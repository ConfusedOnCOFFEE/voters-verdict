use crate::{
    common::{get_users_internal, Empty, Fill, Voting, Votings},
    error::VoteErrorKind,
    persistence::ToPersistence,
    templates::common::render_template,
};
use chrono::Utc;
use rocket::{
    debug, get,
    http::Status,
    request::{FromRequest, Outcome},
    response::status::Unauthorized,
    Request,
};

/////////////////////////////////////////////
//                                         //
//   RENDER TEMPLATES - VOTING             //
//                                         //
/////////////////////////////////////////////

#[get("/")]
pub async fn render_voting_index() -> rocket_dyn_templates::Template {
    let votings: Vec<String> = Votings::empty().index().await.unwrap();
    render_template(
        "votings",
        rocket_dyn_templates::context! {
            votings: votings
        },
    )
}

/////////////////////////////////////////////
//                                         //
//         INVITE CODE                     //
//                                         //
/////////////////////////////////////////////

pub struct VotingGuard {
    pub voting: Voting,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for VotingGuard {
    type Error = VoteErrorKind<'r>;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let default_voting = Voting::empty();
        let voting: Voting = match req.uri().path().segments().get(1) {
            Some(v) => Voting::fill(v, false, "voting").await,
            None => default_voting,
        };
        debug!("FromReuest VotingGuard: {:?}", voting);
        fn is_correct_invite_code(key: &str, voting: &Voting) -> bool {
            key == voting.invite_code
        }
        let unautorized_error =
            VoteErrorKind::Unauthorized(Unauthorized(String::from("Supply an invite code.")));
        match req.query_value::<&str>("invite_code") {
            Some(key) => match key {
                Ok(k) => {
                    if is_correct_invite_code(k, &voting) {
                        Outcome::Success(VotingGuard { voting: voting })
                    } else {
                        Outcome::Error((Status::BadRequest, unautorized_error))
                    }
                }
                Err(_) => Outcome::Error((Status::BadRequest, unautorized_error)),
            },
            None => Outcome::Error((Status::BadRequest, unautorized_error)),
        }
    }
}
#[get("/<voting_id>")]
pub async fn render_voting<'r>(
    guard: VotingGuard,
    voting_id: &str,
) -> rocket_dyn_templates::Template {
    let voting = guard.voting;
    if voting.name.to_lowercase() == voting_id.to_lowercase() {
        let users = get_users_internal().await;
        let candidates = voting.candidates.to_vec();
        let expires_at = format_date(voting.expires_at);
        if voting.expires_at.unwrap() > Utc::now() {
            render_template(
                "voting",
                rocket_dyn_templates::context! {
                    voting,
                    expires_at,
                    candidates,
                    voters: users.voters
                },
            )
        } else {
            render_template(
                "closed-voting",
                rocket_dyn_templates::context! {
                    voting,
                    expires_at,
                },
            )
        }
    } else {
        render_template(
            "error",
            rocket_dyn_templates::context! {
                reason: String::from("Voting doesn't exist.")
            },
        )
    }
}

fn format_date(voted_date: std::option::Option<chrono::DateTime<chrono::Utc>>) -> String {
    match voted_date {
        Some(voted_date) => format!("{}", voted_date.format("%H:%M on %d.%m.%Y")),
        None => String::from("n/a"),
    }
}
