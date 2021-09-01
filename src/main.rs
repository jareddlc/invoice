use std::collections::HashMap;
use std::env;

mod client;
mod csv_reader;
mod transction;

use client::ClientMap;
use csv_reader::CSV;

fn main() {
    // load csv
    let csv = match CSV::load(env::args().collect()) {
        Err(err) => {
            eprintln!("Failed to load csv: {}", err);
            std::process::exit(1);
        }
        Ok(csv) => csv,
    };

    // create a memory store for clients
    let mut clients: ClientMap = HashMap::new();

    // process csv
    for record in csv.records_vec {
        let client = transction::process(&record, &clients, &csv.records_map);

        clients.insert(client.client, client);
    }

    // output csv
    match CSV::output(clients) {
        Err(err) => {
            eprintln!("Failed to output csv: {}", err);
            std::process::exit(1);
        }
        Ok(()) => (),
    };
}
