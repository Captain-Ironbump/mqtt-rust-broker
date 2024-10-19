
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
    pub fn create_payload(variable_header: &dyn VariableHeader, payload_data: Vec<u8>) -> Payload {
        if let Some(connect_header) = variable_header.as_any().downcast_ref::<ConnectHeader>() {
            // The ClientId MUST be the first field in the CONNECT packet [MQTT-3.1.3-1]
            // The ClientId MUST be present and its value MUST be a non-zero-length UTF-8 encoded string [MQTT-3.1.3-3]
            // The ClientId MUST be a UTF-8 encoded string as defined in Section 1.5.3 UTF-8 encoded strings [MQTT-3.1.3-4]
            // The Server MUST allow ClientIds which are between 1 and 23 UTF-8 encoded bytes in length, and that contain only the characters
            // "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ" [MQTT-3.1.3-5]
            
            // take teh first two bytes of the payload data to get the length of the client id
            let client_id_length: usize  = (payload_data[0] as usize) << 8 | payload_data[1] as usize;
            if client_id_length == 0 {
                //TODO: maybe allow for empty client id and generate a random one
                //TODO: set Client Clean Session to 1 if client id is empty
                //TODO: If the Client supplies a zero-byte ClientId with CleanSession set to 0, the Server MUST respond to the CONNECT Packet with a CONNACK return code 0x02 (Identifier rejected) and then close the Network Connection [MQTT-3.1.3-8].
                panic!("Client ID cannot be empty");
            }
            if client_id_length > 23 {
                panic!("Client ID cannot be longer than 23 bytes");
            }
            let mut payload_idx: usize = 2 as usize;
            let client_id: String = String::from_utf8(payload_data[payload_idx..client_id_length + payload_idx].to_vec()).unwrap();
            payload_idx += client_id_length + 1; // +1 to skip the length of the client id
            let mut will_topic: String = "".to_string();
            let mut will_message: String = "".to_string();
            if connect_header.connect_flags & 0b00000100 != 0 {
                let will_topic_length: usize = (payload_data[payload_idx] as usize) << 8 | payload_data[payload_idx + 1] as usize;
                payload_idx += 2;
                will_topic = String::from_utf8(payload_data[payload_idx..will_topic_length + payload_idx].to_vec()).unwrap();
                payload_idx += will_topic_length + 1;
                let will_message_length: usize = (payload_data[payload_idx] as usize) << 8 | payload_data[payload_idx + 1] as usize;
                payload_idx += 2;
                will_message = String::from_utf8(payload_data[payload_idx..will_message_length + payload_idx].to_vec()).unwrap();
                payload_idx += will_message_length + 1;
            }
            
            
            Payload {
                client_id: Some(client_id),
                will_topic: Some(will_topic),
                will_message: Some(will_message),
                ..Default::default()
            }
        } else {
            Payload::default()
        }
    }
    
}
