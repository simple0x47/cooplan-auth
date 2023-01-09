use std::thread::sleep;
use std::time::Duration;

#[cfg(test)]

const IDENTITY_PROVIDER_ENV: &str = "IDENTITY_PROVIDER";
const CLIENT_ID_ENV: &str = "CLIENT_ID";
const CLIENT_SECRET_ENV: &str = "CLIENT_SECRET";
const TOKEN_AUDIENCE: &str = "TOKEN_AUDIENCE";

#[tokio::test]
async fn get_token_successfully_using_environment_variables() {
    use cooplan_auth::client_data::ClientData;
    use cooplan_auth::identity::Identity;

    let identity_provider = std::env::var(IDENTITY_PROVIDER_ENV).unwrap();
    let client_id = std::env::var(CLIENT_ID_ENV).unwrap();
    let client_secret = std::env::var(CLIENT_SECRET_ENV).unwrap();
    let audience = std::env::var(TOKEN_AUDIENCE).unwrap();

    let client_data = ClientData::new(client_id, client_secret, audience);
    let identity = Identity::try_new(identity_provider, client_data)
        .await
        .unwrap();

    let token = identity.try_get_token().await.unwrap();

    assert!(!token.is_expired());
    assert!(!token.value().is_empty());
}

#[tokio::test]
async fn renew_token_correctly_if_expiring_after() {
    use cooplan_auth::client_data::ClientData;
    use cooplan_auth::identity::Identity;

    const SAME_TOKEN_AFTER_SECONDS: u64 = 1u64;
    const NEW_TOKEN_AFTER_SECONDS: u64 = 89000u64;

    let identity_provider = std::env::var(IDENTITY_PROVIDER_ENV).unwrap();
    let client_id = std::env::var(CLIENT_ID_ENV).unwrap();
    let client_secret = std::env::var(CLIENT_SECRET_ENV).unwrap();
    let audience = std::env::var(TOKEN_AUDIENCE).unwrap();

    let client_data = ClientData::new(client_id, client_secret, audience);
    let identity = Identity::try_new(identity_provider, client_data)
        .await
        .unwrap();

    let token = identity.try_get_token().await.unwrap();
    let initial_token_value = token.value();

    let same_token = identity
        .renew_token_if_expiring_after_seconds(token.clone(), SAME_TOKEN_AFTER_SECONDS)
        .await
        .unwrap();

    assert_eq!(initial_token_value, same_token.value());

    // Avoid response caching.
    sleep(Duration::from_secs(1));

    let different_token = identity
        .renew_token_if_expiring_after_seconds(token.clone(), NEW_TOKEN_AFTER_SECONDS)
        .await
        .unwrap();

    assert_ne!(initial_token_value, different_token.value());
}
