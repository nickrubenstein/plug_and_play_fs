use actix_session::Session;
use actix_web::{HttpResponse, web};
use handlebars::Handlebars;
use serde_json::json;

use crate::{models::user::User, util::{error::AppError, forward::ForwardTo}};

pub async fn timelapse(hb: web::Data<Handlebars<'_>>, session: Session) -> Result<HttpResponse, AppError> {
    let user = User::get(session)
        .map_err(|k| AppError::new(k, ForwardTo::Timelapse))?;
    let data = json! ({
        "title": "Timelapse",
        "user": user
    });
    let body = hb.render("timelapse", &data).unwrap();
    Ok(HttpResponse::Ok().body(body))
}