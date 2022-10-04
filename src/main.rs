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

#[cfg(debug_assertions)]
const HOST: &str = "127.0.0.1";
#[cfg(not(debug_assertions))]
const HOST: &str = "0.0.0.0";

#[cfg(debug_assertions)]
const PORT: u16 = 8000;
#[cfg(not(debug_assertions))]
const PORT: u16 = 8000;

#[cfg(debug_assertions)]
const LOG_LEVEL: Level = Level::Debug;
#[cfg(not(debug_assertions))]
const LOG_LEVEL: Level = Level::Info;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new()
        .default_filter_or(if LOG_LEVEL == Level::Debug { "debug" } else { "info" })
    );

    let mut hbars = Handlebars::new();
    hbars
        .register_templates_directory(".hbs", "./static/templates")
        .unwrap();
    let hbars_ref = web::Data::new(hbars);

    let signing_key = Key::generate(); // This will usually come from configuration!
    let message_store = CookieMessageStore::builder(signing_key).build();
    let message_framework = FlashMessagesFramework::builder(message_store)
        .minimum_level(LOG_LEVEL).build();

    log::info!("starting HTTP server at http://{}:{}", HOST, PORT);
    HttpServer::new(move || {
        App::new()
            .app_data(hbars_ref.clone())
            .service(Files::new("/static", "static").show_files_listing())
            .wrap(message_framework.clone())
            .configure(config_app)
            .wrap(Logger::default())
    })
    .bind((HOST, PORT))?
    .run()
    .await
}
