use log::{info, warn, error};

use crate::models::mqtt_headers::{MqttHeaders, PublishHeader};
use crate::models::mqtt_payloads::{Payload, PublishPayload};
use crate::models::mqtt_payloads::PayloadFactory;
use crate::models::mqtt_types::MqttPacketType;

struct Publish {
    fixed_header: MqttHeaders,
    variable_header: PublishHeader,
    payload: Payload,
}

impl Publish {
    pub fn new(fixed_header: MqttHeaders, variable_header: PublishHeader, payload: Payload) -> Self {
        Publish {
            fixed_header,
            variable_header,
            payload,
        }
    }

    pub fn from_bytes(data: Vec<u8>) -> Self {
        let fixed_header = MqttHeaders::parse(&data);
        let variable_header = PublishHeader::from_bytes(&data[2..10]);
        let payload = PayloadFactory::parse_payload(&variable_header, data[10..].to_vec());
        Publish::new(fixed_header.unwrap(), variable_header, payload)
    }
}
