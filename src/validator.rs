use crate::common::{Ballot, Empty, Fill, Index, VoteKind, Voting, Votings};
use regex::Regex;
use rocket::http::Status;
pub const PATTERN: &str = r"[a-zA-Z0-9\.\!-\?]+";
pub fn compare_styles_file_names(a: &str, b: &str) -> bool {
    a.to_lowercase().starts_with(&b.to_lowercase())
}
fn is_correct_formated_value(value: &str, pattern: Option<&str>) -> bool {
    let pattern = match pattern {
        Some(p) => p,
        None => r"[a-zA-Z0-9\.<>\-\)\(\!\?]+",
    };
    let re = Regex::new(pattern).unwrap();
    if re.is_match(value) {
        true
    } else {
        false
    }
}

pub async fn validate(r#type: VoteKind, voting_id: &str, voter_name: &str) -> Result<bool, Status> {
    let r = match r#type {
        VoteKind::Candidate(_) => true,
        VoteKind::Voting(_) => true,
        VoteKind::Ballot(ballot) => {
            validate_requested_voting(voting_id)
                && validate_voting(voting_id).await
                && validate_voter_name(voter_name)
                && validate_candidate(&ballot.candidate)
                && validate_notes(&ballot.notes)
                && validate_points(voting_id, &ballot).await
        }
        VoteKind::Criterion(_) => true,
        _ => true,
    };
    match r {
        true => Ok(r),
        false => Err(Status::UnprocessableEntity),
    }
}

fn validate_requested_voting(voting_id: &str) -> bool {
    let voting_valid = is_correct_formated_value(voting_id, Some(PATTERN));
    if !voting_valid {
        return false;
    }
    true
}
async fn validate_voting(voting_id: &str) -> bool {
    let voting_index = Votings::empty().index().await.unwrap();
    let voting_exist = voting_index
        .iter()
        .find(|v| v.as_str().to_lowercase() == voting_id.to_lowercase())
        .is_some();
    if !voting_exist {
        return false;
    }
    true
}
fn validate_voter_name(voter_name: &str) -> bool {
    let voter_valid = is_correct_formated_value(voter_name, None);
    if !voter_valid {
        return false;
    }
    true
}
fn validate_candidate(candidate: &str) -> bool {
    let candidate_valid = is_correct_formated_value(candidate, Some(PATTERN));
    if !candidate_valid {
        return false;
    }
    true
}

fn validate_notes(note: &Option<String>) -> bool {
    let _ = match &note {
        Some(n) => {
            let notes_valid = is_correct_formated_value(&n, Some(r"[a-zA-Z0-9\. \!-\?]+"));
            if !notes_valid {
                return false;
            }
        }
        None => {}
    };
    true
}

async fn validate_points(voting_id: &str, ballot: &Ballot) -> bool {
    let voting = Voting::fill(voting_id, false).await;
    let mut point_names: Vec<String> = voting
        .categories
        .iter()
        .map(|c| String::from(c.name.as_str()))
        .collect();
    point_names.sort();
    let mut existing_point_names: Vec<String> = ballot
        .clone()
        .votes
        .iter()
        .map(|c| String::from(c.name.as_str()))
        .collect();
    existing_point_names.sort();
    let points_exist = existing_point_names
        .iter()
        .zip(&point_names)
        .filter(|&(a, b)| a != b)
        .count();
    if points_exist != 0 {
        return validate_voting_candidate(&voting, &ballot.candidate);
    };
    true
}

fn validate_voting_candidate(voting: &Voting, candidate: &String) -> bool {
    let candidate_exist = voting
        .candidates
        .iter()
        .filter(|c| !c.voter)
        .find(|c| &c.id.clone().unwrap() == candidate)
        .is_none();
    if !candidate_exist {
        return false;
    }
    true
}
