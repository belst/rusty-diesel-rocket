use rocket::{Outcome, State};
use rocket::request::{self, FromRequest, Request};
use diesel::prelude::*;

#[derive(Queryable, Debug)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password: String,
    pub verification_token: Option<String>,
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
                users.filter(id.eq(uid))
                    .filter(verification_token.eq(None::<String>))
                    .first(&*conn)
                    .unwrap()
            });

        match user {
            Some(user) => Outcome::Success(user),
            None => Outcome::Forward(()),
        }

    }
}
