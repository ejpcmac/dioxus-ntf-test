//! A web service for notifications.

#![expect(
    clippy::missing_panics_doc,
    clippy::expect_used,
    reason = "that’s a PoC"
)]

use std::sync::{Arc, Mutex};

use axum::{
    Router,
    extract::{Path, State, rejection::JsonRejection},
    http::StatusCode,
    response::IntoResponse,
    response::Json,
    routing::{delete, get, post, put},
};
use axum_extra::extract::WithRejection;
use eyre::Result;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use thiserror::Error;

/// The state of the web service.
#[derive(Debug, Default)]
pub struct AppState {
    /// The notifications.
    pub notifications: IndexMap<usize, Notification>,
}

/// A notification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    /// The notification ID.
    pub id: usize,
    /// The message to show.
    pub message: String,
    /// Has the notification been acknowledged?
    pub ack: bool,
}

/// The request payload for `POST /notifications`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNotificationPayload {
    /// The message to show.
    pub message: String,
}

/// The reply payload for `POST /notifications`.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CreateNotificationResult {
    /// The created notification.
    Notification(Notification),
    /// An error has occurred.
    Error(CreateNotificationError),
}

/// Errors that can occur when creating a notification.
#[derive(Debug, Error, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CreateNotificationError {
    /// The payload is invalid.
    #[error("invalid payload: {0}")]
    PayloadError(String),
}

/// The reply payload for `* /notifications/{id}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationResult {
    /// The notification in case of success.
    Notification(Notification),
    /// An error has occurred.
    Error(ResourceError),
}

/// Errors that can occur when operating on a given resource.
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceError {
    /// The resource has not been found.
    #[error("resource {id} not found")]
    NotFound {
        /// The ID of the missing resource.
        id: usize,
    },
}

impl From<JsonRejection> for CreateNotificationError {
    fn from(value: JsonRejection) -> Self {
        Self::PayloadError(value.to_string())
    }
}

impl IntoResponse for Notification {
    fn into_response(self) -> axum::response::Response {
        Json(NotificationResult::Notification(self)).into_response()
    }
}

impl IntoResponse for CreateNotificationError {
    fn into_response(self) -> axum::response::Response {
        let status = match self {
            Self::PayloadError(_) => StatusCode::BAD_REQUEST,
        };

        (status, Json(CreateNotificationResult::Error(self))).into_response()
    }
}

impl IntoResponse for ResourceError {
    fn into_response(self) -> axum::response::Response {
        let status = match self {
            Self::NotFound { .. } => StatusCode::NOT_FOUND,
        };

        (status, Json(NotificationResult::Error(self))).into_response()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

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
async fn list_notifications(
    State(state): State<Arc<Mutex<AppState>>>,
) -> Json<Vec<Notification>> {
    Json(
        state
            .lock()
            .expect("cannot acquire lock on the state")
            .notifications
            .values()
            .cloned()
            .collect(),
    )
}

/// Gets a notification by its ID.
async fn create_notification(
    State(state): State<Arc<Mutex<AppState>>>,
    WithRejection(Json(payload), _): WithRejection<
        Json<CreateNotificationPayload>,
        CreateNotificationError,
    >,
) -> Result<Notification, ResourceError> {
    let mut state = state.lock().expect("cannot acquire lock on the state");
    let id = state.notifications.keys().last().unwrap_or(&0) + 1;
    let notification = Notification {
        id,
        message: payload.message,
        ack: false,
    };
    state.notifications.insert(id, notification.clone());

    Ok(notification)
}

/// Gets a notification by its ID.
async fn get_notification(
    State(state): State<Arc<Mutex<AppState>>>,
    Path(id): Path<usize>,
) -> Result<Notification, ResourceError> {
    state
        .lock()
        .expect("cannot acquire lock on the state")
        .notifications
        .get(&id)
        .cloned()
        .ok_or(ResourceError::NotFound { id })
}

/// Acknowledges a notification.
async fn ack_notification(
    State(state): State<Arc<Mutex<AppState>>>,
    Path(id): Path<usize>,
) -> Result<Notification, ResourceError> {
    let mut state = state.lock().expect("cannot acquire lock on the state");
    match state.notifications.get_mut(&id) {
        Some(notification) => {
            notification.ack = true;
            Ok(notification.clone())
        }
        None => Err(ResourceError::NotFound { id }),
    }
}

/// Delete a notification.
async fn delete_notification(
    State(state): State<Arc<Mutex<AppState>>>,
    Path(id): Path<usize>,
) -> Result<Notification, ResourceError> {
    state
        .lock()
        .expect("cannot acquire lock on the state")
        .notifications
        .shift_remove(&id)
        .ok_or(ResourceError::NotFound { id })
}
