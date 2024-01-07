use rocket::{response::status::Unauthorized, Responder};
/////////////////////////////////////////////
//                                         //
//        VOTE-ERROR-KIND                  //
//                                         //
/////////////////////////////////////////////

#[derive(Debug)]
pub enum VoteErrorKind<'r> {
    MissingField(MissingField<'r>),
    Serialize(rocket::serde::json::serde_json::Error),
    IO(std::io::Error),
    Conflict(String),
    NotFound(String),
    Unauthorized(Unauthorized<String>),
    #[cfg(feature = "sqlx_sqlite")]
    DB(rocket_db_pools::sqlx::Error),
    Internal(String),
}

pub enum FromErrorKind {
    DB(String),
    Serialize(String),
    MultipleRows(bool),
}
impl<'r> From<FromErrorKind> for VoteErrorKind<'r> {
    fn from(error: FromErrorKind) -> Self {
        match error {
            FromErrorKind::DB(b) => VoteErrorKind::Internal(b),
            FromErrorKind::Serialize(b) => VoteErrorKind::Internal(b),
            FromErrorKind::MultipleRows(b) => match b {
                true => {
                    VoteErrorKind::Conflict(String::from("Hit multiple rows.Not detailed enough"))
                }
                false => VoteErrorKind::NotFound(String::from("Doesnt exist.")),
            },
        }
    }
}
impl<'r> ToString for VoteErrorKind<'r> {
    fn to_string(&self) -> String {
        match self {
            VoteErrorKind::Serialize(e) => e.to_string(),
            VoteErrorKind::MissingField(e) => e.to_string(),
            VoteErrorKind::IO(e) => e.to_string(),
            VoteErrorKind::Conflict(e) => e.to_string(),
            VoteErrorKind::NotFound(e) => e.to_string(),
            VoteErrorKind::Unauthorized(e) => e.0.clone(),
            #[cfg(feature = "sqlx_sqlite")]
            VoteErrorKind::DB(e) => e.to_string(),
            VoteErrorKind::Internal(e) => e.to_string(),
        }
    }
}

impl<'r> From<std::io::Error> for VoteErrorKind<'r> {
    fn from(error: std::io::Error) -> Self {
        VoteErrorKind::IO(error)
    }
}

impl<'r> From<rocket::serde::json::serde_json::Error> for VoteErrorKind<'r> {
    fn from(error: rocket::serde::json::serde_json::Error) -> Self {
        VoteErrorKind::Serialize(error)
    }
}
#[cfg(feature = "sqlx_sqlite")]
impl<'r> From<rocket_db_pools::sqlx::Error> for VoteErrorKind<'r> {
    fn from(error: rocket_db_pools::sqlx::Error) -> Self {
        VoteErrorKind::DB(error)
    }
}
#[derive(Responder, Debug)]
#[response(content_type = "application/json")]
pub struct MissingField<'r> {
    field: &'r str,
}
impl<'r> ToString for MissingField<'r> {
    fn to_string(&self) -> String {
        self.field.to_string()
    }
}

impl<'r> std::str::FromStr for VoteErrorKind<'r> {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains("failure") {
            Ok(VoteErrorKind::Internal(String::from("Harcore failure...")))
        } else {
            Err(String::from("Internal error..."))
        }
    }
}
impl<'r> From<<Self as std::str::FromStr>::Err> for VoteErrorKind<'r> {
    fn from(_err: <Self as std::str::FromStr>::Err) -> Self {
        VoteErrorKind::Internal(String::from("Harcore failure..."))
    }
}
