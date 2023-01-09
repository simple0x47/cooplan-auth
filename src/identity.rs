use crate::client_data::ClientData;
use crate::error::{Error, ErrorKind};
use crate::token::Token;
use reqwest::header::{HeaderMap, HeaderValue};
use serde_json::{Map, Value};
use std::sync::Arc;
use std::time::Instant;

const EXPIRATION_ERROR_MARGIN_IN_SECONDS: u64 = 5;

pub struct Identity {
    identity_provider_url: String,
    client_data: ClientData,
}

impl Identity {
    pub async fn try_new(
        identity_provider_url: String,
        client_data: ClientData,
    ) -> Result<Identity, Error> {
        Ok(Identity {
            identity_provider_url,
            client_data,
        })
    }

    pub async fn try_get_token(&self) -> Result<Arc<Token>, Error> {
        let token =
            Arc::new(try_get_new_token(&self.identity_provider_url, &self.client_data).await?);

        Ok(token)
    }

    pub async fn renew_token_if_expiring_after_seconds(
        &self,
        token: Arc<Token>,
        expiring_after_seconds: u64,
    ) -> Result<Arc<Token>, Error> {
        if token.does_expire_after(expiring_after_seconds) {
            return self.try_get_token().await;
        }

        Ok(token)
    }
}

async fn try_get_new_token(
    identity_provider_url: &String,
    client_data: &ClientData,
) -> Result<Token, Error> {
    let client = reqwest::Client::new();
    let mut header = HeaderMap::new();
    let content_type_value = match HeaderValue::from_str("application/json") {
        Ok(value) => value,
        Err(error) => {
            return Err(Error::new(
                ErrorKind::InternalFailure,
                format!("failed to set content type: {}", error),
            ))
        }
    };
    header.insert("content-type", content_type_value);

    let response = match client
        .post(identity_provider_url)
        .headers(header)
        .body(client_data.json())
        .send()
        .await
    {
        Ok(response) => response,
        Err(error) => {
            return Err(Error::new(
                ErrorKind::IdentityProviderFailure,
                format!("failed to get new token: {}", error),
            ))
        }
    };

    let response_object = match response.json::<Map<String, Value>>().await {
        Ok(response_object) => response_object,
        Err(error) => {
            return Err(Error::new(
                ErrorKind::IdentityProviderFailure,
                format!("failed to parse response: {}", error),
            ))
        }
    };

    let token = extract_token_from_response_object(response_object)?;

    Ok(token)
}

fn extract_token_from_response_object(
    mut response_object: Map<String, Value>,
) -> Result<Token, Error> {
    let generated_at = Instant::now();

    let token_string = match response_object.remove("access_token") {
        Some(token_string) => match token_string.as_str() {
            Some(token_string) => token_string.to_string(),
            None => {
                return Err(Error::new(
                    ErrorKind::MalformedResponse,
                    "failed to read token as string",
                ))
            }
        },
        None => return Err(Error::new(ErrorKind::MalformedResponse, "missing token")),
    };

    let mut duration = match response_object.remove("expires_in") {
        Some(duration) => match duration.as_u64() {
            Some(duration) => duration,
            None => {
                return Err(Error::new(
                    ErrorKind::MalformedResponse,
                    "failed to read duration as u64",
                ))
            }
        },
        None => return Err(Error::new(ErrorKind::MalformedResponse, "missing duration")),
    };

    if duration <= EXPIRATION_ERROR_MARGIN_IN_SECONDS {
        return Err(Error::new(
            ErrorKind::MalformedResponse,
            "duration is too short",
        ));
    } else {
        duration -= EXPIRATION_ERROR_MARGIN_IN_SECONDS;
    }

    Ok(Token::new(token_string, generated_at, duration))
}

#[cfg(test)]
use serde_json::Number;

#[test]
fn durations_lower_or_equal_to_error_margin_are_rejected() {
    let mut response_object = Map::new();
    response_object.insert(
        "access_token".to_string(),
        Value::String("token".to_string()),
    );
    response_object.insert("expires_in".to_string(), Value::Number(5.into()));

    assert!(extract_token_from_response_object(response_object).is_err());
}

#[test]
fn extract_token_from_response_object_correctly() {
    const TOKEN: &str = "token";
    let mut response_object = Map::new();
    response_object.insert("access_token".to_string(), Value::String(TOKEN.to_string()));
    response_object.insert("expires_in".to_string(), Value::Number(Number::from(10u64)));

    let token = extract_token_from_response_object(response_object).unwrap();

    assert_eq!(token.value(), TOKEN);
    assert!(!token.is_expired());
}
