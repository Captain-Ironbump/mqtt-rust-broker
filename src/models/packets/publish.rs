use log::{info, warn, error};

use crate::models::mqtt_headers::{MqttHeaders, PublishHeader};
use crate::models::mqtt_payloads::{Payload, PublishPayload};
use crate::models::mqtt_payloads::PayloadFactory;
use crate::models::mqtt_types::MqttPacketType;

pub struct Publish {
    pub fixed_header: MqttHeaders,
    pub variable_header: PublishHeader,
    pub payload: Payload,
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
        let variable_header = PublishHeader::from_bytes(&data[2..]);
        let payload = PayloadFactory::parse_payload(&variable_header, data[10..].to_vec());
        Publish::new(fixed_header.unwrap(), variable_header, payload)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = self.fixed_header.to_bytes();
        bytes.extend(self.variable_header.to_bytes());
        let payload_bytes = match &self.payload {
            Payload::Publish(publish_payload) => publish_payload.to_bytes(),
            _ => panic!("Expected PublishPayload, found {:?}", self.payload),
        };
        bytes.extend(payload_bytes);
        bytes
    }
}
