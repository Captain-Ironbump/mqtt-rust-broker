use std::collections::HashMap;
use std::future::Future;

use futures::stream::SplitSink;
use futures::SinkExt;
use tokio::sync::oneshot;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;
use tokio::net::TcpStream;

use log::{info, warn, error};
use crate::models::mqtt_payloads::Default;
use crate::models::mqtt_headers::{ConnAckHeader, ConnectHeader, MqttHeaders};
use crate::models::packets::{connect::Connect, connack::ConnAck};
use crate::models::mqtt_payloads::{Payload, PayloadFactory};
use crate::models::broker::Broker;

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

pub enum PublishActions {
    None = 0,
    PublishAck = 1,
    PublishRec = 2,
}

pub enum BrokerCommand {
    Connect {
        packet: Connect,
        responder: oneshot::Sender<Result<ConnAck, String>>,
    },
    ConnAck {
        responder: oneshot::Sender<Result<(), String>>,
    },
    Publish {
        packet: Publish,
        responder: oneshot::Sender<Result<PublishActions, String>>,
    },
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
    pub handlers: HashMap<MqttPacketType, fn(&Vec<u8>, &mut Broker) -> Vec<u8>>,
}

impl MqttPacketDispatcher {
    pub fn new() -> Result<Self, &'static str> {
        let mut handlers: HashMap<MqttPacketType, fn(&Vec<u8>, &mut Broker) -> Vec<u8>> = HashMap::new();
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
    fn handle_connect(data: &Vec<u8>, broker: &mut Broker) -> Vec<u8> {
        let connect = Connect::from_bytes(data.clone());
        let connect_payload = match connect.payload as Payload {
            Payload::Connect(connect_payload) => connect_payload,
            _ => {
                error!("Invalid payload type");
                return Vec::new();
            }
        };
        let client_id = connect_payload.client_id.unwrap().clone(); 
        if broker.is_client_connected(&client_id) {
            error!("Client already connected...client will be removed");
            broker.remove_client(&client_id);
            return Vec::new();
        }
        broker.add_client(&client_id, connect.variable_header.keep_alive);
        info!("Client connected: with id: [{}]", client_id);
        //TODO: Send CONNACK packet
        let ack_fixed_header = MqttHeaders::new(MqttPacketType::ConnAck, 0b0000, 2);
        
        let (session_present, return_code) = if connect.variable_header.connect_flags & 0b00000010 == 1 {
            (false, 0b00000000)
        } else {
            (true, 0b00000000) // TODO: check doku and make more checks here
        };
        
        let ack_variable_header = ConnAckHeader::new(session_present, return_code);
        
        let connack = ConnAck::new(ack_fixed_header, ack_variable_header, Payload::Default(Default::default()));
        let connack_packet = connack.to_bytes();
        connack_packet
    }

    fn handle_connack(data: &Vec<u8>, broker: &mut Broker) -> Vec<u8> {
        // Empty function for ConnAck packet
        let packet = Vec::new();
        error!("ConnAck packet not a recive packet for server!");
        packet
    }

    fn handle_publish(data: &Vec<u8>, broker: &mut Broker) -> Vec<u8> {
        // Empty function for Publish packet
        let packet = Vec::new();
        packet
    }

    fn handle_puback(data: &Vec<u8>, broker: &mut Broker) -> Vec<u8> {
        // Empty function for PubAck packet
        let packet = Vec::new();
        packet
    }

    fn handle_pubrec(data: &Vec<u8>, broker: &mut Broker) -> Vec<u8> {
        // Empty function for PubRec packet
        let packet = Vec::new();
        packet
    }

    fn handle_pubrel(data: &Vec<u8>, broker: &mut Broker) -> Vec<u8> {
        // Empty function for PubRel packet
        let packet = Vec::new();
        packet
    }

    fn handle_pubcomp(data: &Vec<u8>, broker: &mut Broker) -> Vec<u8> {
        // Empty function for PubComp packet
        let packet = Vec::new();
        packet
    }

    fn handle_subscribe(data: &Vec<u8>, broker: &mut Broker) -> Vec<u8> {
        // Empty function for Subscribe packet
        let packet = Vec::new();
        packet
    }

    fn handle_suback(data: &Vec<u8>, broker: &mut Broker) -> Vec<u8> {
        // Empty function for SubAck packet
        let packet = Vec::new();
        packet
    }

    fn handle_unsubscribe(data: &Vec<u8>, broker: &mut Broker) -> Vec<u8> {
        // Empty function for Unsubscribe packet
        let packet = Vec::new();
        packet
    }

    fn handle_unsuback(data: &Vec<u8>, broker: &mut Broker) -> Vec<u8> {
        // Empty function for UnsubAck packet
        let packet = Vec::new();
        packet
    }

    fn handle_ping_req(data: &Vec<u8>, broker: &mut Broker) -> Vec<u8> {
        // Empty function for PingReq packet
        let packet = Vec::new();
        packet
    }

    fn handle_ping_resp(data: &Vec<u8>, broker: &mut Broker) -> Vec<u8> {
        // Empty function for PingResp packet
        let packet = Vec::new();
        packet
    }

    fn handle_disconnect(data: &Vec<u8>, broker: &mut Broker) -> Vec<u8> {
        // Empty function for Disconnect packet
        let packet = Vec::new();
        packet
    }
}
