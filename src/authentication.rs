use rocket::{
    http::Status,
    info,
    request::{FromRequest, Outcome},
    response::status::Unauthorized,
    Request,
};

use crate::{
    config::{ADMIN_TOKEN, MAINTAINER_TOKEN},
    error::VoteErrorKind,
};
/////////////////////////////////////////////
//                                         //
//               ADMIN                     //
//                                         //
/////////////////////////////////////////////
#[derive(PartialEq)]
pub enum UserRole {
    Admin,
    Maintainer,
}
pub struct ElevatedUser {
    pub role: UserRole,
}
impl ElevatedUser {
    fn new_admin() -> Self {
        Self {
            role: UserRole::Admin,
        }
    }
    pub fn new_maintainer() -> Self {
        Self {
            role: UserRole::Maintainer,
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ElevatedUser {
    type Error = VoteErrorKind<'r>;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let unautorized_error =
            VoteErrorKind::Unauthorized(Unauthorized(String::from("Provide a valid token")));
        fn is_token_valid(var_key: &str, token_value: &str) -> bool {
            match std::env::var(var_key) {
                Ok(t) => t == token_value,
                Err(_) => false,
            }
        }
        match req.query_value::<&str>("token") {
            Some(possible_token) => match possible_token {
                Ok(t) => {
                    if is_token_valid(MAINTAINER_TOKEN, t) {
                        info!("Maintainer logged in.");
                        return Outcome::Success(ElevatedUser::new_maintainer());
                    }
                    if is_token_valid(ADMIN_TOKEN, t) {
                        info!("Admin logged in.");
                        return Outcome::Success(ElevatedUser::new_admin());
                    }
                    Outcome::Error((Status::BadRequest, unautorized_error))
                }
                Err(_) => Outcome::Error((Status::BadRequest, unautorized_error)),
            },
            None => Outcome::Error((Status::BadRequest, unautorized_error)),
        }
    }
}
