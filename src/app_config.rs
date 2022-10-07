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
                        web::resource("moveup")
                            .route(web::post().to(folders::move_folder_up)) // move folder folder_path up a folder
                    )
                    .service(
                        web::resource("moveinto")
                            .route(web::post().to(folders::move_folder_into)) // move folder folder_path into a sibling folder
                    )
                    .service(
                        web::resource("rename")
                            .route(web::post().to(folders::rename_folder)) // rename folder folder_path
                    )
                    .service(
                        web::resource("flatten")
                            .route(web::post().to(folders::flatten_folder)) // moves everything in folder_path to its parent folder
                    )
                    .service(
                        web::resource("zip")
                            .route(web::post().to(folders::zip_folder)) // delete folder folder_path
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
                                        web::resource("unzip")
                                            .route(web::post().to(files::unzip_file)) // unzip file_name
                                    )
                                    .service(
                                        web::resource("rename")
                                            .route(web::post().to(files::rename_file)) // rename file_name
                                    )
                                    .service(
                                        web::resource("moveup")
                                            .route(web::post().to(files::move_file_up)) // move file_name up a folder
                                    )
                                    .service(
                                        web::resource("moveinto")
                                            .route(web::post().to(files::move_file_into)) // move file_name into a sibling folder
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
