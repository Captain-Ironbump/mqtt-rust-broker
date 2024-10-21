use std::any::Any;

use crate::models::mqtt_types::MqttPacketType;

#[derive(Debug, Clone, Copy)]
pub struct MqttHeaders {
    pub packet_type: MqttPacketType,
    pub flags: u8,
    pub remaining_length: u32,
}

impl MqttHeaders {
    // byte1: message type (4 bits) + flags (4 bits)
    // byte2: remaining length (variable length encoding)
    pub fn parse(buffer: &[u8]) -> Result<Self, &'static str> {
        if buffer.len() < 2 {
            return Err("Buffer is too short to contain an MQTT Fixed Header");
        }

        let byte1 = buffer[0];
        // the first 4 bits of the first byte represent the packet type (right shift by 4 bits)
        let packet_type = match byte1 >> 4 {
            1 => MqttPacketType::Connect,
            2 => MqttPacketType::ConnAck,
            3 => MqttPacketType::Publish,
            4 => MqttPacketType::PubAck,
            5 => MqttPacketType::PubRec,
            6 => MqttPacketType::PubRel,
            7 => MqttPacketType::PubComp,
            8 => MqttPacketType::Subscribe,
            9 => MqttPacketType::SubAck,
            10 => MqttPacketType::Unsubscribe,
            11 => MqttPacketType::UnsubAck,
            12 => MqttPacketType::PingReq,
            13 => MqttPacketType::PingResp,
            14 => MqttPacketType::Disconnect,
            _ => return Err("Invalid MQTT Packet Type"),
        };

        let flags = byte1 & 0x0F;

        let mut multiplier = 1;
        let mut value = 0;
        let mut index = 1;
        while index < buffer.len() {
            let encoded_byte = buffer[index];
            value += (encoded_byte & 127) as u32 * multiplier;
            multiplier *= 128;
            if encoded_byte & 128 == 0 {
                break;
            }
            index += 1;
        }

        Ok(MqttHeaders {
            packet_type,
            flags,
            remaining_length: value,
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        // First Byte: packet Type (4 bits) + Flags (4 bits)
        let byte1 = (self.packet_type as u8) << 4 | (self.flags & 0x0F);
        buffer.push(byte1);

        // Encode Remaining Length using Variable Length Encoding
        let mut remaining_length = self.remaining_length;
        loop {
            let mut encoded_byte = (remaining_length % 128) as u8;
            remaining_length /= 128;
            if remaining_length > 0 {
                encoded_byte |= 128;
            }
            buffer.push(encoded_byte);
            if remaining_length == 0 {
                break;
            }
        }

        buffer
    }
}

pub trait VariableHeader {
    fn header_type(&self) -> MqttPacketType;
    fn as_any(&self) -> &dyn Any;
}


#[derive(Debug, Clone, PartialEq)]
pub struct ConnectHeader {
    pub protocol_name: String,
    pub protocol_level: u8,
    pub connect_flags: u8,
    pub keep_alive: u16,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PublishHeader {
    pub topic_name: String,
    pub packet_id: u16,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SubscribeHeader {
    pub packet_id: u16,
}


impl VariableHeader for ConnectHeader {
    fn header_type(&self) -> MqttPacketType {
        MqttPacketType::Connect
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl VariableHeader for PublishHeader {
    fn header_type(&self) -> MqttPacketType {
        MqttPacketType::Publish
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl VariableHeader for SubscribeHeader {
    fn header_type(&self) -> MqttPacketType {
        MqttPacketType::Subscribe
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl ConnectHeader {
    const PROTOCOL_NAME_LENGTH: usize = 4;
    const PROTOCOL_LEVEL_LENGTH: usize = 1;
    const CONNECT_FLAGS_LENGTH: usize = 1;
    const KEEP_ALIVE_LENGTH: usize = 2; // 2 bytes


   pub fn new(protocol_name: String, protocol_level: u8, connect_flags: u8, keep_alive: u16) -> Result<Self, String> {
       if protocol_name.len() != 4 && protocol_name != "MQTT" {
           return Err("Invalid Protocol Name".to_string());
       }
       Ok(Self {
           protocol_name,
           protocol_level,
           connect_flags,
           keep_alive,
       }) 
    }
    pub fn from_bytes(data: &[u8]) -> Self {
        // the date variable is expected to not hold the fixed header
        let mut idx = 0;
        let protocol_name = String::from_utf8(data[idx..Self::PROTOCOL_NAME_LENGTH].to_vec()).unwrap();
        idx += Self::PROTOCOL_NAME_LENGTH + 1;
        let protocol_level = data[idx - Self::PROTOCOL_LEVEL_LENGTH];
        idx += Self::PROTOCOL_LEVEL_LENGTH + 1;
        let connect_flags = data[idx - Self::CONNECT_FLAGS_LENGTH];
        idx += Self::CONNECT_FLAGS_LENGTH + 2;
        let keep_alive = u16::from_be_bytes([data[idx - Self::KEEP_ALIVE_LENGTH - 2], data[idx - Self::KEEP_ALIVE_LENGTH - 1]]);
        println!("Keep Alive: {}", keep_alive);
        println!("Protocol Name: {}", protocol_name);
        println!("Protocol Level: {}", protocol_level);
        println!("Connect Flags: {}", connect_flags);
        ConnectHeader::new(protocol_name, protocol_level, connect_flags, keep_alive).unwrap()
    }
}

#[cfg(test)]
mod mqtt_headers_tests {
    use super::*; 

    #[test]
    fn test_parse() {
        let buffer = vec![0x10, 0x00];
        let headers = MqttHeaders::parse(&buffer).unwrap();
        assert_eq!(headers.packet_type, MqttPacketType::Connect);
        assert_eq!(headers.flags, 0);
        assert_eq!(headers.remaining_length, 0);
    }

    #[test]
    fn test_to_bytes() {
        let headers = MqttHeaders {
            packet_type: MqttPacketType::Connect,
            flags: 0,
            remaining_length: 10,
        };
        let buffer = headers.to_bytes();
        assert_eq!(buffer, vec![0x10, 0x0A]);
    }

    #[test]
    fn test_connect_header_new() {
        let header = ConnectHeader::new("MQTT".to_string(), 4, 0, 60).unwrap();
        assert_eq!(header.protocol_name, "MQTT");
        assert_eq!(header.protocol_level, 4);
        assert_eq!(header.connect_flags, 0);
        assert_eq!(header.keep_alive, 60);
    }

    #[test]
    fn test_connect_header_new_invalid_protocol_name() {
        let header = ConnectHeader::new("MQT".to_string(), 4, 0, 60);
        assert_eq!(header, Err("Invalid Protocol Name".to_string()));
    }

    #[test]
    fn test_connect_header_from_bytes() {
        let data = vec![0x4D, 0x51, 0x54, 0x54, 0x04, 0x00, 0x00, 0x3C];
        let header = ConnectHeader::from_bytes(&data);
        assert_eq!(header.protocol_name, "MQTT");
        assert_eq!(header.protocol_level, 4);
        assert_eq!(header.connect_flags, 0);
        assert_eq!(header.keep_alive, 60);
    }
}
