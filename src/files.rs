use std::str::FromStr;

use rocket::response::{Redirect, Flash};
use rocket::form::{Form, FromForm};
use rocket::fs::TempFile;
use rocket::tokio::fs;

use rocket_dyn_templates::{Template, handlebars, context};

use self::handlebars::{Handlebars, JsonRender};

const ROOT: &str = ".";

#[derive(FromForm)]
pub struct UploadFile<'f> {
    pub file: TempFile<'f>
}

#[derive(FromForm)]
pub struct NewFolder<'f> {
    pub folder_name: &'f str
}

#[post("/<path>/new_folder", data = "<form>")]
pub async fn post_new_folder(path: String, form: Form<NewFolder<'_>>) -> Result<Flash<Redirect>, Flash<Redirect>> {
    let path = path.replace("..", "").replace("~", "");
    let path_slash = path.replace("+", "/");
    println!("path =>  {}", path_slash);
    let result = fs::create_dir(format!("{}/{}/{}", ROOT, path_slash, form.folder_name.clone())).await;
    let redirect = Redirect::to(uri!("/files", get_folder(path)));
    match result {
        Ok(()) => Ok(Flash::success(redirect, format!("Added new folder '{}'", form.folder_name))),
        Err(e) => Err(Flash::error(redirect, e.to_string()))
    }
}

#[post("/<path>", data = "<form>")]
pub async fn post_file(path: String, form: Form<UploadFile<'_>>) -> Result<Redirect, std::io::Error> {
    let path = path.replace("..", "").replace("~", "");
    let path_slash = path.replace("+", "/");
    println!("path =>  {}", path_slash);
    let mut upload = form.into_inner();
    if let (Some(name), Some(content_type)) = (upload.file.name(), upload.file.content_type()) {
        println!("extension =>  {}", content_type.extension().unwrap());
        upload.file.persist_to(format!("{}/{}/{}.{}", ROOT, path_slash, name, content_type.extension().unwrap())).await?;
    }
    Ok(Redirect::to(uri!("/files", get_folder(path))))
}

#[get("/")]
pub async fn get_root_folder() -> Result<Template, std::io::Error> {
    get_folder(String::new()).await
}

#[get("/<path>")]
pub async fn get_folder(path: String) -> Result<Template, std::io::Error> {
    let path = path.replace("..", "").replace("~", "");
    let mut path_list:Vec<String> = path.split("+").map(|s| String::from_str(s).unwrap()).collect();
    if path_list[0].len() > 0 {
        path_list.push(String::from_str("").unwrap());
    }
    let mut path_dir = fs::read_dir(format!("{}/{}", ROOT, path_list.join("/"))).await?;
    let mut files = Vec::new();
    while let Some(entry) = path_dir.next_entry().await? {
        if let Ok(name) = entry.file_name().into_string() {
            let file_type = entry.file_type().await?;
            files.push(context! {
                path: format!("{}/{}{}", "/files", path_list.join("+"), name),
                name: name,
                is_folder: file_type.is_dir()
            });
        }
    }
    let mut path_display = Vec::new();
    path_display.push((path_list[0].clone(), path_list[0].clone()));
    for i in 1..path_list.len() - 1 {
        let full_path = format!("{}+{}", path_list[i - 1], path_list[i]);
        path_display.push((path_list[i].clone(), full_path.clone()));
        path_list[i] = full_path;
    }
    Ok(Template::render("files", context! {
        title: "FS",
        path: path,
        paths: path_display,
        items: files,
    }))
}

pub fn routes() -> Vec<rocket::Route> {
    routes![post_new_folder, post_file, get_folder, get_root_folder]
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
