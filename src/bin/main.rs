#[macro_use]
extern crate rocket;

#[cfg(feature = "templates")]
use rocket::{get, response::Redirect};
use rocket::{Build, Rocket};
use voters_verdict::{
    ballots::{
        get_ballots_by_candidate, get_ballots_by_voted_on, get_ballots_by_voter,
        get_ballots_by_voting, get_ballots_sorted, post_ballot,
    },
    criteria::{get_criterias, get_criterion, post_criterion},
    plumping::{
        internal_server_error, not_found, unauthorized, unprocessable_content, version_handler,
        CORS,
    },
    routes::{API_BALLOTS, API_CRITERIA, API_USERS, API_VOTINGS},
    users::{get_user, get_users, get_users_by_type, post_user},
    votes::{close_vote, get_full_vote, get_raw_vote, post_vote},
};

#[cfg(feature = "templates")]
use rocket::fs::FileServer;
#[cfg(feature = "templates")]
use rocket_dyn_templates::tera::{self, Value};
#[cfg(feature = "templates")]
use rocket_dyn_templates::{Engines, Template};
#[cfg(feature = "templates")]
use std::collections::HashMap;
#[cfg(feature = "templates")]
use voters_verdict::{
    config::{ASSET_DIR, ENVIRONMENT},
    routes::API_ADMIN,
    templates::{
        render_admin_manage_panel, render_admin_panel, render_ballots_by_candidate,
        render_ballots_by_voted_on, render_ballots_by_voter, render_ballots_sorted,
        render_dev_admin_panel, render_voting, render_voting_index,
    },
};
#[cfg(feature = "templates")]
fn format_critera(orignal_value: &Value, _: &HashMap<String, Value>) -> tera::Result<Value> {
    let value = String::from(orignal_value.as_str().unwrap());
    let mut v: Vec<&str> = value.split_terminator('_').collect();
    let weight = "(W:".to_owned() + v.pop().unwrap() + ")";
    let max = "(Max:".to_owned() + v.pop().unwrap() + ")";
    let min = "(Min:".to_owned() + v.pop().unwrap() + ")";
    Ok(Value::String(
        [v.pop().unwrap(), &min, &max, &weight].join(" "),
    ))
}

#[launch]
#[cfg(feature = "templates")]
fn rocket() -> Rocket<Build> {
    #[cfg(not(test))]
    env_logger::init();
    let admin_routes = match std::env::var(ENVIRONMENT) {
        Ok(e) => match e.as_str() {
            "dev" | "d" | "DEV" => routes![render_dev_admin_panel, render_admin_manage_panel],
            _ => routes![render_admin_panel, render_admin_manage_panel],
        },
        Err(_) => routes![render_admin_panel, render_admin_manage_panel],
    };
    let file_dir = match std::env::var(ASSET_DIR) {
        Ok(file_dir) => file_dir,
        Err(_) => String::from("/tmp"),
    };
    info!("Currently asset directory: {}", file_dir);
    rocket::build()
        .register(
            "/",
            catchers![
                unprocessable_content,
                unauthorized,
                not_found,
                internal_server_error
            ],
        )
        .attach(CORS)
        .attach(Template::custom(|engines: &mut Engines| {
            engines
                .tera
                .register_filter("format_criteria", format_critera);
        }))
        .mount("/", routes![login])
        .mount(API_ADMIN, admin_routes)
        .mount("/votings", routes![render_voting_index, render_voting,])
        .mount(
            "/ballots",
            routes![
                render_ballots_by_voted_on,
                render_ballots_sorted,
                render_ballots_by_voter,
                render_ballots_by_candidate
            ],
        )
        .mount("/info", routes![version_handler])
        .mount("/static", FileServer::from(file_dir).rank(1))
        .mount(
            API_BALLOTS,
            routes![
                post_ballot,
                get_ballots_by_voting,
                get_ballots_by_voted_on,
                get_ballots_sorted,
                get_ballots_by_voter,
                get_ballots_by_candidate
            ],
        )
        .mount(
            API_CRITERIA,
            routes![post_criterion, get_criterias, get_criterion],
        )
        .mount(
            API_USERS,
            routes![get_users, get_users_by_type, get_user, post_user],
        )
        .mount(
            API_VOTINGS,
            routes![get_raw_vote, post_vote, get_full_vote, close_vote],
        )
}
#[launch]
#[cfg(not(feature = "templates"))]
fn rocket() -> Rocket<Build> {
    rocket::build()
        .register(
            "/",
            catchers![
                unprocessable_content,
                unauthorized,
                not_found,
                internal_server_error
            ],
        )
        .attach(CORS)
        .mount("/info", routes![version_handler])
        .mount(
            API_BALLOTS,
            routes![
                post_ballot,
                get_ballots_by_voting,
                get_ballots_by_voted_on,
                get_ballots_sorted,
                get_ballots_by_voter,
                get_ballots_by_candidate
            ],
        )
        .mount(
            API_CRITERIA,
            routes![post_criterion, get_criterias, get_criterion],
        )
        .mount(
            API_USERS,
            routes![get_users, get_users_by_type, get_user, post_user],
        )
        .mount(
            API_VOTINGS,
            routes![get_raw_vote, get_full_vote, post_vote, close_vote],
        )
}
#[get("/")]
#[cfg(feature = "templates")]
fn login() -> Redirect {
    Redirect::to("/votings")
}

#[cfg(test)]
mod test {
    use super::rocket;
    use rocket::http::Status;
    use rocket::local::blocking::Client;
    use rocket::uri;
    use voters_verdict::{
        ballots::{Table, TableRow},
        common::Criterion,
        config::{FILE_DIR, MANIFEST_DIR},
    };
    fn get_env_manifest_dir() -> String {
        match std::env::var(MANIFEST_DIR) {
            Ok(d) => String::from(d),
            Err(_) => String::from("."),
        }
    }

    fn build_test_client() -> Client {
        let cargo_manifest_dir = get_env_manifest_dir();
        std::env::set_var(FILE_DIR, cargo_manifest_dir.to_owned() + "/test-data/");
        Client::tracked(rocket()).expect("valid rocket instance")
    }

    #[cfg(feature = "templates")]
    #[test]
    fn redirect_to_votings() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let response = client.get(uri!(super::login)).dispatch();
        assert_eq!(response.status(), Status::SeeOther);
    }

    #[cfg(feature = "templates")]
    mod rendered_ballots {
        use super::*;

        #[test]
        fn get_ballots_by_voting_on() {
            let client = build_test_client();
            let response = client.get(uri!("/ballots/Voting")).dispatch();
            assert_eq!(response.status(), Status::Ok);
        }

        #[test]
        fn get_ballots_by_voting_by_user() {
            let client = build_test_client();
            let response = client.get(uri!("/ballots/Voting/voters/Obama")).dispatch();
            assert_eq!(response.status(), Status::Ok);
        }

        #[test]
        fn get_ballots_by_voting_by_sort_sum() {
            let client = build_test_client();
            let response = client
                .get(uri!("/ballots/Voting/results?sort=sum"))
                .dispatch();
            assert_eq!(response.status(), Status::Ok);
        }
        #[test]
        fn get_ballots_by_voting_by_sort_weight() {
            let client = build_test_client();
            let response = client
                .get(uri!("/ballots/Voting/results?sort=weight"))
                .dispatch();
            assert_eq!(response.status(), Status::Ok);
        }

        #[test]
        fn get_ballots_by_voting_by_candidates() {
            let client = build_test_client();
            let response = client
                .get(uri!("/ballots/Voting/candidates/John"))
                .dispatch();
            assert_eq!(response.status(), Status::Ok);
        }
    }
    mod api_ballots {
        use super::*;

        fn it_should_have_sorted_entries(
            response: rocket::local::blocking::LocalResponse,
            rows: usize,
            value: i8,
        ) {
            let table = response.into_json::<Table<Criterion, TableRow>>().unwrap();
            assert_eq!(table.headers.len(), 2);
            assert_eq!(table.rows.len(), rows);
            assert_eq!(table.rows.get(0).unwrap().sum, value);
        }
        #[test]
        fn get_ballots_by_voting_on() {
            let client = build_test_client();
            let response = client.get(uri!("/api/v1/ballots/Voting")).dispatch();
            assert_eq!(response.status(), Status::Ok);

            it_should_have_sorted_entries(response, 3, 15);
        }

        #[test]
        fn get_ballots_by_voting_by_lowercase_user() {
            let client = build_test_client();
            let response = client
                .get(uri!("/api/v1/ballots/Voting/voters/Michelle"))
                .dispatch();
            assert_eq!(response.status(), Status::Ok);
            it_should_have_sorted_entries(response, 1, 14);
        }
        #[test]
        fn get_ballots_by_voting_by_uppercase_user() {
            let client = build_test_client();
            let response = client
                .get(uri!("/api/v1/ballots/Voting/voters/michelle"))
                .dispatch();
            assert_eq!(response.status(), Status::Ok);
            it_should_have_sorted_entries(response, 1, 14);
        }
        #[test]
        fn get_ballots_by_voting_by_sort_sum() {
            let client = build_test_client();
            let response = client
                .get(uri!("/api/v1/ballots/Voting/results?sort=sum"))
                .dispatch();
            assert_eq!(response.status(), Status::Ok);

            it_should_have_sorted_entries(response, 3, 12);
        }
        #[test]
        fn get_ballots_by_voting_by_sort_weight() {
            let client = build_test_client();
            let response = client
                .get(uri!("/api/v1/ballots/Voting/results?sort=weight"))
                .dispatch();
            assert_eq!(response.status(), Status::Ok);

            it_should_have_sorted_entries(response, 3, 12);
        }

        #[test]
        fn get_ballots_by_voting_by_lowercase_candidates() {
            let client = build_test_client();
            let response = client
                .get(uri!("/api/v1/ballots/Voting/candidates/doe"))
                .dispatch();
            assert_eq!(response.status(), Status::Ok);
            let table = response.into_json::<Table<Criterion, TableRow>>().unwrap();
            assert_eq!(table.rows.len(), 1);
            assert_eq!(table.rows.get(0).unwrap().sum, 15);
        }

        #[test]
        fn get_ballots_by_voting_by_uppercase_candidates() {
            let client = build_test_client();
            let response = client
                .get(uri!("/api/v1/ballots/Voting/candidates/Doe"))
                .dispatch();
            assert_eq!(response.status(), Status::Ok);
            let table = response.into_json::<Table<Criterion, TableRow>>().unwrap();
            assert_eq!(table.rows.len(), 1);
            assert_eq!(table.rows.get(0).unwrap().sum, 15);
        }
    }
    mod api_users {
        use super::*;
        #[test]
        fn get_users() {
            let client = build_test_client();
            let response = client.get(uri!("/api/v1/users")).dispatch();
            assert_eq!(response.status(), Status::Ok);
        }

        #[test]
        fn get_user() {
            let client = build_test_client();
            let response = client.get(uri!("/api/v1/users/Obama")).dispatch();
            assert_eq!(response.status(), Status::Ok);
        }
    }

    mod api_criteria {
        use super::*;
        #[test]
        fn get_criterias() {
            let client = build_test_client();
            let response = client.get(uri!("/api/v1/criteria")).dispatch();
            assert_eq!(response.status(), Status::Ok);
        }

        #[test]
        fn get_criteria() {
            let client = build_test_client();
            let response = client.get(uri!("/api/v1/criteria/Style")).dispatch();
            assert_eq!(response.status(), Status::Ok);
        }
    }
    mod api_votings {
        use super::*;
        #[test]
        fn get_raw_vote() {
            let client = build_test_client();
            let response = client.get(uri!("/api/v1/votings/raw/Voting")).dispatch();
            assert_eq!(response.status(), Status::Ok);
        }

        #[test]
        fn get_full_vote() {
            let client = build_test_client();
            let response = client
                .get(uri!("/api/v1/votings/raw/Voting?full"))
                .dispatch();
            assert_eq!(response.status(), Status::Ok);
        }
    }
}
