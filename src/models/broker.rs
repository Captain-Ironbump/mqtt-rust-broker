use core::panic;
use std::{collections::{HashMap, HashSet}, time::{Duration, SystemTime}};
use tokio_tungstenite::tungstenite::protocol::Message;
use log::{info, error};
use tokio::sync::mpsc;

use super::{mqtt_payloads::Payload, mqtt_types::BrokerCommand, packets::connack::ConnAck, packets::publish::Publish};
use crate::models::packets::connack;

#[derive(Debug, Eq, PartialEq)]
enum ConnectionStatus {
    Connected,
    Disconnected,
    AwaitingReconnect,
}

#[derive(Debug)]
struct ClientState {
    client_id: String,
    connected_status: ConnectionStatus,
    subscriptions: HashSet<String>,
    last_seen: SystemTime,
    keep_alive: Duration,
    ws_sender: mpsc::Sender<Message>,
}

impl ClientState {
    pub fn new(client_id: &str, keep_alive: Duration, ws_sender: mpsc::Sender<Message>) -> Self {
        ClientState {
            client_id: client_id.to_string(),
            connected_status: ConnectionStatus::Connected,
            subscriptions: HashSet::new(),
            last_seen: SystemTime::now(),
            keep_alive,
            ws_sender,
        }
    }

    pub fn update_last_seen(&mut self) {
        self.last_seen = SystemTime::now();
    }
    
    pub fn is_alive(&self) -> bool {
        self.last_seen.elapsed().unwrap_or(Duration::ZERO) <= self.keep_alive
    }

    pub async fn publish(&self, publish: Publish) {
        let publish_binary = publish.to_bytes();
        let _send = self.ws_sender.send(Message::Binary(publish_binary.to_vec())).await;
    }
}

#[derive(Debug)]
pub struct Broker {
    clients: HashMap<String, ClientState>,
}

impl Broker {
    async fn run(mut self, mut command_rx: mpsc::Receiver<BrokerCommand>) {
        while let Some(command) = command_rx.recv().await {
            match command {
                BrokerCommand::Connect { packet, ws_sender, responder } => {
                    let variable_header = packet.variable_header;
                    let payload = packet.payload;
                    let connect_payload = match payload {
                        Payload::Connect(connect_payload) => connect_payload,
                        _ => {
                            let _ = responder.send(Err("Invalid payload".to_string()));
                            continue;
                        }                    
                    };
                    if let Some(client_id) = connect_payload.client_id {
                        self.add_client(&client_id, variable_header.keep_alive, ws_sender);
                        let connack = ConnAck::new_success(&variable_header);
                        let _ = responder.send(Ok(connack));
                    } else {
                        let _ = responder.send(Err("Client ID not provided".to_string()));
                    }
                }
                BrokerCommand::ConnAck { responder } => {
                    error!("Received CONNACK packet, but not expected");
                    let _ = responder.send(Err("Broker should never recieve a CONNACK packet".to_string()));
                }, 
                BrokerCommand::Publish { packet, responder } => {
                    let fixed_header = packet.fixed_header;
                    let variable_header = packet.variable_header;
                    //let payload = packet.payload;
                    let publish_payload = match packet.payload {
                        Payload::Publish(publish_payload) => publish_payload,
                        _ => {
                            let _ = responder.send(Err("Invalid payload".to_string()));
                            continue;
                        }
                    };
                    
                    let topic_name = variable_header.get_topic_name();
                    let mut client_queue: Vec<&mut ClientState> = Vec::new();
                    //check for subscribers
                    for client in self.clients.values_mut() {
                        if client.subscriptions.contains(&topic_name) && client.connected_status == ConnectionStatus::Connected {
                            //push client to the queue
                            client_queue.push(client);
                        }
                    }

                    if client_queue.is_empty() {
                        error!("No subscribers found for topic: {}", topic_name);
                        let _ = responder.send(Err("No subscribers found for topic".to_string()));
                        continue;
                    }


                    //check for QoS qos_level
                    // QoS 0: At most once delivery -> no acknoledgement needed; if client is not
                    // connected, message is lost
                    //
                    // QoS 1: At least once delivery -> acknoledgement needed (PUBACK); if client is not
                    // connected, message is stored and sent when client reconnects only of the
                    // retain flag is set, otherwise message is discarded
                    //
                    // QoS 2: Exactly once delivery -> acknoledgement needed (PUBREC, PUBREL,
                    // PUBCOMP); if client is not connected, message is stored and sent when client 
                    // reconnects only of the retain flag is set, otherwise message is discarded
                    
                    let qos_level = fixed_header.get_qos_level(); 
                    match qos_level {
                        0 => {
                            for client in &mut client_queue {
                                //send message to client
                                client.update_last_seen();
                            }
                        },
                        1 => {
                            for client in client_queue {
                                //send message to client
                            }
                        },
                        2 => {
                            for client in client_queue {
                                //send message to client
                            }
                        },
                        _ => {
                            error!("Invalid QoS level");
                        }
                    }
                },
            }
        }
    }

}


impl Broker {
    pub fn new() -> Self {
        Broker {
            clients: HashMap::new(),
        }
    }

    pub fn add_client(&mut self, client_id: &str, keep_alive: u16, ws_sender: mpsc::Sender<Message>) {
        let keep_alive_duration = Duration::from_secs(keep_alive as u64);
        let client = ClientState::new(client_id, keep_alive_duration, ws_sender);
        self.clients.insert(client_id.to_string(), client);
    }

    pub fn remove_client(&mut self, client_id: &str) -> String {
        self.clients.remove(client_id).unwrap().client_id    
    }

   
    pub fn update_client_activity(&mut self, client_id: &str) {
        if let Some(client) = self.clients.get_mut(client_id) {
            client.update_last_seen();
            info!("updated client actifity");
        }
    }

    pub fn get_client(&self, client_id: &str) -> Option<&ClientState> {
        self.clients.get(client_id)
    }

    pub fn is_client_connected(&self, client_id: &str) -> bool {
        self.clients.contains_key(client_id)
    }
}
