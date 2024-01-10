use leptos::{html::Input, *};
use leptos_router::*;

use reqwest;
use serde_json::{json, Value};

use crate::{
    app::toast,
    core::{
        helper::{access_token_to_uuid_email, local_storage},
        models::User,
    },
    env,
};

#[component]
pub fn LogIn(user: Signal<User>, set_user: WriteSignal<User>) -> impl IntoView {
    let email_ref = NodeRef::<Input>::new();
    let pass_ref = NodeRef::<Input>::new();
    let disable_login_btn = RwSignal::new(false);
    let login = move |email: String, password: String| async move {
        let client = reqwest::Client::new();
        let body = json!({"email":email,"password":password}).to_string();
        let res = client
            .post(env::APP_MANUAL_LOGIN_URL)
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
                                .map(|access_token| {
                                    access_token_to_uuid_email(access_token.as_str())
                                })
                                .flatten();
                            match (access_token, refresh_token, uuid_email) {
                                (Some(access_token), Some(refresh_token), Some((uuid, email))) => {
                                    if user.get().uuid != uuid {
                                        local_storage()
                                            .clear()
                                            .expect("Can't access to local storage");
                                    }
                                    set_user.set(User { access_token, refresh_token, uuid, email });
                                    use_navigate()("/leptos_supabase_example/", Default::default());
                                }
                                _ => {
                                    toast(String::from("User token is not valid"));
                                }
                            }
                        }
                        None => {
                            toast(String::from("User token is not valid"));
                        }
                    }
                } else {
                    toast(format!(
                        "Login Failed. Response message: {}",
                        res.status().canonical_reason().unwrap_or("Nothing")
                    ));
                }
            }
            Err(err) => {
                toast(format!("Login Failed. Error: {}", err.to_string()));
            }
        }
    };
    view! {
        <div id="login-container">
            <form
                id="login-signup-form"
                on:submit=move |event| {
                    event.prevent_default();
                    spawn_local(async move {
                        disable_login_btn.set(true);
                        login(email_ref.get().unwrap().value(), pass_ref.get().unwrap().value())
                            .await;
                        disable_login_btn.set(false);
                    });
                }
            >

                <h1>Login</h1>
                <label for="email">
                    Email: <input node_ref=email_ref type="email" name="email" id="email" required/>
                </label>
                <label for="pass">
                    Password:
                    <input node_ref=pass_ref type="password" name="pass" id="pass" required/>
                </label>
                <input
                    type="submit"
                    class="primary-button"
                    value="LogIn"
                    disabled=disable_login_btn
                />
                <input
                    type="button"
                    class="secondary-button"
                    value="SignUp"
                    on:click=move |_| {
                        use_navigate()("/leptos_supabase_example/signup", Default::default());
                    }
                />

                <div class="or">OR</div>

                <a type="button" class="login-with-google-btn" href=env::APP_GOOGLE_LOGIN_URL>
                    "Continue with Google"
                </a>
            </form>

        </div>
    }
}

