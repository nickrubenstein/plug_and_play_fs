use std::{fs, io::Write, time::SystemTime};
use std::io::{Error, ErrorKind};
use std::path::MAIN_SEPARATOR;
use actix_multipart::{Multipart, MultipartError};
use actix_web::web;
use actix_http::error::ParseError;
use serde_json::json;
use futures_util::TryStreamExt;
use time::UtcOffset;
use time::{format_description::well_known::Rfc2822, OffsetDateTime};

use crate::util::error::AppErrorKind;
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
    pub fn new(path: &str) -> Result<Self, AppErrorKind> {
        if !path.starts_with(ROOT_FOLDER) || path.contains("+..+") {
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
        self.path.replace(&format!("{}+",ROOT_FOLDER), &format!(".{}", MAIN_SEPARATOR))
                 .replace(ROOT_FOLDER, &format!(".{}", MAIN_SEPARATOR))
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
        self.path == ROOT_FOLDER
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

    // pub fn copy(&self, entity_name: &str, new_folder: &Folder) -> Result<(), AppErrorKind> {
    //     fs::copy(self.join(entity_name)?.to_path(), new_folder.join(entity_name)?.to_path()).map_err(Into::into)
    // }

    pub fn copy_file(&self, entity_name: &str) -> Result<u64, AppErrorKind> {
        let file = self.join(entity_name)?;
        fs::copy(file.to_path(), file.create_unique_name()).map_err(Into::into)
    }

    pub fn create_unique_name(&self) -> String {
        let mut count = 2;
        let mut path = self.to_path();
        let p = path.to_owned();
        let stem = format!("{}/{}",
            std::path::Path::new(&p).parent().unwrap().to_str().unwrap(),
            std::path::Path::new(&p).file_stem().unwrap().to_str().unwrap()
        );
        loop {
            let path_obj = std::path::Path::new(&path);
            let exists = path_obj.try_exists();
            if exists.is_ok() && exists.unwrap() == false {
                return path;
            }
            match path_obj.extension() {
                Some(ext) => {
                    path = format!("{}({}).{}", stem, count, ext.to_str().unwrap());
                },
                None => {
                    path = format!("{}({})", stem, count);
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
                "value": self.system_time(data.created().unwrap_or(SystemTime::UNIX_EPOCH))
            },
            {
                "name": "Modified", 
                "value": self.system_time(data.modified().unwrap_or(SystemTime::UNIX_EPOCH))
            },
            {
                "name": "Accessed", 
                "value": self.system_time(data.accessed().unwrap_or(SystemTime::UNIX_EPOCH))
            }
        ]))
    }

    fn system_time<T: Into<OffsetDateTime>>(&self, dt: T) -> String {
        let mut date_time: OffsetDateTime = dt.into();
        date_time = date_time.to_offset(UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC));
        date_time.format(&Rfc2822).unwrap_or(String::from("Unknown"))
    }
}
