use rocket::{
    debug, get,
    http::Status,
    info, post, put,
    response::status::{Conflict, Created},
    serde::{json::Json, Deserialize, Serialize},
};
use std::collections::BTreeMap;

use crate::{
    authentication::ElevatedUser,
    common::{Candidate, CreateVoting, Criterion, Fill, Voting, VotingStyles},
    routes::API_VOTINGS,
    serialize::ToStorage,
};

use regex::Regex;

#[get("/raw/<voting>")]
pub async fn get_raw_vote(voting: &str) -> Result<Json<Voting>, Conflict<String>> {
    Ok(Voting::fill_json(voting, false, "voting").await)
}
#[get("/raw/<voting>?full")]
pub async fn get_full_vote(voting: &str) -> Result<Json<Voting>, Conflict<String>> {
    let voting = Voting::fill(voting, false, "voting").await;
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
pub async fn query_full_voting(voting: CreateVoting) -> Voting {
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
        .iter()
        .map(|b| Criterion::fill(b, false, "criteria"))
        .collect();
    let mut collected_criterias: Vec<Criterion> =
        futures::future::join_all(requested_criterias).await;
    vote_to_be_created
        .categories
        .append(&mut collected_criterias);
    let requested_candidates: Vec<_> = voting
        .candidates
        .iter()
        .map(|b| Candidate::fill(b, false, "candidate"))
        .collect();
    let mut collected_candidates: Vec<Candidate> =
        futures::future::join_all(requested_candidates).await;
    vote_to_be_created
        .candidates
        .append(&mut collected_candidates);
    vote_to_be_created
}

#[post("/", format = "application/json", data = "<voting>")]
pub async fn post_vote(voting: Json<CreateVoting>) -> Result<Created<&'static str>, Status> {
    let re = Regex::new(r"[a-zA-Z]{1}[a-zA-Z0-9]{4}").unwrap();
    let extracted_voting = voting.into_inner();
    if re.is_match(&extracted_voting.invite_code) {
        let vote_to_be_created = query_full_voting(extracted_voting).await;
        match vote_to_be_created.save().await {
            Ok(_done) => Ok(Created::new(
                API_VOTINGS.to_owned() + "/" + &vote_to_be_created.name.clone(),
            )),
            Err(e) => {
                info!("{:?}", e);
                Err(Status::Conflict)
            }
        }
    } else {
        Err(Status::UnprocessableEntity)
    }
}

#[put("/<voting>/close")]
#[cfg(not(feature = "sqlx_sqlite"))]
pub async fn close_vote<'r>(_elevated_user: ElevatedUser, voting: &str) -> Result<String, Status> {
    let mut voting = Voting::fill(voting, true, "voting").await;
    voting.expires_at = Some(chrono::Utc::now());
    match voting.update().await {
        Ok(done) => Ok(done),
        Err(_e) => Err(Status::Conflict),
    }
}

#[put("/<voting>/close")]
#[cfg(feature = "sqlx_sqlite")]
pub async fn close_vote<'r>(_elevated_user: ElevatedUser, voting: &str) -> Result<String, Status> {
    let voting = Voting::fill(voting, true, "voting").await;
    let my_new_closing_vote = chrono::Utc::now().to_string();
    let mut tree = BTreeMap::new();
    tree.insert("expires_at", &my_new_closing_vote);
    match voting.update(tree).await {
        Ok(done) => Ok(done),
        Err(_e) => Err(Status::Conflict),
    }
}

#[derive(Debug, Serialize, PartialEq, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct PossibleVotingModification {
    candidates: Option<Vec<String>>,
    categories: Option<Vec<String>>,
}
#[put("/<voting>/add", format = "application/json", data = "<modifications>")]
pub async fn modify_voting<'r>(
    _elevated_user: ElevatedUser,
    voting: &str,
    modifications: Json<PossibleVotingModification>,
) -> Result<String, Status> {
    let unpacked_payload = modifications.into_inner();
    let mut voting = Voting::fill(&voting.to_lowercase(), true, "voting").await;
    let a = match unpacked_payload.candidates {
        Some(new_candidates) => {
            let requested_candidates: Vec<_> = new_candidates
                .iter()
                .filter(|c| {
                    voting
                        .candidates
                        .iter()
                        .find(|existing_candidate| {
                            !c.matches(
                                &existing_candidate
                                    .id
                                    .clone()
                                    .unwrap()
                                    .strip_prefix("candidate_")
                                    .expect("Always exist as prefix"),
                            )
                            .collect::<Vec<&str>>()
                            .is_empty()
                        })
                        .is_some()
                })
                .map(|c| Candidate::fill(c, false, "candidate"))
                .collect();
            let mut collected_candidates: Vec<Candidate> =
                futures::future::join_all(requested_candidates).await;
            voting.candidates.append(&mut collected_candidates);
            Ok(())
        }
        None => Err(()),
    };

    let b = match unpacked_payload.categories {
        Some(mut new_categories) => {
            let requested_criterias: Vec<_> = new_categories
                .iter_mut()
                .filter(|c| {
                    debug!("Filtered: {}", c);
                    voting
                        .categories
                        .iter()
                        .find(|existing_categories| {
                            debug!("Find: {}-{:?}", c, existing_categories);
                            !c.matches(&existing_categories.name)
                                .collect::<Vec<&str>>()
                                .is_empty()
                        })
                        .is_some()
                })
                .map(|c| Criterion::fill(c, false, "criteria"))
                .collect();
            let mut collected_criterias: Vec<Criterion> =
                futures::future::join_all(requested_criterias).await;
            voting.categories.append(&mut collected_criterias);
            Ok(())
        }
        None => Err(()),
    };
    if a.is_ok() && b.is_ok() {
        info!("{:?}", voting);
        #[cfg(not(feature = "sqlx_sqlite"))]
        match voting.update().await {
            Ok(done) => Ok(done),
            Err(_e) => Err(Status::Conflict),
        }
        #[cfg(feature = "sqlx_sqlite")]
        update_sqlite(voting).await
    } else {
        Err(Status::Conflict)
    }
}

#[cfg(feature = "sqlx_sqlite")]
async fn update_sqlite(voting: Voting) -> Result<String, Status> {
    let mut tree = BTreeMap::new();
    let serialize_candidates = match rocket::serde::json::to_string(&voting.candidates) {
        Ok(s) => Ok(s),
        Err(_) => Err(Status::NotFound),
    };

    let serialize_categories = match rocket::serde::json::to_string(&voting.categories) {
        Ok(s) => Ok(s),
        Err(_) => Err(Status::NotFound),
    };
    if serialize_categories.is_ok() && serialize_candidates.is_ok() {
        let unwrapped_candidate = serialize_candidates.unwrap();
        let unwrapped_categories = serialize_categories.unwrap();
        tree.insert("candidates", &unwrapped_candidate);
        tree.insert("categories", &unwrapped_categories);
        match voting.update(tree).await {
            Ok(done) => Ok(done),
            Err(_e) => Err(Status::Conflict),
        }
    } else {
        Err(Status::NotFound)
    }
}
#[cfg(test)]
mod test {
    use super::*;
    use crate::config::ADMIN_TOKEN;
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
        let elevated_user = ElevatedUser::new_maintainer();
        let response = super::close_vote(elevated_user, "voting").await;
        assert_eq!(response, Err(Status { code: 401 }));
    }

    #[rocket::async_test]
    async fn close_vote_conflict() {
        let elevated_user = ElevatedUser::new_maintainer();
        let response = super::close_vote(elevated_user, "votings").await;
        assert_eq!(response, Ok(String::from("Done")));
    }
    #[rocket::async_test]
    async fn close_vote_unauthores_no_token() {
        let elevated_user = ElevatedUser::new_maintainer();
        let response = super::close_vote(elevated_user, "voting").await;
        assert_eq!(response, Err(Status::Unauthorized));
    }
    #[rocket::async_test]
    async fn close_vote_unauthores_wrong_token() {
        let elevated_user = ElevatedUser::new_maintainer();
        let response = super::close_vote(elevated_user, "voting").await;
        assert_eq!(response, Err(Status::Unauthorized));
    }
}
