use csv::{ReaderBuilder, Trim};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::io;

use crate::client::ClientMap;

// Hashmap for fast transaction lookup
pub type RecordMap = HashMap<u32, Record>;
pub type RecordVec = Vec<Record>;

// Spec of the input csv file.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Record {
    pub r#type: Type,
    pub client: u16,
    pub tx: u32,
    pub amount: Option<f64>,
    // additional field
    pub status: Option<Status>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq)]
pub enum Type {
    #[serde(rename = "deposit")]
    DEPOSIT,
    #[serde(rename = "withdrawal")]
    WITHDRAWAL,
    #[serde(rename = "dispute")]
    DISPUTE,
    #[serde(rename = "resolve")]
    RESOLVE,
    #[serde(rename = "chargeback")]
    CHARGEBACK,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq)]
pub enum Status {
    NONE,
    DISPUTED,
    RESOLVED,
    CHARGEBACKED,
}

// Contains a vector and hashmap of records
#[derive(Clone, Debug, Default)]
pub struct CSV {
    pub records_vec: RecordVec,
    pub records_map: RecordMap,
}

impl CSV {
    // Loads the provided csv file and returns a CSV
    pub fn load(args: Vec<String>) -> Result<Self, Box<dyn Error>> {
        // check for file path
        if args.len() < 2 {
            return Err("Error: No file path provided")?;
        }

        // create vector and hashmap
        let mut records_vec = vec![];
        let mut records_map: RecordMap = HashMap::new();

        // read csv with options
        // FIXME: determine headers manually
        // as it will allow us to take both file types
        let mut rdr = ReaderBuilder::new()
            .has_headers(true)
            .trim(Trim::All)
            .from_path(&args[1])?;

        // convert data into Record and add to vector
        for result in rdr.deserialize() {
            let record: Record = result?;
            records_vec.push(record.clone());

            // update record with disputed field
            match record.r#type {
                Type::DEPOSIT => records_map.insert(record.tx, record),
                Type::WITHDRAWAL => records_map.insert(record.tx, record),
                Type::DISPUTE => {
                    let orignal_record = records_map.get(&record.tx);

                    if orignal_record.is_some() {
                        let mut cloned_record =
                            orignal_record.expect("Failed to get record").clone();
                        cloned_record.status = Some(Status::DISPUTED);
                        records_map.insert(cloned_record.tx, cloned_record.clone());
                    }

                    None
                }
                Type::RESOLVE => None,
                Type::CHARGEBACK => None,
            };
        }

        // return CSV
        Ok(CSV {
            records_vec,
            records_map,
        })
    }

    // Writes the provided clients as csv to the stdout
    pub fn output(clients: ClientMap) -> Result<(), Box<dyn Error>> {
        let mut wtr = csv::Writer::from_writer(io::stdout());

        for (_id, client) in clients {
            wtr.serialize(client)?;
        }

        wtr.flush()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_argument() {
        let csv = CSV::load(vec!["".to_string()]);

        assert_eq!(csv.is_err(), true);
    }

    #[test]
    fn test_non_existant_file() {
        let csv = CSV::load(vec!["main".to_string(), "no_file.csv".to_string()]);

        assert_eq!(csv.is_err(), true);
    }

    #[test]
    fn test_load() {
        let csv = CSV::load(vec!["main".to_string(), "sample.csv".to_string()])
            .expect("Failed load csv file");

        assert_eq!(csv.records_vec.len(), 17);
    }
}
