pub struct ClientData {
    id: String,
    secret: String,
    audience: String,
}

impl ClientData {
    pub fn new(id: String, secret: String, audience: String) -> ClientData {
        ClientData {
            id,
            secret,
            audience,
        }
    }

    pub fn json(&self) -> String {
        format!(
            "{{ \"client_id\": \"{}\", \"client_secret\": \"{}\", \"audience\": \"{}\", \"grant_type\":\"client_credentials\" }}",
            self.id, self.secret, self.audience
        )
    }
}
