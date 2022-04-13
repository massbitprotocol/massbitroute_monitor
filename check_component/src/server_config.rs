#[derive(Debug, Clone)]
pub struct AccessControl {
    pub access_control_allow_headers: String,
    pub access_control_allow_origin: String,
    pub access_control_allow_methods: String,
    pub content_type: String,
}

impl Default for AccessControl {
    fn default() -> Self {
        AccessControl {
            access_control_allow_headers:
                "Content-Type, User-Agent, Authorization, Access-Control-Allow-Origin".to_string(),
            access_control_allow_origin: "*".to_string(),
            access_control_allow_methods: "text/html".to_string(),
            content_type: "application/json".to_string(),
        }
    }
}

impl AccessControl {
    pub fn get_access_control_allow_headers(&self) -> Vec<String> {
        self.access_control_allow_headers
            .split(",")
            .into_iter()
            .map(|header| header.replace(" ", ""))
            .collect()
    }
}
