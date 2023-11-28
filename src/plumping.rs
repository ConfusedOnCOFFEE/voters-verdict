use crate::config::{DATE_DEPLOY, ENVIRONMENT};
use rocket::{
    catch,
    fairing::{Fairing, Info, Kind},
    get,
    http::{Cookie, CookieJar, Header, Method},
    Request, Response,
};

/////////////////////////////////////////////
//                                         //
//               FAIRING                   //
//                                         //
/////////////////////////////////////////////

pub struct CORS;
#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        let env = match std::env::var(ENVIRONMENT) {
            Ok(e) => e,
            Err(_) => String::from("TEST"),
        };
        response.set_header(Header::new(
            "set-cookie",
            Cookie::new("cv-".to_owned() + &env, env).to_string(),
        ));
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, GET, OPTIONS",
        ));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

/////////////////////////////////////////////
//                                         //
//        VERSION HANDLER                  //
//                                         //
/////////////////////////////////////////////

#[get("/version")]
pub fn version_handler(_jar: &CookieJar<'_>) -> String {
    match std::env::var(DATE_DEPLOY) {
        Ok(val) => val.clone(),
        Err(_e) => "n/a".to_string(),
    }
}

/////////////////////////////////////////////
//                                         //
//               CATCHERS                  //
//                                         //
/////////////////////////////////////////////

#[catch(500)]
pub fn internal_server_error(req: &Request<'_>) -> String {
    if req.method() == Method::Options {
        return "".to_string();
    }
    "Sorry, Problem on our side. We are learning RUST here...".to_string()
}

#[catch(400)]
pub fn unauthorized(req: &Request<'_>) -> String {
    if req.method() == Method::Options {
        return "".to_string();
    }
    "UNAUTHORIZED. You are not allowed to vote. Wrong invite_code or token?".to_string()
}

#[catch(422)]
pub fn unprocessable_content(req: &Request<'_>) -> String {
    if req.method() == Method::Options {
        return "".to_string();
    }
    "Your request body is malinformed or has missing properties.".to_string()
}

#[catch(404)]
pub fn not_found(req: &Request<'_>) -> String {
    if req.method() == Method::Options {
        return "".to_string();
    }
    format!(
        "The voting, candidate, user or whatever you wanted {}. It does not exist!",
        req.uri()
    )
}
