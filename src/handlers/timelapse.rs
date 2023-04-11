use std::sync::{Arc, Mutex};

use actix_session::Session;
use actix_web::{HttpResponse, web};
use actix_web_flash_messages::{FlashMessage, IncomingFlashMessages};
use handlebars::Handlebars;
use serde_json::json;

use crate::{models::user::User, util::{error::AppError, forward::{ForwardTo, self}, timelapse::{TimelapseThread, TimelapseSettings}}};


pub async fn timelapse(hb: web::Data<Handlebars<'_>>, timelapse_mutex: web::Data<Arc<Mutex<TimelapseThread>>>, session: Session, flashes: IncomingFlashMessages) -> Result<HttpResponse, AppError> {
    let user = User::get(session)
        .map_err(|k| AppError::new(k, ForwardTo::Timelapse))?;
    let mut is_running = false;
    match timelapse_mutex.lock() {
        Ok(timelapse) => {
            is_running = timelapse.is_running();
        },
        Err(err) => FlashMessage::error(format!("timelapse mutex lock error: {}", err)).send()
    };
    let flashes: Vec<(String,String)> = flashes.iter().map(|f| {(f.level().to_string(), f.content().to_string())}).collect();
    let data = json! ({
        "title": "Timelapse",
        "user": user,
        "flashes": flashes,
        "is_running": is_running
    });
    let body = hb.render("timelapse", &data).unwrap();
    Ok(HttpResponse::Ok().body(body))
}

pub async fn start(timelapse_mutex: web::Data<Arc<Mutex<TimelapseThread>>>, form: web::Form<TimelapseSettings>, session: Session) -> Result<HttpResponse, AppError> {
    let _user = User::get(session)
        .map_err(|k| AppError::new(k, ForwardTo::Timelapse))?;
    match timelapse_mutex.lock() {
        Ok(mut timelapse) => {
            if !timelapse.is_running() {
                timelapse.start(form.0);
            }
            else {
                FlashMessage::error("Timelapse is already running").send()
            }
        },
        Err(err) => FlashMessage::error(format!("timelapse mutex lock error: {}", err)).send()
    };
    Ok(forward::to(ForwardTo::Timelapse))
}

pub async fn stop(timelapse_mutex: web::Data<Arc<Mutex<TimelapseThread>>>, session: Session) -> Result<HttpResponse, AppError> {
    let _user = User::get(session)
        .map_err(|k| AppError::new(k, ForwardTo::Timelapse))?;
    match timelapse_mutex.lock() {
        Ok(mut timelapse) => {
            if timelapse.is_running() {
                timelapse.stop();
            }
            else {
                FlashMessage::error("Timelapse is already not running").send()
            }
        },
        Err(err) => FlashMessage::error(format!("timelapse mutex lock error: {}", err)).send()
    };
    Ok(forward::to(ForwardTo::Timelapse))
}