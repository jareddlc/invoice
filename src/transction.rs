use crate::client::{Client, ClientMap};
use crate::csv_reader::{Record, RecordMap, Status, Type};

// process a single record
pub fn process(record: &Record, clients: &ClientMap, records: &RecordMap) -> Client {
    // find client
    let client = Client::find(&record, &clients);

    // process record
    let result = match &record.r#type {
        Type::DEPOSIT => deposit(record, client),
        Type::WITHDRAWAL => withdrawal(record, client),
        Type::DISPUTE => dispute(record, client, records),
        Type::RESOLVE => resolve(record, client, records),
        Type::CHARGEBACK => chargeback(record, client, records),
    };

    // return result
    result
}

fn deposit(record: &Record, mut client: Client) -> Client {
    // do nothing if we do not have a valid amount
    let amount = match record.amount {
        Some(amount) => amount,
        None => return client,
    };

    // increase available and total
    client.available = client.available + amount;
    client.total = client.total + amount;

    client
}

fn withdrawal(record: &Record, mut client: Client) -> Client {
    // do nothing if we do not have a valid amount
    let amount = match record.amount {
        Some(amount) => amount,
        None => return client,
    };

    // can only withdrawal if the client has enough available
    if client.available > amount {
        client.available = client.available - amount;
        client.total = client.total - amount;
    }

    client
}

fn dispute(record: &Record, mut client: Client, records: &RecordMap) -> Client {
    // find transaction or ignore
    let tx = match records.get(&record.tx) {
        Some(tx) => tx,
        None => return client,
    };

    // do nothing if we do not have a valid amount
    let amount = match tx.amount {
        Some(amount) => amount,
        None => return client,
    };

    // decrease available
    if tx.r#type == Type::DEPOSIT {
        client.available = client.available - amount;
    }

    // increase held
    client.held = client.held + amount;

    client
}

fn resolve(record: &Record, mut client: Client, records: &RecordMap) -> Client {
    // find transaction or ignore
    let tx = match records.get(&record.tx) {
        Some(tx) => tx,
        None => return client,
    };

    // verify if the tx is under dispute
    if tx.status.is_none() {
        return client;
    }
    let status = tx.status.expect("Failed to get transaction status");
    if status != Status::DISPUTED {
        return client;
    }

    // do nothing if we do not have a valid amount
    let amount = match tx.amount {
        Some(amount) => amount,
        None => return client,
    };

    // decrease held
    client.held = client.held - amount;

    // increase available
    if tx.r#type == Type::DEPOSIT {
        client.available = client.available + amount;
    }

    client
}

fn chargeback(record: &Record, mut client: Client, records: &RecordMap) -> Client {
    // find transaction or ignore
    let tx = match records.get(&record.tx) {
        Some(tx) => tx,
        None => return client,
    };

    // verify if the tx is under dispute
    if tx.status.is_none() {
        return client;
    }
    let status = tx.status.expect("Failed to get transaction status");
    if status != Status::DISPUTED {
        return client;
    }

    // do nothing if we do not have a valid amount
    let amount = match tx.amount {
        Some(amount) => amount,
        None => return client,
    };

    // decrease held
    client.held = client.held - amount;

    // decrease total
    if tx.r#type == Type::DEPOSIT {
        client.total = client.total - amount;
    }

    // increase available and total
    if tx.r#type == Type::WITHDRAWAL {
        client.total = client.total + amount;
        client.available = client.available + amount;
    }

    // freeze client
    client.locked = true;

    client
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::csv_reader::{Status, Type};
    use std::collections::HashMap;

    #[test]
    fn test_deposit() {
        let clients: ClientMap = HashMap::new();
        let record_deposit = Record {
            r#type: Type::DEPOSIT,
            client: 1,
            tx: 1,
            amount: Some(10.0),
            status: None,
        };
        let mut records_map = HashMap::new();
        records_map.insert(record_deposit.tx, record_deposit.clone());

        let client = process(&record_deposit, &clients, &records_map);

        assert_eq!(client.available, 10.0);
        assert_eq!(client.total, 10.0);
        assert_eq!(client.locked, false);
    }

    #[test]
    fn test_withdrawal() {
        let mut clients: ClientMap = HashMap::new();
        let record_deposit = Record {
            r#type: Type::DEPOSIT,
            client: 1,
            tx: 1,
            amount: Some(10.0),
            status: None,
        };
        let record_withdrawal = Record {
            r#type: Type::WITHDRAWAL,
            client: 1,
            tx: 2,
            amount: Some(2.0),
            status: None,
        };
        let mut records_map = HashMap::new();
        records_map.insert(record_deposit.tx, record_deposit.clone());
        records_map.insert(record_withdrawal.tx, record_withdrawal.clone());

        let mut client = process(&record_deposit, &clients, &records_map);
        clients.insert(client.client, client);
        client = process(&record_withdrawal, &clients, &records_map);
        clients.insert(client.client, client);

        assert_eq!(client.available, 8.0);
        assert_eq!(client.total, 8.0);
        assert_eq!(client.locked, false);
    }

    #[test]
    fn test_dispute_deposit() {
        let mut clients: ClientMap = HashMap::new();
        let record_deposit = Record {
            r#type: Type::DEPOSIT,
            client: 1,
            tx: 1,
            amount: Some(10.0),
            status: None,
        };
        let record_withdrawal = Record {
            r#type: Type::DEPOSIT,
            client: 1,
            tx: 2,
            amount: Some(2.0),
            status: Some(Status::DISPUTED),
        };
        let record_dispute = Record {
            r#type: Type::DISPUTE,
            client: 1,
            tx: 2,
            amount: None,
            status: None,
        };
        let mut records_map = HashMap::new();
        records_map.insert(record_deposit.tx, record_deposit.clone());
        records_map.insert(record_withdrawal.tx, record_withdrawal.clone());

        let mut client = process(&record_deposit, &clients, &records_map);
        clients.insert(client.client, client);
        client = process(&record_withdrawal, &clients, &records_map);
        clients.insert(client.client, client);
        client = process(&record_dispute, &clients, &records_map);
        clients.insert(client.client, client);

        assert_eq!(client.available, 10.0);
        assert_eq!(client.held, 2.0);
        assert_eq!(client.total, 12.0);
        assert_eq!(client.locked, false);
    }

    #[test]
    fn test_dispute_withdrawal() {
        let mut clients: ClientMap = HashMap::new();
        let record_deposit = Record {
            r#type: Type::DEPOSIT,
            client: 1,
            tx: 1,
            amount: Some(10.0),
            status: None,
        };
        let record_withdrawal = Record {
            r#type: Type::WITHDRAWAL,
            client: 1,
            tx: 2,
            amount: Some(2.0),
            status: Some(Status::DISPUTED),
        };
        let record_dispute = Record {
            r#type: Type::DISPUTE,
            client: 1,
            tx: 2,
            amount: None,
            status: None,
        };
        let mut records_map = HashMap::new();
        records_map.insert(record_deposit.tx, record_deposit.clone());
        records_map.insert(record_withdrawal.tx, record_withdrawal.clone());

        let mut client = process(&record_deposit, &clients, &records_map);
        clients.insert(client.client, client);
        client = process(&record_withdrawal, &clients, &records_map);
        clients.insert(client.client, client);
        client = process(&record_dispute, &clients, &records_map);
        clients.insert(client.client, client);

        assert_eq!(client.available, 8.0);
        assert_eq!(client.held, 2.0);
        assert_eq!(client.total, 8.0);
        assert_eq!(client.locked, false);
    }

    #[test]
    fn test_resolve_deposit() {
        let mut clients: ClientMap = HashMap::new();
        let record_deposit = Record {
            r#type: Type::DEPOSIT,
            client: 1,
            tx: 1,
            amount: Some(10.0),
            status: None,
        };
        let record_withdrawal = Record {
            r#type: Type::DEPOSIT,
            client: 1,
            tx: 2,
            amount: Some(2.0),
            status: Some(Status::DISPUTED),
        };
        let record_dispute = Record {
            r#type: Type::DISPUTE,
            client: 1,
            tx: 2,
            amount: None,
            status: None,
        };
        let record_resolve = Record {
            r#type: Type::RESOLVE,
            client: 1,
            tx: 2,
            amount: None,
            status: None,
        };

        let mut records_map = HashMap::new();
        records_map.insert(record_deposit.tx, record_deposit.clone());
        records_map.insert(record_withdrawal.tx, record_withdrawal.clone());

        let mut client = process(&record_deposit, &clients, &records_map);
        clients.insert(client.client, client);
        client = process(&record_withdrawal, &clients, &records_map);
        clients.insert(client.client, client);
        client = process(&record_dispute, &clients, &records_map);
        clients.insert(client.client, client);
        client = process(&record_resolve, &clients, &records_map);
        clients.insert(client.client, client);

        assert_eq!(client.available, 12.0);
        assert_eq!(client.held, 0.0);
        assert_eq!(client.total, 12.0);
        assert_eq!(client.locked, false);
    }

    #[test]
    fn test_resolve_withdrawal() {
        let mut clients: ClientMap = HashMap::new();
        let record_deposit = Record {
            r#type: Type::DEPOSIT,
            client: 1,
            tx: 1,
            amount: Some(10.0),
            status: None,
        };
        let record_withdrawal = Record {
            r#type: Type::WITHDRAWAL,
            client: 1,
            tx: 2,
            amount: Some(2.0),
            status: Some(Status::DISPUTED),
        };
        let record_dispute = Record {
            r#type: Type::DISPUTE,
            client: 1,
            tx: 2,
            amount: None,
            status: None,
        };
        let record_resolve = Record {
            r#type: Type::RESOLVE,
            client: 1,
            tx: 2,
            amount: None,
            status: None,
        };

        let mut records_map = HashMap::new();
        records_map.insert(record_deposit.tx, record_deposit.clone());
        records_map.insert(record_withdrawal.tx, record_withdrawal.clone());

        let mut client = process(&record_deposit, &clients, &records_map);
        clients.insert(client.client, client);
        client = process(&record_withdrawal, &clients, &records_map);
        clients.insert(client.client, client);
        client = process(&record_dispute, &clients, &records_map);
        clients.insert(client.client, client);
        client = process(&record_resolve, &clients, &records_map);
        clients.insert(client.client, client);

        assert_eq!(client.available, 8.0);
        assert_eq!(client.held, 0.0);
        assert_eq!(client.total, 8.0);
        assert_eq!(client.locked, false);
    }

    #[test]
    fn test_chargeback_deposit() {
        let mut clients: ClientMap = HashMap::new();
        let record_deposit = Record {
            r#type: Type::DEPOSIT,
            client: 1,
            tx: 1,
            amount: Some(10.0),
            status: None,
        };
        let record_withdrawal = Record {
            r#type: Type::DEPOSIT,
            client: 1,
            tx: 2,
            amount: Some(2.0),
            status: Some(Status::DISPUTED),
        };
        let record_dispute = Record {
            r#type: Type::DISPUTE,
            client: 1,
            tx: 2,
            amount: None,
            status: None,
        };
        let record_resolve = Record {
            r#type: Type::CHARGEBACK,
            client: 1,
            tx: 2,
            amount: None,
            status: None,
        };

        let mut records_map = HashMap::new();
        records_map.insert(record_deposit.tx, record_deposit.clone());
        records_map.insert(record_withdrawal.tx, record_withdrawal.clone());

        let mut client = process(&record_deposit, &clients, &records_map);
        clients.insert(client.client, client);
        client = process(&record_withdrawal, &clients, &records_map);
        clients.insert(client.client, client);
        client = process(&record_dispute, &clients, &records_map);
        clients.insert(client.client, client);
        client = process(&record_resolve, &clients, &records_map);
        clients.insert(client.client, client);

        assert_eq!(client.available, 10.0);
        assert_eq!(client.held, 0.0);
        assert_eq!(client.total, 10.0);
        assert_eq!(client.locked, true);
    }

    #[test]
    fn test_chargeback_withdrawal() {
        let mut clients: ClientMap = HashMap::new();
        let record_deposit = Record {
            r#type: Type::DEPOSIT,
            client: 1,
            tx: 1,
            amount: Some(10.0),
            status: None,
        };
        let record_withdrawal = Record {
            r#type: Type::WITHDRAWAL,
            client: 1,
            tx: 2,
            amount: Some(2.0),
            status: Some(Status::DISPUTED),
        };
        let record_dispute = Record {
            r#type: Type::DISPUTE,
            client: 1,
            tx: 2,
            amount: None,
            status: None,
        };
        let record_resolve = Record {
            r#type: Type::CHARGEBACK,
            client: 1,
            tx: 2,
            amount: None,
            status: None,
        };

        let mut records_map = HashMap::new();
        records_map.insert(record_deposit.tx, record_deposit.clone());
        records_map.insert(record_withdrawal.tx, record_withdrawal.clone());

        let mut client = process(&record_deposit, &clients, &records_map);
        clients.insert(client.client, client);
        client = process(&record_withdrawal, &clients, &records_map);
        clients.insert(client.client, client);
        client = process(&record_dispute, &clients, &records_map);
        clients.insert(client.client, client);
        client = process(&record_resolve, &clients, &records_map);
        clients.insert(client.client, client);

        assert_eq!(client.available, 10.0);
        assert_eq!(client.held, 0.0);
        assert_eq!(client.total, 10.0);
        assert_eq!(client.locked, true);
    }
}
