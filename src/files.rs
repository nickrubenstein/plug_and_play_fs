use rocket::response::{Redirect, Flash};
use rocket::form::{Form, FromForm};
use rocket::fs::TempFile;
use rocket::tokio::fs;

use rocket_dyn_templates::{Template, handlebars, context};

use self::handlebars::{Handlebars, JsonRender};

use crate::file_path::FilePath;

#[derive(FromForm)]
pub struct UploadFile<'f> {
    pub file: TempFile<'f>
}

#[derive(FromForm)]
pub struct NewFolder<'f> {
    pub folder_name: &'f str
}

#[post("/new_folder", data = "<form>")]
pub async fn post_root_new_folder(form: Form<NewFolder<'_>>) -> Result<Flash<Redirect>, Flash<Redirect>> {
    post_new_folder(String::new(), form).await
}

#[post("/new_folder/<path>", data = "<form>")]
pub async fn post_new_folder(path: String, form: Form<NewFolder<'_>>) -> Result<Flash<Redirect>, Flash<Redirect>> {
    let path = match FilePath::new(path) {
        Ok(file_path) => file_path,
        Err(e) => return Err(Flash::error(Redirect::to("/files"), e.to_string()))
    };
    let redirect = Redirect::to(uri!("/files", get_folder(path.uri_path())));
    println!("path =>  {}", path.append_to_file_path(&form.folder_name.to_string()));
    let result = fs::create_dir(path.append_to_file_path(&form.folder_name.to_string())).await;
    match result {
        Ok(()) => Ok(Flash::success(redirect, format!("Added folder '{}'", form.folder_name))),
        Err(e) => Err(Flash::error(redirect, e.to_string()))
    }
}

#[post("/", data = "<form>")]
pub async fn post_root_file(form: Form<UploadFile<'_>>) -> Result<Flash<Redirect>, Flash<Redirect>> {
    post_file(String::new(), form).await
}

#[post("/<path>", data = "<form>")]
pub async fn post_file(path: String, form: Form<UploadFile<'_>>) -> Result<Flash<Redirect>, Flash<Redirect>> {
    let path = match FilePath::new(path) {
        Ok(file_path) => file_path,
        Err(e) => return Err(Flash::error(Redirect::to("/files"), e.to_string()))
    };
    let redirect = Redirect::to(uri!("/files", get_folder(path.uri_path())));
    let mut upload = form.into_inner();
    if let (Some(name), Some(content_type)) = (upload.file.name(), upload.file.content_type()) {
        let file_name = format!("{}.{}", name, content_type.extension().unwrap());
        println!("path =>  {}", path.append_to_file_path(&file_name));
        match upload.file.persist_to(path.append_to_file_path(&file_name)).await {
            Ok(()) => return Ok(Flash::success(redirect, format!("Added file '{}'", file_name))),
            Err(e) => return Err(Flash::error(redirect, e.to_string()))
        }
    }
    else {
        return Err(Flash::error(redirect, "Could not get file name or extension"))
    }
}

#[get("/delete/<path>")]
pub async fn delete_file(path: String) -> Result<Flash<Redirect>, Flash<Redirect>> {
    let path = match FilePath::new(path) {
        Ok(file_path) => file_path,
        Err(e) => return Err(Flash::error(Redirect::to("/files"), e.to_string()))
    };
    println!("parent =>  {}", path.parent().uri_path());
    let redirect = Redirect::to(uri!("/files", get_folder(path.parent().uri_path())));
    let file_meta = fs::metadata(path.file_path()).await;
    let file_meta = match file_meta {
        Ok(meta) => meta,
        Err(e) => return Err(Flash::error(redirect, e.to_string()))
    };
    // println!("created =>  {}", file_meta.created().unwrap().duration_since(std::time::SystemTime::now()).unwrap().as_secs());
    if file_meta.is_file() {
        let result = fs::remove_file(path.file_path()).await;
        match result {
            Ok(()) => Ok(Flash::success(redirect, format!("Deleted file '{}'", path.file_path()))),
            Err(e) => Err(Flash::error(redirect, e.to_string()))
        }
    }
    else {
        let result = fs::remove_dir(path.file_path()).await;
        match result {
            Ok(()) => Ok(Flash::success(redirect, format!("Deleted folder '{}'", path.file_path()))),
            Err(e) => Err(Flash::error(redirect, e.to_string()))
        }
    }
    
}

#[get("/")]
pub async fn get_root_folder() -> Result<Template, std::io::Error> {
    get_folder(String::new()).await
}

#[get("/<path>")]
pub async fn get_folder(path: String) -> Result<Template, std::io::Error> {
    let path = match FilePath::new(path) {
        Ok(file_path) => file_path,
        Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, e))
    };
    let mut path_dir = fs::read_dir(path.file_path()).await?;
    let mut files = Vec::new();
    while let Some(entry) = path_dir.next_entry().await? {
        if let Ok(name) = entry.file_name().into_string() {
            let file_type = entry.file_type().await?;
            files.push(context! {
                path: path.append_to_uri_path(&name),
                name: name,
                is_folder: file_type.is_dir()
            });
        }
    }
    Ok(Template::render("files", context! {
        title: "FS",
        path: path.uri_path(),
        paths: path.path_list_aggrigate(),
        items: files,
    }))
}

pub fn routes() -> Vec<rocket::Route> {
    routes![
        post_new_folder, 
        post_root_new_folder, 
        post_file, 
        post_root_file, 
        delete_file,
        get_folder, 
        get_root_folder
    ]
}

pub fn catchers() -> Vec<rocket::Catcher> {
    catchers![]
}

fn wow_helper(
    h: &handlebars::Helper<'_, '_>,
    _: &handlebars::Handlebars,
    _: &handlebars::Context,
    _: &mut handlebars::RenderContext<'_, '_>,
    out: &mut dyn handlebars::Output
) -> handlebars::HelperResult {
    if let Some(param) = h.param(0) {
        out.write("<b><i>")?;
        out.write(&param.value().render())?;
        out.write("</b></i>")?;
    }

    Ok(())
}

pub fn customize(hbs: &mut Handlebars) {
    hbs.register_helper("wow", Box::new(wow_helper));
}
