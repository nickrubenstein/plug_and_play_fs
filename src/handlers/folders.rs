use actix_web::{web, HttpResponse, http};
use actix_web_flash_messages::{FlashMessage};
use serde::Deserialize;

use crate::models::folder::Folder;

#[derive(Deserialize)]
pub struct NewFolderFormData {
    folder_name: String,
}

pub async fn add_folder(folder_path: web::Path<String>, form: web::Form<NewFolderFormData>) -> HttpResponse {
    let folder = match Folder::new(folder_path.into_inner()) {
        Ok(file_path) => file_path,
        Err(e) => {
            FlashMessage::error(e.to_string()).send();
            return HttpResponse::TemporaryRedirect().append_header((http::header::LOCATION, "/")).finish();
        }
    };
    match folder.create_dir(&form.folder_name) {
        Ok(()) => FlashMessage::success(format!("created folder '{}'", form.folder_name)).send(),
        Err(e) => FlashMessage::error(e.to_string()).send()
    }
    HttpResponse::SeeOther().append_header((http::header::LOCATION, format!("/fs/{}/files", folder.uri_path()))).finish()
}