use rocket::{
    get, post,
    response::status::{Conflict, NotFound},
    serde::json::Json,
    FromFormField,
};

use crate::common::{
    get_users_internal, Candidate, Empty, FromJsonFile, IdGenerator, IsUnique, ToJsonFile, Users,
};

#[derive(Debug, PartialEq, FromFormField)]
pub enum User {
    Candidate,
    Voter,
}
#[get("/?<type>")]
pub async fn get_users_by_type(r#type: User) -> Json<Vec<String>> {
    let res = get_users_internal().await;
    let queried_answered = match r#type {
        User::Candidate => res.candidates,
        User::Voter => res.voters,
    };
    Json(queried_answered)
}
#[get("/")]
pub async fn get_users() -> Json<Users> {
    let res = get_users_internal().await;
    Json(res)
}
#[get("/<id>")]
pub async fn get_user(id: &str) -> Result<Json<Candidate>, NotFound<String>> {
    match Candidate::empty().is_unique(id).await {
        Ok(long_id) => Ok(Candidate::from_json_file(&Candidate::empty(), &long_id, false).await),
        Err(_) => Err(NotFound(id.to_owned() + " not found.")),
    }
}
#[post("/", format = "application/json", data = "<user>")]
pub async fn post_user(user: Json<Candidate>) -> Result<String, Conflict<String>> {
    let mut payload = user.into_inner();
    payload.set_id(&payload.generate_id());
    match payload.to_json_file().await {
        Ok(done) => Ok(done),
        Err(e) => Err(Conflict(e.to_string())),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[rocket::async_test]
    async fn get_users() {
        let response = super::get_users().await;
        assert_eq!(
            response.into_inner(),
            Users {
                candidates: vec![String::from("joe"), String::from("doe")],
                voters: vec![String::from("obama"), String::from("michelle")]
            }
        );
    }
    #[rocket::async_test]
    async fn get_user() {
        let response = super::get_user("Obama").await.unwrap();
        assert_eq!(response.into_inner().label, "Obama");
    }

    #[rocket::async_test]
    async fn get_user_not_available() {
        let response = super::get_user("john").await;
        assert_eq!(response, Err(NotFound(String::from("john not found."))));
    }

    #[rocket::async_test]
    async fn post_user_conflict() {
        let json = Json(Candidate {
            id: Some("Obama".to_string()),
            label: "Obama".to_string(),
            voter: true,
        });
        let response = super::post_user(json).await;
        assert_eq!(
            response,
            Err(Conflict(String::from("Voter already exist.")))
        );
    }
    #[rocket::async_test]
    async fn post_user() {
        let json = Json(Candidate {
            id: Some("john".to_string()),
            label: "John".to_string(),
            voter: true,
        });
        let response = super::post_user(json).await;
        assert_eq!(response, Ok(String::from("Saved and index updated.")));
    }

    #[rocket::async_test]
    async fn post_user_conflict_lowercase() {
        let json = Json(Candidate {
            id: Some("obama".to_string()),
            label: "obama".to_string(),
            voter: true,
        });
        let response = super::post_user(json).await;
        assert_eq!(
            response,
            Err(Conflict(String::from("Voter already exist.")))
        )
    }
}
