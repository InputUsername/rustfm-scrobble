// Last.fm scrobble API 2.0 client

use std::collections::HashMap;
use std::io::Read;
use reqwest::{Client, StatusCode};
use serde_json;

use auth::AuthCredentials;
use dto::AuthResponseDto;

pub struct LastFmClient {
    auth: AuthCredentials,
    http_client: Client
}

impl LastFmClient {

    pub fn new(api_key: String, api_secret: String) -> LastFmClient {
        let partial_auth = AuthCredentials::new_partial(api_key, api_secret);
        let http_client = Client::new().unwrap();

        LastFmClient{
            auth: partial_auth,
            http_client: http_client
        }
    }

    pub fn set_user_credentials(&mut self, username: String, password: String) {
        self.auth.set_user_credentials(username, password);
    }

    pub fn authenticate(&mut self) -> Result<(), String> {
        if !self.auth.is_valid() {
            return Err("Invalid authentication parameters".to_string())
        }

        let params = self.auth.get_auth_request_params();

        match self.send_request("auth.getMobileSession", params) {
            Ok(body) => {
                let decoded: AuthResponseDto = serde_json::from_str(body.as_str()).unwrap();
                self.auth.set_session_key(decoded.session.key);

                Ok(())
            },
            Err(msg) => Err(format!("Authentication failed: {}", msg))
        }
    }

    pub fn send_authenticated_request(&self, object: &str, params: &HashMap<&str, String>) -> Result<String, String> {
        if !self.auth.is_authenticated() {
            return Err("Not authenticated".to_string())
        }

        let mut req_params = self.auth.get_request_params();
        for (k, v) in params {
            req_params.insert(k, v.clone());
        }

        self.send_request(object, req_params)
    }

    fn send_request(&self, object: &str, params: HashMap<&str, String>) -> Result<String, String> {
        let url = "https://ws.audioscrobbler.com/2.0/?format=json";
        let signature = self.auth.get_signature(object, &params);

        let mut req_params = params.clone();
        req_params.insert("method", object.to_string());
        req_params.insert("api_sig", signature);

        let result = self.http_client.post(url)
            .form(&req_params)
            .send();

        match result {
            Ok(mut resp) => {
                let status = *resp.status();
                if status != StatusCode::Ok {
                    return Err(format!("Non Success status ({})", status));
                }

                let mut resp_body = String::new();
                match resp.read_to_string(&mut resp_body) {
                    Ok(_) => return Ok(resp_body),
                    Err(_) => return Err("Failed to read response body".to_string())
                }
            },
            Err(msg) => return Err(format!("{}", msg))
        }
    }

}
