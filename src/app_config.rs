use actix_web::web;

use crate::handlers::{root, files, folders};

pub fn config_app(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/fs")
            .service(
                web::scope("/{folder_path}")
                    .service(
                        web::resource("")
                            .route(web::get().to(folders::get_folder_detail)) // get details of folder_path
                            .route(web::post().to(folders::add_folder))// add new folder to folder_path
                    )
                    .service(
                        web::resource("rename")
                            .route(web::post().to(folders::rename_folder)) // rename folder folder_path
                    )
                    .service(
                        web::resource("remove")
                            .route(web::post().to(folders::remove_folder)) // delete folder folder_path
                    )
                    .service(
                        web::scope("/files")
                            .service(
                                web::resource("")
                                    .route(web::get().to(files::get_files)) // get list of files and folders in folder_path
                                    .route(web::post().to(files::upload_file)) // add file to folder_path
                            )
                            .service(
                                web::scope("/{file_name}")
                                    .service(
                                        web::resource("")
                                            .route(web::get().to(files::get_file_detail)) // get details of file_name
                                    )
                                    .service(
                                        web::resource("download")
                                            .route(web::post().to(files::download_file)) // downloads file_name
                                    )
                                    .service(
                                        web::resource("rename")
                                            .route(web::post().to(files::rename_file)) // rename file_name
                                    )
                                    .service(
                                        web::resource("remove")
                                            .route(web::post().to(files::remove_file)) // delete file_name
                                    )
                            )
                    )
            )
    )
    .route("/", web::get().to(root::index))
    .route("/about", web::get().to(root::about));
}
