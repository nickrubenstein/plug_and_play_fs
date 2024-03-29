use std::{rc::Rc, fmt::{Debug, Formatter, Result}};

use actix_session::Session;
use actix_web::{HttpResponse, ResponseError};
use actix_web_flash_messages::FlashMessage;

use crate::util::forward::{self, ForwardTo};

#[derive(Debug)]
pub struct AppError {
    kind: AppErrorKind,
    forward: ForwardTo
}

pub enum AppErrorKind {
    FolderPathInvalid,
    FolderPathNotFound,
    FileNotFound,
    CannotGetParentOfRoot,
    CannotRenameRoot,
    CannotDeleteRoot,
    CannotCopyRoot,
    CannotMoveRoot,
    CannotZipRoot,
    CannotMoveAboveRoot,
    CannotMoveFolderIntoItself,
    FailedToReadFile,
    FailedToZipFolder,
    FailedToUnzipFile,
    InvalidUserCredentials,
    Io(std::io::Error),
    Session(String, Option<Session>)
}

impl Debug for AppErrorKind {
    fn fmt(&self, fmt: &mut Formatter) -> Result {
        fmt.debug_struct("AppErrorKind").finish()
    }
}

fn match_error_kind(kind: &AppErrorKind, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match kind {
        AppErrorKind::FolderPathInvalid => write!(f, "folder_path must start with 'root' and cannot have '..'"),
        AppErrorKind::FolderPathNotFound => write!(f, "folder_path does not match a system folder path"),
        AppErrorKind::FileNotFound => write!(f, "file could not be found"),
        AppErrorKind::CannotGetParentOfRoot => write!(f, "cannot get the parent of root"),
        AppErrorKind::CannotRenameRoot => write!(f, "cannot rename root folder"),
        AppErrorKind::CannotCopyRoot => write!(f, "cannot copy root"),
        AppErrorKind::CannotMoveRoot => write!(f, "cannot move root"),
        AppErrorKind::CannotZipRoot => write!(f, "cannot zip root"),
        AppErrorKind::CannotDeleteRoot => write!(f, "cannot delete root folder"),
        AppErrorKind::CannotMoveAboveRoot => write!(f, "cannot move above root"),
        AppErrorKind::CannotMoveFolderIntoItself => write!(f, "cannot move folder into itself"),
        AppErrorKind::FailedToReadFile => write!(f, "failed to read file"),
        AppErrorKind::FailedToZipFolder => write!(f, "failed to zip folder"),
        AppErrorKind::FailedToUnzipFile => write!(f, "failed to unzip file"),
        AppErrorKind::InvalidUserCredentials => write!(f, "failed to login username or password invalid"),
        AppErrorKind::Io(io_err) => write!(f, "{}", io_err.to_string()),
        AppErrorKind::Session(session_err,_) => write!(f, "{}", session_err),
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match_error_kind(&self.kind, f)
    }
}

impl std::fmt::Display for AppErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match_error_kind(self, f)
    }
}

impl AppError {
    pub fn new(kind: AppErrorKind, forward: ForwardTo) -> Self {
        match kind {
            AppErrorKind::Session(err, Some(session)) => {
                Self { 
                    kind: AppErrorKind::Session(err, None), 
                    forward: ForwardTo::LoginRedirect(Rc::new(forward), session)
                }
            }
            _ => {
                Self { kind, forward }
            }
        }
    }

    pub fn root(kind: AppErrorKind) -> Self {
        Self::new(kind, ForwardTo::Root)
    }

    pub fn login(kind: AppErrorKind) -> Self {
        Self::new(kind, ForwardTo::Login)
    }
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        log::debug!("{}", self);
        FlashMessage::error(self.to_string()).send();
        forward::to(self.forward.clone())
    }
}

impl From<std::io::Error> for AppErrorKind {
    fn from(io_err: std::io::Error) -> Self {
        match io_err.kind() {
            std::io::ErrorKind::NotFound => AppErrorKind::FolderPathNotFound,
            _ => {
                AppErrorKind::Io(io_err)
            }
        }
    }
}
