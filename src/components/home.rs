use std::pin::Pin;

use chrono::Utc;
use futures::{future::join_all, Future};
use leptos::leptos_dom::logging::console_error;
use leptos::{html::Dialog, *};
use leptos_router::*;
use leptos_use::storage::use_local_storage;
use leptos_use::storage::JsonCodec;

use postgrest::Postgrest;
use serde_json::json;
use web_sys::MouseEvent;

use crate::{
    core::{
        helper::{local_storage, refresh_token},
        models::{Company, Job, RefreshTokenError, Status, User},
    },
    env,
};

#[component]
pub fn Home(user: Signal<User>, set_user: WriteSignal<User>) -> impl IntoView {
    let postgrest_client = StoredValue::new(
        Postgrest::new(env::APP_DATABASE_URL).insert_header("apikey", env::APP_API_KEY),
    );
    let (companies, set_companies, _) = use_local_storage::<Vec<Company>, JsonCodec>("companies");

    let update_companies = move |company: &Company| {
        // To trigger companies signal to update
        batch(|| {
            set_companies.update(|f| {
                f.iter_mut().filter(|c| c.date_added == company.date_added).next().and_then(
                    |res| {
                        res.name.set(company.name.get());
                        res.phone.set(company.phone.get());
                        res.jobs.set(company.jobs.get());
                        res.status.set(company.status.get());
                        Some(res)
                    },
                );
            });
        });
    };

    let access_token_expired = RwSignal::new(false);

    let logout_alert = NodeRef::<Dialog>::new();

    let editing_company = RwSignal::new(Option::<Company>::None);

    let input_company_name = RwSignal::new(String::new());
    let input_phone = RwSignal::new(String::new());
    let input_jobs =
        RwSignal::new(vec![(RwSignal::new("".to_string()), RwSignal::new("".to_string()))]);
    let clear_form = move || {
        input_company_name.set(String::new());
        input_phone.set(String::new());
        input_jobs.set(vec![(RwSignal::new("".to_string()), RwSignal::new("".to_string()))]);
    };
    let on_edit_button_clicked = move |company: Company| {
        editing_company.set(Some(company.clone()));

        input_company_name.set(company.name.get());
        input_phone.set(company.phone.get());
        input_jobs.set(
            company
                .jobs
                .get()
                .into_iter()
                .map(|Job { name: a, qualification: b }| (RwSignal::new(a), RwSignal::new(b)))
                .collect::<Vec<_>>(),
        );
    };

    let sync_insert_to_database = move |company: Company| async move {
        let postgrest_client = postgrest_client.get_value();
        let user = user.get();
        company.status.set(Status::SyncingInsert);
        update_companies(&company); // To trigger companies signal to update
        let response = postgrest_client
      .from("companies")
      .auth(user.access_token)
      .insert(
          json!( {"user_id":user.uuid,"name": company.name.get() ,"phone":company.phone.get(),"jobs":company.jobs.get(),"date_added":company.date_added } )
            .to_string(),
          )
      .execute()
      .await;

        match response {
            Ok(response) => {
                if response.status().is_success() {
                    company.status.set(Status::Synced);
                    update_companies(&company); // To trigger companies signal to update
                } else {
                    if response.status().as_u16() == 401 {
                        access_token_expired.set(true);
                        update_companies(&company); // To trigger companies signal to update
                    } else if response.status().as_u16() == 409 {
                        // On Conflict in (added_date, user_id):
                        company.status.set(Status::EditFailed);
                    }
                    company.status.set(Status::InsertFailed);
                    update_companies(&company); // To trigger companies signal to update
                }
            }
            Err(err) => {
                console_error(format!("{err:?}").as_str());
                company.status.set(Status::InsertFailed);
                update_companies(&company); // To trigger companies signal to update
            }
        };
    };
    let sync_edit_to_database = move |company: Company| async move {
        let postgrest_client = postgrest_client.get_value();
        let user = user.get();
        company.status.set(Status::SyncingEdit);
        update_companies(&company); // To trigger companies signal to update
        let response = postgrest_client
      .from("companies")
      .auth(user.access_token)
      .eq("date_added", company.date_added.to_string())
      .update(json!({"user_id":user.uuid,"name":company.name.get(),"phone":company.phone.get(),"jobs":company.jobs.get()}).to_string())
      .execute()
      .await;
        match response {
            Ok(response) => {
                if response.status().is_success() {
                    company.status.set(Status::Synced);
                    update_companies(&company); // To trigger companies signal to update
                } else {
                    if response.status().as_u16() == 401 {
                        access_token_expired.set(true);
                        update_companies(&company); // To trigger companies signal to update
                    }
                    company.status.set(Status::EditFailed);
                    update_companies(&company); // To trigger companies signal to update
                }
            }
            Err(err) => {
                console_error(format!("{err:?}").as_str());
                company.status.set(Status::EditFailed);
                update_companies(&company); // To trigger companies signal to update
            }
        };

        editing_company.set(None);
    };
    let sync_delete_to_database = move |company: Company| async move {
        let postgrest_client = postgrest_client.get_value();
        let user = user.get();
        company.status.set(Status::SyncingDelete);
        update_companies(&company); // To trigger companies signal to update
        let response = postgrest_client
            .from("companies")
            .auth(user.access_token)
            .eq("date_added", company.date_added.to_string())
            .delete()
            .execute()
            .await;
        match response {
            Ok(response) => {
                if response.status().is_success() {
                    set_companies.update(|f| {
                        f.remove(
                            f.iter().position(|e| e.date_added == company.date_added).unwrap(),
                        );
                    });
                } else {
                    if response.status().as_u16() == 401 {
                        access_token_expired.set(true);
                        update_companies(&company); // To trigger companies signal to update
                    }
                    company.status.set(Status::DeleteFailed);
                    update_companies(&company); // To trigger companies signal to update
                }
            }
            Err(err) => {
                console_error(format!("{err:?}").as_str());
                company.status.set(Status::DeleteFailed);
                update_companies(&company); // To trigger companies signal to update
            }
        };
    };

    let logout = move || {
        local_storage().clear().expect("Can't access to local storage");
        set_user.set(User::default());
        use_navigate()("/leptos_supabase_example/login", Default::default());
    };
    let retry_all_faileds = move || {
        spawn_local(async move {
            let futs: Vec<Pin<Box<dyn Future<Output = ()>>>> = companies
                .get()
                .iter()
                .map(move |company| match company.status.get() {
                    Status::SyncingInsert => Box::pin(sync_insert_to_database(company.clone()))
                        as Pin<Box<dyn Future<Output = ()>>>,
                    Status::SyncingEdit => Box::pin(sync_edit_to_database(company.clone()))
                        as Pin<Box<dyn Future<Output = ()>>>,
                    Status::SyncingDelete => Box::pin(sync_delete_to_database(company.clone()))
                        as Pin<Box<dyn Future<Output = ()>>>,
                    Status::InsertFailed => Box::pin(sync_insert_to_database(company.clone()))
                        as Pin<Box<dyn Future<Output = ()>>>,
                    Status::EditFailed => Box::pin(sync_edit_to_database(company.clone()))
                        as Pin<Box<dyn Future<Output = ()>>>,
                    Status::DeleteFailed => Box::pin(sync_delete_to_database(company.clone()))
                        as Pin<Box<dyn Future<Output = ()>>>,
                    Status::Synced => {
                        Box::pin(std::future::ready(())) as Pin<Box<dyn Future<Output = ()>>>
                    }
                })
                .collect::<Vec<_>>();
            join_all(futs).await;
        });
    };

    let logout_btn_clicked = move |event: MouseEvent| {
        event.prevent_default();
        if companies.with(|f| f.iter().any(|c| c.status.get() != Status::Synced)) {
            logout_alert.get().unwrap().show_modal().unwrap_or_default();
        } else {
            logout();
        }
    };

    let init_fetch = move || async move {
        let response = postgrest_client
            .get_value()
            .from("companies")
            .auth(user.get().access_token)
            // .eq("user_id", auth.get_value().uuid) // It is handled on the RLS side
            .select("date_added,name,phone,jobs")
            .order("date_added")
            .execute()
            .await;
        match response {
            Ok(response) => {
                if response.status().is_success() {
                    let comps = response
                        .bytes()
                        .await
                        .ok()
                        .map(|bytes| {
                            serde_json::from_str::<Vec<Company>>(
                                format!("{}", String::from_utf8(bytes.to_vec()).unwrap()).as_str(),
                            )
                            .ok()
                        })
                        .flatten();
                    match comps {
                        Some(new_companies) => {
                            batch(|| {
                                set_companies.update(move |current_companies| {
                                    for new_company in new_companies.iter() {
                                        // Adding the items have created in other devices
                                        if current_companies.iter().any(|current_company| {
                                            current_company.date_added == new_company.date_added
                                        }) == false
                                        {
                                            current_companies.push(new_company.clone());
                                        }
                                        // Replace the same items have edited in other devices AFTER us
                                        // And add the items that were about to insert/edit but the tab was closed
                                        // and the signal wasn't updated but the insert/edit request was sent by the browser. Yes Chrome keeps requests alive.)
                                        if let Some(replacing_company_index) =
                                            current_companies.iter().position(|current_company| {
                                                new_company.date_added == current_company.date_added
                                                    && (current_company.status.get()
                                                        == Status::Synced
                                                        || current_company.status.get()
                                                            == Status::SyncingInsert
                                                        || current_company.status.get()
                                                            == Status::SyncingEdit)
                                            })
                                        {
                                            // Changed and refresh the reactive subfiealds
                                            // The whole companies will be updated too because we are in set_companies.update !
                                            current_companies[replacing_company_index]
                                                .name
                                                .set(new_company.name.get());
                                            current_companies[replacing_company_index]
                                                .phone
                                                .set(new_company.phone.get());
                                            current_companies[replacing_company_index]
                                                .jobs
                                                .set(new_company.jobs.get());
                                            current_companies[replacing_company_index]
                                                .status
                                                .set(new_company.status.get());
                                            // current_companies.remove(replacing_company_index);
                                            // current_companies.push(new_company.clone());
                                        }
                                    }
                                    // Delete the items has have been removed by other devices (Including the ones that were about to be deleted but the tab was closed
                                    // and the signal wasn't updated but the delete request was sent by the browser. Yes Chrome keeps requests alive.)
                                    current_companies.retain(|current_company| {
                                        new_companies.iter().any(|new_company| {
                                            new_company.date_added == current_company.date_added
                                        }) || (current_company.status.get() != Status::Synced
                                            && current_company.status.get()
                                                != Status::SyncingDelete)
                                    });
                                    current_companies.sort_by_key(|f| f.date_added);
                                });
                            });
                            retry_all_faileds();
                        }
                        None => {}
                    }
                } else {
                    if response.status().as_u16() == 401 {
                        access_token_expired.set(true);
                    }
                }
            }
            Err(_) => {}
        }
    };
    Effect::new(move |previous| {
        // To ignore incoming access_token_expired updates when it's already working on it
        if access_token_expired.get() && previous.map(|f| f == false).unwrap_or(true) {
            spawn_local(async move {
                let res = refresh_token(user).await;

                match res {
                    Ok(new_user) => {
                        set_user.set(new_user);
                        spawn_local(async move {
                            init_fetch().await;
                        });
                    }
                    Err(err) => match err {
                        RefreshTokenError::NetworkError => {}
                        RefreshTokenError::JsonParseError => {}
                        RefreshTokenError::RefreshTokenExpirationError => {
                            set_user.set(User::default());
                            use_navigate()("/leptos_supabase_example/login", Default::default())
                        }
                        RefreshTokenError::UnknownError => {}
                    },
                }
                access_token_expired.set(false);
            })
        }
        access_token_expired.get()
    });

    spawn_local(async move { init_fetch().await });

    view! {
        <div id="main">

            <div id="main-column">
                <div id="user-info">
                    <h1 id="email">{move || user.get().email}</h1>
                    <button type="button" on:click=logout_btn_clicked id="logout-button">

                        Log Out
                    </button>
                </div>
                <form id="input-form">
                    <label for="company-name">"Company Name:"</label>
                    <input
                        type="text"
                        prop:value=input_company_name
                        on:input=move |event| input_company_name.set(event_target_value(&event))
                        name="company-name"
                        id="company-name"
                    />
                    <div class="gap"></div>
                    <label for="phone">Phone:</label>
                    <input
                        type="phone"
                        prop:value=input_phone
                        on:input=move |event| input_phone.set(event_target_value(&event))
                        name="phone"
                        id="phone"
                    />
                    <div class="gap"></div>
                    <table style="width:100%;">
                        <thead>
                            <tr>
                                <td>"Job Title:"</td>
                                <td>"Qualification:"</td>
                            </tr>
                        </thead>
                        <tbody>
                            {move || {
                                input_jobs
                                    .get()
                                    .into_iter()
                                    .enumerate()
                                    .map(|(index, job)| {
                                        view! {
                                            <tr>
                                                <td>
                                                    <input
                                                        prop:value=job.0
                                                        on:input=move |event| job.0.set(event_target_value(&event))
                                                        type="text"
                                                        name="job"
                                                        class="job"
                                                    />
                                                </td>
                                                <td>
                                                    <input
                                                        prop:value=job.1
                                                        on:input=move |event| job.1.set(event_target_value(&event))
                                                        type="text"
                                                        name="qualification"
                                                        class="qualification"
                                                    />
                                                </td>
                                                <td>
                                                    {move || {
                                                        (input_jobs.get().len() > 1)
                                                            .then_some(
                                                                view! {
                                                                    <button
                                                                        class="gg-close"
                                                                        on:click=move |_| {
                                                                            input_jobs
                                                                                .update(|f| {
                                                                                    f.remove(index);
                                                                                })
                                                                        }

                                                                        style="display:inline;"
                                                                    >
                                                                        ""
                                                                    </button>
                                                                },
                                                            )
                                                    }}

                                                </td>
                                            </tr>
                                        }
                                    })
                                    .collect_view()
                            }}
                            <div class="gap"></div> <tr>
                                <td colspan="2">
                                    <input
                                        type="button"
                                        style="width:100%;"
                                        id="add"
                                        on:click=move |_| {
                                            input_jobs
                                                .update(|f| {
                                                    f.push((
                                                        RwSignal::new("".to_string()),
                                                        RwSignal::new("".to_string()),
                                                    ))
                                                })
                                        }

                                        value="ADD JOB"
                                    />
                                </td>
                            </tr>
                        </tbody>
                    </table>
                    <div class="gap"></div>
                    <div id="send-cancel">
                        <input
                            type="button"
                            style="display:inline;"
                            id="send"
                            class="primary-button"
                            value=move || {
                                editing_company
                                    .with(|f| {
                                        f.as_ref().and_then(|_| Some("Edit")).unwrap_or("Send")
                                    })
                            }

                            on:click=move |_| {
                                if editing_company.with(|f| f.is_none()) {
                                    let company = Company {
                                        name: RwSignal::new(input_company_name.get()),
                                        phone: RwSignal::new(input_phone.get()),
                                        jobs: RwSignal::new(
                                            input_jobs
                                                .get()
                                                .iter()
                                                .map(|job_signals| Job {
                                                    name: job_signals.0.get(),
                                                    qualification: job_signals.1.get(),
                                                })
                                                .collect::<Vec<_>>(),
                                        ),
                                        date_added: Utc::now(),
                                        status: RwSignal::new(Status::SyncingInsert),
                                    };
                                    set_companies
                                        .update(|f| {
                                            f.push(company.clone());
                                        });
                                    spawn_local(async move {
                                        sync_insert_to_database(company).await;
                                    });
                                } else if editing_company
                                    .with(|f| {
                                        f.as_ref().unwrap().status.get() == Status::InsertFailed
                                    })
                                {
                                    editing_company
                                        .with(|f| {
                                            f.as_ref().unwrap().name.set(input_company_name.get());
                                            f.as_ref().unwrap().phone.set(input_phone.get());
                                            f.as_ref()
                                                .unwrap()
                                                .jobs
                                                .set(
                                                    input_jobs
                                                        .get()
                                                        .iter()
                                                        .map(|job_signals| Job {
                                                            name: job_signals.0.get(),
                                                            qualification: job_signals.1.get(),
                                                        })
                                                        .collect::<Vec<_>>(),
                                                );
                                        });
                                    update_companies(&editing_company.get().unwrap());
                                    spawn_local(async move {
                                        sync_insert_to_database(editing_company.get().unwrap())
                                            .await;
                                    });
                                } else {
                                    editing_company
                                        .with(|f| {
                                            f.as_ref().unwrap().name.set(input_company_name.get());
                                            f.as_ref().unwrap().phone.set(input_phone.get());
                                            f.as_ref()
                                                .unwrap()
                                                .jobs
                                                .set(
                                                    input_jobs
                                                        .get()
                                                        .iter()
                                                        .map(|job_signals| Job {
                                                            name: job_signals.0.get(),
                                                            qualification: job_signals.1.get(),
                                                        })
                                                        .collect::<Vec<_>>(),
                                                );
                                        });
                                    update_companies(&editing_company.get().unwrap());
                                    spawn_local(async move {
                                        sync_edit_to_database(editing_company.get().unwrap()).await;
                                    });
                                }
                                clear_form();
                            }
                        />

                        <input
                            type="button"
                            value="Cancel"
                            style="display:inline;"
                            id="cancel"
                            class="error-button"
                            on:click=move |_| {
                                if editing_company.with(|f| f.is_some()) {
                                    editing_company.set(None);
                                }
                                clear_form();
                            }
                        />

                    </div>
                </form>
            </div>

            <Show when=move || companies.with(|f| f.is_empty() == false)>
                <table class="styled-table">
                    <thead>
                        <tr>
                            <th class="status-cell">Status</th>
                            <th>Company Name</th>
                            <th>Phone</th>
                            <th>Jobs</th>
                            <th></th>
                        </tr>
                    </thead>
                    <tbody>
                        <For
                            each=move || companies.get()
                            key=move |company| company.date_added
                            let:company
                        >

                            {
                                let stored_company = store_value(company);
                                move || {
                                    view! {
                                        <tr>
                                            <td class="status-cell">
                                                {move || {
                                                    stored_company.get_value().status.get().to_string()
                                                }}
                                                {move || {
                                                    match stored_company.get_value().status.get() {
                                                        Status::InsertFailed => {
                                                            Some(
                                                                view! {
                                                                    <button
                                                                        type="button"
                                                                        on:click=move |event| {
                                                                            event.prevent_default();
                                                                            spawn_local(async move {
                                                                                sync_insert_to_database(stored_company.get_value()).await;
                                                                            });
                                                                        }
                                                                    >

                                                                        "Retry Insert"
                                                                    </button>
                                                                },
                                                            )
                                                        }
                                                        Status::EditFailed => {
                                                            Some(
                                                                view! {
                                                                    <button
                                                                        type="button"
                                                                        on:click=move |event| {
                                                                            event.prevent_default();
                                                                            spawn_local(async move {
                                                                                sync_edit_to_database(stored_company.get_value()).await;
                                                                            });
                                                                        }
                                                                    >

                                                                        "Retry Edit"
                                                                    </button>
                                                                },
                                                            )
                                                        }
                                                        Status::DeleteFailed => {
                                                            Some(
                                                                view! {
                                                                    <button
                                                                        type="button"
                                                                        on:click=move |event| {
                                                                            event.prevent_default();
                                                                            spawn_local(async move {
                                                                                sync_delete_to_database(stored_company.get_value()).await;
                                                                            });
                                                                        }
                                                                    >

                                                                        "Retry Delete"
                                                                    </button>
                                                                },
                                                            )
                                                        }
                                                        _ => None,
                                                    }
                                                }}

                                            </td>
                                            <td>{stored_company.get_value().name}</td>
                                            <td>{stored_company.get_value().phone}</td>
                                            <td class="jobs-cell">
                                                {move || {
                                                    format!("{:?}", stored_company.get_value().jobs.get())
                                                }}

                                            </td>
                                            <td>

                                                <button
                                                    type="button"
                                                    class="edit-button"
                                                    on:click=move |_| {
                                                        on_edit_button_clicked(stored_company.get_value());
                                                    }

                                                    disabled=move || {
                                                        stored_company
                                                            .get_value()
                                                            .status
                                                            .with(|f| {
                                                                *f == Status::SyncingInsert || *f == Status::SyncingDelete
                                                                    || *f == Status::SyncingEdit
                                                            })
                                                    }
                                                >

                                                    Edit
                                                </button>
                                                <button
                                                    type="button"
                                                    class="delete-button"
                                                    on:click=move |_| {
                                                        spawn_local(async move {
                                                            sync_delete_to_database(stored_company.get_value()).await
                                                        });
                                                    }

                                                    disabled=move || {
                                                        editing_company
                                                            .with(|f| match f {
                                                                None => false,
                                                                Some(ref f) => {
                                                                    stored_company
                                                                        .with_value(|data| data.date_added == f.date_added)
                                                                }
                                                            })
                                                            || stored_company
                                                                .with_value(|c| {
                                                                    c.status.get() == Status::SyncingInsert
                                                                        || c.status.get() == Status::SyncingEdit
                                                                        || c.status.get() == Status::SyncingDelete
                                                                })
                                                    }
                                                >

                                                    Delete
                                                </button>

                                            </td>
                                        </tr>
                                    }
                                }
                            }

                        </For>
                    </tbody>
                </table>
            </Show>
            <dialog class="dialog" node_ref=logout_alert>
                <div class="dialog-inner-box">
                    <h1>"⚠ You have unsynced data"</h1>
                    <p>
                        "If you log out now, you will lose your unsynced data." <br/>
                        " Do you want to sync your changes before logging out?"
                    </p>
                    <input type="button" class="primary-button" value="Sync and Log Out"/>
                    <input
                        type="button"
                        class="error-button"
                        value="Log Out Without Syncing"
                        on:click=move |_| logout()
                    />
                    <input
                        type="button"
                        class="secondary-button"
                        value="Cancel"
                        on:click=move |_| {
                            logout_alert.get().unwrap().close();
                        }
                    />

                </div>

            </dialog>
        </div>
    }
}

