use crate::models::mqtt_headers::{MqttHeaders, ConnectHeader};

pub struct Connect {
    pub fixed_header: MqttHeaders,
    pub variable_header: ConnectHeader,
    pub payload: ConnectPayload,
}