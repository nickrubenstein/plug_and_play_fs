use std::{rc::Rc, fmt::{Formatter, Debug, Result}};
use actix_session::Session;
use actix_web::{HttpResponse, http};

use crate::models::folder::Folder;

#[derive(Clone)]
pub enum ForwardTo {
    Root,
    Login,
    Timelapse,
    LoginRedirect(Rc<ForwardTo>, Session),
    Folder(Folder),
    FolderDetail(Folder),
    FileDetail(Folder, String)
}

impl Debug for ForwardTo {
    fn fmt(&self, fmt: &mut Formatter) -> Result {
        fmt.debug_struct("ForwardTo").finish()
    }
}

pub fn to(forward: ForwardTo) -> HttpResponse {
    to_string(&location(&forward))
}

pub fn to_string(forward: &str) -> HttpResponse {
    HttpResponse::SeeOther().insert_header((http::header::LOCATION, forward)).finish()
}

pub fn location(forward: &ForwardTo) -> String {
    match forward {
        ForwardTo::Root => {
            "/fs/root/files".to_string()
        },
        ForwardTo::Login => {
            "/login".to_string()
        },
        ForwardTo::Timelapse => {
            "/timelapse".to_string()
        },
        ForwardTo::LoginRedirect(redirect, session) => {
            match session.insert("redirect", location(&redirect)) {
                Ok(()) => (),
                Err(err) => {
                    log::error!("ForwardTo::LoginRedirect SessionInsertError: {}", err);
                }
            }
            location(&ForwardTo::Login)
        },
        ForwardTo::Folder(folder) => {
            format!("/fs/{}/files", folder.to_string())
        },
        ForwardTo::FolderDetail(folder) => {
            format!("/fs/{}", folder.to_string())
        },
        ForwardTo::FileDetail(folder, file_name) => {
            format!("/fs/{}/files/{}", folder.to_string(), file_name)
        }
    }
}
