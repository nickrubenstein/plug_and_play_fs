use actix_session::Session;
use actix_web::{HttpResponse, web};
use actix_web_flash_messages::{IncomingFlashMessages, FlashMessage};
use handlebars::Handlebars;
use serde::{Serialize, Deserialize};
use serde_json::json;

use crate::{models::user::{UserAuthority, User}, util::{error::AppError, forward::{ForwardTo, self}}};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Login {
    username: String,
    password: String
}

pub async fn admin(session: Session) -> Result<HttpResponse, AppError> {
    let user = User::get(session)
        .map_err(AppError::login)?;
    if user.authority != UserAuthority::Admin {
        return Ok(HttpResponse::Unauthorized().finish())
    }
    Ok(HttpResponse::Ok().body("You are an admin"))
}

pub async fn account(session: Session, hb: web::Data<Handlebars<'_>>, flashes: IncomingFlashMessages) -> Result<HttpResponse, AppError> {
    let user = User::get(session)
        .map_err(AppError::login)?;
    let flashes: Vec<(String,String)> = flashes.iter().map(|f| {(f.level().to_string(), f.content().to_string())}).collect();
    let data = json! ({
        "title": "FS",
        "flashes": flashes,
        "user": user
    });
    let body = hb.render("account", &data).unwrap();
    Ok(HttpResponse::Ok().body(body))
}

pub async fn login(session: Session, hb: web::Data<Handlebars<'_>>, flashes: IncomingFlashMessages) -> Result<HttpResponse, AppError> {
    let user = User::get(session).ok();
    let flashes: Vec<(String,String)> = flashes.iter().map(|f| {(f.level().to_string(), f.content().to_string())}).collect();
    let data = json! ({
        "title": "FS",
        "flashes": flashes,
        "user": user
    });
    let body = hb.render("login", &data).unwrap();
    Ok(HttpResponse::Ok().body(body))
}

pub async fn try_login(login: web::Form<Login>, session: Session) -> Result<HttpResponse, AppError> {
    let login = login.into_inner();
    let user = User::fetch(&login.username, &login.password).await
        .map_err(AppError::login)?;
    let forward = session.get("redirect")
        .unwrap_or_else(|_| {Some(forward::location(&ForwardTo::Root))})
        .unwrap_or_else(|| forward::location(&ForwardTo::Root));
    session.remove("redirect");
    user.insert(session)
        .map_err(AppError::login)?;
    FlashMessage::success("logged in successfully").send();
    Ok(forward::to_string(&forward))
}

pub async fn logout(session: Session) -> Result<HttpResponse, AppError> {
    User::remove(session)
        .map_err(AppError::login)?;
    FlashMessage::success("logged out successfully").send();
    Ok(forward::to(ForwardTo::Login))
}