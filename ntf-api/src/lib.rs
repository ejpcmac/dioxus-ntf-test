//! Client library for the notification API.

pub use ntf_api_types::Notification;

use reqwest::{Client, Method};
use serde_json::Value;
use thiserror::Error;

use ntf_api_types::{
    CreateNotificationError, CreateNotificationPayload,
    CreateNotificationResult, NotificationResult, ResourceError,
};
use ntf_poc_helpers::tracing::LogResult as _;

/// API client for the notification web service.
#[derive(Debug)]
pub struct ApiClient {
    /// The base URL of the API.
    base_url: String,
    /// The reqwest client.
    client: Client,
}

/// Errors that can occur when listing notifications.
#[derive(Debug, Error)]
pub enum ListError {
    /// An error occurred during the API request.
    #[error(transparent)]
    ApiError(ApiError),
}

/// Errors that can occur when creating notifications.
#[derive(Debug, Error)]
pub enum CreateError {
    /// An error occurred during the API request.
    #[error(transparent)]
    ApiError(ApiError),
}

/// Errors that can occur when getting a notification.
pub type GetError = ResourceAccessError;

/// Errors that can occur when acknowledging a notification.
pub type AckError = ResourceAccessError;

/// Errors that can occur when deleting a notification.
pub type DeleteError = ResourceAccessError;

/// Errors that can occur when accessing a resource.
#[derive(Debug, Error)]
pub enum ResourceAccessError {
    /// An error occurred during the API request.
    #[error(transparent)]
    ApiError(ApiError),
    /// The resource has not been found.
    #[error("the resource has not been found (id = {id}).")]
    NotFound {
        /// The ID of the missing resource.
        id: usize,
    },
}

/// Errors that can occur when making API calls.
#[derive(Debug, Error)]
pub enum ApiError {
    /// An error occurend during the API request.
    #[error("an error occurred during the API request")]
    RequestError(#[source] BoxedError),
    /// The response from the server is invalid.
    #[error("an error occurred while handling the response from the server")]
    ResponseError(#[source] BoxedError),
}

/// A boxed, type-erased error.
type BoxedError = Box<dyn std::error::Error + Send + Sync>;

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
            .request(Method::GET, "notifications", None)
            .await
            .map_err(ListError::ApiError)?;

        serde_json::from_value(value)
            .wrap_err_with_type_info()
            .map_err(ListError::deserialisation_error)
    }

    /// Creates a notification.
    #[expect(
        clippy::missing_panics_doc,
        reason = "cannot actually panic (see reason on expect_used below)"
    )]
    pub async fn create_notification(
        &self,
        message: &str,
    ) -> Result<Notification, CreateError> {
        #[expect(
            clippy::expect_used,
            reason = "the payload is known to be serialisable to JSON"
        )]
        let body = serde_json::to_value(CreateNotificationPayload {
            message: message.to_owned(),
        })
        .expect("could not serialise to JSON");

        let response = self
            .request(Method::POST, "notifications", Some(&body))
            .await
            .map_err(CreateError::ApiError)?;

        let response = serde_json::from_value(response)
            .wrap_err_with_type_info()
            .map_err(CreateError::deserialisation_error)?;

        match response {
            CreateNotificationResult::Notification(notification) => {
                Ok(notification)
            }
            CreateNotificationResult::Error(error) => {
                Err(CreateError::payload_error(error))
            }
        }
    }

    /// Gets a notification by its ID.
    pub async fn get_notification(
        &self,
        id: usize,
    ) -> Result<Notification, GetError> {
        self.request_notification(id, Method::GET).await
    }

    /// Acknowledges a notification by its ID.
    pub async fn ack_notification(
        &self,
        id: usize,
    ) -> Result<Notification, AckError> {
        self.request_notification(id, Method::PUT).await
    }

    /// Deletes a notification by its ID.
    pub async fn delete_notification(
        &self,
        id: usize,
    ) -> Result<Notification, DeleteError> {
        self.request_notification(id, Method::DELETE).await
    }

    /// Requests a notification by its ID with the given `method`.
    async fn request_notification(
        &self,
        id: usize,
        method: Method,
    ) -> Result<Notification, ResourceAccessError> {
        let value = self
            .request(method, &format!("notifications/{id}"), None)
            .await
            .map_err(ResourceAccessError::ApiError)?;

        let response = serde_json::from_value(value)
            .wrap_err_with_type_info()
            .map_err(ResourceAccessError::deserialisation_error)?;

        match response {
            NotificationResult::Notification(notification) => Ok(notification),
            NotificationResult::Error(error) => match error {
                ResourceError::NotFound { id } => {
                    Err(ResourceAccessError::NotFound { id })
                }
            },
        }
    }

    /// Performs a request on the given route.
    async fn request(
        &self,
        method: Method,
        route: &str,
        body: Option<&Value>,
    ) -> Result<Value, ApiError> {
        self.client
            .request(method, format!("{}/{}", self.base_url, route))
            .json(&body)
            .send()
            .await
            .map_err(ApiError::request_error)
            .log_err()?
            .json()
            .await
            .map_err(ApiError::response_error)
            .log_err()
    }
}

impl ListError {
    /// Builds a [`ListError::ApiError`] from a [`DeserialisationError`].
    fn deserialisation_error(error: DeserialisationError) -> Self {
        Self::ApiError(ApiError::ResponseError(Box::new(error)))
    }
}

impl CreateError {
    /// Builds a [`CreateError::ApiError`] from a [`CreateNotificationError`].
    fn payload_error(error: CreateNotificationError) -> Self {
        Self::ApiError(ApiError::RequestError(Box::new(error)))
    }

    /// Builds a [`CreateError::ApiError`] from a [`DeserialisationError`].
    fn deserialisation_error(error: DeserialisationError) -> Self {
        Self::ApiError(ApiError::ResponseError(Box::new(error)))
    }
}

impl ResourceAccessError {
    /// Builds a [`ResourceAccessError::ApiError`] from a [`DeserialisationError`].
    fn deserialisation_error(error: DeserialisationError) -> Self {
        Self::ApiError(ApiError::ResponseError(Box::new(error)))
    }
}

impl ApiError {
    /// Builds an [`ApiError::RequestError`] from a [`reqwest::Error`].
    fn request_error(error: reqwest::Error) -> Self {
        Self::RequestError(Box::new(error))
    }

    /// Builds an [`ApiError::ResponseError`] from a [`reqwest::Error`].
    fn response_error(error: reqwest::Error) -> Self {
        Self::ResponseError(Box::new(error))
    }
}

/// A deserialisation error with destination type information.
#[derive(Debug, Error)]
#[error("failed to deserialise the payload into a `{type_name}`")]
struct DeserialisationError {
    /// The name of the type we are trying to deserialise to.
    pub type_name: &'static str,
    /// The source of the error.
    pub source: BoxedError,
}

impl DeserialisationError {
    /// Builds a [`DeserialisationError`] from a [`serde_json::Error`].
    fn from_serde_json<T>(error: serde_json::Error) -> Self {
        Self {
            type_name: std::any::type_name::<T>(),
            source: Box::new(error),
        }
    }
}

/// An extension trait for deserialisation results.
trait DeserialisationResult<T> {
    /// Maps the error to [`DeserialisationError`].
    fn wrap_err_with_type_info(self) -> Result<T, DeserialisationError>;
}

impl<T> DeserialisationResult<T> for Result<T, serde_json::Error> {
    fn wrap_err_with_type_info(self) -> Result<T, DeserialisationError> {
        self.map_err(DeserialisationError::from_serde_json::<T>)
            .log_err()
    }
}
