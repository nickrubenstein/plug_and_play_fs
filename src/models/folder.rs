use std::{fs, io::Write, time::SystemTime, env};
use std::io::{Error, ErrorKind};
use std::path::MAIN_SEPARATOR;
use actix_multipart::{Multipart, MultipartError};
use actix_web::web;
use actix_http::error::ParseError;
use serde_json::json;
use futures_util::TryStreamExt;

use crate::util::error::AppErrorKind;
use crate::util::{zip, time_format};

const ROOT_URL: &str = "root";
const ROOT_FOLDER_ENV: &str = "FS_ROOT_FOLDER";

#[derive(Clone, Debug)]
pub struct Folder {
    path: String
}

impl Default for Folder {
    fn default() -> Self {
        Self { path: ROOT_URL.to_owned() }
    }
}

/// Handles folder path logic
impl Folder {

    pub fn root_folder() -> String {
        match env::var(ROOT_FOLDER_ENV) {
            Ok(val) => format!("{}{}", val, MAIN_SEPARATOR),
            Err(_) => String::from("")
        }
    }

    /// Creates a new Folder with the given path. The path must start with "root" and 
    /// follow the pattern of folder names separated by '+'. Ex. "root+test_files+folder name"
    pub fn new(path: &str) -> Result<Self, AppErrorKind> {
        if !path.starts_with(ROOT_URL) || path.contains("+..+") {
            log::error!("------>{}<-----", path);
            return Err(AppErrorKind::FolderPathInvalid);
        }
        Ok(Self { path: path.to_owned() })
    }

    /// Returns the folder path structure separated with '+'
    pub fn to_string(&self) -> String {
        self.path.to_string()
    }

    /// Returns the folder path structure compliant to the current OS
    pub fn to_path(&self) -> String {
        self.path.replace(&format!("{}+",ROOT_URL), &format!(".{}{}", MAIN_SEPARATOR, Folder::root_folder()))
                 .replace(ROOT_URL, &format!(".{}{}", MAIN_SEPARATOR, Folder::root_folder()))
                 .replace("+", &String::from(MAIN_SEPARATOR))
    }

    /// Returns a new Folder with path appended on to self.path
    pub fn join(&self, path: &str) -> Result<Self, AppErrorKind> {
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
    pub fn parent(&self) -> Result<Self, AppErrorKind> {
        if self.is_root() {
            return Err(AppErrorKind::CannotGetParentOfRoot);
        }
        let mut folders: Vec<&str> = self.path.split("+").collect();
        folders.pop();
        Self::new(&folders.join("+"))
    }

    pub fn is_root(&self) -> bool {
        self.path == ROOT_URL
    }
}

/// Handles calls to fs functions
impl Folder {

    pub fn entity_list(&self, folders_only: bool) -> Result<(Vec<serde_json::Value>, Vec<serde_json::Value>), AppErrorKind> {
        let entities = self.entities(folders_only)?;
        Ok((
            entities.0.into_iter().map(|folder| { json!({
                    "path": folder.to_string(),
                    "name": folder.name()
                })
            }).collect(),
            entities.1.into_iter().map(|file| { json!({
                    "path": file.to_string(),
                    "name": file.name()
                })
            }).collect()
        ))
    }

    pub fn entities(&self, folders_only: bool) -> Result<(Vec<Self>, Vec<Self>), AppErrorKind> {
        let mut dir = fs::read_dir(self.to_path())?;
        let mut entities = (Vec::new(), Vec::new());
        while let Some(entry) = dir.next() {
            if let Ok(dir_entry) = entry {
                if let (Ok(file_name), Ok(file_type)) = (dir_entry.file_name().into_string(), dir_entry.file_type()) {
                    if file_type.is_dir() {
                        entities.0.push(Self {
                            path: self.join(&file_name)?.to_string()
                        });
                    } else if !folders_only {
                        entities.1.push(Self {
                            path: self.join(&file_name)?.to_string()
                        });
                    }

                }
            }
        }
        entities.0.sort_by(|a,b| { 
            a.name().cmp(b.name())
        });
        entities.1.sort_by(|a,b| { 
            a.name().cmp(b.name())
        });
        Ok(entities)
    }

    pub fn details(&self) -> Result<serde_json::Value, AppErrorKind> {
        self.common_details(None).map_err(Into::into)
    }

    pub fn file_details(&self, file_name: &str) -> Result<serde_json::Value, AppErrorKind> {
        self.common_details(Some(file_name)).map_err(Into::into)
    }

    pub fn create_dir(&self, folder_name: &str) -> Result<(), AppErrorKind> {
        fs::create_dir(self.join(&folder_name)?.to_path()).map_err(Into::into)
    }

    pub fn rename(&mut self, name: &str) -> Result<(), AppErrorKind> {
        if self.is_root() {
            return Err(AppErrorKind::CannotRenameRoot);
        }
        let new_folder = self.parent()?.join(&name)?;
        let result = fs::rename(self.to_path(), new_folder.to_path());
        if let Ok(()) = result {
            self.path = new_folder.path
        }
        result.map_err(Into::into)
    }

    pub fn rename_file(&self, old_name: &str, new_name: &str) -> Result<(), AppErrorKind> {
        fs::rename(self.join(old_name)?.to_path(), self.join(new_name)?.to_path()).map_err(Into::into)
    }

    pub fn move_entity(&self, entity_name: &str, new_folder: &Folder) -> Result<(), AppErrorKind> {
        fs::rename(self.join(entity_name)?.to_path(), new_folder.join(entity_name)?.to_path()).map_err(Into::into)
    }

    pub fn copy(&self) -> Result<Self, AppErrorKind> {
        if self.is_root() {
            return Err(AppErrorKind::CannotCopyRoot);
        }
        let new_name = self.create_unique_name();
        let new_folder = self.parent()?.join(&new_name)?;
        fs::create_dir(new_folder.to_path())?;
        self.copy_to(self, &new_folder)?;
        Ok(new_folder)
    }

    pub fn copy_to(&self, source_folder: &Self, target_folder: &Self) -> Result<(), AppErrorKind> {
        let entities = source_folder.entities(false)?;
        for folder in entities.0 {
            let new_folder = target_folder.join(folder.name())?;
            fs::create_dir(new_folder.to_path())?;
            self.copy_to(&folder, &new_folder)?;
        }
        for file in entities.1 {
            fs::copy(file.to_path(), target_folder.join(file.name())?.to_path())?;
        }
        Ok(())
    }

    pub fn copy_file(&self, entity_name: &str) -> Result<Folder, AppErrorKind> {
        let file = self.join(entity_name)?;
        let new_name = file.create_unique_name();
        fs::copy(file.to_path(), self.join(&new_name)?.to_path())?;
        Ok(self.join(&new_name)?)
    }

    pub fn create_unique_name(&self) -> String {
        let mut count = 2;
        let path = self.to_path();
        let path = std::path::Path::new(&path);
        let stem = path.file_stem().unwrap().to_str().unwrap();
        let parent_path = path.parent().unwrap();
        let mut name = path.file_name().unwrap().to_str().unwrap().to_owned();
        loop {
            let path = parent_path.join(&name);
            let exists = path.try_exists();
            if exists.is_ok() && exists.unwrap() == false {
                return name;
            }
            match path.extension() {
                Some(ext) => {
                    name = format!("{}({}).{}", stem, count, ext.to_str().unwrap());
                },
                None => {
                    name = format!("{}({})", stem, count);
                }
            }
            count += 1;
        }
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

    pub async fn read_file(&self, name: &str) -> Result<Vec<u8>, AppErrorKind> {
        let file_path = self.join(name)?.to_path();
        match web::block(move || fs::read(file_path)).await {
            Ok(result) => result.map_err(Into::into),
            Err(_e) => Err(AppErrorKind::FailedToReadFile)
        }
    }

    pub fn remove_file(&self, name: &str) -> Result<(), AppErrorKind> {
        fs::remove_file(self.join(name)?.to_path()).map_err(Into::into)
    }

    pub fn remove(&self) -> Result<(), AppErrorKind> {
        if self.is_root() {
            return Err(AppErrorKind::CannotDeleteRoot);
        }
        fs::remove_dir_all(self.to_path()).map_err(Into::into)
    }

    pub async fn zip(&self) -> Result<(), AppErrorKind> {
        let parent_path = self.parent()?;
        let folder_name = self.name().to_owned();
        match web::block(move || zip::create_zip_from_folder(&parent_path.to_path(), &folder_name)).await {
            Ok(result) => result.map_err(Into::into),
            Err(_e) => Err(AppErrorKind::FailedToZipFolder)
        }
    }

    pub async fn unzip_file(&self, file_name: &str) -> Result<(), AppErrorKind> {
        let file_path = self.join(file_name)?.to_path();
        match web::block(move || zip::extract_zip(&file_path)).await {
            Ok(result) => result.map_err(Into::into),
            Err(_e) => Err(AppErrorKind::FailedToUnzipFile)
        }
    }

    fn common_details(&self, file_name: Option<&str>) -> Result<serde_json::Value, AppErrorKind> {
        let data = if let Some(name) = file_name {
            fs::metadata(self.join(name)?.to_path())?
        }
        else {
            fs::metadata(self.to_path())?
        };
        Ok(json!([
            {
                "name": "Size", 
                "value": data.len()
            },
            {
                "name": "Readonly", 
                "value": data.permissions().readonly()
            },
            {
                "name": "Created", 
                "value": time_format::format_time(data.created().unwrap_or(SystemTime::UNIX_EPOCH), None)
            },
            {
                "name": "Modified", 
                "value": time_format::format_time(data.modified().unwrap_or(SystemTime::UNIX_EPOCH), None)
            },
            {
                "name": "Accessed", 
                "value": time_format::format_time(data.accessed().unwrap_or(SystemTime::UNIX_EPOCH), None)
            }
        ]))
    }
}
