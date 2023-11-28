use rocket::{get, http::Status, info, post, put, response::status::Conflict, serde::json::Json};

use crate::{
    common::{Candidate, CreateVoting, Criterion, Fill, ToJsonFile, Voting, VotingStyles},
    config::ADMIN_TOKEN,
};

use regex::Regex;

#[get("/raw/<voting>")]
pub async fn get_raw_vote(voting: &str) -> Result<Json<Voting>, Conflict<String>> {
    Ok(Voting::fill_json(voting, false).await)
}
#[get("/raw/<voting>?full")]
pub async fn get_full_vote(voting: &str) -> Result<Json<Voting>, Conflict<String>> {
    let voting = Voting::fill(voting, false).await;
    let requested_voting = CreateVoting {
        name: voting.name,
        expires_at: voting.expires_at.unwrap(),
        candidates: vec![],
        criterias: vec![],
        styles: None,
        invite_code: voting.invite_code,
    };
    Ok(Json(query_full_voting(requested_voting).await))
}

// TODO refactor and make each call more obious
pub async fn query_full_voting(mut voting: CreateVoting) -> Voting {
    let styles = match voting.styles {
        Some(s) => VotingStyles {
            background: String::from(s.background.as_str()),
            font: String::from(s.font.as_str()),
            selection: String::from(s.selection.as_str()),
            fields: String::from(s.fields.as_str()),
        },
        None => VotingStyles::default(),
    };
    let mut vote_to_be_created = Voting {
        name: String::from(voting.name.as_str()),
        expires_at: Some(voting.expires_at),
        created_at: Some(chrono::Utc::now()),
        candidates: vec![],
        categories: vec![],
        styles,
        invite_code: voting.invite_code,
    };
    let requested_criterias: Vec<_> = voting
        .criterias
        .iter_mut()
        .map(|b| Criterion::fill(b, false))
        .collect();
    let mut collected_criterias: Vec<Criterion> =
        futures::future::join_all(requested_criterias).await;
    vote_to_be_created
        .categories
        .append(&mut collected_criterias);
    let requested_candidates: Vec<_> = voting
        .candidates
        .iter()
        .map(|b| Candidate::fill(b, false))
        .collect();
    let mut collected_candidates: Vec<Candidate> =
        futures::future::join_all(requested_candidates).await;
    vote_to_be_created
        .candidates
        .append(&mut collected_candidates);
    vote_to_be_created
}

#[post("/", format = "application/json", data = "<voting>")]
pub async fn post_vote(voting: Json<CreateVoting>) -> Result<String, Status> {
    let re = Regex::new(r"[a-zA-Z]{1}[a-zA-Z0-9]{4}").unwrap();
    let extracted_voting = voting.into_inner();
    if re.is_match(&extracted_voting.invite_code) {
        let vote_to_be_created = query_full_voting(extracted_voting).await;
        match vote_to_be_created.to_json_file().await {
            Ok(done) => Ok(done),
            Err(e) => {
                info!("{:?}", e);
                Err(Status::Conflict)
            }
        }
    } else {
        Err(Status::UnprocessableEntity)
    }
}

#[put("/<voting>/close?<token>")]
pub async fn close_vote<'r>(voting: &str, token: &str) -> Result<String, Status> {
    let mut voting = Voting::fill(voting, true).await;
    match std::env::var(ADMIN_TOKEN) {
        Ok(t) if t == token => {
            voting.expires_at = Some(chrono::Utc::now());
            match voting.update_json_file().await {
                Ok(done) => Ok(done),
                Err(_e) => Err(Status::Conflict),
            }
        }
        Ok(_) => Err(Status::Unauthorized),
        Err(_) => Err(Status::Unauthorized),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[rocket::async_test]
    async fn query_full_voting() {
        let create_voting = CreateVoting {
            name: "name".to_string(),
            expires_at: chrono::Utc::now(),
            candidates: vec![],
            criterias: vec![],
            styles: None,
            invite_code: "T1234".to_string(),
        };
        let response = super::query_full_voting(create_voting).await;
        assert_eq!(response.styles.background, "#30363d");
    }

    #[rocket::async_test]
    async fn query_full_voting_no_color() {
        let create_voting = CreateVoting {
            name: "name".to_string(),
            expires_at: chrono::Utc::now(),
            candidates: vec![],
            criterias: vec![],
            styles: Some(VotingStyles {
                background: String::from("1"),
                font: String::from("2"),
                selection: String::from("3"),
                fields: String::from("4"),
            }),
            invite_code: "T1234".to_string(),
        };
        let response = super::query_full_voting(create_voting).await;
        assert_eq!(response.styles.background, "1");
    }

    #[rocket::async_test]
    async fn close_vote() {
        let response = super::close_vote("voting", "W123").await;
        assert_eq!(response, Err(Status { code: 401 }));
    }

    #[rocket::async_test]
    async fn close_vote_conflict() {
        std::env::set_var(ADMIN_TOKEN, "W1234");
        let response = super::close_vote("votings", "W1234").await;
        assert_eq!(response, Ok(String::from("Done")));
    }
    #[rocket::async_test]
    async fn close_vote_unauthores_no_token() {
        let response = super::close_vote("voting", "").await;
        assert_eq!(response, Err(Status::Unauthorized));
    }
    #[rocket::async_test]
    async fn close_vote_unauthores_wrong_token() {
        let response = super::close_vote("voting", "W123").await;
        assert_eq!(response, Err(Status::Unauthorized));
    }
}
