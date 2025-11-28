//! An application that can receive and show notifications.

use dioxus::prelude::*;
use ntf_api::ApiClient;

/// Version info.
const VERSION_WITH_GIT: &str = env!("VERSION_WITH_GIT");
/// CSS for the app.
const CSS: Asset = asset!("/assets/app.css");
/// API endpoint.
const ENDPOINT: &str = "http://localhost:3000";

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: CSS }

        Version {}
        Notifications {}
    }
}

#[component]
fn Version() -> Element {
    rsx! {
        div { class: "absolute top-2 right-2 badge badge-sm badge-ghost", "{VERSION_WITH_GIT}" }
    }
}

#[component]
fn Notifications() -> Element {
    let fetch_notifications = async move || {
        ApiClient::new(ENDPOINT)
            .list_notifications()
            .await
            .unwrap_or_default()
    };

    let mut notifications = use_resource(fetch_notifications);

    let mut update_notifications = async move || {
        notifications.set(Some(fetch_notifications().await));
    };

    let reload_notifications = move |_| async move {
        update_notifications().await;
    };

    let ack_notification = move |id| async move {
        let _ignored = ApiClient::new(ENDPOINT).ack_notification(id).await;
        update_notifications().await;
    };

    let delete_notification = move |id| async move {
        let _ignored = ApiClient::new(ENDPOINT).delete_notification(id).await;
        update_notifications().await;
    };

    rsx! {
        ul { class: "list bg-base-100 rounded-box shadow-md",
            li { class: "p-4 pb-2 text-xs tracking-wide",
                button { class: "btn btn-primary", onclick: reload_notifications, "Reload notifications" }
            }

            if let Some(ntfs) = notifications.read().as_deref() {
                for ntf in ntfs.iter().cloned() {
                    li { class: "list-row",
                        h2 { class: "text-4xl", "#{ntf.id}" }
                        p { "{ntf.message}" }
                        if ntf.ack {
                            button { class: "btn btn-active btn-success", "✓" }
                        } else {

                            button {
                                class: "btn btn-soft btn-success",
                                onclick: move |_| ack_notification(ntf.id),
                                "✓"
                            }
                        }
                        button {
                            class: "btn btn-soft btn-error",
                            onclick: move |_| delete_notification(ntf.id),
                            "✗"
                        }
                    }
                }
            } else {
                li { class: "list-row",
                    h2 { class: "text-2xl", "Loading..." }
                }
            }
        }
    }
}
