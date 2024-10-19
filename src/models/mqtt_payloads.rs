
use super::mqtt_headers::{ConnectHeader, VariableHeader};

#[derive(Debug, Default)]
pub struct Payload {
    pub client_id: Option<String>,
    pub will_topic: Option<String>,
    pub will_message: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
}

struct PayloadFactory;

impl PayloadFactory {
    pub fn create_payload(variable_header: &dyn VariableHeader) -> Payload {
        if let Some(connect_header) = variable_header.as_any().downcast_ref::<ConnectHeader>() {
            Payload::default()
        } else {
            Payload::default()
        }
    }
    
}