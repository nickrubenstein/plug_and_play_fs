use std::str::FromStr;

const ROOT: &str = ".";

pub struct FilePath {
    path: String
}

impl FilePath {
    pub fn new(path: String) -> Result<FilePath, &'static str> {
        if path.contains("..") {
            return Err("Path cannot be relative and include any ../");
        }
        if path.contains("~") {
            return Err("Path cannot be relative and include ~/");
        }
        Ok(FilePath {
            path
        })
    }

    pub fn uri_path(&self) -> String {
        self.path.clone()
    }

    pub fn file_path(&self) -> String {
        format!("{}/{}", ROOT, self.path.replace("+", "/"))
    }

    pub fn parent(&self) -> FilePath {
        let mut path_list = self.path_list();
        path_list.pop();
        if path_list.len() > 0 {
            FilePath::new(path_list.join("+")).unwrap()
        }
        else {
            FilePath::new(String::new()).unwrap()
        }
    }

    pub fn append_to_file_path(&self, name: &String) -> String {
        if self.path.len() > 0 {
            format!("{}/{}", self.file_path(), name)
        }
        else {
            format!("{}{}", self.file_path(), name)
        }
    }

    pub fn append_to_uri_path(&self, name: &String) -> String {
        if self.path.len() > 0 {
            format!("{}+{}", self.uri_path(), name)
        }
        else {
            name.clone()
        }
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

    fn path_list(&self) -> Vec<String> {
        self.path.split("+").map(|s| String::from_str(s).unwrap()).collect()
    }
}
