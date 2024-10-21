use crate::models::mqtt_headers::{MqttHeaders, ConnectHeader};
use crate::models::mqtt_payloads::ConnectPayload;

pub struct Connect {
    pub fixed_header: MqttHeaders,
    pub variable_header: ConnectHeader,
    pub payload: ConnectPayload,
}

impl Connect {
    pub fn new(fixed_header: MqttHeaders, variable_header: ConnectHeader, payload: ConnectPayload) -> Self {
        Connect {
            fixed_header,
            variable_header,
            payload,
        }
    }

    pub fn from_bytes(data: Vec<u8>) -> Self {
        let fixed_header = MqttHeaders::parse(data[0]);
        let variable_header = ConnectHeader::from_bytes(&data[1..]);
        let payload = ConnectPayload::from_bytes(&data[1..]);
        Connect {
            fixed_header,
            variable_header,
            payload,
        }
    }
}
