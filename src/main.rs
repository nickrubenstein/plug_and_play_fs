use std::{fs::File, io::BufReader};
use actix_web::{middleware::Logger, App, HttpServer, web};
use actix_files::Files;
use actix_web_flash_messages::{FlashMessagesFramework, storage::CookieMessageStore, Level};
use actix_session::{SessionMiddleware, storage::CookieSessionStore, config::PersistentSession};
use handlebars::Handlebars;
use rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile::{certs, pkcs8_private_keys};
use time::Duration;

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
const PORT: u16 = 443;

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
    hbars.register_templates_directory(".hbs", "./static/templates").unwrap();
    let hbars_ref = web::Data::new(hbars);

    let private_key = actix_web::cookie::Key::derive_from(&open_key_file());
    let message_store = CookieMessageStore::builder(private_key.to_owned()).build();
    let message_framework = FlashMessagesFramework::builder(message_store)
        .minimum_level(LOG_LEVEL).build();

    let rustls_config = init_rustls_config();
    
    log::info!("starting HTTP server at http://{}:{}", HOST, PORT);
    HttpServer::new(move || {
        App::new()
            .app_data(hbars_ref.clone())
            .wrap(SessionMiddleware::builder(
                    CookieSessionStore::default(),
                    private_key.to_owned())
                .cookie_name("session_cookie".to_string())
                .session_lifecycle(PersistentSession::default()
                    .session_ttl(Duration::days(1)))
                .build()
            )
            .service(Files::new("/static", "static").show_files_listing())
            .wrap(message_framework.clone())
            .configure(config_app)
            .wrap(Logger::default())
    })
    .bind_rustls((HOST, PORT), rustls_config)?
    .run()
    .await
}

fn init_rustls_config() -> rustls::ServerConfig {
    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth();

    let key = PrivateKey(open_key_file());
    let cert_chain = open_cert_file();
    config.with_single_cert(cert_chain, key).unwrap()
}

fn open_key_file() -> Vec<u8> {
    let key_file = &mut BufReader::new(File::open("private/key.pem").unwrap());
    let mut keys: Vec<Vec<u8>> = pkcs8_private_keys(key_file).unwrap();
    // exit if no keys could be parsed
    if keys.is_empty() {
        eprintln!("Could not locate PKCS 8 private keys.");
        std::process::exit(1);
    }
    keys.remove(0)
}

fn open_cert_file() -> Vec<Certificate> {
    let cert_file = &mut BufReader::new(File::open("private/cert.pem").unwrap());
    let cert_chain = certs(cert_file)
        .unwrap()
        .into_iter()
        .map(Certificate)
        .collect();
    cert_chain
}
