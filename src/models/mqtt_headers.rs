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



#[cfg(test)]
mod mytests {
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
}