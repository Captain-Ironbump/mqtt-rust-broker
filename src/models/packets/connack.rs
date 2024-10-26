use crate::models::mqtt_headers::MqttHeaders;
use crate::models::mqtt_payloads::{Payload, PayloadFactory};
use crate::models::mqtt_headers::ConnAckHeader;

pub struct ConnAck {
    pub fixed_header: MqttHeaders,
    pub variable_header: ConnAckHeader,
    pub payload: Payload,
}

impl ConnAck {
    pub fn new(fixed_header: MqttHeaders, variable_header: ConnAckHeader, payload: Payload) -> Self {
        ConnAck {
            fixed_header,
            variable_header,
            payload,
        }
    }

    pub fn from_bytes(data: Vec<u8>) -> Self {
        let fixed_header = MqttHeaders::parse(&data);
        let fixed_header_size = fixed_header.unwrap().incomming_byte_size();
        let variable_header = ConnAckHeader::from_bytes(&data[fixed_header_size..ConnAckHeader::incomming_byte_size() + fixed_header_size]);
        let payload = PayloadFactory::parse_payload(&variable_header, data[0..0].to_vec());
        ConnAck::new(fixed_header.unwrap(), variable_header, payload)
    }
}
