use std::io::{self};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Node {
    id: usize,
    name: String,
}

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let stream = serde_json::Deserializer::from_reader(stdin.lock()).into_iter::<Node>();

    for node in stream {
        match node {
            Ok(inner) => println!("{:?}", inner),
            Err(e) => {
                println!("We got an error: {e}");
                continue;
            }
        }
    }

    Ok(())
}
