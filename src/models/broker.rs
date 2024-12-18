use core::panic;
use std::{collections::{HashMap, HashSet}, time::{Duration, SystemTime}};

use log::info;

#[derive(Debug)]
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
}

impl ClientState {
    pub fn new(client_id: &str, keep_alive: Duration) -> Self {
        ClientState {
            client_id: client_id.to_string(),
            connected_status: ConnectionStatus::Connected,
            subscriptions: HashSet::new(),
            last_seen: SystemTime::now(),
            keep_alive,
        }
    }

    pub fn update_last_seen(&mut self) {
        self.last_seen = SystemTime::now();
    }
    
    pub fn is_alive(&self) -> bool {
        self.last_seen.elapsed().unwrap_or(Duration::ZERO) <= self.keep_alive
    }
}

#[derive(Debug)]
pub struct Broker {
    clients: HashMap<String, ClientState>,
}


impl Broker {
    pub fn new() -> Self {
        Broker {
            clients: HashMap::new(),
        }
    }

    pub fn add_client(&mut self, client_id: &str, keep_alive: u16) {
        let keep_alive_duration = Duration::from_secs(keep_alive as u64);
        let client = ClientState::new(client_id, keep_alive_duration);
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
