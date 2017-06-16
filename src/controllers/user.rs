extern crate argon2;
extern crate rand;

use rocket::request::{Form, FlashMessage, FromFormValue};
use rocket::response::{Redirect, Flash};
use rocket::http::{Cookie, Cookies, RawStr};
use rocket_contrib::Template;

use diesel::prelude::*;
use ::models::user::{User, NewUser};
use std::error::Error;
use std::collections::HashMap;
use self::rand::Rng;
use ::db;

#[derive(Debug)]
pub enum Identifier {
    Username(String),
    Email(String),
}

impl<'v> FromFormValue<'v> for Identifier {
    type Error = &'v str;

    // TODO: Add more sanity checking
    fn from_form_value(form_value: &'v RawStr) -> Result<Self, &'v str> {
        let form_value = form_value.percent_decode().map_err(|_| "Utf8 decode error")?;
        if form_value.contains('@') {
            Ok(Identifier::Email(form_value.into()))
        } else {
            Ok(Identifier::Username(form_value.into()))
        }
    }
}

#[derive(FromForm, Debug)]
pub struct Credentials {
    identifier: Identifier,
    password: String,
}

#[post("/login")]
pub fn logged_user(_user: User) -> Redirect {
    Redirect::to("/")
}

#[post("/login", data = "<creds>", rank = 2)]
pub fn login(conn: db::PgSqlConn,
             mut session: Cookies,
             creds: Form<Credentials>)
             -> Flash<Redirect> {
    use ::schema::users::dsl::*;
    
    // TODO: FIX
    //let user = users.filter(verification_token.eq(None::<String>));
    let user = match creds.get().identifier {
        Identifier::Username(ref uname) => users.filter(username.eq(uname)).first(&*conn),
        Identifier::Email(ref mail) => users.filter(email.eq(mail)).first(&*conn),
    };

    let user: User = match user {
        Err(e) => return Flash::error(Redirect::to("/login"), e.description()),
        Ok(u) => u,
    };

    match argon2::verify_encoded(&user.password, creds.get().password.as_bytes()) {
        Err(e) => Flash::error(Redirect::to("/login"), e.description()),
        Ok(check) if !check => Flash::error(Redirect::to("/login"), "Invalid Password!"),
        Ok(_) => {
            session.add_private(Cookie::new("user_id", user.id.to_string()));
            Flash::success(Redirect::to("/"), "Login successful")
        }
    }
}

#[post("/logout")]
pub fn logout(mut session: Cookies) -> Flash<Redirect> {
    session.remove_private(Cookie::named("user_id"));
    Flash::success(Redirect::to("/login"), "Logout successful")
}

/// Redirect already logged in users
#[get("/login")]
pub fn login_user(_user: User) -> Redirect {
    Redirect::to("/")
}

#[get("/login", rank = 2)]
pub fn login_page(flash: Option<FlashMessage>) -> Template {
    let mut context = HashMap::new();
    if let Some(ref msg) = flash {
        context.insert("flash", msg.msg());
    }

    Template::render("login", &context)
}

#[get("/")]
pub fn user_index(user: User) -> Template {
    let mut context = HashMap::new();
    context.insert("id", user.id.to_string());
    context.insert("username", user.username);
    Template::render("index", &context)
}

#[get("/", rank = 2)]
fn index() -> Redirect {
    Redirect::to("/login")
}

#[get("/register")]
pub fn register_user(_user: User) -> Redirect {
    Redirect::to("/")
}

#[get("/register", rank = 2)]
pub fn register_page(flash: Option<FlashMessage>) -> Template {
    let mut context = HashMap::new();
    if let Some(ref msg) = flash {
        context.insert("flash", msg.msg());
    }

    Template::render("register", &context)
}

#[post("/register")]
pub fn registered_user(_user: User) -> Redirect {
    Redirect::to("/")
}

#[post("/register", data = "<creds>", rank = 2)]
pub fn register(conn: db::PgSqlConn, creds: Form<NewUser>) -> Flash<Redirect> {
    use ::schema::users;

    let salt = rand::thread_rng()
        .gen_ascii_chars()
        .take(10)
        .collect::<String>();

    let mut creds = creds.into_inner();
    creds.password = match argon2::hash_encoded(creds.password.as_bytes(),
                                        salt.as_bytes(),
                                        &argon2::Config::default()) {
        Err(e) => return Flash::error(Redirect::to("/register"), e.description()),
        Ok(p) => p,
    };

    match ::diesel::insert(&creds).into(users::table).execute(&*conn) {
        Err(e) => Flash::error(Redirect::to("/register"), e.description()),
        Ok(_) => {
            Flash::success(Redirect::to("/login"),
                           "Registration successful. You can now login")
        }
    }
}
