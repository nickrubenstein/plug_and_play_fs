use std::io::prelude::*;
use std::io::{Write, Seek};
use std::iter::Iterator;
use zip::write::FileOptions;

use std::path::Path;
use std::fs::{File, self, DirEntry, ReadDir};

// const ZIP_METHOD : zip::CompressionMethod = zip::CompressionMethod::Stored;
// const ZIP_METHOD : zip::CompressionMethod = zip::CompressionMethod::Deflated;
const ZIP_METHOD: zip::CompressionMethod = zip::CompressionMethod::Bzip2;

pub fn write_zip_from_folder(folder_path: String, folder_name: String) -> std::io::Result<()> {
    let src_dir = format!("{}/{}", folder_path, folder_name);
    let dst_file = format!("{}/{}.tar.bz2", folder_path, folder_name);
    // log::debug!("zip: {} to {}", src_dir, dst_file);
    if !Path::new(&src_dir).is_dir() {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Can only zip folders"));
    }
    let path = Path::new(&dst_file);
    let file = File::create(&path)?;
    let dir = fs::read_dir(src_dir.to_string())?;
    let all_dirs = read_all_dirs(dir);

    zip_dir(all_dirs, &src_dir, file)?;
    Ok(())
}

fn zip_dir<T>(all_dirs: Vec<DirEntry>, prefix: &str, writer: T)
              -> zip::result::ZipResult<()>
    where T: Write+Seek
{
    let mut zip = zip::ZipWriter::new(writer);
    let options = FileOptions::default()
        .compression_method(ZIP_METHOD)
        .unix_permissions(0o755);

    let mut buffer = Vec::new();
    for entry in all_dirs {
        let path = entry.path();
        let name = path.strip_prefix(Path::new(prefix)).unwrap().to_str().unwrap();
        // Write file or directory explicitly
        // Some unzip tools unzip files with directory paths correctly, some do not!
        if path.is_file() {
            // log::debug!("adding file {:?} as {:?} ...", path, name);
            zip.start_file(name, options)?;
            let mut f = File::open(path)?;
            f.read_to_end(&mut buffer)?;
            zip.write_all(&*buffer)?;
            buffer.clear();
        } else if name.len() != 0 {
            // Only if not root! Avoids path spec / warning
            // and mapname conversion failed error on unzip
            // log::debug!("adding dir {:?} as {:?} ...", path, name);
            zip.add_directory(name, options)?;
        }
    }
    zip.finish()?;
    Ok(())
}

fn read_all_dirs(dir: ReadDir) -> Vec<DirEntry> {
    let mut sub_dirs = Vec::new();
    let mut dir: Vec<DirEntry> = dir.into_iter().filter_map(|e| e.ok()).map(|entity| {
        if entity.path().is_dir() {
            if let Ok(sub_dir) = fs::read_dir(entity.path().to_str().unwrap()) {
                sub_dirs.append(&mut read_all_dirs(sub_dir));
            }
        }
        entity
    }).collect();
    dir.append(&mut sub_dirs);
    dir
}