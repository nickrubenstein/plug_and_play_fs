use actix_web::web;

use crate::handlers::{root, files, folders};

pub fn config_app(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/fs")
            .service(
                web::scope("/{folder_path}")
                    .service(
                        web::resource("")
                    //         .route(web::get().to(folders::get_folder_detail)) // get details of folder_path
                            .route(web::post().to(folders::add_folder))// add new folder to folder_path
                    //         .route(web::put().to(folders::rename_folder)) // rename folder folder_path
                    //         .route(web::delete().to(folders::delete_folder))    // delete folder folder_path
                    )
                    .service(
                        web::scope("/files")
                            .service(
                                web::resource("")
                                    .route(web::get().to(files::get_files)) // get list of files and folders in folder_path
                                    .route(web::post().to(files::add_file))// add file to folder_path
                            )
                            .service(
                                web::resource("/{file_name}")
                            //         .route(web::get().to(files::get_file_detail)) // get details of file_name
                            //         .route(web::put().to(files::rename_file)) // rename file_name
                            //         .route(web::delete().to(files::delete_file))    // delete file_name
                            )
                    )
            )
    )
    .route("/", web::get().to(root::index))
    .route("/about", web::get().to(root::about));
}
