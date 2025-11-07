//! Types for the notification API.

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[cfg(feature = "axum")]
use axum::{
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};

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

#[cfg(feature = "axum")]
impl From<JsonRejection> for CreateNotificationError {
    fn from(value: JsonRejection) -> Self {
        Self::PayloadError(value.to_string())
    }
}

#[cfg(feature = "axum")]
impl IntoResponse for Notification {
    fn into_response(self) -> Response {
        Json(NotificationResult::Notification(self)).into_response()
    }
}

#[cfg(feature = "axum")]
impl IntoResponse for CreateNotificationError {
    fn into_response(self) -> Response {
        let status = match self {
            Self::PayloadError(_) => StatusCode::BAD_REQUEST,
        };

        (status, Json(CreateNotificationResult::Error(self))).into_response()
    }
}

#[cfg(feature = "axum")]
impl IntoResponse for ResourceError {
    fn into_response(self) -> Response {
        let status = match self {
            Self::NotFound { .. } => StatusCode::NOT_FOUND,
        };

        (status, Json(NotificationResult::Error(self))).into_response()
    }
}
