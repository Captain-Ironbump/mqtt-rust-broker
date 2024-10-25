use log::{info, warn, error};

use crate::models::mqtt_headers::{MqttHeaders, ConnectHeader};
use crate::models::mqtt_payloads::{Payload, ConnectPayload};
use crate::models::mqtt_payloads::PayloadFactory;
use crate::models::mqtt_types::MqttPacketType;

pub struct Connect {
    pub fixed_header: MqttHeaders,
    pub variable_header: ConnectHeader,
    pub payload: Payload,
}

impl Connect {
    pub fn new(fixed_header: MqttHeaders, variable_header: ConnectHeader, payload: Payload) -> Self {
        Connect {
            fixed_header,
            variable_header,
            payload,
        }
    }

    pub fn from_bytes(data: Vec<u8>) -> Self {
        let fixed_header = MqttHeaders::parse(&data);
        let variable_header = ConnectHeader::from_bytes(&data[2..10]);
        info!("{:?}", fixed_header);
        info!("{:?}", variable_header);
        let payload = PayloadFactory::parse_payload(&variable_header, data[10..].to_vec());
        info!("{:?}", payload);
        //let connect_payload = match payload {
        //    Payload::Connect(connect_payload) => connect_payload, // Extract ConnectPayload
        //    _ => panic!("Expected ConnectPayload, found {:?}", payload), // Handle other cases
        //};
        Connect::new(fixed_header.unwrap(), variable_header, payload)
    }
}

#[cfg(test)]
mod connect_tests {
    use super::*;

    #[test]
    fn test_connect_from_bytes() {
        //let data = vec![0x10, 0x00, 0x04, 0x4D, 0x51, 0x54, 0x54, 0x04, 0x02, 0x00, 0x3C, 0x00, 0x0A, 0x74, 0x65, 0x73, 0x74, 0x75, 0x73, 0x65, 0x72, 0x6E, 0x61, 0x6D, 0x65, 0x00, 0x0A, 0x74, 0x65, 0x73, 0x74, 0x75, 0x73, 0x65, 0x72, 0x70, 0x77, 0x64];
        let header_data = vec![0x10, 0x00];
        let connect_variable_header_data = vec![0x4D, 0x51, 0x54, 0x54, 0x04, 0xC4, 0x00, 0x3C];
        let connect_payload_data: Vec<u8> = vec![
            0x00, 0x04, 0x74, 0x65, 0x73, 0x74, // Client ID: test
            0x00, 0x04, 0x74, 0x65, 0x73, 0x74, // Will Topic: test
            0x00, 0x04, 0x74, 0x65, 0x73, 0x74, // Will Message: test
            0x00, 0x04, 0x74, 0x65, 0x73, 0x74, // User Name: test
            0x00, 0x04, 0x74, 0x65, 0x73, 0x74, // Password: test
        ]; 

        let data = [&header_data[..], &connect_variable_header_data[..], &connect_payload_data[..]].concat();
        let connect = Connect::from_bytes(data);
        assert_eq!(connect.fixed_header.packet_type, MqttPacketType::Connect);
        //assert_eq!(connect.fixed_header.flags, 0);
        //assert_eq!(connect.fixed_header.remaining_length, 0);
        assert_eq!(connect.variable_header.protocol_name, "MQTT");
        assert_eq!(connect.variable_header.protocol_level, 4);
        assert_eq!(connect.variable_header.connect_flags, 0xC4);
        assert_eq!(connect.variable_header.keep_alive, 60);

        let connect_payload = match connect.payload {
            Payload::Connect(connect_payload) => connect_payload, // Extract ConnectPayload
            _ => panic!("Expected ConnectPayload, found {:?}", connect.payload), // Handle other cases
        };

        assert_eq!(connect_payload.client_id.unwrap(), "test");
        assert_eq!(connect_payload.will_topic.unwrap(), "test");
        assert_eq!(connect_payload.will_message.unwrap(), "test");
        assert_eq!(connect_payload.username.unwrap(), "test");
        assert_eq!(connect_payload.password.unwrap(), "test");
    }
}
