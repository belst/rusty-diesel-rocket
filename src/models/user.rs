use rocket::{Outcome, State};
use rocket::request::{self, FromRequest, Request};
use diesel::prelude::*;
use ::schema::users;

#[derive(Queryable, Debug)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password: String,
    pub verification_token: Option<String>,
}

// TODO: Add validation to FromForm
#[derive(Insertable, FromForm)]
#[table_name="users"]
pub struct NewUser {
    pub username: String,
    pub email: String,
    pub password: String,
}

impl<'a, 'r> FromRequest<'a, 'r> for User {
    type Error = ();

    fn from_request(req: &'a Request<'r>) -> request::Outcome<User, ()> {
        use ::schema::users::dsl::*;
        let pool = match <State<::PgSqlConn> as FromRequest>::from_request(req) {
            Outcome::Success(conn) => conn,
            _ => return Outcome::Forward(()),
        };

        let conn = match pool.get() {
            Ok(c) => c,
            _ => return Outcome::Forward(()),
        };

        let user = req.session()
            .get("user_id")
            .and_then(|cookie| cookie.value().parse::<i32>().ok())
            .map(|uid| {
                println!("{}", uid);
                users.filter(id.eq(uid))
                    // TODO: FIX verification token
                    //.filter(verification_token.eq(None::<String>))
                    .first(&*conn)
            });


        let user = match user {
            None => return Outcome::Forward(()),
            Some(u) => u,
        };
        match user {
            Err(e) => {
                println!("{:#?}", e);
                return Outcome::Failure((::rocket::http::Status::NotFound, ()));
            }
            Ok(u) => Outcome::Success(u),
        }
    }
}
