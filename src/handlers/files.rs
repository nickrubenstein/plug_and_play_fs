use actix_multipart::Multipart;
use actix_web::{web, HttpResponse, 
    http::{self, header::{ContentDisposition, DispositionType, DispositionParam}}
};
use actix_web_flash_messages::{FlashMessage, IncomingFlashMessages};
use serde::Deserialize;
use serde_json::json;
use handlebars::Handlebars;

use crate::{models::folder::Folder, util::{error::{AppError, AppErrorKind}, forward::ForwardTo}};
use crate::util::forward;

const PARENT_OPTION: &str = "|Move to parent folder|";

#[derive(Deserialize)]
pub struct RenameFileFormData {
    file_name: String
}

#[derive(Deserialize)]
pub struct MoveFileIntoFormData {
    folder_name: String
}

#[derive(Deserialize)]
pub struct MoveEntitiesIntoFormData {
    selected_folders: String,
    selected_files: String,
    folder_name: String
}

#[derive(Deserialize)]
pub struct RemoveEntitiesFormData {
    selected_folders: String,
    selected_files: String
}

pub async fn get_files(folder_path: web::Path<String>, hb: web::Data<Handlebars<'_>>, flashes: IncomingFlashMessages) -> Result<HttpResponse, AppError> {
    let folder = Folder::new(&folder_path.into_inner())
        .map_err(AppError::root)?;
    let (folders, files) = folder.entity_list(false)
        .map_err(|k| AppError::new(k, ForwardTo::Folder(folder.parent().unwrap_or_default())))?;
    let flashes: Vec<(String,String)> = flashes.iter().map(|f| {(f.level().to_string(), f.content().to_string())}).collect();
    let crumbs: Vec<(String,String)> = folder.ancestors(true).iter().map(|a| { (a.to_string(), a.name().to_owned())}).collect();
    let data = json! ({
        "title": "FS",
        "flashes": flashes,
        "folder_path": folder.to_string(),
        "crumbs": crumbs,
        "entity_count": folders.len() + files.len(),
        "folders": folders,
        "files": files,
        "parent_option": PARENT_OPTION.clone()
    });
    let body = hb.render("files", &data).unwrap();
    Ok(HttpResponse::Ok().body(body))
}

pub async fn get_file_detail(path: web::Path<(String,String)>, hb: web::Data<Handlebars<'_>>, flashes: IncomingFlashMessages) -> Result<HttpResponse, AppError> {
    let (folder_path, file_name) = path.into_inner();
    let folder = Folder::new(&folder_path)
        .map_err(AppError::root)?;
    let details = folder.file_details(&file_name)
        .map_err(|k| AppError::new(k, ForwardTo::Folder(folder.clone())))?;
    let folders = folder.entity_list(true)
        .map_err(|k| AppError::new(k, ForwardTo::Folder(folder.clone())))?.0;
    let flashes: Vec<(String,String)> = flashes.iter().map(|f| {(f.level().to_string(), f.content().to_string())}).collect();
    let crumbs: Vec<(String,String)> = folder.ancestors(true).iter().map(|a| { (a.to_string(), a.name().to_owned())}).collect();
    let data = json! ({
        "title": "FS",
        "flashes": flashes,
        "folder_path": folder.to_string(),
        "crumbs": crumbs,
        "file_name": file_name,
        "details": details,
        "folders": folders,
        "parent_option": PARENT_OPTION.clone()
    });
    let body = hb.render("file-detail", &data).unwrap();
    Ok(HttpResponse::Ok().body(body))
}

pub async fn upload_file(folder_path: web::Path<String>, payload: Multipart) -> Result<HttpResponse, AppError> {
    let folder = Folder::new(&folder_path.into_inner())
        .map_err(AppError::root)?;
    match folder.upload_file(payload).await {
        Ok(file_names) if file_names.len() == 1 => FlashMessage::success(format!("uploaded file '{}'", file_names[0])).send(),
        Ok(file_names) if file_names.len() > 1 => FlashMessage::success(format!("uploaded {} files", file_names.len())).send(),
        Ok(_) => FlashMessage::error("No files were uploaded").send(),
        Err(e) => FlashMessage::error(e.to_string()).send()
    }
    Ok(forward::to(&ForwardTo::Folder(folder)))
}

pub async fn download_file(path: web::Path<(String,String)>) -> Result<HttpResponse, AppError> {
    let (folder_path, file_name) = path.into_inner();
    let folder = Folder::new(&folder_path)
        .map_err(AppError::root)?;
    let file_content = folder.read_file(&file_name).await
        .map_err(|k| AppError::new(k, ForwardTo::FileDetail(folder, file_name.clone())))?;
    let content_disposition = ContentDisposition {
        disposition: DispositionType::Attachment,
        parameters: vec![DispositionParam::Filename(file_name)],
    };
    Ok(HttpResponse::Ok().append_header((http::header::CONTENT_DISPOSITION, content_disposition)).body(file_content))
}

pub async fn rename_file(path: web::Path<(String,String)>, form: web::Form<RenameFileFormData>) -> Result<HttpResponse, AppError> {
    let (folder_path, file_name) = path.into_inner();
    let folder = Folder::new(&folder_path)
        .map_err(AppError::root)?;
    folder.rename_file(&file_name, &form.file_name)
        .map_err(|k| AppError::new(k, ForwardTo::FileDetail(folder.clone(), file_name.clone())))?;
    FlashMessage::success(format!("renamed file '{}' to '{}'", &file_name, &form.file_name)).send();
    Ok(forward::to(&ForwardTo::FileDetail(folder, form.file_name.clone())))
}

pub async fn move_file(path: web::Path<(String,String)>, form: web::Form<MoveFileIntoFormData>) -> Result<HttpResponse, AppError> {
    if form.folder_name == PARENT_OPTION {
        return move_file_up(path).await;
    }
    let (folder_path, file_name) = path.into_inner();
    let folder = Folder::new(&folder_path)
        .map_err(AppError::root)?;
    let child_folder = folder.join(&form.folder_name)
        .map_err(|k| AppError::new(k, ForwardTo::FileDetail(folder.clone(), file_name.clone())))?;
    folder.move_entity(&file_name, &child_folder)
        .map_err(|k| AppError::new(k, ForwardTo::FileDetail(folder, file_name.clone())))?;
    FlashMessage::success(format!("moved file '{}' to '{}'", &file_name, child_folder.name())).send();
    Ok(forward::to(&ForwardTo::FileDetail(child_folder, file_name)))
}

pub async fn move_file_up(path: web::Path<(String,String)>) -> Result<HttpResponse, AppError> {
    let (folder_path, file_name) = path.into_inner();
    let folder = Folder::new(&folder_path)
        .map_err(AppError::root)?;
    if folder.is_root() {
        return Err(AppError::new(AppErrorKind::CannotDeleteRoot, ForwardTo::FileDetail(folder, file_name)));
    }
    let parent = folder.parent().unwrap_or_default();
    folder.move_entity(&file_name, &parent)
        .map_err(|k| AppError::new(k, ForwardTo::FileDetail(folder, file_name.clone())))?;
    FlashMessage::success(format!("moved file '{}' up a folder", file_name)).send();
    Ok(forward::to(&ForwardTo::FileDetail(parent, file_name)))
}

pub async fn move_entities(folder_path: web::Path<String>, form: web::Form<MoveEntitiesIntoFormData>) -> Result<HttpResponse, AppError> {
    if form.folder_name == PARENT_OPTION {
        return move_entities_up(folder_path, form).await;
    }
    let folder = Folder::new(&folder_path.into_inner())
        .map_err(AppError::root)?;
    let new_folder = folder.join(&form.folder_name)
        .map_err(|k| AppError::new(k, ForwardTo::Folder(folder.clone())))?;
    let selected_entities = form.selected_folders.split("/").chain(form.selected_files.split("/"))
        .filter_map(|e| { if e.len() == 0 { None } else { folder.join(e).ok() }});
    let mut count = 0;
    for entity in selected_entities {
        if entity.name() == new_folder.name() {
            FlashMessage::error("Cannot move a folder into itself").send();
            continue;
        }
        match folder.move_entity(entity.name(), &new_folder) {
            Ok(()) => count = count + 1,
            Err(e) => {
                FlashMessage::error(e.to_string()).send();
            }
        }
    }
    if count > 0 {
        FlashMessage::success(format!("moved {} files/folders into '{}'", count, new_folder.name())).send();
    }
    Ok(forward::to(&ForwardTo::Folder(folder)))
}

pub async fn move_entities_up(folder_path: web::Path<String>, form: web::Form<MoveEntitiesIntoFormData>) -> Result<HttpResponse, AppError> {
    let folder = Folder::new(&folder_path.into_inner())
        .map_err(AppError::root)?;
    if folder.is_root() {
        return Err(AppError::new(AppErrorKind::CannotGetParentOfRoot, ForwardTo::Folder(folder)));
    }
    let selected_entities = form.selected_folders.split("/").chain(form.selected_files.split("/"))
        .filter_map(|e| { if e.len() == 0 { None } else { folder.join(e).ok() }});
    let parent = folder.parent().unwrap_or_default();
    let mut count = 0;
    for entity in selected_entities {
        match folder.move_entity(entity.name(), &parent) {
            Ok(()) => count = count + 1,
            Err(e) => {
                FlashMessage::error(e.to_string()).send();
            }
        }
    }
    if count > 0 {
        FlashMessage::success(format!("moved {} files/folders up a folder", count)).send();
    }
    Ok(forward::to(&ForwardTo::Folder(folder)))
}

pub async fn unzip_file(path: web::Path<(String,String)>) -> Result<HttpResponse, AppError> {
    let (folder_path, file_name) = path.into_inner();
    let folder = Folder::new(&folder_path)
        .map_err(AppError::root)?;
    folder.unzip_file(&file_name).await
        .map_err(|k| AppError::new(k, ForwardTo::FileDetail(folder.clone(), file_name.clone())))?;
    FlashMessage::success(format!("unzipped file '{}'", file_name)).send();
    Ok(forward::to(&ForwardTo::Folder(folder)))
}

pub async fn remove_file(path: web::Path<(String,String)>) -> Result<HttpResponse, AppError> {
    let (folder_path, file_name) = path.into_inner();
    let folder = Folder::new(&folder_path)
        .map_err(AppError::root)?;
    folder.remove_file(&file_name)
        .map_err(|k| AppError::new(k, ForwardTo::FileDetail(folder.clone(), file_name.clone())))?;
    FlashMessage::success(format!("removed file '{}'", file_name)).send();
    Ok(forward::to(&ForwardTo::Folder(folder)))
}

pub async fn remove_entities(folder_path: web::Path<String>, form: web::Form<RemoveEntitiesFormData>) -> Result<HttpResponse, AppError> {
    let folder = Folder::new(&folder_path.into_inner())
        .map_err(AppError::root)?;
    let selected_folders = form.selected_folders.split("/")
        .filter_map(|e| { if e.len() == 0 { None } else { folder.join(e).ok() }});
    let mut count = 0;
    for remove_folder in selected_folders {
        match remove_folder.remove() {
            Ok(()) => count = count + 1,
            Err(e) => FlashMessage::error(e.to_string()).send()
        }
    }
    let selected_files = form.selected_files.split("/")
        .filter_map(|e| { if e.len() == 0 { None } else { folder.join(e).ok() }});
    for remove_file in selected_files {
        match folder.remove_file(remove_file.name()) {
            Ok(()) => count = count + 1,
            Err(e) => FlashMessage::error(e.to_string()).send()
        }
    }
    if count > 0 {
        FlashMessage::success(format!("removed {} files/folders", count)).send();
    }
    Ok(forward::to(&ForwardTo::Folder(folder)))
}
