#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]
extern crate rocket;
extern crate rocket_contrib;

#[macro_use]
extern crate diesel_codegen;

mod schema;
mod models;
mod controllers;
mod db;

#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate r2d2;
extern crate r2d2_diesel;

use rocket_contrib::Template;

use controllers::user::*;

fn main() {
    let pool = db::establish_connection();

    rocket::ignite()
        .attach(Template::fairing())
        .mount("/",
               routes![index,
                       user_index,
                       login_page,
                       login_user,
                       logout,
                       login,
                       logged_user,
                       register,
                       registered_user,
                       register_page,
                       register_user
               ])
        .manage(pool)
        .launch();
}
