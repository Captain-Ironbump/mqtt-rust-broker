use super::mqtt_headers::{ConnectHeader, PublishHeader, SubscribeHeader, VariableHeader};
use log::{info, warn, error};

#[derive(Debug)]
pub struct ConnectPayload {
    pub client_id: Option<String>,
    pub will_topic: Option<String>,
    pub will_message: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug)]
pub struct PublishPayload {
    pub payload: Vec<u8>,
}

#[derive(Debug)]
pub struct SubscribePayload {
    pub subscription_topic: String,
    pub qos: u8,
}

#[derive(Debug, Default)]
pub struct Default;

#[derive(Debug)]
pub enum Payload {
    Connect(ConnectPayload),
    Publish(PublishPayload),
    Subscribe(SubscribePayload),
    Default(Default),
}

pub struct PayloadFactory;

impl PayloadFactory {
    const WILL_FLAG: u8 = 0b00000100;
    const USER_NAME_FLAG: u8 = 0b10000000;
    const PASSWORD_FLAG: u8 = 0b01000000;
    const QOS_MASK_VALID: u8 = 0b00000011;
    const QOS_MASK_INVALID: u8 = 0b11111100;

    fn extract_utf8_string(payload_data: &[u8], start_idx: &mut usize) -> (usize, String) {
        let string_length: usize = (payload_data[*start_idx] as usize) << 8 | payload_data[*start_idx + 1] as usize;
        *start_idx += 2;
        let extracted_string: String = String::from_utf8(payload_data[*start_idx..string_length + *start_idx].to_vec()).unwrap();
        *start_idx += string_length;
        (string_length, extracted_string)
    }

    pub fn parse_payload(variable_header: &dyn VariableHeader, payload_data: Vec<u8>) -> Payload {
        if let Some(connect_header) = variable_header.as_any().downcast_ref::<ConnectHeader>() {
            // The ClientId MUST be the first field in the CONNECT packet [MQTT-3.1.3-1]
            // The ClientId MUST be present and its value MUST be a non-zero-length UTF-7 encoded string [MQTT-3.1.3-3]
            // The ClientId MUST be a UTF-8 encoded string as defined in Section 1.5.3 UTF-8 encoded strings [MQTT-3.1.3-4]
            // The Server MUST allow ClientIds which are between 1 and 23 UTF-8 encoded bytes in length, and that contain only the characters
            // "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ" [MQTT-3.1.3-5]
            
            // take teh first two bytes of the payload data to get the length of the client id
            let mut payload_idx: usize = 0 as usize;
            let (client_id_length, client_id) = Self::extract_utf8_string(&payload_data, &mut payload_idx);
            info!("Client ID: [{}] with a length of {}", client_id, client_id_length);

            if client_id_length == 0 {
                //TODO: maybe allow for empty client id and generate a random one
                //TODO: set Client Clean Session to 1 if client id is empty
                //TODO: If the Client supplies a zero-byte ClientId with CleanSession set to 0, the Server MUST respond to the CONNECT Packet with a CONNACK return code 0x02 (Identifier rejected) and then close the Network Connection [MQTT-3.1.3-8].
                error!("Client ID cannot be empty");
            }
            if client_id_length > 23 {
                error!("Client ID cannot be longer than 23 bytes");
            }

            let (will_topic, will_message) = if connect_header.connect_flags & Self::WILL_FLAG != 0 {
                let (will_topic_length, will_topic) = Self::extract_utf8_string(&payload_data, &mut payload_idx);
                let (will_message_length, will_message) = Self::extract_utf8_string(&payload_data, &mut payload_idx);
                info!("Will Topic: [{}] with a length of {}", will_topic, will_topic_length);
                info!("Will Message: [{}] with a length of {}", will_message, will_message_length);
                (will_topic, will_message)
            } else {
                (String::new(), String::new())
            };

            let user_name = if connect_header.connect_flags & Self::USER_NAME_FLAG != 0 {
                let (user_name_length, user_name) = Self::extract_utf8_string(&payload_data, &mut payload_idx);
                info!("User Name: [{}] with a length of {}", user_name, user_name_length);
                user_name
            } else {
                String::new()
            };

            let password = if connect_header.connect_flags & Self::PASSWORD_FLAG != 0 {
                let (password_length, password) = Self::extract_utf8_string(&payload_data, &mut payload_idx);
                info!("Password: [{}] with a length of {}", password, password_length);
                password
            } else {
                String::new()
            };
            
            Payload::Connect(ConnectPayload {
                client_id: Some(client_id),
                will_topic: Some(will_topic),
                will_message: Some(will_message),
                username: Some(user_name),
                password: Some(password),
            })
        } else if let Some(_publish_header) = variable_header.as_any().downcast_ref::<PublishHeader>() {
            Payload::Publish(PublishPayload {
                payload: payload_data,
            })
        } else if let Some(_subscribe_header) = variable_header.as_any().downcast_ref::<SubscribeHeader>() {
            let mut payload_idx: usize = 0 as usize;
            let (subscription_topic_length, subscription_topic) = Self::extract_utf8_string(&payload_data, &mut payload_idx);
            info!("Subscription Topic: [{}] with a length of {}", subscription_topic, subscription_topic_length);
            let mut qos = payload_data[payload_idx];
            // validate qos byte format top most 6 bits should be 0
            if qos & Self::QOS_MASK_INVALID != 0 {
                error!("Invalid QoS value");
            }
            qos &= Self::QOS_MASK_VALID;
            Payload::Subscribe(SubscribePayload {
                subscription_topic,
                qos,
            })
        }
        else {
            Payload::Default(Default::default())
        }
    }
    
}


#[cfg(test)]
mod payload_tests {
    use super::*;

    #[test]
    fn test_connect_payload_empty() {
        let connect_header = ConnectHeader {
            connect_flags: 0b00000000,
            keep_alive: 60,
            protocol_name: "MQTT".to_string(),
            protocol_level: 4,
        };
        let payload_data: Vec<u8> = vec![
            0x00, 0x04, 0x74, 0x65, 0x73, 0x74, // Client ID: test
            0x00, 0x00, // Will Topic: 
            0x00, 0x00, // Will Message: 
            0x00, 0x00, // User Name: 
            0x00, 0x00, // Password: 
        ];
        let payload = PayloadFactory::parse_payload(&connect_header, payload_data);
        match payload {
            Payload::Connect(connect_payload) => {
                assert_eq!(connect_payload.client_id.unwrap(), "test");
                assert_eq!(connect_payload.will_topic.unwrap(), "");
                assert_eq!(connect_payload.will_message.unwrap(), "");
                assert_eq!(connect_payload.username.unwrap(), "");
                assert_eq!(connect_payload.password.unwrap(), "");
            },
            _ => error!("Invalid payload type"),
        }
    }

    #[test]
    fn test_connect_payload_non_empty() {
        let connect_header = ConnectHeader {
            connect_flags: 0b11000100,
            keep_alive: 60,
            protocol_name: "MQTT".to_string(),
            protocol_level: 4,
        };
        let payload_data: Vec<u8> = vec![
            0x00, 0x04, 0x74, 0x65, 0x73, 0x74, // Client ID: test
            0x00, 0x04, 0x74, 0x65, 0x73, 0x74, // Will Topic: test
            0x00, 0x04, 0x74, 0x65, 0x73, 0x74, // Will Message: test
            0x00, 0x04, 0x74, 0x65, 0x73, 0x74, // User Name: test
            0x00, 0x04, 0x74, 0x65, 0x73, 0x74, // Password: test
        ];
        let payload = PayloadFactory::parse_payload(&connect_header, payload_data);
        match payload {
            Payload::Connect(connect_payload) => {
                assert_eq!(connect_payload.client_id.unwrap(), "test");
                assert_eq!(connect_payload.will_topic.unwrap(), "test");
                assert_eq!(connect_payload.will_message.unwrap(), "test");
                assert_eq!(connect_payload.username.unwrap(), "test");
                assert_eq!(connect_payload.password.unwrap(), "test");
            },
            _ => error!("Invalid payload type"),
        }
    } 

    #[test]
    fn test_publish_payload() {
        let publish_header = PublishHeader {
            topic_name: "test".to_string(),
            packet_id: 0,
        };
        let payload_data: Vec<u8> = vec![0x00, 0x01, 0x02, 0x03];
        let payload = PayloadFactory::parse_payload(&publish_header, payload_data);
        match payload {
            Payload::Publish(publish_payload) => {
                assert_eq!(publish_payload.payload, vec![0x00, 0x01, 0x02, 0x03]);
            },
            _ => error!("Invalid payload type"),
        }
    }

    #[test]
    fn test_subscribe_payload() {
        let subscribe_header = SubscribeHeader {
            packet_id: 0,
        };
        let payload_data: Vec<u8> = vec![
            0x00, 0x04, 0x74, 0x65, 0x73, 0x74, // Subscription Topic: test
            0x01, // QoS: 1
        ];
        let payload = PayloadFactory::parse_payload(&subscribe_header, payload_data);
        match payload {
            Payload::Subscribe(subscribe_payload) => {
                assert_eq!(subscribe_payload.subscription_topic, "test");
                assert_eq!(subscribe_payload.qos, 1);
            },
            _ => error!("Invalid payload type"),
        }
    }
}

