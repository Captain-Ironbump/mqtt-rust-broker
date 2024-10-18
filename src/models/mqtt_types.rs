use std::collections::HashMap;

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

pub struct MqttPacketDispatcher {
    pub handlers: HashMap<MqttPacketType, fn()>
}

impl MqttPacketDispatcher {
    pub fn new() -> Result<Self, &'static str> {
        let mut handlers: HashMap<MqttPacketType, fn()> = HashMap::new();
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
    fn handle_connect() {
        // Empty function for Connect packet
    }

    fn handle_connack() {
        // Empty function for ConnAck packet
    }

    fn handle_publish() {
        // Empty function for Publish packet
    }

    fn handle_puback() {
        // Empty function for PubAck packet
    }

    fn handle_pubrec() {
        // Empty function for PubRec packet
    }

    fn handle_pubrel() {
        // Empty function for PubRel packet
    }

    fn handle_pubcomp() {
        // Empty function for PubComp packet
    }

    fn handle_subscribe() {
        // Empty function for Subscribe packet
    }

    fn handle_suback() {
        // Empty function for SubAck packet
    }

    fn handle_unsubscribe() {
        // Empty function for Unsubscribe packet
    }

    fn handle_unsuback() {
        // Empty function for UnsubAck packet
    }

    fn handle_ping_req() {
        // Empty function for PingReq packet
    }

    fn handle_ping_resp() {
        // Empty function for PingResp packet
    }

    fn handle_disconnect() {
        // Empty function for Disconnect packet
    }
}