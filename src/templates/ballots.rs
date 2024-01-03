use crate::{
    ballots::{ballots_by_candidate, ballots_by_voted_on, ballots_by_voter, ballots_sorted},
    common::{Fill, Voting},
    templates::common::render_template,
};

use rocket::get;

/////////////////////////////////////////////
//                                         //
//  RENDER TEMPLATES - CAST BALLOTS        //
//                                         //
////////////////////////////////////////////

#[get("/<voting_id>")]
pub async fn render_ballots_by_voted_on(voting_id: &str) -> rocket_dyn_templates::Template {
    let voting: Voting = Voting::fill(voting_id, false, "voting").await;
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
    let voting: Voting = Voting::fill(voting_id, false, "voting").await;
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
    let voting: Voting = Voting::fill(voting_id, false, "voting").await;
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
    let voting: Voting = Voting::fill(voting_id, false, "voting").await;
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
