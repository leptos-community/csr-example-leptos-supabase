use std::ops::Not;
use std::time::Duration;

use leptos::*;

use leptos_router::*;
use leptos_use::storage::{use_local_storage, JsonCodec};

use crate::components::{Home, LogIn, SignUp};
use crate::core::helper::{local_storage, url_hash_to_user};
use crate::core::models::User;

#[component]
pub fn App() -> impl IntoView {
    let (user, set_user, _) = use_local_storage::<User, JsonCodec>("user");
    let show_toast = RwSignal::new(false);
    let toast_text = RwSignal::new(String::new());
    provide_context(Callback::new(move |text: String| {
        toast_text.set(text);
        set_timeout(move || show_toast.set(false), Duration::from_secs(10));
        show_toast.set(true);
    }));
    view! {
        <Router>
            <Routes base="/leptos_supabase_example".to_string()>
                <Route
                    path="/login"
                    view=move || {
                        if user.get_untracked().access_token.is_empty() {
                            view! { <LogIn user=user set_user=set_user/> }
                        } else {
                            view! { <Redirect path="/leptos_supabase_example/"/> }
                        }
                    }
                />

                <Route path="/signup" view=SignUp/>

                <Route
                    path="/redirect"
                    view=move || {
                        let new_user = url_hash_to_user(use_location().hash.get());
                        match new_user {
                            Some(new_user) => {
                                if user.get_untracked().uuid != new_user.uuid {
                                    local_storage().clear().expect("Can't access to local storage");
                                }
                                set_user.set(new_user);
                                view! { <Redirect path="/leptos_supabase_example/"/> }
                            }
                            /// (Some(access_token), None) => _Never happens! This path is for login from google so both tokens always are provided
                            _ => view! { <Redirect path="/leptos_supabase_example/login"/> },
                        }
                    }
                />

                <Route
                    path="/"
                    view=move || {
                        if user.get_untracked().access_token.is_empty().not() {
                            view! { <Home user=user set_user=set_user/> }
                        } else {
                            view! { <Redirect path="/leptos_supabase_example/login"/> }
                        }
                    }
                />

                <Route
                    path="/signup/confirmation"
                    view=move || {
                        view! {
                            <div style="position: absolute; top: 50%; left: 50%; transform: translate(-50%, -50%);font-size:22px;">
                                "Please confirm your email from your inbox"
                            </div>
                        }
                    }
                />

                <Route
                    path="/*"
                    view=move || {
                        view! {
                            // view! { <LogIn user=user set_user=set_user/> }
                            <div style="position: absolute; top: 50%; left: 50%; transform: translate(-50%, -50%);font-size:22px;">
                                "Page Not Found :("
                            </div>
                        }
                    }
                />

            </Routes>
            <div id="toast" class:show=show_toast>
                {toast_text}
            </div>

        </Router>
    }
}

pub fn toast(text: String) {
    use_context::<Callback<String, ()>>().expect("Can't send toast from here").call(text);
}

