use leptos::{html::Input, *};
use leptos_router::*;
use serde_json::json;

use crate::{app::toast, env};
#[component]
pub fn SignUp() -> impl IntoView {
    let email_ref = NodeRef::<Input>::new();
    let pass_ref = NodeRef::<Input>::new();
    let disable_login_btn = RwSignal::new(false);

    let signup = move |email: String, password: String| async move {
        let client = reqwest::Client::new();
        let body = json!({"email":email,"password":password}).to_string();
        let res = client
            .post(env::APP_SIGNUP_URL)
            .body(body)
            .header("apikey", env::APP_API_KEY)
            .send()
            .await;
        match res {
            Ok(res) => {
                if res.status().is_success() {
                    use_navigate()(
                        "/leptos_supabase_example/signup/confirmation",
                        Default::default(),
                    );
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
                        signup(email_ref.get().unwrap().value(), pass_ref.get().unwrap().value())
                            .await;
                        disable_login_btn.set(false);
                    })
                }
            >

                <h1>SignUp</h1>

                <label for="email">
                    Email: <input node_ref=email_ref type="email" name="email" id="email" required/>
                </label>
                <label for="pass">
                    Password:
                    <input
                        node_ref=pass_ref
                        type="password"
                        name="pass"
                        id="pass"
                        required
                        minlength="6"
                    />
                </label>
                <input
                    type="submit"
                    class="primary-button"
                    value="Signup"
                    disabled=disable_login_btn
                />

                <input
                    type="button"
                    class="secondary-button"
                    value="LogIn"
                    on:click=move |_| {
                        use_navigate()("/leptos_supabase_example/login", Default::default());
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

