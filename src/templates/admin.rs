use crate::{
    authentication::{ElevatedUser, UserRole},
    common::{Empty, Fill, Voting, VotingStyles, Votings},
    criteria::get_criterias,
    persistence::ToPersistence,
    routes::{API_CRITERIA, API_USERS, API_VOTINGS},
    templates::common::render_template,
    users::{get_users_by_type, User},
};

use rocket::get;

#[get("/")]
pub async fn render_admin_panel<'r>(
    _elevated_user: ElevatedUser,
) -> rocket_dyn_templates::Template {
    let html_file = "admin";
    match get_criterias().await {
        Ok(c) => build_admin_panel(c.into_inner(), html_file).await,
        Err(_) => build_admin_panel(vec![], html_file).await,
    }
}
#[get("/votings")]
pub async fn render_votings_admin_panel<'r>(
    _elevated_user: ElevatedUser,
) -> rocket_dyn_templates::Template {
    let html_file = "admin-voting";
    match get_criterias().await {
        Ok(c) => build_admin_panel(c.into_inner(), html_file).await,
        Err(_) => build_admin_panel(vec![], html_file).await,
    }
}
#[get("/")]
pub async fn render_dev_admin_panel() -> rocket_dyn_templates::Template {
    let html_file = "admin";
    match get_criterias().await {
        Ok(c) => build_admin_panel(c.into_inner(), html_file).await,
        Err(_) => build_admin_panel(vec![], html_file).await,
    }
}
#[get("/votings/<voting>")]
pub async fn render_voting_admin_panel(
    elevated_user: ElevatedUser,
    voting: &str,
) -> rocket_dyn_templates::Template {
    if elevated_user.role == UserRole::Admin {
        match get_criterias().await {
            Ok(c) => build_voting_admin_panel(c.into_inner(), voting).await,
            Err(_) => build_voting_admin_panel(vec![], voting).await,
        }
    } else {
        render_template(
            "error",
            rocket_dyn_templates::context! {
                reason: String::from("Unauthorized")
            },
        )
    }
}

async fn build_voting_admin_panel(
    criterias: Vec<String>,
    voting: &str,
) -> rocket_dyn_templates::Template {
    let voting = Voting::fill(voting, false, "voting").await;
    let candidates = get_users_by_type(User::Candidate).await.into_inner();
    render_template(
        "admin-modify-voting",
        rocket_dyn_templates::context! {
            candidates,
            criterias,
            voting,
            votings_route: API_VOTINGS,
        },
    )
}
#[get("/votings")]
pub async fn render_votings_dev_admin_panel() -> rocket_dyn_templates::Template {
    let html_file = "admin-voting";
    match get_criterias().await {
        Ok(c) => build_admin_panel(c.into_inner(), html_file).await,
        Err(_) => build_admin_panel(vec![], html_file).await,
    }
}
#[get("/votings/<voting>")]
pub async fn render_voting_dev_admin_panel(voting: &str) -> rocket_dyn_templates::Template {
    match get_criterias().await {
        Ok(c) => build_voting_admin_panel(c.into_inner(), voting).await,
        Err(_) => build_voting_admin_panel(vec![], voting).await,
    }
}
async fn build_admin_panel(
    criterias: Vec<String>,
    html_file: &'static str,
) -> rocket_dyn_templates::Template {
    let candidates = get_users_by_type(User::Candidate).await.into_inner();
    render_template(
        html_file,
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

#[get("/manage")]
pub async fn render_admin_manage_panel<'r>(
    elevated_user: ElevatedUser,
) -> rocket_dyn_templates::Template {
    if elevated_user.role == UserRole::Admin {
        let voting_index = Votings::empty().index().await.unwrap();
        let votings: Vec<_> = voting_index
            .iter()
            .map(|b| Voting::fill(b, true, "voting"))
            .collect();
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
