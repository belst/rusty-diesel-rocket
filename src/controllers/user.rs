extern crate argon2;

use rocket::State;
use rocket::request::{Form, FlashMessage, FromFormValue};
use rocket::response::{Redirect, Flash};
use rocket::http::{Cookie, Session, RawStr};
use rocket_contrib::Template;

use diesel::prelude::*;
use ::models::user::User;
use std::error::Error;
use std::collections::HashMap;

#[derive(Debug)]
pub enum Identifier {
    Username(String),
    Email(String),
}

impl<'v> FromFormValue<'v> for Identifier {
    type Error = &'v str;

    // TODO: Add more sanity checking
    fn from_form_value(form_value: &'v RawStr) -> Result<Self, &'v str> {
        if form_value.contains('@') {
            Ok(Identifier::Email(form_value.percent_decode().unwrap().into()))
        } else {
            Ok(Identifier::Username(form_value.percent_decode().unwrap().into()))
        }
    }
}

#[derive(FromForm, Debug)]
pub struct Credentials {
    identifier: Identifier,
    password: String,
}

#[post("/login", data = "<creds>")]
pub fn login(pool: State<::PgSqlConn>,
             mut session: Session,
             creds: Form<Credentials>)
             -> Flash<Redirect> {
    use ::schema::users::dsl::*;
    if session.get("user_id").is_some() {
        return Flash::success(Redirect::to("/"), "Already logged in");
    }
    let conn = match pool.get() {
        Err(e) => return Flash::error(Redirect::to("/login"), e.description()),
        Ok(c) => c,
    };

    let user = users.filter(verification_token.eq(None::<String>));
    let user = match creds.get().identifier {
        Identifier::Username(ref uname) => user.filter(username.eq(uname.trim())).first(&*conn),
        Identifier::Email(ref mail) => user.filter(email.eq(mail.trim())).first(&*conn),
    };

    let user: User = match user {
        Err(e) => return Flash::error(Redirect::to("/login"), e.description()),
        Ok(u) => u,
    };

    match argon2::verify_encoded(&user.password, creds.get().password.as_bytes()) {
        Err(e) => Flash::error(Redirect::to("/login"), e.description()),
        Ok(check) if !check => Flash::error(Redirect::to("/login"), "Invalid Password!"),
        Ok(_) => {
            session.set(Cookie::new("user_id", user.id.to_string()));
            Flash::success(Redirect::to("/"), "Login successful")
        }
    }
}

#[post("/logout")]
pub fn logout(mut session: Session) -> Flash<Redirect> {
    session.remove(Cookie::named("user_id"));
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
