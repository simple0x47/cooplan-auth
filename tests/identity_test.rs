#[cfg(test)]
#[tokio::test]
async fn get_token_successfully_using_environment_variables() {
    use cooplan_auth::client_data::ClientData;
    use cooplan_auth::identity::Identity;

    const IDENTITY_PROVIDER_ENV: &str = "IDENTITY_PROVIDER";
    const CLIENT_ID_ENV: &str = "CLIENT_ID";
    const CLIENT_SECRET_ENV: &str = "CLIENT_SECRET";
    const TOKEN_AUDIENCE: &str = "TOKEN_AUDIENCE";

    let identity_provider = std::env::var(IDENTITY_PROVIDER_ENV).unwrap();
    let client_id = std::env::var(CLIENT_ID_ENV).unwrap();
    let client_secret = std::env::var(CLIENT_SECRET_ENV).unwrap();
    let audience = std::env::var(TOKEN_AUDIENCE).unwrap();

    let client_data = ClientData::new(client_id, client_secret, audience);
    let mut identity = Identity::try_new(identity_provider, client_data)
        .await
        .unwrap();

    let token = identity.try_get_token().await.unwrap();

    assert!(!token.is_empty())
}
