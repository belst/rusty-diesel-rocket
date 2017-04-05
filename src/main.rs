#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]
extern crate rocket;
extern crate rocket_contrib;

#[macro_use]
extern crate diesel_codegen;

mod schema;
mod models;
mod controllers;

#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate r2d2;
extern crate r2d2_diesel;


use diesel::pg::PgConnection;
use r2d2::Pool;
use r2d2_diesel::ConnectionManager;
use dotenv::dotenv;
use std::env;
use self::controllers::user::*;

type PgSqlConn = Pool<ConnectionManager<PgConnection>>;


pub fn establish_connection() -> PgSqlConn {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let config = r2d2::Config::default();
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::new(config, manager).expect("Failed to create database pool.")
}

fn main() {
    let pool = establish_connection();

    rocket::ignite()
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
