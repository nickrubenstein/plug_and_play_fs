use actix_session::Session;
use actix_web::{HttpResponse, http, Responder, web};
use handlebars::Handlebars;
use serde_json::json;

use crate::models::user::User;


pub async fn index() -> impl Responder {
    HttpResponse::PermanentRedirect().insert_header((http::header::LOCATION, "/fs/root/files")).finish()
}

pub async fn about(hb: web::Data<Handlebars<'_>>, session: Session) -> impl Responder {
    let user = User::get(session).ok();
    let data = json! ({
        "title": "About FS",
        "user": user
    });
    let body = hb.render("about", &data).unwrap();
    HttpResponse::Ok().body(body)
}