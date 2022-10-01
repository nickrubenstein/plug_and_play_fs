use actix_web::{HttpResponse, http, Responder, web};
use handlebars::Handlebars;
use serde_json::json;


pub async fn index() -> impl Responder {
    HttpResponse::PermanentRedirect().insert_header((http::header::LOCATION, "/fs/root/files")).finish()
}

pub async fn about(hb: web::Data<Handlebars<'_>>) -> impl Responder {
    let data = json! ({
        "title": "About FS",
    });
    let body = hb.render("about", &data).unwrap();
    HttpResponse::Ok().body(body)
}