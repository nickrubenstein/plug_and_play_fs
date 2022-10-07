use actix_web::{web, HttpResponse, http};
use actix_web_flash_messages::{FlashMessage, IncomingFlashMessages};
use handlebars::Handlebars;
use serde::Deserialize;
use serde_json::json;

use crate::models::folder::Folder;

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

pub async fn get_folder_detail(folder_path: web::Path<String>, hb: web::Data<Handlebars<'_>>, flashes: IncomingFlashMessages) -> HttpResponse {
    let folder = match Folder::new(&folder_path.into_inner()) {
        Ok(file_path) => file_path,
        Err(e) => {
            FlashMessage::error(e.to_string()).send();
            return HttpResponse::SeeOther().insert_header((http::header::LOCATION, "/")).finish();
        }
    };
    let details = match folder.details() {
        Ok(list) => list,
        Err(e) => {
            FlashMessage::error(e.to_string()).send();
            return HttpResponse::SeeOther().insert_header((http::header::LOCATION, format!("/fs/{}/files", folder.parent().unwrap_or_default().to_string()))).finish();
        }
    };
    let folders = match folder.parent() {
        Ok(parent) => match parent.entity_list(true) {
            Ok(list) => list,
            Err(_) => Vec::new()
        },
        Err(_) => Vec::new()
    };
    let flashes: Vec<(String,String)> = flashes.iter().map(|f| {(f.level().to_string(), f.content().to_string())}).collect();
    let crumbs: Vec<(String,String)> = folder.ancestors(true).iter().map(|a| { (a.to_string(), a.name().to_owned())}).collect();
    let data = json! ({
        "title": "FS",
        "flashes": flashes,
        "folder_path": folder.to_string(),
        "crumbs": crumbs,
        "folders": folders,
        "details": details
    });
    let body = hb.render("folder-detail", &data).unwrap();
    HttpResponse::Ok().body(body)
}

pub async fn add_folder(folder_path: web::Path<String>, form: web::Form<NewFolderFormData>) -> HttpResponse {
    let folder = match Folder::new(&folder_path.into_inner()) {
        Ok(file_path) => file_path,
        Err(e) => {
            FlashMessage::error(e.to_string()).send();
            return HttpResponse::SeeOther().append_header((http::header::LOCATION, "/")).finish();
        }
    };
    match folder.create_dir(&form.folder_name) {
        Ok(()) => FlashMessage::success(format!("created folder '{}'", form.folder_name)).send(),
        Err(e) => FlashMessage::error(e.to_string()).send()
    }
    HttpResponse::SeeOther().append_header((http::header::LOCATION, format!("/fs/{}/files", folder.to_string()))).finish()
}

pub async fn rename_folder(folder_path: web::Path<String>, form: web::Form<RenameFolderFormData>) -> HttpResponse {
    let mut folder = match Folder::new(&folder_path.into_inner()) {
        Ok(file_path) => file_path,
        Err(e) => {
            FlashMessage::error(e.to_string()).send();
            return HttpResponse::SeeOther().append_header((http::header::LOCATION, "/")).finish();
        }
    };
    let old_folder_name = folder.name().to_owned();
    match folder.rename(&form.folder_name) {
        Ok(()) => FlashMessage::success(format!("renamed folder '{}' to '{}'", old_folder_name, form.folder_name)).send(),
        Err(e) => {
            FlashMessage::error(e.to_string()).send();
            return HttpResponse::SeeOther().append_header((http::header::LOCATION, format!("/fs/{}", folder.to_string()))).finish()
        }
    }
    HttpResponse::SeeOther().append_header((http::header::LOCATION, format!("/fs/{}", folder.to_string()))).finish()
}

pub async fn move_folder_up(folder_path: web::Path<String>) -> HttpResponse {
    let folder = match Folder::new(&folder_path.into_inner()) {
        Ok(file_path) => file_path,
        Err(e) => {
            FlashMessage::error(e.to_string()).send();
            return HttpResponse::SeeOther().append_header((http::header::LOCATION, "/")).finish();
        }
    };
    
    let parent_folder = folder.parent().unwrap_or_default();
    if parent_folder.is_root() {
        FlashMessage::error("Cannot move folder above root").send();
        return HttpResponse::SeeOther().append_header((http::header::LOCATION, format!("/fs/{}", folder.to_string()))).finish();
    }
    let grandparent_folder = parent_folder.parent().unwrap_or_default();
    match parent_folder.move_entity(folder.name(), &grandparent_folder) {
        Ok(()) => FlashMessage::success(format!("moved folder '{}' up a folder", folder.name())).send(),
        Err(e) => {
            FlashMessage::error(e.to_string()).send();
            return HttpResponse::SeeOther().append_header((http::header::LOCATION, format!("/fs/{}", folder.to_string()))).finish()
        }
    }
    HttpResponse::SeeOther().append_header((http::header::LOCATION, format!("/fs/{}", grandparent_folder.join(folder.name()).unwrap_or_default().to_string()))).finish()
}

pub async fn move_folder_into(folder_path: web::Path<String>, form: web::Form<MoveFolderIntoFormData>) -> HttpResponse {
    let folder = match Folder::new(&folder_path.into_inner()) {
        Ok(file_path) => file_path,
        Err(e) => {
            FlashMessage::error(e.to_string()).send();
            return HttpResponse::SeeOther().append_header((http::header::LOCATION, "/")).finish();
        }
    };
    if folder.is_root() {
        FlashMessage::error("Cannot move root folder").send();
        return HttpResponse::SeeOther().append_header((http::header::LOCATION, format!("/fs/{}", folder.to_string()))).finish();
    }
    let parent_folder = folder.parent().unwrap_or_default();
    let sibling_folder = match parent_folder.join(&form.folder_name) {
        Ok(folder) => folder,
        Err(e) => {
            FlashMessage::error(e.to_string()).send();
            return HttpResponse::SeeOther().append_header((http::header::LOCATION, format!("/fs/{}", folder.to_string()))).finish();
        }
    };
    if folder.name() == sibling_folder.name() {
        FlashMessage::error("Cannot move folder into itself").send();
        return HttpResponse::SeeOther().append_header((http::header::LOCATION, format!("/fs/{}", folder.to_string()))).finish();
    }
    match parent_folder.move_entity(folder.name(), &sibling_folder) {
        Ok(()) => FlashMessage::success(format!("moved folder '{}' into '{}'", folder.name(), sibling_folder.name())).send(),
        Err(e) => {
            FlashMessage::error(e.to_string()).send();
            return HttpResponse::SeeOther().append_header((http::header::LOCATION, format!("/fs/{}", folder.to_string()))).finish()
        }
    }
    HttpResponse::SeeOther().append_header((http::header::LOCATION, format!("/fs/{}", sibling_folder.join(folder.name()).unwrap_or_default().to_string()))).finish()
}

pub async fn flatten_folder(folder_path: web::Path<String>) -> HttpResponse {
    let folder = match Folder::new(&folder_path.into_inner()) {
        Ok(file_path) => file_path,
        Err(e) => {
            FlashMessage::error(e.to_string()).send();
            return HttpResponse::SeeOther().append_header((http::header::LOCATION, "/")).finish();
        }
    };
    if folder.is_root() {
        FlashMessage::error("Cannot flatten root folder").send();
        return HttpResponse::SeeOther().append_header((http::header::LOCATION, format!("/fs/{}", folder.to_string()))).finish();
    }
    let parent = folder.parent().unwrap_or_default();
    match folder.entities(false) {
        Ok(entities) => {
            for (entity, _) in entities {
                match folder.move_entity(entity.name(), &parent) {
                    Ok(()) => (),
                    Err(e) => FlashMessage::error(e.to_string()).send()
                }
            }
        }
        Err(e) => FlashMessage::error(e.to_string()).send()
    }
    let old_folder_name = folder.name();
    match folder.remove() {
        Ok(()) => (),
        Err(e) => {
            FlashMessage::error(e.to_string()).send();
            return HttpResponse::SeeOther().append_header((http::header::LOCATION, format!("/fs/{}/files", folder.to_string()))).finish()
        }
    }
    FlashMessage::success(format!("flattened and removed folder '{}'", old_folder_name)).send();
    HttpResponse::SeeOther().append_header((http::header::LOCATION, format!("/fs/{}/files", parent.to_string()))).finish()
}

pub async fn remove_folder(folder_path: web::Path<String>) -> HttpResponse {
    let folder = match Folder::new(&folder_path.into_inner()) {
        Ok(file_path) => file_path,
        Err(e) => {
            FlashMessage::error(e.to_string()).send();
            return HttpResponse::SeeOther().append_header((http::header::LOCATION, "/")).finish();
        }
    };
    
    let old_folder_name = folder.name();
    let parent_folder = folder.parent().unwrap_or_default();
    match folder.remove() {
        Ok(()) => FlashMessage::success(format!("removed folder '{}'", old_folder_name)).send(),
        Err(e) => {
            FlashMessage::error(e.to_string()).send();
            return HttpResponse::SeeOther().append_header((http::header::LOCATION, format!("/fs/{}", folder.to_string()))).finish()
        }
    }
    HttpResponse::SeeOther().append_header((http::header::LOCATION, format!("/fs/{}/files", parent_folder.to_string()))).finish()
}

pub async fn zip_folder(folder_path: web::Path<String>) -> HttpResponse {
    let folder = match Folder::new(&folder_path.into_inner()) {
        Ok(file_path) => file_path,
        Err(e) => {
            FlashMessage::error(e.to_string()).send();
            return HttpResponse::SeeOther().append_header((http::header::LOCATION, "/")).finish();
        }
    };
    if folder.is_root() {
        FlashMessage::error("Cannot zip root folder").send();
        return HttpResponse::SeeOther().append_header((http::header::LOCATION, format!("/fs/{}", folder.to_string()))).finish();
    }
    match folder.zip().await {
        Ok(()) => FlashMessage::success(format!("zipped folder '{}'", folder.name())).send(),
        Err(e) => {
            FlashMessage::error(e.to_string()).send();
            return HttpResponse::SeeOther().append_header((http::header::LOCATION, format!("/fs/{}", folder.to_string()))).finish()
        }
    }
    HttpResponse::SeeOther().append_header((http::header::LOCATION, format!("/fs/{}/files", folder.parent().unwrap_or_default().to_string()))).finish()
}