use actix_web::{middleware::Logger, cookie::Key, App, HttpServer, web};
use actix_files::Files;
use actix_web_flash_messages::{FlashMessagesFramework, storage::CookieMessageStore, Level};
use handlebars::Handlebars;

pub mod tests;
pub mod app_config;
pub mod handlers;
pub mod models;
pub mod util;

use app_config::config_app;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let mut hbars = Handlebars::new();
    hbars
        .register_templates_directory(".hbs", "./static/templates")
        .unwrap();
    let hbars_ref = web::Data::new(hbars);

    let signing_key = Key::generate(); // This will usually come from configuration!
    let message_store = CookieMessageStore::builder(signing_key).build();
    let message_framework = FlashMessagesFramework::builder(message_store)
        .minimum_level(flash_min_level()).build();

    log::info!("starting HTTP server at http://localhost:8000");
    HttpServer::new(move || {
        App::new()
            .app_data(hbars_ref.clone())
            .service(Files::new("/static", "static").show_files_listing())
            .wrap(message_framework.clone())
            .configure(config_app)
            .wrap(Logger::default())
    })
    .bind(("127.0.0.1", 8000))?
    .run()
    .await
}

#[cfg(debug_assertions)] // For debug
fn flash_min_level() -> Level {
    log::info!("Debugging enabled");
    Level::Debug
}

#[cfg(not(debug_assertions))] // For release
fn flash_min_level() -> Level {
    log::info!("Debugging disabled");
    Level::Info
}