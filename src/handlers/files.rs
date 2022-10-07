use actix_multipart::Multipart;
use actix_web::{web, HttpResponse, 
    http::{self, header::{ContentDisposition, DispositionType, DispositionParam}}
};
use actix_web_flash_messages::{FlashMessage, IncomingFlashMessages};
use serde::Deserialize;
use serde_json::json;
use handlebars::Handlebars;

use crate::models::folder::Folder;


#[derive(Deserialize)]
pub struct RenameFileFormData {
    file_name: String,
}

#[derive(Deserialize)]
pub struct MoveFileDownFormData {
    folder_name: String,
}

pub async fn get_files(folder_path: web::Path<String>, hb: web::Data<Handlebars<'_>>, flashes: IncomingFlashMessages) -> HttpResponse {
    let folder = match Folder::new(&folder_path.into_inner()) {
        Ok(file_path) => file_path,
        Err(e) => {
            FlashMessage::error(e.to_string()).send();
            return HttpResponse::SeeOther().insert_header((http::header::LOCATION, "/")).finish();
        }
    };
    let entities = match folder.entity_list(false) {
        Ok(list) => list,
        Err(e) => {
            FlashMessage::error(e.to_string()).send();
            return HttpResponse::SeeOther().insert_header((http::header::LOCATION, format!("/fs/{}/files", folder.parent().unwrap_or_default().to_string()))).finish();
        }
    };
    let flashes: Vec<(String,String)> = flashes.iter().map(|f| {(f.level().to_string(), f.content().to_string())}).collect();
    let crumbs: Vec<(String,String)> = folder.ancestors(true).iter().map(|a| { (a.to_string(), a.name().to_owned())}).collect();
    let data = json! ({
        "title": "FS",
        "flashes": flashes,
        "folder_path": folder.to_string(),
        "crumbs": crumbs,
        "entities": entities
    });
    let body = hb.render("files", &data).unwrap();
    HttpResponse::Ok().body(body)
}

pub async fn get_file_detail(path: web::Path<(String,String)>, hb: web::Data<Handlebars<'_>>, flashes: IncomingFlashMessages) -> HttpResponse {
    let (folder_path, file_name) = path.into_inner();
    let folder = match Folder::new(&folder_path) {
        Ok(file_path) => file_path,
        Err(e) => {
            FlashMessage::error(e.to_string()).send();
            return HttpResponse::SeeOther().insert_header((http::header::LOCATION, "/")).finish();
        }
    };
    let details = match folder.file_details(file_name.clone()) {
        Ok(list) => list,
        Err(e) => {
            FlashMessage::error(e.to_string()).send();
            return HttpResponse::SeeOther().insert_header((http::header::LOCATION, format!("/fs/{}/files", folder.to_string()))).finish();
        }
    };
    let folders = match folder.entity_list(true) {
        Ok(list) => list,
        Err(_) => Vec::new()
    };
    let flashes: Vec<(String,String)> = flashes.iter().map(|f| {(f.level().to_string(), f.content().to_string())}).collect();
    let crumbs: Vec<(String,String)> = folder.ancestors(true).iter().map(|a| { (a.to_string(), a.name().to_owned())}).collect();
    let data = json! ({
        "title": "FS",
        "flashes": flashes,
        "folder_path": folder.to_string(),
        "crumbs": crumbs,
        "file_name": file_name,
        "details": details,
        "folders": folders
    });
    let body = hb.render("file-detail", &data).unwrap();
    HttpResponse::Ok().body(body)
}

pub async fn upload_file(folder_path: web::Path<String>, payload: Multipart) -> HttpResponse {
    let folder = match Folder::new(&folder_path.into_inner()) {
        Ok(file_path) => file_path,
        Err(e) => {
            FlashMessage::error(e.to_string()).send();
            return HttpResponse::SeeOther().append_header((http::header::LOCATION, "/")).finish();
        }
    };
    
    match folder.upload_file(payload).await {
        Ok(file_names) if file_names.len() == 1 => FlashMessage::success(format!("uploaded file '{}'", file_names[0])).send(),
        Ok(file_names) if file_names.len() > 1 => FlashMessage::success(format!("uploaded {} files", file_names.len())).send(),
        Ok(_) => FlashMessage::error("No files were uploaded").send(),
        Err(e) => FlashMessage::error(e.to_string()).send()
    }
    HttpResponse::SeeOther().append_header((http::header::LOCATION, format!("/fs/{}/files", folder.to_string()))).finish()
}

pub async fn download_file(path: web::Path<(String,String)>) -> HttpResponse {
    let (folder_path, file_name) = path.into_inner();
    let folder = match Folder::new(&folder_path) {
        Ok(file_path) => file_path,
        Err(e) => {
            FlashMessage::error(e.to_string()).send();
            return HttpResponse::SeeOther().append_header((http::header::LOCATION, "/")).finish();
        }
    };
    
    let file_content = match folder.read_file(file_name.clone()).await {
        Ok(content) => content,
        Err(e) => {
            FlashMessage::error(e.to_string()).send();
            return HttpResponse::SeeOther().append_header((http::header::LOCATION, format!("/fs/{}/files/{}", folder.to_string(), file_name))).finish()
        }
    };
    let content_disposition = ContentDisposition {
        disposition: DispositionType::Attachment,
        parameters: vec![DispositionParam::Filename(file_name)],
    };
    HttpResponse::Ok().append_header((http::header::CONTENT_DISPOSITION, content_disposition)).body(file_content)
}

pub async fn rename_file(path: web::Path<(String,String)>, form: web::Form<RenameFileFormData>) -> HttpResponse {
    let (folder_path, file_name) = path.into_inner();
    let folder = match Folder::new(&folder_path) {
        Ok(file_path) => file_path,
        Err(e) => {
            FlashMessage::error(e.to_string()).send();
            return HttpResponse::SeeOther().append_header((http::header::LOCATION, "/")).finish();
        }
    };
    
    match folder.rename_file(file_name.clone(), form.file_name.clone()) {
        Ok(()) => FlashMessage::success(format!("renamed file '{}' to '{}'", file_name, form.file_name)).send(),
        Err(e) => {
            FlashMessage::error(e.to_string()).send();
            return HttpResponse::SeeOther().append_header((http::header::LOCATION, format!("/fs/{}/files/{}", folder.to_string(), file_name))).finish()
        }
    }
    HttpResponse::SeeOther().append_header((http::header::LOCATION, format!("/fs/{}/files/{}", folder.to_string(), form.file_name))).finish()
}

pub async fn remove_file(path: web::Path<(String,String)>) -> HttpResponse {
    let (folder_path, file_name) = path.into_inner();
    let folder = match Folder::new(&folder_path) {
        Ok(file_path) => file_path,
        Err(e) => {
            FlashMessage::error(e.to_string()).send();
            return HttpResponse::SeeOther().append_header((http::header::LOCATION, "/")).finish();
        }
    };
    
    match folder.remove_file(file_name.clone()) {
        Ok(()) => FlashMessage::success(format!("removed file '{}'", file_name)).send(),
        Err(e) => FlashMessage::error(e.to_string()).send()
    }
    HttpResponse::SeeOther().append_header((http::header::LOCATION, format!("/fs/{}/files", folder.to_string()))).finish()
}

pub async fn move_file_up(path: web::Path<(String,String)>) -> HttpResponse {
    let (folder_path, file_name) = path.into_inner();
    let folder = match Folder::new(&folder_path) {
        Ok(file_path) => file_path,
        Err(e) => {
            FlashMessage::error(e.to_string()).send();
            return HttpResponse::SeeOther().append_header((http::header::LOCATION, "/")).finish();
        }
    };
    if folder.is_root() {
        FlashMessage::error("Cannot move file above root").send();
        return HttpResponse::SeeOther().append_header((http::header::LOCATION, format!("/fs/{}/files/{}", folder.to_string(), file_name))).finish();
    }
    let parent = folder.parent().unwrap_or_default();
    match folder.move_entity(&file_name, &parent) {
        Ok(()) => FlashMessage::success(format!("moved file '{}' up a folder", file_name)).send(),
        Err(e) => FlashMessage::error(e.to_string()).send()
    }
    HttpResponse::SeeOther().append_header((http::header::LOCATION, format!("/fs/{}/files/{}", parent.to_string(), file_name))).finish()
}

pub async fn move_file_into(path: web::Path<(String,String)>, form: web::Form<MoveFileDownFormData>) -> HttpResponse {
    let (folder_path, file_name) = path.into_inner();
    let folder = match Folder::new(&folder_path) {
        Ok(file_path) => file_path,
        Err(e) => {
            FlashMessage::error(e.to_string()).send();
            return HttpResponse::SeeOther().append_header((http::header::LOCATION, "/")).finish();
        }
    };
    let child_folder = match folder.join(&form.folder_name) {
        Ok(folder) => folder,
        Err(e) => {
            FlashMessage::error(e.to_string()).send();
            return HttpResponse::SeeOther().append_header((http::header::LOCATION, format!("/fs/{}/files/{}", folder.to_string(), &file_name))).finish();
        }
    };
    match folder.move_entity(&file_name, &child_folder) {
        Ok(()) => FlashMessage::success(format!("moved file '{}' to '{}'", &file_name, child_folder.name())).send(),
        Err(e) => {
            FlashMessage::error(e.to_string()).send();
            return HttpResponse::SeeOther().append_header((http::header::LOCATION, format!("/fs/{}/files/{}", folder.to_string(), &file_name))).finish()
        }
    }
    HttpResponse::SeeOther().append_header((http::header::LOCATION, format!("/fs/{}/files/{}", child_folder.to_string(), &file_name))).finish()
}

pub async fn unzip_file(path: web::Path<(String,String)>) -> HttpResponse {
    let (folder_path, file_name) = path.into_inner();
    let folder = match Folder::new(&folder_path) {
        Ok(file_path) => file_path,
        Err(e) => {
            FlashMessage::error(e.to_string()).send();
            return HttpResponse::SeeOther().append_header((http::header::LOCATION, "/")).finish();
        }
    };
    
    match folder.unzip_file(&file_name).await {
        Ok(()) => FlashMessage::success(format!("extracted file '{}'", file_name)).send(),
        Err(e) => FlashMessage::error(e.to_string()).send()
    }
    HttpResponse::SeeOther().append_header((http::header::LOCATION, format!("/fs/{}/files", folder.to_string()))).finish()
}
