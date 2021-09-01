# invoice

# Running
```shell
$ cargo run -- sample.csv > invoice.csv
```

# Tests
```shell
$ cargo test
```

# Overview
This project requires an csv file path as an argument.
The  csv file must contain headers (see csv_reader.rs section for more info)


## client.rs
This file contains the definitions of a Client and associated functions for it.
The Client struct is what determines the output to stdout.

There were a few questions I had on creating a Client. For example what is a valid Client ID,
that way i can check for correctness. If a client ID isnt provided we should create one,
however without knowing the spec for Client ID, im likely to run into a collision.
So my assumption here for the sake of brevity, was that if Client ID is 0,
they did not have a Client ID and i generated one using a random number generator.
The use of random number generator is not something i would use ever for an ID,
but for the sake of time contrictions, i decided to use what was most simplest.

## csv_reader.rs
This file contains the CSV parser and the definitions of a Record.

There are a few issues I wish i was able to solve regarding csv parsing.
I've made it so that the headers are required, if and no headers are provided,
the application will exit. This is obviously a drawback, but I didn't get time
to add the functionality to manually check for headers.

I decided to use vectors and hashmaps for the Records.
The vector will contain all rows from the csv, and would guarantee order,
while the hashmap only contains deposit and withdrawal transactions,
that will serve a purpose for use during processing of the transactions.

The main drawback here is that everything gets loaded into memory.
So given a large enough file, it is possible to run into issues.

Another issue is since im relying on serde for serialization/deserialization,
I am not able to catch anomalies that may exist with the CSV file.

These issues are all fixable, but due to time limits,
I decided to take the approach i did.

## main.rs
Entry to the application. I tried to keep this as basic as possible,
in hopes that it is clear as to what is happening.

## transaction.rs
This file contains all the functions that correspond to transactions.

There was a question about the code running in a server which could potentially
respond to thousands of requests, I decided to begin work in such a way that
the function would be portable.

There are many cases i do not catch for transactions, and it was mainly due to stay
within the time contraints. If time wasn't an issue, the design would have been slightly different
to prevent things like duplicate transactions, better validity checks, etc. So in this code,
transaction history, duplicate prevention, etc are not present.
