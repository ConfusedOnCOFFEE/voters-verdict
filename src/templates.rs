use crate::{
    ballots::{ballots_by_candidate, ballots_by_voted_on, ballots_by_voter, ballots_sorted},
    common::{
        get_users_internal, Empty, Fill, Index, VoteErrorKind, Voting, VotingStyles, Votings,
    },
    config::{ADMIN_TOKEN, MAINTAINER_TOKEN},
    criteria::get_criterias,
    routes::{API_CRITERIA, API_USERS, API_VOTINGS},
    users::{get_users_by_type, User},
};
use chrono::Utc;
use rocket::{
    get,
    http::Status,
    info,
    request::{FromRequest, Outcome},
    response::status::Unauthorized,
    Request,
};

/////////////////////////////////////////////
//                                         //
//               ADMIN                     //
//                                         //
/////////////////////////////////////////////
#[derive(PartialEq)]
enum UserRole {
    Admin,
    Maintainer,
}
pub struct ElevatedUser {
    role: UserRole,
}
impl ElevatedUser {
    fn new_admin() -> Self {
        Self {
            role: UserRole::Admin,
        }
    }
    fn new_maintainer() -> Self {
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

#[cfg(feature = "templates")]
#[get("/")]
pub async fn render_admin_panel<'r>(_admin: ElevatedUser) -> rocket_dyn_templates::Template {
    match get_criterias().await {
        Ok(c) => build_admin_panel(c.into_inner()).await,
        Err(_) => build_admin_panel(vec![]).await,
    }
}
#[cfg(feature = "templates")]
async fn build_admin_panel(criterias: Vec<String>) -> rocket_dyn_templates::Template {
    let candidates = get_users_by_type(User::Candidate).await.into_inner();
    render_template(
        "admin",
        rocket_dyn_templates::context! {
            candidates,
            criterias,
            users_route: API_USERS,
            votings_route: API_VOTINGS,
            criteria_route: API_CRITERIA,
            default_styles: VotingStyles::default()
        },
    )
}
#[cfg(feature = "templates")]
#[get("/")]
pub async fn render_dev_admin_panel() -> rocket_dyn_templates::Template {
    match get_criterias().await {
        Ok(c) => build_admin_panel(c.into_inner()).await,
        Err(_) => build_admin_panel(vec![]).await,
    }
}
#[cfg(feature = "templates")]
#[get("/manage")]
pub async fn render_admin_manage_panel<'r>(admin: ElevatedUser) -> rocket_dyn_templates::Template {
    if admin.role == UserRole::Admin {
        let voting_index = Votings::empty().index().await.unwrap();
        let votings: Vec<_> = voting_index.iter().map(|b| Voting::fill(b, true)).collect();
        let collected_votings: Vec<Voting> = futures::future::join_all(votings).await;
        render_template(
            "manage-votings",
            rocket_dyn_templates::context! {
                votings: collected_votings
            },
        )
    } else {
        render_template(
            "error",
            rocket_dyn_templates::context! {
                reason: String::from("Unauthorized")
            },
        )
    }
}

/////////////////////////////////////////////
//                                         //
//   RENDER TEMPLATES - VOTING             //
//                                         //
/////////////////////////////////////////////

pub fn render_template<T: rocket::serde::Serialize>(
    name: &'static str,
    context: T,
) -> rocket_dyn_templates::Template {
    rocket_dyn_templates::Template::render(name, context)
}
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
            Some(v) => Voting::fill(v, false).await,
            None => default_voting,
        };
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

/////////////////////////////////////////////
//                                         //
//  RENDER TEMPLATES - CAST BALLOTS        //
//                                         //
////////////////////////////////////////////

fn format_date(voted_date: std::option::Option<chrono::DateTime<chrono::Utc>>) -> String {
    match voted_date {
        Some(voted_date) => format!("{}", voted_date.format("%H:%M on %d.%m.%Y")),
        None => String::from("n/a"),
    }
}

#[get("/<voting_id>")]
pub async fn render_ballots_by_voted_on(voting_id: &str) -> rocket_dyn_templates::Template {
    let voting: Voting = Voting::fill(voting_id, false).await;
    let table = ballots_by_voted_on(voting_id).await;
    render_template(
        "cast-ballots",
        rocket_dyn_templates::context! {
            voting,
            ballots: table.rows
        },
    )
}
#[get("/<voting_id>/results?<sort>")]
pub async fn render_ballots_sorted(voting_id: &str, sort: &str) -> rocket_dyn_templates::Template {
    let voting: Voting = Voting::fill(voting_id, false).await;
    let table = ballots_sorted(voting_id, sort).await;
    render_template(
        "cast-ballots",
        rocket_dyn_templates::context! {
            voting,
            ballots: table.rows
        },
    )
}
#[get("/<voting_id>/voters/<voter>")]
pub async fn render_ballots_by_voter(
    voting_id: &str,
    voter: &str,
) -> rocket_dyn_templates::Template {
    let voting: Voting = Voting::fill(voting_id, false).await;
    let table = ballots_by_voter(voting_id, voter).await;
    if table.rows.len() != 0 {
        render_template(
            "cast-user-ballots",
            rocket_dyn_templates::context! {
                requested_col: String::from("candidate"),
                voting,
                ballots: table.rows,
                filter: voter
            },
        )
    } else {
        render_missing_data(voting_id, voter, false)
    }
}
#[get("/<voting_id>/candidates/<candidate>")]
pub async fn render_ballots_by_candidate(
    voting_id: &str,
    candidate: &str,
) -> rocket_dyn_templates::Template {
    let voting: Voting = Voting::fill(voting_id, false).await;
    let table = ballots_by_candidate(voting_id, candidate).await;
    if table.rows.len() != 0 {
        render_template(
            "cast-user-ballots",
            rocket_dyn_templates::context! {
                requested_col: String::from("voter"),
                voting,
                ballots: table.rows,
                filter: candidate
            },
        )
    } else {
        render_missing_data(voting_id, candidate, true)
    }
}
fn render_missing_data(
    voting_id: &str,
    query: &str,
    candidate: bool,
) -> rocket_dyn_templates::Template {
    render_template(
        "missing-data",
        rocket_dyn_templates::context! {
            name: voting_id,
            user: query,
            candidate
        },
    )
}
