#[macro_use] extern crate rocket;
use rocket::Request;
use rocket::response::Redirect;
use rocket_dyn_templates::{Template, context};

mod files;

#[cfg(test)] mod tests;

#[get("/")]
pub fn index() -> Redirect {
    Redirect::to(uri!("/files"))
}

#[get("/about")]
pub fn about() -> Template {
    Template::render("about", context! {
        title: "About",
        parent: "layout",
    })
}

#[catch(404)]
pub fn not_found(req: &Request<'_>) -> Template {
    Template::render("error/404", context! {
        uri: req.uri()
    })
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index, about])
        .register("/", catchers![not_found])
        .mount("/files", files::routes())
        .register("/files", files::catchers())
        .attach(Template::custom(|engines| {
            files::customize(&mut engines.handlebars);
        }))
}
