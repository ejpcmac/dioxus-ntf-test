//! A web service for notifications.

#![expect(
    clippy::missing_panics_doc,
    clippy::expect_used,
    reason = "that’s a PoC"
)]

use std::sync::{Arc, Mutex};

use axum::{
    Router,
    extract::{Path, State},
    response::Json,
    routing::{delete, get, post, put},
};
use axum_extra::extract::WithRejection;
use eyre::Result;
use indexmap::IndexMap;
use serde_json::{Value, json};

use ntf_api_types::{
    CreateNotificationError, CreateNotificationPayload, Notification,
    ResourceError,
};
use ntf_poc_helpers::tracing::LogResult as _;

/// The state of the web service.
#[derive(Debug, Default)]
pub struct AppState {
    /// The notifications.
    pub notifications: IndexMap<usize, Notification>,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt().init();

    let state = Arc::new(Mutex::new(AppState::default()));

    let app = Router::new()
        .route("/status", get(status))
        .route("/notifications", get(list_notifications))
        .route("/notifications", post(create_notification))
        .route("/notifications/{id}", get(get_notification))
        .route("/notifications/{id}", put(ack_notification))
        .route("/notifications/{id}", delete(delete_notification))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Returns the status.
async fn status() -> Json<Value> {
    Json(json!({"status": "ok"}))
}

/// Lists the notifications.
#[tracing::instrument(skip(state))]
async fn list_notifications(
    State(state): State<Arc<Mutex<AppState>>>,
) -> Json<Vec<Notification>> {
    let notifications = state
        .lock()
        .expect("poisoned lock")
        .notifications
        .values()
        .cloned()
        .collect();

    tracing::info!(?notifications, "LIST");
    Json(notifications)
}

/// Gets a notification by its ID.
#[tracing::instrument(skip(state))]
async fn create_notification(
    State(state): State<Arc<Mutex<AppState>>>,
    WithRejection(Json(payload), _): WithRejection<
        Json<CreateNotificationPayload>,
        CreateNotificationError,
    >,
) -> Result<Notification, ResourceError> {
    let mut state = state.lock().expect("poisoned lock");
    let id = state.notifications.keys().last().unwrap_or(&0) + 1;
    let notification = Notification {
        id,
        message: payload.message,
        ack: false,
    };
    state.notifications.insert(id, notification.clone());

    tracing::info!(?notification, "CREATE");
    Ok(notification)
}

/// Gets a notification by its ID.
#[tracing::instrument(skip(state))]
async fn get_notification(
    State(state): State<Arc<Mutex<AppState>>>,
    Path(id): Path<usize>,
) -> Result<Notification, ResourceError> {
    let notification = state
        .lock()
        .expect("poisoned lock")
        .notifications
        .get(&id)
        .cloned()
        .ok_or(ResourceError::NotFound { id })
        .log_err()?;

    tracing::info!(?notification, "GET");
    Ok(notification)
}

/// Acknowledges a notification.
#[tracing::instrument(skip(state))]
async fn ack_notification(
    State(state): State<Arc<Mutex<AppState>>>,
    Path(id): Path<usize>,
) -> Result<Notification, ResourceError> {
    let mut state = state.lock().expect("poisoned lock");
    match state.notifications.get_mut(&id) {
        Some(notification) => {
            notification.ack = true;
            tracing::info!(?notification, "ACK");
            Ok(notification.clone())
        }
        None => Err(ResourceError::NotFound { id }).log_err(),
    }
}

/// Delete a notification.
#[tracing::instrument(skip(state))]
async fn delete_notification(
    State(state): State<Arc<Mutex<AppState>>>,
    Path(id): Path<usize>,
) -> Result<Notification, ResourceError> {
    let notification = state
        .lock()
        .expect("poisoned lock")
        .notifications
        .shift_remove(&id)
        .ok_or(ResourceError::NotFound { id })
        .log_err()?;

    tracing::info!(?notification, "DELETE");
    Ok(notification)
}
