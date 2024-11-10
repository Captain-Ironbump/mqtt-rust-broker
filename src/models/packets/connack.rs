use crate::models::mqtt_headers::{ConnectHeader, MqttHeaders};
use crate::models::mqtt_payloads::{Payload, PayloadFactory};
use crate::models::mqtt_headers::ConnAckHeader;
use crate::models::mqtt_types::MqttPacketType;

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

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        let fixed_header_buffer = self.fixed_header.to_bytes();
        let variable_header_buffer = self.variable_header.to_bytes();
        buffer.extend(fixed_header_buffer);
        buffer.extend(variable_header_buffer);
        buffer
    }

    pub fn new_success(connect_header: &ConnectHeader) -> Self {
        let (session_present, return_code) = if connect_header.connect_flags & 0b00000010 == 1 {
            (false, 0b00000000)
        } else {
            (true, 0b00000000) // TODO: check doku and make more checks here
        };
        
        let fixed_header = MqttHeaders::new(MqttPacketType::ConnAck, 0b0000, 2);
        let variable_header = ConnAckHeader::new(session_present, return_code);
        ConnAck::new(fixed_header, variable_header, Payload::Default(Default::default()))
    }
}
