use actix_multipart::Multipart;
use actix_web::{web, HttpResponse, http};
use actix_web_flash_messages::{FlashMessage, IncomingFlashMessages};
use serde::Deserialize;
use serde_json::json;
use handlebars::Handlebars;

use crate::models::folder::Folder;


#[derive(Deserialize)]
pub struct RenameFileFormData {
    file_name: String,
}

pub async fn get_files(folder_path: web::Path<String>, hb: web::Data<Handlebars<'_>>, flashes: IncomingFlashMessages) -> HttpResponse {
    let folder = match Folder::new(folder_path.into_inner()) {
        Ok(file_path) => file_path,
        Err(e) => {
            FlashMessage::error(e.to_string()).send();
            return HttpResponse::TemporaryRedirect().insert_header((http::header::LOCATION, "/")).finish();
        }
    };
    let files = match folder.file_list() {
        Ok(list) => list,
        Err(_e) => return HttpResponse::InternalServerError().finish()
    };
    let flashes: Vec<(String,String)> = flashes.iter().map(|f| {(f.level().to_string(), f.content().to_string())}).collect();
    log::info!("{:?}", flashes);
    let data = json! ({
        "title": "FS",
        "flashes": flashes,
        "folder_path": folder.uri_path(),
        "paths": folder.path_list_aggrigate(),
        "items": files
    });
    let body = hb.render("files", &data).unwrap();
    HttpResponse::Ok().body(body)
}

pub async fn get_file_detail(path: web::Path<(String,String)>, hb: web::Data<Handlebars<'_>>, flashes: IncomingFlashMessages) -> HttpResponse {
    let (folder_path, file_name) = path.into_inner();
    let folder = match Folder::new(folder_path) {
        Ok(file_path) => file_path,
        Err(e) => {
            FlashMessage::error(e.to_string()).send();
            return HttpResponse::TemporaryRedirect().insert_header((http::header::LOCATION, "/")).finish();
        }
    };
    let details = match folder.file_details(file_name.clone()) {
        Ok(list) => list,
        Err(_e) => return HttpResponse::InternalServerError().finish()
    };
    let flashes: Vec<(String,String)> = flashes.iter().map(|f| {(f.level().to_string(), f.content().to_string())}).collect();
    log::info!("{:?}", flashes);
    let data = json! ({
        "title": "FS",
        "flashes": flashes,
        "folder_path": folder.uri_path(),
        "paths": folder.path_list_aggrigate(),
        "file_name": file_name,
        "items": details
    });
    let body = hb.render("file-detail", &data).unwrap();
    HttpResponse::Ok().body(body)
}

pub async fn add_file(folder_path: web::Path<String>, payload: Multipart) -> HttpResponse {
    let folder = match Folder::new(folder_path.into_inner()) {
        Ok(file_path) => file_path,
        Err(e) => {
            FlashMessage::error(e.to_string()).send();
            return HttpResponse::TemporaryRedirect().append_header((http::header::LOCATION, "/")).finish();
        }
    };
    
    match folder.upload_file(payload).await {
        Ok(file_names) if file_names.len() == 1 => FlashMessage::success(format!("uploaded file '{}'", file_names[0])).send(),
        Ok(file_names) if file_names.len() > 1 => FlashMessage::success(format!("uploaded {} files", file_names.len())).send(),
        Ok(_) => FlashMessage::error("No files were uploaded").send(),
        Err(e) => FlashMessage::error(e.to_string()).send()
    }
    HttpResponse::SeeOther().append_header((http::header::LOCATION, format!("/fs/{}/files", folder.uri_path()))).finish()
}

pub async fn rename_file(path: web::Path<(String,String)>, form: web::Form<RenameFileFormData>) -> HttpResponse {
    let (folder_path, file_name) = path.into_inner();
    let folder = match Folder::new(folder_path) {
        Ok(file_path) => file_path,
        Err(e) => {
            FlashMessage::error(e.to_string()).send();
            return HttpResponse::TemporaryRedirect().append_header((http::header::LOCATION, "/")).finish();
        }
    };
    
    match folder.rename_file(file_name.clone(), form.file_name.clone()) {
        Ok(()) => FlashMessage::success(format!("renamed file '{}' to '{}'", file_name, form.file_name)).send(),
        Err(e) => {
            FlashMessage::error(e.to_string()).send();
            return HttpResponse::SeeOther().append_header((http::header::LOCATION, format!("/fs/{}/files/{}", folder.uri_path(), file_name))).finish()
        }
    }
    HttpResponse::SeeOther().append_header((http::header::LOCATION, format!("/fs/{}/files/{}", folder.uri_path(), form.file_name))).finish()
}

pub async fn remove_file(path: web::Path<(String,String)>) -> HttpResponse {
    let (folder_path, file_name) = path.into_inner();
    let folder = match Folder::new(folder_path) {
        Ok(file_path) => file_path,
        Err(e) => {
            FlashMessage::error(e.to_string()).send();
            return HttpResponse::TemporaryRedirect().append_header((http::header::LOCATION, "/")).finish();
        }
    };
    
    match folder.remove_file(file_name.clone()) {
        Ok(()) => FlashMessage::success(format!("removed file '{}'", file_name)).send(),
        Err(e) => FlashMessage::error(e.to_string()).send()
    }
    HttpResponse::SeeOther().append_header((http::header::LOCATION, format!("/fs/{}/files", folder.uri_path()))).finish()
}

// #[post("/", data = "<form>")]
// pub async fn post_root_file(form: Form<UploadFile<'_>>) -> Result<Flash<Redirect>, Flash<Redirect>> {
//     post_file(String::new(), form).await
// }

// #[post("/<path>", data = "<form>")]
// pub async fn post_file(path: String, form: Form<UploadFile<'_>>) -> Result<Flash<Redirect>, Flash<Redirect>> {
//     let path = match FilePath::new(path) {
//         Ok(file_path) => file_path,
//         Err(e) => return Err(Flash::error(Redirect::to("/files"), e.to_string()))
//     };
//     let redirect = Redirect::to(uri!("/files", get_folder(path.uri_path())));
//     let mut upload = form.into_inner();
//     if let (Some(name), Some(content_type)) = (upload.file.name(), upload.file.content_type()) {
//         let file_name = format!("{}.{}", name, content_type.extension().unwrap());
//         println!("path =>  {}", path.append_to_file_path(&file_name));
//         match upload.file.persist_to(path.append_to_file_path(&file_name)).await {
//             Ok(()) => return Ok(Flash::success(redirect, format!("Added file '{}'", file_name))),
//             Err(e) => return Err(Flash::error(redirect, e.to_string()))
//         }
//     }
//     else {
//         return Err(Flash::error(redirect, "Could not get file name or extension"))
//     }
// }

// #[get("/")]
// pub async fn get_root_folder() -> Result<Template, std::io::Error> {
//     get_folder(String::new()).await
// }

// #[get("/<path>")]
// pub async fn get_folder(path: String) -> Result<Template, std::io::Error> {
//     let path = match FilePath::new(path) {
//         Ok(file_path) => file_path,
//         Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, e))
//     };
//     let mut path_dir = fs::read_dir(path.file_path()).await?;
//     let mut files = Vec::new();
//     while let Some(entry) = path_dir.next_entry().await? {
//         if let Ok(name) = entry.file_name().into_string() {
//             let file_type = entry.file_type().await?;
//             files.push(context! {
//                 path: format!("{}/{}", "/files", path.append_to_uri_path(&name)),
//                 name: name,
//                 is_folder: file_type.is_dir()
//             });
//         }
//     }
//     Ok(Template::render("files", context! {
//         title: "FS",
//         path: path.uri_path(),
//         paths: path.path_list_aggrigate(),
//         items: files,
//     }))
// }
