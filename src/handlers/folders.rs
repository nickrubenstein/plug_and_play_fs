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

pub async fn get_folder_detail(folder_path: web::Path<String>, hb: web::Data<Handlebars<'_>>, flashes: IncomingFlashMessages) -> HttpResponse {
    let folder = match Folder::new(folder_path.into_inner()) {
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
            return HttpResponse::SeeOther().insert_header((http::header::LOCATION, format!("/fs/{}/files", folder.parent()))).finish();
        }
    };
    let flashes: Vec<(String,String)> = flashes.iter().map(|f| {(f.level().to_string(), f.content().to_string())}).collect();
    let data = json! ({
        "title": "FS",
        "flashes": flashes,
        "folder_path": folder.uri_path(),
        "crumbs": folder.path_list_aggrigate(),
        "items": details
    });
    let body = hb.render("folder-detail", &data).unwrap();
    HttpResponse::Ok().body(body)
}

pub async fn add_folder(folder_path: web::Path<String>, form: web::Form<NewFolderFormData>) -> HttpResponse {
    let folder = match Folder::new(folder_path.into_inner()) {
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
    HttpResponse::SeeOther().append_header((http::header::LOCATION, format!("/fs/{}/files", folder.uri_path()))).finish()
}

pub async fn rename_folder(folder_path: web::Path<String>, form: web::Form<RenameFolderFormData>) -> HttpResponse {
    let mut folder = match Folder::new(folder_path.into_inner()) {
        Ok(file_path) => file_path,
        Err(e) => {
            FlashMessage::error(e.to_string()).send();
            return HttpResponse::SeeOther().append_header((http::header::LOCATION, "/")).finish();
        }
    };
    let old_folder_name = folder.name();
    match folder.rename(&form.folder_name) {
        Ok(()) => FlashMessage::success(format!("renamed folder '{}' to '{}'", old_folder_name, form.folder_name)).send(),
        Err(e) => {
            FlashMessage::error(e.to_string()).send();
            return HttpResponse::SeeOther().append_header((http::header::LOCATION, format!("/fs/{}", folder.uri_path()))).finish()
        }
    }
    HttpResponse::SeeOther().append_header((http::header::LOCATION, format!("/fs/{}", folder.uri_path()))).finish()
}

pub async fn remove_folder(folder_path: web::Path<String>) -> HttpResponse {
    let folder = match Folder::new(folder_path.into_inner()) {
        Ok(file_path) => file_path,
        Err(e) => {
            FlashMessage::error(e.to_string()).send();
            return HttpResponse::SeeOther().append_header((http::header::LOCATION, "/")).finish();
        }
    };
    
    let old_folder_name = folder.name();
    let parent_folder = folder.parent();
    match folder.remove() {
        Ok(()) => FlashMessage::success(format!("removed folder '{}'", old_folder_name)).send(),
        Err(e) => {
            FlashMessage::error(e.to_string()).send();
            return HttpResponse::SeeOther().append_header((http::header::LOCATION, format!("/fs/{}", folder.uri_path()))).finish()
        }
    }
    HttpResponse::SeeOther().append_header((http::header::LOCATION, format!("/fs/{}/files", parent_folder))).finish()
}

pub async fn zip_folder(folder_path: web::Path<String>) -> HttpResponse {
    let folder = match Folder::new(folder_path.into_inner()) {
        Ok(file_path) => file_path,
        Err(e) => {
            FlashMessage::error(e.to_string()).send();
            return HttpResponse::SeeOther().append_header((http::header::LOCATION, "/")).finish();
        }
    };
    if folder.uri_path() == "root" {
        FlashMessage::error("Cannot zip root folder").send();
        return HttpResponse::SeeOther().append_header((http::header::LOCATION, format!("/fs/{}", folder.uri_path()))).finish();
    }
    match folder.zip().await {
        Ok(()) => FlashMessage::success(format!("zipped folder '{}'", folder.name())).send(),
        Err(e) => {
            FlashMessage::error(e.to_string()).send();
            return HttpResponse::SeeOther().append_header((http::header::LOCATION, format!("/fs/{}", folder.uri_path()))).finish()
        }
    }
    HttpResponse::SeeOther().append_header((http::header::LOCATION, format!("/fs/{}/files", folder.parent()))).finish()
}