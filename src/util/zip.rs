use std::io::prelude::*;
use std::io::{Write, Seek};
use std::iter::Iterator;
use zip::write::FileOptions;

use std::path::Path;
use std::fs::{File, self, DirEntry, ReadDir};

// const ZIP_METHOD : zip::CompressionMethod = zip::CompressionMethod::Stored;
const DEFLATED_METHOD : zip::CompressionMethod = zip::CompressionMethod::Deflated;

pub fn create_zip_from_folder(folder_path: String, folder_name: String) -> std::io::Result<()> {
    let src_dir = format!("{}/{}", folder_path, folder_name);
    let dst_file = format!("{}/{}.zip", folder_path, folder_name);
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
        .compression_method(DEFLATED_METHOD)
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

pub fn extract_zip(archive_path: &str) -> std::io::Result<()> {
    let archive_path = std::path::Path::new(archive_path);
    let archive_file = fs::File::open(archive_path)?;
    let file_name = archive_path.file_name().unwrap().to_str().unwrap().replace(".tar.bz2", "").replace(".zip", "");
    let extracted_path = archive_path.parent().unwrap().join(file_name);
    let mut archive = zip::ZipArchive::new(archive_file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => extracted_path.join(path),
            None => continue,
        };

        {
            let comment = file.comment();
            if !comment.is_empty() {
                log::debug!("File {} comment: {}", i, comment);
            }
        }

        if (*file.name()).ends_with('/') {
            log::debug!("File {} extracted to \"{}\"", i, outpath.display());
            fs::create_dir_all(&outpath)?;
        } else {
            log::debug!("File {} extracted to \"{}\" ({} bytes)", i, outpath.display(), file.size());
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(&p)?;
                }
            }
            let mut outfile = fs::File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }

        // Get and Set permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            if let Some(mode) = file.unix_mode() {
                fs::set_permissions(&outpath, fs::Permissions::from_mode(mode))?;
            }
        }
    }
    Ok(())
}