use std::{fs::{ReadDir, self}, io::Write};
use actix_multipart::{Multipart, MultipartError};
use actix_web::web;
use actix_http::error::ParseError;
use serde_json::json;
use futures_util::{stream::StreamExt as _, TryStreamExt, Stream};

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

    pub fn read_dir(&self) -> std::io::Result<ReadDir> {
        fs::read_dir(self.file_path())
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

    pub fn append_to_uri_path(&self, name: String) -> String {
        if self.path.len() > 0 {
            format!("{}+{}", self.uri_path(), name)
        }
        else {
            name.clone()
        }
    }

    pub fn file_list(&self) -> Result<Vec<serde_json::Value>, std::io::Error> {
        let mut dir = match fs::read_dir(Folder::path(self.file_path())) {
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
}
