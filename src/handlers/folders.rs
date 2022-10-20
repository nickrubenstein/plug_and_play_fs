use actix_session::Session;
use actix_web::{web, HttpResponse};
use actix_web_flash_messages::{FlashMessage, IncomingFlashMessages};
use handlebars::Handlebars;
use serde::Deserialize;
use serde_json::json;

use crate::{models::{folder::Folder, user::User}, util::{error::{AppError, AppErrorKind}, forward::{ForwardTo, self}}};

const PARENT_OPTION: &str = "|Move to parent folder|";

#[derive(Deserialize)]
pub struct NewFolderFormData {
    folder_name: String,
}

#[derive(Deserialize)]
pub struct RenameFolderFormData {
    folder_name: String,
}

#[derive(Deserialize)]
pub struct MoveFolderIntoFormData {
    folder_name: String,
}

pub async fn get_folder_detail(
    folder_path: web::Path<String>,
    session: Session,
    hb: web::Data<Handlebars<'_>>,
    flashes: IncomingFlashMessages
) -> Result<HttpResponse, AppError> {
    let folder = Folder::new(&folder_path.into_inner())
        .map_err(AppError::root)?;
    let user = User::get(session)
        .map_err(|k| AppError::new(k, ForwardTo::FolderDetail(folder.clone())))?;
    let details = folder.details()
        .map_err(|k| AppError::new(k, ForwardTo::Folder(folder.parent().unwrap_or_default())))?;
    let folders = match folder.parent() {
        Ok(parent) => match parent.entity_list(true) {
            Ok(list) => list.0,
            Err(_) => Vec::new()
        },
        Err(_) => Vec::new()
    };
    let flashes: Vec<(String,String)> = flashes.iter().map(|f| {(f.level().to_string(), f.content().to_string())}).collect();
    let crumbs: Vec<(String,String)> = folder.ancestors(true).iter().map(|a| { (a.to_string(), a.name().to_owned())}).collect();
    let data = json! ({
        "title": "FS",
        "user": user,
        "flashes": flashes,
        "folder_path": folder.to_string(),
        "crumbs": crumbs,
        "folders": folders,
        "details": details,
        "parent_option": PARENT_OPTION.clone()
    });
    let body = hb.render("folder-detail", &data).unwrap();
    Ok(HttpResponse::Ok().body(body))
}

pub async fn add_folder(folder_path: web::Path<String>, form: web::Form<NewFolderFormData>) -> Result<HttpResponse, AppError> {
    let folder = Folder::new(&folder_path.into_inner())
        .map_err(AppError::root)?;
    folder.create_dir(&form.folder_name)
        .map_err(|k| AppError::new(k, ForwardTo::Folder(folder.clone())))?;
    FlashMessage::success(format!("created folder '{}'", form.folder_name)).send();
    Ok(forward::to(ForwardTo::Folder(folder)))
}

pub async fn rename_folder(folder_path: web::Path<String>, form: web::Form<RenameFolderFormData>) -> Result<HttpResponse, AppError> {
    let mut folder = Folder::new(&folder_path.into_inner())
        .map_err(AppError::root)?;
    let old_folder = folder.clone();
    folder.rename(&form.folder_name)
        .map_err(|k| AppError::new(k, ForwardTo::FolderDetail(folder.clone())))?;
    FlashMessage::success(format!("renamed folder '{}' to '{}'", old_folder.name(), form.folder_name)).send();
    Ok(forward::to(ForwardTo::FolderDetail(folder.clone())))
}

pub async fn move_folder(folder_path: web::Path<String>, form: web::Form<MoveFolderIntoFormData>) -> Result<HttpResponse, AppError> {
    if form.folder_name == PARENT_OPTION {
        return move_folder_up(folder_path).await;
    }
    let folder = Folder::new(&folder_path.into_inner())
        .map_err(AppError::root)?;
    if folder.is_root() {
        return Err(AppError::new(AppErrorKind::CannotMoveRoot, ForwardTo::FolderDetail(folder)));
    }
    let parent_folder = folder.parent().unwrap_or_default();
    let sibling_folder = parent_folder.join(&form.folder_name)
        .map_err(|k| AppError::new(k, ForwardTo::FolderDetail(folder.clone())))?;
    if folder.name() == sibling_folder.name() {
        return Err(AppError::new(AppErrorKind::CannotMoveFolderIntoItself, ForwardTo::FolderDetail(folder)));
    }
    parent_folder.move_entity(folder.name(), &sibling_folder) 
        .map_err(|k| AppError::new(k, ForwardTo::FolderDetail(folder.clone())))?;
    FlashMessage::success(format!("moved folder '{}' into '{}'", folder.name(), sibling_folder.name())).send();
    Ok(forward::to(ForwardTo::FolderDetail(sibling_folder.join(folder.name()).unwrap_or_default())))
}

pub async fn move_folder_up(folder_path: web::Path<String>) -> Result<HttpResponse, AppError> {
    let folder = Folder::new(&folder_path.into_inner())
        .map_err(AppError::root)?;
    let parent_folder = folder.parent().unwrap_or_default();
    if parent_folder.is_root() {
        return Err(AppError::new(AppErrorKind::CannotMoveAboveRoot, ForwardTo::FolderDetail(folder)));
    }
    let grandparent_folder = parent_folder.parent().unwrap_or_default();
    parent_folder.move_entity(folder.name(), &grandparent_folder)
        .map_err(|k| AppError::new(k, ForwardTo::FolderDetail(folder.clone())))?;
    FlashMessage::success(format!("moved folder '{}' up a folder", folder.name())).send();
    Ok(forward::to(ForwardTo::FolderDetail(grandparent_folder.join(folder.name()).unwrap_or_default())))
}

pub async fn zip_folder(folder_path: web::Path<String>) -> Result<HttpResponse, AppError> {
    let folder = Folder::new(&folder_path.into_inner())
        .map_err(AppError::root)?;
    if folder.is_root() {
        return Err(AppError::new(AppErrorKind::CannotZipRoot, ForwardTo::FolderDetail(folder)));
    }
    folder.zip().await
        .map_err(|k| AppError::new(k, ForwardTo::FolderDetail(folder.clone())))?;
    FlashMessage::success(format!("zipped folder '{}'", folder.name())).send();
    Ok(forward::to(ForwardTo::Folder(folder.parent().unwrap_or_default())))
}

pub async fn remove_folder(folder_path: web::Path<String>) -> Result<HttpResponse, AppError> {
    let folder = Folder::new(&folder_path.into_inner())
        .map_err(AppError::root)?;
    let old_folder_name = folder.name();
    let parent = folder.parent().unwrap_or_default();
    folder.remove()
        .map_err(|k| AppError::new(k, ForwardTo::FolderDetail(folder.clone())))?;
    FlashMessage::success(format!("removed folder '{}'", old_folder_name)).send();
    Ok(forward::to(ForwardTo::FolderDetail(parent)))
}
