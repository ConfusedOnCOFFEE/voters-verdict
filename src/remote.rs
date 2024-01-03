use rocket::{
    debug, error,
    tokio::{fs::File, io::AsyncReadExt},
};
#[rocket::async_trait]
pub trait RemoteIndex {
    async fn prepare(&self, uri: &str) -> Result<Vec<String>, String> {
        debug!("Remote URL: {:?}", uri);
        match std::env::var("VOTERS_VERDICT_SELF_CERT") {
            Ok(cert) => {
                let mut buf = Vec::new();
                match File::open(cert).await {
                    Ok(mut f) => {
                        let _ = match f.read_to_end(&mut buf).await {
                            Ok(_) => {}
                            Err(e) => error!("{:?}", e),
                        };

                        match reqwest::Certificate::from_der(&buf) {
                            Ok(c) => {
                                let client_builder =
                                    reqwest::Client::builder().add_root_certificate(c);
                                self.build_client(&uri, Some(client_builder)).await
                            }
                            Err(_e) => {
                                error!("Self signed cert couldn't be loaded.");
                                self.build_client(&uri, None).await
                            }
                        }
                    }
                    Err(_) => {
                        error!("Self signed cert couldn't be loaded.");
                        self.build_client(&uri, None).await
                    }
                }
            }
            Err(_) => self.build_client(&uri, None).await,
        }
    }
    async fn build_client(
        &self,
        uri: &str,
        client_builder: Option<reqwest::ClientBuilder>,
    ) -> Result<Vec<String>, String> {
        let custom_client = match client_builder {
            Some(builder) => builder,
            None => reqwest::Client::builder(),
        };
        let client = match custom_client.build() {
            Ok(c) => c,
            Err(_) => reqwest::Client::new(),
        };
        self.http_get(uri, client).await
    }
    async fn http_get(&self, uri: &str, client: reqwest::Client) -> Result<Vec<String>, String> {
        let get_client = client.get(uri);
        match std::env::var("VOTERS_VERDICT_REMOTE_CREDENTIALS") {
            Ok(t) => match std::env::var("VOTERS_VERDICT_REMOTE_AUTH") {
                Ok(auths) => {
                    debug!("Detected remote auth: {}", auths);
                    match auths.as_str() {
                        "bearer" => {
                            debug!("Detected bearer auth");
                            self.get(get_client.bearer_auth("Bearer ".to_owned() + &t))
                                .await
                        }
                        "basic" => {
                            debug!("Detected basic auth");
                            let credentails = auths.split_once(':');
                            match credentails {
                                Some((user, pw)) => {
                                    self.get(get_client.basic_auth(user, Some(pw))).await
                                }
                                None => {
                                    error!("Credentials for basic auth not provided");
                                    self.get(get_client).await
                                }
                            }
                        }
                        _ => self.get(get_client).await,
                    }
                }
                Err(_) => self.get(get_client).await,
            },
            Err(_) => self.get(get_client).await,
        }
    }
    async fn get(&self, request_builder: reqwest::RequestBuilder) -> Result<Vec<String>, String> {
        match request_builder
            .header(
                reqwest::header::ACCEPT,
                reqwest::header::HeaderValue::from_static("application/json"),
            )
            .send()
            .await
        {
            Ok(r) => {
                debug!("Raw request response: {:?}", r);
                match r.json::<Vec<String>>().await {
                    Ok(d) => Ok(d),
                    Err(e) => {
                        error!("{:?}", e);
                        Ok(vec![])
                    }
                }
            }
            Err(e) => {
                error!("{:?}", e);
                Ok(vec![])
            }
        }
    }
}

impl RemoteIndex for EmojiCategories {}
impl RemoteIndex for Criterion {}
impl RemoteIndex for Criteria {}
impl RemoteIndex for Candidate {}
impl RemoteIndex for CastBallots {}
impl RemoteIndex for Voting {}
impl RemoteIndex for Votings {}
