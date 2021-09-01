use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::csv_reader::Record;

// Create a type for easier fn definitions
pub type ClientMap = HashMap<u16, Client>;

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct Client {
    pub client: u16,
    pub available: f64,
    pub held: f64,
    pub total: f64,
    pub locked: bool,
}

impl Client {
    // Returns the client from the given record. If client does not exist, it will create one with default values.
    pub fn find(record: &Record, clients: &ClientMap) -> Self {
        // return the client if we already have them
        if clients.contains_key(&record.client) {
            return clients
                .get(&record.client)
                .expect("Failed to get client")
                .clone();
        }

        // otherwise create one
        Client::from(record)
    }
}

// Create a new Client from a Record
impl From<&Record> for Client {
    fn from(record: &Record) -> Self {
        // generate an id if necessary
        let mut client: u16 = record.client;
        if client == 0 {
            let mut rng = rand::thread_rng();
            client = rng.gen();
        }

        Self {
            client: client,
            ..Default::default()
        }
    }
}

// Default values for a Client
impl Default for Client {
    fn default() -> Client {
        Client {
            client: 0,
            available: 0.0,
            held: 0.0,
            total: 0.0,
            locked: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::csv_reader::Type;

    #[test]
    fn test_create_client() {
        let clients: ClientMap = HashMap::new();
        let record = Record {
            r#type: Type::DEPOSIT,
            client: 0,
            tx: 1,
            amount: Some(1.0),
            status: None,
        };

        let client = Client::find(&record, &clients);

        assert_ne!(client.client, 0);
        assert_eq!(client.available, 0.0);
        assert_eq!(client.held, 0.0);
        assert_eq!(client.total, 0.0);
        assert_eq!(client.locked, false);
    }
}
