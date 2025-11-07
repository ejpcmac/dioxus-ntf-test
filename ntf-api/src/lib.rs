//! Client library for the notification API.

#![expect(
    clippy::missing_panics_doc,
    clippy::expect_used,
    reason = "that’s a PoC"
)]

use ntf_api_types::{
    CreateNotificationPayload, CreateNotificationResult, Notification,
    NotificationResult, ResourceError,
};
use ntf_poc_helpers::tracing::LogResult as _;
use reqwest::Client;
use serde_json::Value;
use thiserror::Error;

/// API client for the notification web service.
#[derive(Debug)]
pub struct ApiClient {
    /// The base URL of the API.
    base_url: String,
    /// The reqwest client.
    client: Client,
}

/// Errors that can occur when making API calls.
#[derive(Debug, Error)]
pub enum ApiError {
    /// An error occurend during the API request.
    #[error("An error occurrend during the API request.")]
    RequestError(#[source] reqwest::Error),
}

/// Errors that can occur when listing notifications.
#[derive(Debug, Error)]
pub enum ListError {
    /// An error occurred during the API request.
    #[error("An error occurred during the API request.")]
    RequestError(#[source] ApiError),
    /// Failed to parse the data.
    #[error("Failed to parse the data")]
    FormatError(serde_json::Error),
}

/// Errors that can occur when creating notifications.
#[derive(Debug, Error)]
pub enum CreateError {
    /// An error occurred during the API request.
    #[error("An error occurred during the API request.")]
    RequestError(#[source] ApiError),
    /// Failed to parse the data.
    #[error("Failed to parse the data")]
    FormatError(#[from] serde_json::Error),
}

/// Errors that can occur when listing notifications.
#[derive(Debug, Error)]
pub enum GetError {
    /// An error occurred during the API request.
    #[error("An error occurred during the API request.")]
    RequestError(#[source] ApiError),
    /// The resource has not been found.
    #[error("The resource has not been found.")]
    NotFound,
    /// Failed to parse the data.
    #[error("Failed to parse the data")]
    FormatError(serde_json::Error),
}

impl ApiClient {
    /// Creates a new API client.
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_owned(),
            client: Client::new(),
        }
    }

    /// Lists the notifications.
    pub async fn list_notifications(
        &self,
    ) -> Result<Vec<Notification>, ListError> {
        let value = self
            .get("notifications")
            .await
            .map_err(ListError::RequestError)?;

        serde_json::from_value(value)
            .map_err(ListError::FormatError)
            .log_err()
    }

    /// Creates a notification.
    pub async fn create_notification(
        &self,
        message: &str,
    ) -> Result<Notification, CreateError> {
        let payload = CreateNotificationPayload {
            message: message.to_owned(),
        };

        let body =
            serde_json::to_value(payload).expect("could not serialise to JSON");

        let response = self
            .post("notifications", &body)
            .await
            .map_err(CreateError::RequestError)?;

        let response = serde_json::from_value(response)
            .map_err(CreateError::FormatError)
            .log_err()?;

        match response {
            CreateNotificationResult::Notification(notification) => {
                Ok(notification)
            }
            CreateNotificationResult::Error(error) => match error {
                ntf_api_types::CreateNotificationError::PayloadError(_) => {
                    todo!()
                }
            },
        }
    }

    /// Gets a notification by its ID.
    pub async fn get_notification(
        &self,
        id: usize,
    ) -> Result<Notification, GetError> {
        let value = self
            .get(&format!("notifications/{id}"))
            .await
            .map_err(GetError::RequestError)?;

        let response = serde_json::from_value(value)
            .map_err(GetError::FormatError)
            .log_err()?;

        match response {
            NotificationResult::Notification(notification) => Ok(notification),
            NotificationResult::Error(error) => match error {
                ResourceError::NotFound { id } => Err(GetError::NotFound),
            },
        }
    }

    /// Perform a GET request on the given route.
    async fn get(&self, route: &str) -> Result<Value, ApiError> {
        self.client
            .get(format!("{}/{}", self.base_url, route))
            .send()
            .await
            .map_err(ApiError::RequestError)
            .log_err()?
            .json()
            .await
            .map_err(ApiError::RequestError)
            .log_err()
    }

    /// Perform a POST request on the given route.
    async fn post(&self, route: &str, body: &Value) -> Result<Value, ApiError> {
        self.client
            .post(format!("{}/{}", self.base_url, route))
            .json(body)
            .send()
            .await
            .map_err(ApiError::RequestError)
            .log_err()?
            .json()
            .await
            .map_err(ApiError::RequestError)
    }
}
