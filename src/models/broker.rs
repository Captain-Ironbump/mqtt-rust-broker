use core::panic;

#[derive(Debug)]
pub struct Broker {
    clients: Vec<String>,
}


impl Broker {
    pub fn new() -> Self {
        Broker {
            clients: Vec::new(),
        }
    }

    pub fn connect_client(&mut self, client_id: &String) -> bool {
        self.clients.push(client_id.clone());
        true
    }

    pub fn is_client_connected(&mut self, client_id: &String) -> bool {
        if self.clients.contains(client_id) {
            return true;
        }
        false
    }

    fn get_index_of_client(&mut self, client_id: &String) -> usize {
        if !self.is_client_connected(client_id) {
            panic!("Not found");
        }

        for (idx, connected_client) in self.clients.iter().enumerate() {
            if client_id.eq(connected_client) {
                return idx;
            }
        }
        0
    }

    pub fn disconnect_client(&mut self, client_id: &String) -> (String, bool) {
        if !self.is_client_connected(client_id) {
            return (String::new(), false);
        }
        let idx = self.get_index_of_client(client_id);
        let disconnected_client = self.clients.remove(idx);
        (disconnected_client, true)
    }
}
