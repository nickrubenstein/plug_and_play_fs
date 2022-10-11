use actix_web::{HttpResponse, http};

use crate::models::folder::Folder;

#[derive(Debug)]
pub enum ForwardTo {
    Root,
    Folder(Folder),
    FolderDetail(Folder),
    FileDetail(Folder, String)
}

pub fn to(forward: &ForwardTo) -> HttpResponse {
    match forward {
        ForwardTo::Root => {
            to_root()
        },
        ForwardTo::Folder(folder) => {
            to_folder(folder)
        },
        ForwardTo::FolderDetail(folder) => {
            to_folder_details(folder)
        },
        ForwardTo::FileDetail(folder, file_name) => {
            to_file_details(folder, file_name)
        }
    }
}

fn to_root() -> HttpResponse {
    HttpResponse::SeeOther().insert_header((http::header::LOCATION, "/")).finish()
}

fn to_folder(folder: &Folder) -> HttpResponse {
    HttpResponse::SeeOther().insert_header((http::header::LOCATION, format!("/fs/{}/files", folder.to_string()))).finish()
}

fn to_folder_details(folder: &Folder) -> HttpResponse {
    HttpResponse::SeeOther().insert_header((http::header::LOCATION, format!("/fs/{}", folder.to_string()))).finish()
}

fn to_file_details(folder: &Folder, file_name: &str) -> HttpResponse {
    HttpResponse::SeeOther().insert_header((http::header::LOCATION, format!("/fs/{}/files/{}", folder.to_string(), file_name))).finish()
}