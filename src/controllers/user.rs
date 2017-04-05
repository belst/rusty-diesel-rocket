extern crate argon2;

use rocket::State;
use diesel::prelude::*;

#[derive(Debug)]
pub enum Identifier {
    Username(String),
    Email(String),
}

impl<'v> FromFormValue<'v> for Identifier {
    type Error = &'v str;

    // TODO: Add more sanity checking
    fn from_form_value(form_value: &'v str) -> Result<Self, &'v str> {
        if form_value.contains('@') {
            Ok(Identifier::Email(form_value))
        } else {
            Ok(Identifier::Username(form_value))
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
             -> Flash<Redirect>
{
    use ::schema::users::dsl::*;
    if session.get("user_id").is_some() {
        return Flash::success(Redirect::to("/"), "Already logged in");
    }

    let mut user = users.filter(verification_token.eq(None::<String>));
    let user = match creds.get().identifier {
        use Identifier::*;
        Username(uname) => user.filter(username.eq(uname.trim())).first(),
        Email(mail) => user.filter(email.eq(mail.trim())).first(),
    };

    let user = match user {
        Err(e) => return Flash::error(Redirect::to("/login"), e.description()),
        Ok(u) => u,
    };

    match argon2::verify_encoded(&user.password, creds.get().password) {
        Err(e) => Flash::error(Redirect::to("/login"), e.description()),
        Ok(check) if !check => Flash::error(Redirect::to("/login"), "Invalid Password!"),
        Ok(_) => {
            session.set(Cookie::new("user_id", user.id.to_string()));
            Flash::success(Redirect::to("/"), "Login successful")
        }
    }
}
