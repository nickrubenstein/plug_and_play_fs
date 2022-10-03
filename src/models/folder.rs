use std::{fs::{ReadDir, self}, io::Write, time::SystemTime};
use actix_multipart::{Multipart, MultipartError};
use actix_web::web;
use actix_http::error::ParseError;
use serde_json::json;
use futures_util::TryStreamExt;
use time::{format_description::well_known::Iso8601, OffsetDateTime};

const ROOT: &str = ".";

pub struct Folder {
    path: String
}

impl Folder {
    pub fn new(path: String) -> std::io::Result<Folder> {
        if path.contains("..") { //todo make custom errors that deescribe these permission denied better
            return Err(std::io::ErrorKind::PermissionDenied.into()); // Path cannot be relative and include any ../
        }
        if path.contains("~") {
            return Err(std::io::ErrorKind::PermissionDenied.into()); // Path cannot be relative and include ~/
        }
        if !path.starts_with("root") {
            return Err(std::io::ErrorKind::InvalidInput.into()); // Path cannot be relative and include ~/
        }
        Ok(Folder { path })
    }

    pub fn uri_path(&self) -> String {
        self.path.clone()
    }

    pub fn file_path(&self) -> String {
        Folder::path(self.path.clone())
    }

    pub fn path(path: String) -> String {
        format!("{}/{}", ROOT, path.replace("root+","").replace("root","").replace("+", "/"))
    }

    pub fn append_to_uri_path(&self, name: String) -> String {
        if self.path.len() > 0 {
            format!("{}+{}", self.uri_path(), name)
        }
        else {
            name.clone()
        }
    }

    fn path_list(&self) -> Vec<String> {
        self.path.split("+").map(|s| String::from(s)).collect()
    }

    pub fn path_list_aggrigate(&self) -> Vec<(String, String)> {
        let mut path_list = self.path_list();
        let mut path_display = Vec::new();
        path_display.push((path_list[0].clone(), path_list[0].clone()));
        for i in 1..path_list.len() {
            let full_path = format!("{}+{}", path_list[i - 1], path_list[i]);
            path_display.push((path_list[i].clone(), full_path.clone()));
            path_list[i] = full_path;
        }
        path_display
    }

    pub fn name(&self) -> String {
        let path = self.uri_path();
        let folders = path.split("+");
        folders.last().unwrap().to_string()
    }

    pub fn parent(&self) -> String {
        let path = self.uri_path();
        let mut folders: Vec<&str> = path.split("+").collect();
        folders.pop();
        folders.join("+")
    }

    pub fn read_dir(&self) -> std::io::Result<ReadDir> {
        fs::read_dir(self.file_path())
    }

    pub fn file_list(&self) -> Result<Vec<serde_json::Value>, std::io::Error> {
        let mut dir = match fs::read_dir(self.file_path()) {
            Ok(d) => d,
            Err(e) => return Err(e)
        };
        let mut files = Vec::new();
        while let Some(entry) = dir.next() {
            if let Ok(dir_entry) = entry {
                if let (Ok(file_name), Ok(file_type)) = (dir_entry.file_name().into_string(), dir_entry.file_type()) {
                    files.push(json!({
                        "path": self.append_to_uri_path(file_name.clone()),
                        "name": file_name, 
                        "is_folder": file_type.is_dir()
                    }));
                }
            }
        }
        Ok(files)
    }

    fn systemtime_to_iso<T: Into<OffsetDateTime>>(&self, dt: T) -> Result<String, time::error::Format> {
        dt.into().format(&Iso8601::DEFAULT)
    }

    pub fn details(&self) -> Result<serde_json::Value, std::io::Error> {
        let data = fs::metadata(self.file_path())?;
        let created = self.systemtime_to_iso(data.created().unwrap_or(SystemTime::UNIX_EPOCH), )
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "could not get create date"))?;
        let modified = self.systemtime_to_iso(data.modified().unwrap_or(SystemTime::UNIX_EPOCH))
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "could not get modified date"))?;
        let accessed = self.systemtime_to_iso(data.accessed().unwrap_or(SystemTime::UNIX_EPOCH))
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

    pub fn file_details(&self, file_name: String) -> Result<serde_json::Value, std::io::Error> {
        let data = fs::metadata(Folder::path(self.append_to_uri_path(file_name)))?;
        let created = self.systemtime_to_iso(data.created().unwrap_or(SystemTime::UNIX_EPOCH), )
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "could not get create date"))?;
        let modified = self.systemtime_to_iso(data.modified().unwrap_or(SystemTime::UNIX_EPOCH))
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "could not get modified date"))?;
        let accessed = self.systemtime_to_iso(data.accessed().unwrap_or(SystemTime::UNIX_EPOCH))
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

    pub fn create_dir(&self, name: &str) -> std::io::Result<()> {
        let path = self.file_path();
        if path.len() > 0 {
            fs::create_dir(format!("{}/{}", self.file_path(), name))
        }
        else {
            fs::create_dir(name)
        }
    }

    pub fn rename(&mut self, name: &str) -> std::io::Result<()> {
        let uri_path = self.uri_path();
        if uri_path == "root" {
            return Err(std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Cannot rename root folder"));
        }
        let mut new_path: Vec<&str> = uri_path.split("+").collect();
        new_path.pop();
        new_path.push(name);
        let new_path = new_path.join("+");
        
        let result = fs::rename(self.file_path(), Folder::path(new_path.clone()));
        if let Ok(()) = result {
            self.path = new_path;
        }
        result
    }

    pub fn rename_file(&self, old_name: String, new_name: String) -> std::io::Result<()> {
        fs::rename(Folder::path(self.append_to_uri_path(old_name)), Folder::path(self.append_to_uri_path(new_name)))
    }

    pub async fn upload_file(&self, mut payload: Multipart) -> Result<Vec<String>, MultipartError> {
        let mut file_names = Vec::new();
        // iterate over multipart stream
        while let Some(mut field) = payload.try_next().await? {
            let file_name = field.content_disposition().get_filename().unwrap_or_default().to_string();
            file_names.push(file_name.clone());
            log::info!("field: {:?}", file_name);
            // File::create is blocking operation, use threadpool
            let file_path = Folder::path(self.append_to_uri_path(file_name.to_string()));
            let mut file = match web::block(move || std::fs::File::create(file_path)).await {
                Ok(Ok(f)) => f,
                Ok(Err(e)) => return Err(MultipartError::Parse(ParseError::Io(e))),
                Err(e) => return Err(MultipartError::Parse(ParseError::Io(
                    std::io::Error::new(std::io::ErrorKind::WouldBlock, e))))
            };
            // Field in turn is stream of *Bytes* object
            while let Some(chunk) = field.try_next().await? {
                // let field_size = chunk.con();
                // log::info!("field: {:?}", field_size);
                // filesystem operations are blocking, we have to use threadpool
                file = match web::block(move || file.write_all(&chunk).map(|_| file)).await {
                    Ok(Ok(f)) => f,
                    Ok(Err(e)) => return Err(MultipartError::Parse(ParseError::Io(e))),
                    Err(e) => return Err(MultipartError::Parse(ParseError::Io(
                        std::io::Error::new(std::io::ErrorKind::WouldBlock, e))))
                };
            }
        }
        Ok(file_names)
    }

    pub fn remove_file(&self, name: String) -> std::io::Result<()> {
        fs::remove_file(Folder::path(self.append_to_uri_path(name)))
    }

    pub fn remove(&self) -> std::io::Result<()> {
        let uri_path = self.uri_path();
        if uri_path == "root" {
            return Err(std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Cannot remove root folder"));
        }
        fs::remove_dir(self.file_path())
    }
}
