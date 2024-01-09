use crate::{core::models::RefreshTokenError, core::models::User, env};
use base64::{self, Engine};
use leptos::{Signal, SignalGet};
use serde_json::{json, Value};
use web_sys::Storage;

#[inline]
pub fn local_storage() -> Storage {
    web_sys::window()
        .expect("Can't access to the window")
        .local_storage()
        .expect("Can't access to local storage")
        .expect("Can't access to local storage")
}

pub fn url_hash_to_user(mut url_hash: String) -> Option<User> {
    if url_hash.is_empty() {
        return None;
    }
    let mut access_token = None;
    let mut refresh_token = None;
    url_hash.remove(0);
    for q in url_hash.split("&") {
        let Some((key, value)) = q.split_once("=") else {
            break;
        };
        if key == "access_token" {
            access_token = Some(value.to_owned());
        } else if key == "refresh_token" {
            refresh_token = Some(value.to_owned());
        }
    }
    let uuid_email = access_token
        .as_ref()
        .map(|access_token| access_token_to_uuid_email(access_token.as_str()))
        .flatten();
    match (uuid_email, access_token, refresh_token) {
        (Some((uuid, email)), Some(access_token), Some(refresh_token)) => {
            Some(User { uuid, email, access_token, refresh_token })
        }
        _ => None,
    }
}
pub fn access_token_to_uuid_email(token: &str) -> Option<(String, String)> {
    if token.is_empty() {
        return None;
    }
    let output_size = base64::decoded_len_estimate(token.len());
    let mut payload_buffer = Vec::<u8>::with_capacity(output_size);
    let payload_base64 = token.split(".").nth(1)?;
    base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode_vec(payload_base64, &mut payload_buffer)
        .ok()?;
    let payload_json: Value = serde_json::from_slice(&payload_buffer[..]).ok()?;
    let uuid = payload_json.get("sub")?.as_str()?.to_owned();
    let email = payload_json.get("email")?.as_str()?.to_owned();
    Some((uuid, email))
}

pub async fn refresh_token(user: Signal<User>) -> Result<User, RefreshTokenError> {
    let client = reqwest::Client::new();
    let body = json!({"refresh_token": user.get().refresh_token}).to_string();
    let res = client
        .post(env::APP_REFRESH_TOKEN_URL)
        .body(body)
        .header("apikey", env::APP_API_KEY)
        .send()
        .await;
    match res {
        Ok(res) => {
            if res.status().is_success() {
                let user_details = res
                    .bytes()
                    .await
                    .ok()
                    .map(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
                    .flatten();

                match user_details {
                    Some(user_details) => {
                        let access_token = user_details
                            .get("access_token")
                            .as_ref()
                            .map(|access_token| access_token.as_str())
                            .flatten()
                            .map(str::to_string);
                        let refresh_token = user_details
                            .get("refresh_token")
                            .as_ref()
                            .map(|refresh_token| refresh_token.as_str())
                            .flatten()
                            .map(str::to_string);
                        let uuid_email = access_token
                            .as_ref()
                            .map(|access_token| access_token_to_uuid_email(access_token.as_str()))
                            .flatten();

                        match (access_token, refresh_token, uuid_email) {
                            (Some(access_token), Some(refresh_token), Some((uuid, email))) => {
                                return Ok(User { access_token, uuid, refresh_token, email });
                            }
                            // When server decides to just renew the accesss token not the refresh token:
                            (Some(access_token), None, Some((uuid, email))) => {
                                // local_storage().set("token", access_token).expect("Can't write to local storage");
                                return Ok(User {
                                    access_token,
                                    uuid,
                                    email,
                                    refresh_token: user.get().refresh_token,
                                });
                            }
                            // When server expires the refresh token and user needs to relogin (It hardly happens because then the response wouldn't be successfull at all):
                            (None, None, None) => {
                                return Err(RefreshTokenError::RefreshTokenExpirationError);
                            }
                            _ => {
                                return Err(RefreshTokenError::UnknownError);
                            }
                        }
                    }
                    // Parse problem!
                    None => {
                        return Err(RefreshTokenError::JsonParseError);
                    }
                }
            } else {
                if res.status().as_u16() == 401
                    && res
                        .headers()
                        .get("Www-Authenticate")
                        .and_then(|f| f.to_str().ok().map(|f| f.contains("expired")))
                        .unwrap_or(false)
                {
                    // When server expires the refresh token and user needs to relogin (It can happen!):
                    return Err(RefreshTokenError::RefreshTokenExpirationError);
                } else {
                    // Regular internet problems
                    return Err(RefreshTokenError::NetworkError);
                }
            }
        }
        Err(_) => {
            return Err(RefreshTokenError::NetworkError);
        }
    }
}

