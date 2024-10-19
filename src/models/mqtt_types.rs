use std::collections::HashMap;

use futures::stream::SplitSink;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;
use tokio::net::TcpStream;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MqttPacketType {
    Connect = 1,
    ConnAck = 2,
    Publish = 3,
    PubAck = 4,
    PubRec = 5,
    PubRel = 6,
    PubComp = 7,
    Subscribe = 8,
    SubAck = 9,
    Unsubscribe = 10,
    UnsubAck = 11,
    PingReq = 12,
    PingResp = 13,
    Disconnect = 14,
}

impl MqttPacketType {
    pub fn from_u8(value: u8) -> Result<Self, &'static str> {
        match value {
            1 => Ok(MqttPacketType::Connect),
            2 => Ok(MqttPacketType::ConnAck),
            3 => Ok(MqttPacketType::Publish),
            4 => Ok(MqttPacketType::PubAck),
            5 => Ok(MqttPacketType::PubRec),
            6 => Ok(MqttPacketType::PubRel),
            7 => Ok(MqttPacketType::PubComp),
            8 => Ok(MqttPacketType::Subscribe),
            9 => Ok(MqttPacketType::SubAck),
            10 => Ok(MqttPacketType::Unsubscribe),
            11 => Ok(MqttPacketType::UnsubAck),
            12 => Ok(MqttPacketType::PingReq),
            13 => Ok(MqttPacketType::PingResp),
            14 => Ok(MqttPacketType::Disconnect),
            _ => Err("Invalid MQTT Packet Type"),
        }
    }
}

#[cfg(test)]
mod packet_type_tests {
    use super::*;

    #[test]
    fn test_from_u8() {
        assert_eq!(MqttPacketType::from_u8(1), Ok(MqttPacketType::Connect));
        assert_eq!(MqttPacketType::from_u8(2), Ok(MqttPacketType::ConnAck));
        assert_eq!(MqttPacketType::from_u8(3), Ok(MqttPacketType::Publish));
        assert_eq!(MqttPacketType::from_u8(4), Ok(MqttPacketType::PubAck));
        assert_eq!(MqttPacketType::from_u8(5), Ok(MqttPacketType::PubRec));
        assert_eq!(MqttPacketType::from_u8(6), Ok(MqttPacketType::PubRel));
        assert_eq!(MqttPacketType::from_u8(7), Ok(MqttPacketType::PubComp));
        assert_eq!(MqttPacketType::from_u8(8), Ok(MqttPacketType::Subscribe));
        assert_eq!(MqttPacketType::from_u8(9), Ok(MqttPacketType::SubAck));
        assert_eq!(MqttPacketType::from_u8(10), Ok(MqttPacketType::Unsubscribe));
        assert_eq!(MqttPacketType::from_u8(11), Ok(MqttPacketType::UnsubAck));
        assert_eq!(MqttPacketType::from_u8(12), Ok(MqttPacketType::PingReq));
        assert_eq!(MqttPacketType::from_u8(13), Ok(MqttPacketType::PingResp));
        assert_eq!(MqttPacketType::from_u8(14), Ok(MqttPacketType::Disconnect));
        assert_eq!(MqttPacketType::from_u8(15), Err("Invalid MQTT Packet Type"));
    }
}


#[derive(Debug, Clone)]
pub struct MqttPacketDispatcher {
    pub handlers: HashMap<MqttPacketType, fn(&mut SplitSink<WebSocketStream<TcpStream>, Message>, &Vec<u8>)>
}

impl MqttPacketDispatcher {
    pub fn new() -> Result<Self, &'static str> {
        let mut handlers: HashMap<MqttPacketType, fn(&mut SplitSink<WebSocketStream<TcpStream>, Message>, &Vec<u8>)> = HashMap::new();
        handlers.insert(MqttPacketType::Connect, MqttPacketDispatcher::handle_connect);
        handlers.insert(MqttPacketType::ConnAck, MqttPacketDispatcher::handle_connack);
        handlers.insert(MqttPacketType::Publish, MqttPacketDispatcher::handle_publish);
        handlers.insert(MqttPacketType::PubAck, MqttPacketDispatcher::handle_puback);
        handlers.insert(MqttPacketType::PubRec, MqttPacketDispatcher::handle_pubrec);
        handlers.insert(MqttPacketType::PubRel, MqttPacketDispatcher::handle_pubrel);
        handlers.insert(MqttPacketType::PubComp, MqttPacketDispatcher::handle_pubcomp);
        handlers.insert(MqttPacketType::Subscribe, MqttPacketDispatcher::handle_subscribe);
        handlers.insert(MqttPacketType::SubAck, MqttPacketDispatcher::handle_suback);
        handlers.insert(MqttPacketType::Unsubscribe, MqttPacketDispatcher::handle_unsubscribe);
        handlers.insert(MqttPacketType::UnsubAck, MqttPacketDispatcher::handle_unsuback);
        handlers.insert(MqttPacketType::PingReq, MqttPacketDispatcher::handle_ping_req);
        handlers.insert(MqttPacketType::PingResp, MqttPacketDispatcher::handle_ping_resp);
        handlers.insert(MqttPacketType::Disconnect, MqttPacketDispatcher::handle_disconnect);

        Ok(MqttPacketDispatcher { handlers })
    }

        // Empty handler functions for each packet type
    fn handle_connect(sender: &mut SplitSink<WebSocketStream<TcpStream>, Message>, data: &Vec<u8>) {
        
    }

    fn handle_connack(sender: &mut SplitSink<WebSocketStream<TcpStream>, Message>, data: &Vec<u8>) {
        // Empty function for ConnAck packet
    }

    fn handle_publish(sender: &mut SplitSink<WebSocketStream<TcpStream>, Message>, data: &Vec<u8>) {
        // Empty function for Publish packet
    }

    fn handle_puback(sender: &mut SplitSink<WebSocketStream<TcpStream>, Message>, data: &Vec<u8>) {
        // Empty function for PubAck packet
    }

    fn handle_pubrec(sender: &mut SplitSink<WebSocketStream<TcpStream>, Message>, data: &Vec<u8>) {
        // Empty function for PubRec packet
    }

    fn handle_pubrel(sender: &mut SplitSink<WebSocketStream<TcpStream>, Message>, data: &Vec<u8>) {
        // Empty function for PubRel packet
    }

    fn handle_pubcomp(sender: &mut SplitSink<WebSocketStream<TcpStream>, Message>, data: &Vec<u8>) {
        // Empty function for PubComp packet
    }

    fn handle_subscribe(sender: &mut SplitSink<WebSocketStream<TcpStream>, Message>, data: &Vec<u8>) {
        // Empty function for Subscribe packet
    }

    fn handle_suback(sender: &mut SplitSink<WebSocketStream<TcpStream>, Message>, data: &Vec<u8>) {
        // Empty function for SubAck packet
    }

    fn handle_unsubscribe(sender: &mut SplitSink<WebSocketStream<TcpStream>, Message>, data: &Vec<u8>) {
        // Empty function for Unsubscribe packet
    }

    fn handle_unsuback(sender: &mut SplitSink<WebSocketStream<TcpStream>, Message>, data: &Vec<u8>) {
        // Empty function for UnsubAck packet
    }

    fn handle_ping_req(sender: &mut SplitSink<WebSocketStream<TcpStream>, Message>, data: &Vec<u8>) {
        // Empty function for PingReq packet
    }

    fn handle_ping_resp(sender: &mut SplitSink<WebSocketStream<TcpStream>, Message>, data: &Vec<u8>) {
        // Empty function for PingResp packet
    }

    fn handle_disconnect(sender: &mut SplitSink<WebSocketStream<TcpStream>, Message>, data: &Vec<u8>) {
        // Empty function for Disconnect packet
    }
}