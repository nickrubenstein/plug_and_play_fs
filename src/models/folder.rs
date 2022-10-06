use std::{fs, io::Write, time::SystemTime};
use std::io::{Error, ErrorKind, self};
use std::path::MAIN_SEPARATOR;
use actix_multipart::{Multipart, MultipartError};
use actix_web::web;
use actix_http::error::ParseError;
use serde_json::json;
use futures_util::TryStreamExt;
use time::{format_description::well_known::Rfc2822, OffsetDateTime};

use crate::util::zip;

const ROOT_FOLDER: &str = "root";

#[derive(Clone, Debug)]
pub struct Folder {
    path: String
}

impl Default for Folder {
    fn default() -> Self {
        Self { path: ROOT_FOLDER.to_owned() }
    }
}

/// Handles folder path logic
impl Folder {

    /// Creates a new Folder with the given path. The path must start with "root" and 
    /// follow the pattern of folder names separated by '+'. Ex. "root+test_files+folder name"
    pub fn new(path: &str) -> io::Result<Self> {
        if path.contains("..") { //todo make custom errors that deescribe these permission denied better
            return Err(ErrorKind::PermissionDenied.into()); // Path cannot be relative and include any ../
        }
        if path.contains("~") {
            return Err(ErrorKind::PermissionDenied.into()); // Path cannot be relative and include ~/
        }
        if !path.starts_with(ROOT_FOLDER) {
            return Err(ErrorKind::InvalidInput.into());
        }
        Ok(Self { path: path.to_owned() })
    }

    /// Returns the folder path structure separated with '+'
    pub fn to_string(&self) -> String {
        self.path.to_string()
    }

    /// Returns the folder path structure compliant to the current OS
    pub fn to_path(&self) -> String {
        self.path.replace(&format!("{}+",ROOT_FOLDER), &format!(".{}", MAIN_SEPARATOR))
                 .replace(ROOT_FOLDER, &format!(".{}", MAIN_SEPARATOR))
                 .replace("+", &String::from(MAIN_SEPARATOR))
    }

    /// Returns a new Folder with path appended on to self.path
    pub fn join(&self, path: &str) -> io::Result<Self> {
        let join = format!("{}+{}", self.path, path);
        Self::new(&join)
    }

    /// Returns a list of the folder path parents starting with root
    pub fn ancestors(&self, include_self: bool) -> Vec<Self> {
        let mut ancestors = Vec::new();
        let mut parent = self.clone();
        if include_self {
            ancestors.push(parent.clone());
        }
        while let Ok(p) = parent.parent() {
            parent = p;
            ancestors.insert(0, parent.clone());
        }
        ancestors
    }

    /// Returns the name of the folder
    pub fn name(&self) -> &str {
        let folders = self.path.split("+");
        folders.last().unwrap()
    }

    /// Returns a Folder representing the parent of the current folder.
    /// Returns Error if trying to get the parent of root
    pub fn parent(&self) -> io::Result<Self> {
        if self.is_root() {
            return Err(Error::new(ErrorKind::PermissionDenied, "Cannot get parent of root folder"));
        }
        let mut folders: Vec<&str> = self.path.split("+").collect();
        folders.pop();
        Self::new(&folders.join("+"))
    }

    pub fn is_root(&self) -> bool {
        self.path == ROOT_FOLDER
    }
}

/// Handles calls to fs functions
impl Folder {

    pub fn entity_list(&self) -> io::Result<Vec<serde_json::Value>> {
        let mut dir = match fs::read_dir(self.to_path()) {
            Ok(d) => d,
            Err(e) => return Err(e)
        };
        let mut files = Vec::new();
        while let Some(entry) = dir.next() {
            if let Ok(dir_entry) = entry {
                if let (Ok(file_name), Ok(file_type)) = (dir_entry.file_name().into_string(), dir_entry.file_type()) {
                    files.push(json!({
                        "path": self.join(&file_name)?.to_string(),
                        "name": file_name, 
                        "is_folder": file_type.is_dir()
                    }));
                }
            }
        }
        Ok(files)
    }

    pub fn details(&self) -> io::Result<serde_json::Value> {
        let common_json = self.common_details(None)?;
        Ok(common_json)
    }

    pub fn file_details(&self, file_name: String) -> io::Result<serde_json::Value> {
        let common_json = self.common_details(Some(file_name))?;
        Ok(common_json)
    }

    pub fn create_dir(&self, folder_name: &str) -> io::Result<()> {
        fs::create_dir(self.join(&folder_name)?.to_path())
    }

    pub fn rename(&mut self, name: &str) -> io::Result<()> {
        if self.is_root() {
            return Err(Error::new(ErrorKind::PermissionDenied, "Cannot rename root folder"));
        }
        let new_folder = self.parent()?.join(&name)?;
        let result = fs::rename(self.to_path(), new_folder.to_path());
        if let Ok(()) = result {
            self.path = new_folder.path
        }
        result
    }

    pub fn rename_file(&self, old_name: String, new_name: String) -> io::Result<()> {
        fs::rename(self.join(&old_name)?.to_path(), self.join(&new_name)?.to_path())
    }

    pub async fn upload_file(&self, mut payload: Multipart) -> Result<Vec<String>, MultipartError> {
        let mut file_names = Vec::new();
        // iterate over multipart stream
        while let Some(mut field) = payload.try_next().await? {
            let file_name = field.content_disposition().get_filename().unwrap_or_default().to_string();
            file_names.push(file_name.clone());
            // log::debug!("field: {:?}", file_name);
            // File::create is blocking operation, use threadpool
            let file_path = self.join(&file_name).unwrap().to_path();
            let mut file = match web::block(move || std::fs::File::create(file_path)).await {
                Ok(Ok(f)) => f,
                Ok(Err(e)) => return Err(MultipartError::Parse(ParseError::Io(e))),
                Err(e) => return Err(MultipartError::Parse(ParseError::Io(
                    Error::new(ErrorKind::WouldBlock, e))))
            };
            // Field in turn is stream of *Bytes* object
            while let Some(chunk) = field.try_next().await? {
                // let field_size = chunk.con();
                // log::debug!("field: {:?}", field_size);
                // filesystem operations are blocking, we have to use threadpool
                file = match web::block(move || file.write_all(&chunk).map(|_| file)).await {
                    Ok(Ok(f)) => f,
                    Ok(Err(e)) => return Err(MultipartError::Parse(ParseError::Io(e))),
                    Err(e) => return Err(MultipartError::Parse(ParseError::Io(
                        Error::new(ErrorKind::WouldBlock, e))))
                };
            }
        }
        Ok(file_names)
    }

    pub async fn read_file(&self, name: String) -> io::Result<Vec<u8>> {
        let file_path = self.join(&name)?.to_path();
        match web::block(move || fs::read(file_path)).await {
            Ok(result) => result,
            Err(e) => Err(Error::new(ErrorKind::WouldBlock, e))
        }
    }

    pub fn remove_file(&self, name: String) -> io::Result<()> {
        fs::remove_file(self.join(&name)?.to_path())
    }

    pub fn remove(&self) -> io::Result<()> {
        if self.is_root() {
            return Err(Error::new(ErrorKind::PermissionDenied, "Cannot remove root folder"));
        }
        fs::remove_dir(self.to_path())
    }

    pub async fn zip(&self) -> io::Result<()> {
        let parent_path = self.parent()?;
        let folder_name = self.name().to_owned();
        match web::block(move || zip::create_zip_from_folder(&parent_path.to_path(), &folder_name)).await {
            Ok(result) => result,
            Err(e) => Err(Error::new(ErrorKind::WouldBlock, e))
        }
    }

    pub async fn extract_file(&self, file_name: &str) -> io::Result<()> {
        let file_path = self.join(file_name)?.to_path();
        match web::block(move || zip::extract_zip(&file_path)).await {
            Ok(result) => result,
            Err(e) => Err(Error::new(ErrorKind::WouldBlock, e))
        }
    }

    fn common_details(&self, file_name: Option<String>) -> io::Result<serde_json::Value> {
        let data = if let Some(name) = file_name {
            fs::metadata(self.join(&name)?.to_path())?
        }
        else {
            fs::metadata(self.to_path())?
        };
        let created = self.system_time(data.created().unwrap_or(SystemTime::UNIX_EPOCH), )
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "could not get create date"))?;
        let modified = self.system_time(data.modified().unwrap_or(SystemTime::UNIX_EPOCH))
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "could not get modified date"))?;
        let accessed = self.system_time(data.accessed().unwrap_or(SystemTime::UNIX_EPOCH))
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "could not get modified date"))?;
        Ok(json!([
            {
                "name": "Size", 
                "value": data.len()
            },
            {
                "name": "readonly", 
                "value": data.permissions().readonly()
            },
            {
                "name": "created", 
                "value": created
            },
            {
                "name": "modified", 
                "value": modified
            },
            {
                "name": "accessed", 
                "value": accessed
            }
        ]))
    }

    fn system_time<T: Into<OffsetDateTime>>(&self, dt: T) -> Result<std::string::String, time::error::Format> {
        dt.into().format(&Rfc2822)
    }
}
