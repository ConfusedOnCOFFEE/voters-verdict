use rocket::{
    get, post,
    response::status::{Conflict, NotFound},
    serde::json::Json,
};

use crate::{
    common::{Criteria, Criterion, Empty, Fill, Index, ToJsonFile},
    validator::compare_styles_file_names,
};

#[get("/", format = "application/json")]
pub async fn get_criterias() -> Result<Json<Vec<String>>, Conflict<String>> {
    match Criteria::empty().index().await {
        Ok(done) => Ok(Json(done)),
        Err(e) => Err(Conflict(e.to_string())),
    }
}

#[get("/<criterion>")]
pub async fn get_criterion(criterion: &str) -> Result<Json<Vec<Criterion>>, NotFound<String>> {
    match Criteria::empty().index().await {
        Ok(done) => {
            let possible_criteria = done
                .iter()
                .filter(|c| compare_styles_file_names(c, criterion))
                .map(|c| Criterion::fill(c, false));
            let all_criterias = futures::future::join_all(possible_criteria).await;
            Ok(Json(all_criterias))
        }
        Err(_e) => Err(NotFound(format!("{:?} not found", criterion))),
    }
}

#[post("/", format = "application/json", data = "<criterion>")]
pub async fn post_criterion(criterion: Json<Criterion>) -> Result<String, Conflict<String>> {
    match criterion.into_inner().to_json_file().await {
        Ok(done) => Ok(done),
        Err(e) => Err(Conflict(e.to_string())),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[rocket::async_test]
    async fn get_criterias() {
        let response = super::get_criterias().await.unwrap();
        assert_eq!(
            response.into_inner(),
            vec![
                String::from("style_0_10_15"),
                String::from("weather_0_15_15"),
                String::from("style_10_15_50"),
            ]
        );
    }

    #[test]
    fn test_string_compare() {
        assert_eq!(compare_styles_file_names("test_0_15_15", "test"), true);
        assert_eq!(compare_styles_file_names("test_0_15_15", "Test_0"), true);
        assert_eq!(compare_styles_file_names("test_0_15_15", "test_0"), true);
        assert_eq!(compare_styles_file_names("test_0_15_15", "test_0_15"), true);
    }
    #[rocket::async_test]
    async fn get_single_criterion() {
        let response = super::get_criterion("Style_10").await.unwrap();
        assert_eq!(
            response.into_inner(),
            vec![Criterion {
                name: String::from("Style"),
                min: 10,
                max: 15,
                weight: Some(50.0)
            }]
        );
    }
    #[rocket::async_test]
    async fn get_multiple_criterion() {
        let response = super::get_criterion("Style").await.unwrap();
        assert_eq!(
            response.into_inner(),
            vec![
                Criterion {
                    name: String::from("Style"),
                    min: 0,
                    max: 10,
                    weight: Some(15.0)
                },
                Criterion {
                    name: String::from("Style"),
                    min: 10,
                    max: 15,
                    weight: Some(50.0)
                }
            ]
        );
    }
    #[rocket::async_test]
    async fn post_criterion() {
        let criterion = Criterion {
            name: String::from("test"),
            min: 1,
            max: 2,
            weight: Some(80.0),
        };
        let response = super::post_criterion(Json(criterion)).await.unwrap();
        assert_eq!(response, "Saved and index updated.");
    }

    #[rocket::async_test]
    async fn post_criterion_conflict() {
        let criterion = Criterion {
            name: String::from("weather"),
            min: 0,
            max: 15,
            weight: Some(15.0),
        };
        let response = super::post_criterion(Json(criterion)).await;
        assert_eq!(
            response,
            Err(Conflict(String::from("Criterion already exist.")))
        );
    }
}
