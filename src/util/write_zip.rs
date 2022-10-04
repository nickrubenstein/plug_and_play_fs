use std::io::prelude::*;
use std::io::{Write, Seek};
use std::iter::Iterator;
use zip::write::FileOptions;
use zip::result::ZipError;

use std::path::Path;
use std::fs::{File, self, DirEntry, ReadDir};

const METHOD_STORED : Option<zip::CompressionMethod> = Some(zip::CompressionMethod::Stored);

#[cfg(feature = "deflate")]
const METHOD_DEFLATED : Option<zip::CompressionMethod> = Some(zip::CompressionMethod::Deflated);
#[cfg(not(feature = "deflate"))]
const METHOD_DEFLATED : Option<zip::CompressionMethod> = None;

#[cfg(feature = "bzip2")]
const METHOD_BZIP2 : Option<zip::CompressionMethod> = Some(zip::CompressionMethod::Bzip2);
#[cfg(not(feature = "bzip2"))]
const METHOD_BZIP2 : Option<zip::CompressionMethod> = None;

pub fn write_zip_from_folder(folder_path: String, folder_name: String) -> std::io::Result<()> {
    let src_dir = format!("{}/{}", folder_path, folder_name);
    let dst_file = format!("{}/{}.zip", folder_path, folder_name);
    log::info!("done: {} written to {}", src_dir, dst_file);
    for &method in [METHOD_STORED, METHOD_DEFLATED, METHOD_BZIP2].iter() {
        if method.is_none() { continue }
        doit(src_dir.clone(), dst_file.clone(), method.unwrap())?
    }
    Ok(())
}

fn zip_dir<T>(all_dirs: Vec<DirEntry>, prefix: &str, writer: T, method: zip::CompressionMethod)
              -> zip::result::ZipResult<()>
    where T: Write+Seek
{
    let mut zip = zip::ZipWriter::new(writer);
    let options = FileOptions::default()
        .compression_method(method)
        .unix_permissions(0o755);

    let mut buffer = Vec::new();
    for entry in all_dirs {
        let path = entry.path();
        let name = path.strip_prefix(Path::new(prefix)).unwrap().to_str().unwrap();
        // Write file or directory explicitly
        // Some unzip tools unzip files with directory paths correctly, some do not!
        if path.is_file() {
            // log::info!("adding file {:?} as {:?} ...", path, name);
            zip.start_file(name, options)?;
            let mut f = File::open(path)?;
            f.read_to_end(&mut buffer)?;
            zip.write_all(&*buffer)?;
            buffer.clear();
        } else if name.len() != 0 {
            // Only if not root! Avoids path spec / warning
            // and mapname conversion failed error on unzip
            // log::info!("adding dir {:?} as {:?} ...", path, name);
            zip.add_directory(name, options)?;
        }
    }
    zip.finish()?;
    Ok(())
}

fn doit(src_dir: String, dst_file: String, method: zip::CompressionMethod) -> zip::result::ZipResult<()> {
    if !Path::new(&src_dir).is_dir() {
        return Err(ZipError::FileNotFound);
    }
    let path = Path::new(&dst_file);
    let file = File::create(&path)?;
    let dir = fs::read_dir(src_dir.to_string())?;
    let all_dirs = read_all_dirs(dir);

    zip_dir(all_dirs, &src_dir, file, method)?;
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