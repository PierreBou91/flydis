use std::{
    collections::{HashMap, HashSet},
    io::{self, BufRead, StdinLock, Stdout, Write},
    time::SystemTime,
};

use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    src: String,
    dest: String,
    body: Body,
}

#[derive(Serialize, Deserialize, Debug)]
struct Body {
    #[serde(flatten)]
    specific_fields: SpecificBodyFields,
    #[serde(skip_serializing_if = "Option::is_none")]
    msg_id: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    in_reply_to: Option<usize>,
}

#[derive(Serialize, Deserialize, Debug)]
// internally tagging this enum allows to match the maelstrom protocol specs
// https://serde.rs/enum-representations.html
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum SpecificBodyFields {
    Init {
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk,
    Echo {
        echo: String,
    },
    EchoOk {
        echo: String,
    },
    Generate,
    GenerateOk {
        id: String,
    },
    Broadcast {
        #[serde(rename = "message")]
        broadcast_message: usize,
    },
    BroadcastOk,
    Read,
    ReadOk {
        messages: HashSet<usize>,
    },
    Topology {
        topology: HashMap<String, Vec<String>>,
    },
    TopologyOk,
    MultiBroadcast {
        messages: HashSet<usize>,
    },
    MultiBroadcastOk,
}

impl SpecificBodyFields {
    #[allow(unused)]
    fn type_name(&self) -> String {
        match self {
            SpecificBodyFields::Init { .. } => String::from("INIT"),
            SpecificBodyFields::InitOk => String::from("INIT_OK"),
            SpecificBodyFields::Echo { .. } => String::from("ECHO"),
            SpecificBodyFields::EchoOk { .. } => String::from("ECHO_OK"),
            SpecificBodyFields::Generate => String::from("GENERATE"),
            SpecificBodyFields::GenerateOk { .. } => String::from("GENERATE_OK"),
            SpecificBodyFields::Broadcast { .. } => String::from("BROADCAST"),
            SpecificBodyFields::BroadcastOk => String::from("BROADCAST_OK"),
            SpecificBodyFields::Read => String::from("READ"),
            SpecificBodyFields::ReadOk { .. } => String::from("READ_OK"),
            SpecificBodyFields::Topology { .. } => String::from("TOPOLOGY"),
            SpecificBodyFields::TopologyOk => String::from("TOPOLOGY_OK"),
            SpecificBodyFields::MultiBroadcast { .. } => String::from("MULTI_BROADCAST"),
            SpecificBodyFields::MultiBroadcastOk => String::from("MULTI_BROADCAST_OK"),
        }
    }
}

// Generic over any BufRead to allow for different input sources like a TcpStream
// might be dumb
struct Node<R: BufRead, W: Write> {
    id: String,
    ears: R,
    mouth: W,
    message_counter: usize,
    store: HashSet<usize>,
    neighbours: Vec<String>,
    to_transmit: HashMap<usize, usize>,
}

impl<'a> Default for Node<StdinLock<'a>, Stdout> {
    fn default() -> Self {
        Self {
            id: String::from("NO_ID"),
            ears: io::stdin().lock(),
            mouth: io::stdout(),
            message_counter: 0,
            store: HashSet::new(),
            neighbours: Vec::new(),
            to_transmit: HashMap::new(),
        }
    }
}

impl<R: BufRead, W: Write> Node<R, W> {
    fn run(&mut self) -> io::Result<()> {
        let mut buf = String::new();

        loop {
            buf.clear();
            let n = self.ears.read_line(&mut buf)?;
            if n == 0 {
                break;
            } // EOF, maybe don't break ?

            let line = buf.trim_end();
            // eprintln!("RAW RECEIVED: {}", line);
            match serde_json::from_str::<Message>(line) {
                Ok(msg) => self.handle_message(msg),
                Err(e) => writeln!(io::stderr().lock(), "Error deserializing input: {e}")?,
            }
        }
        Ok(())
    }

    fn handle_message(&mut self, message: Message) {
        eprintln!(
            "RECEIVED {}: {}",
            message.body.specific_fields.type_name(),
            json!(message)
        );
        match message.body.specific_fields {
            SpecificBodyFields::Init { node_id, node_ids } => {
                assert_eq!(self.id, "NO_ID");
                let _ = node_ids;
                self.id = node_id;
                let answer = Message {
                    src: self.id.clone(),
                    dest: message.src,
                    body: Body {
                        specific_fields: SpecificBodyFields::InitOk,
                        msg_id: Some(self.message_counter),
                        in_reply_to: message.body.msg_id,
                    },
                };
                writeln!(&mut self.mouth, "{:}", json!(answer)).unwrap();
                eprintln!("SENT INIT_OK: {}", json!(answer));
                self.message_counter += 1;
            }
            SpecificBodyFields::InitOk => unreachable!(),
            SpecificBodyFields::Echo { echo } => {
                let answer = Message {
                    src: self.id.clone(),
                    dest: message.src,
                    body: Body {
                        specific_fields: SpecificBodyFields::EchoOk { echo },
                        msg_id: Some(self.message_counter),
                        in_reply_to: message.body.msg_id,
                    },
                };
                writeln!(&mut self.mouth, "{:}", json!(answer)).unwrap();
                eprintln!("SENT ECHO_OK: {}", json!(answer));
                self.message_counter += 1;
            }
            SpecificBodyFields::EchoOk { .. } => unreachable!(),
            SpecificBodyFields::Generate => {
                let answer = Message {
                    src: self.id.clone(),
                    dest: message.src,
                    body: Body {
                        specific_fields: SpecificBodyFields::GenerateOk {
                            id: format!(
                                "{}-{}",
                                self.id,
                                std::time::SystemTime::now()
                                    .duration_since(SystemTime::UNIX_EPOCH)
                                    .unwrap()
                                    .as_micros()
                            ),
                        },
                        msg_id: Some(self.message_counter),
                        in_reply_to: message.body.msg_id,
                    },
                };
                writeln!(&mut self.mouth, "{:}", json!(answer)).unwrap();
                eprintln!("SENT GENERATE_OK: {}", json!(answer));
                self.message_counter += 1;
            }
            SpecificBodyFields::GenerateOk { .. } => unreachable!(),
            SpecificBodyFields::Broadcast { broadcast_message } => {
                if message.dest == self.id {
                    self.store.insert(broadcast_message);
                    let answer = Message {
                        src: self.id.clone(),
                        dest: message.src.clone(),
                        body: Body {
                            specific_fields: SpecificBodyFields::BroadcastOk {},
                            msg_id: Some(self.message_counter),
                            in_reply_to: message.body.msg_id,
                        },
                    };
                    writeln!(&mut self.mouth, "{:}", json!(answer)).unwrap();
                    self.message_counter += 1;
                    eprintln!("SENT BROADCAST_OK: {}", json!(answer));
                }
                self.to_transmit
                    .insert(self.message_counter, broadcast_message);
                // Broadcast any new value to my neighbor if he's not the one who sent it
                for nei in &self.neighbours {
                    if nei != &message.src {
                        // for (transmit_id, value) in &self.to_transmit {
                        //     let answer = Message {
                        //         src: self.id.clone(),
                        //         dest: nei.into(),
                        //         body: Body {
                        //             specific_fields: SpecificBodyFields::Broadcast {
                        //                 broadcast_message: *value,
                        //             },
                        //             msg_id: Some(*transmit_id),
                        //             in_reply_to: message.body.msg_id,
                        //         },
                        //     };
                        //     writeln!(&mut self.mouth, "{:}", json!(answer)).unwrap();
                        //     self.message_counter += 1;
                        //     eprintln!("SENT BROADCAST: {}", json!(answer));
                        // }

                        let answer = Message {
                            src: self.id.clone(),
                            dest: nei.to_string(),
                            body: Body {
                                specific_fields: SpecificBodyFields::Read,
                                msg_id: Some(self.message_counter),
                                in_reply_to: None,
                            },
                        };
                        writeln!(&mut self.mouth, "{}", json!(answer)).unwrap();
                        self.message_counter += 1;
                        eprintln!("SENT READ: {}", json!(answer));
                    }
                }
            }
            SpecificBodyFields::BroadcastOk => {
                self.to_transmit.remove(&message.body.in_reply_to.unwrap());
                eprintln!(
                    "BROADCAST REMOVED {}\nREMAINING {:?}",
                    json!(&message.body.in_reply_to.unwrap()),
                    self.to_transmit
                );
            }
            SpecificBodyFields::Read => {
                let answer = Message {
                    src: self.id.clone(),
                    dest: message.src,
                    body: Body {
                        specific_fields: SpecificBodyFields::ReadOk {
                            messages: self.store.clone(),
                        },
                        msg_id: Some(self.message_counter),
                        in_reply_to: message.body.msg_id,
                    },
                };
                writeln!(&mut self.mouth, "{:}", json!(answer)).unwrap();
                self.message_counter += 1;
                eprintln!("SENT READ_OK: {}", json!(answer));
            }
            SpecificBodyFields::ReadOk { messages } => {
                for value in &self.store {
                    if !messages.contains(value) {
                        let answer = Message {
                            src: self.id.clone(),
                            dest: message.src.clone(),
                            body: Body {
                                specific_fields: SpecificBodyFields::Broadcast {
                                    broadcast_message: *value,
                                },
                                msg_id: Some(self.message_counter),
                                in_reply_to: message.body.msg_id,
                            },
                        };
                        writeln!(&mut self.mouth, "{:}", json!(answer)).unwrap();
                        self.message_counter += 1;
                        eprintln!("SENT BROADCAST: {}", json!(answer));
                    }
                }
            }
            SpecificBodyFields::Topology { topology } => {
                if let Some(nei) = topology.get(&self.id) {
                    self.neighbours = nei.to_vec();
                }
                let answer = Message {
                    src: self.id.clone(),
                    dest: message.src,
                    body: Body {
                        specific_fields: SpecificBodyFields::TopologyOk,
                        msg_id: Some(self.message_counter),
                        in_reply_to: message.body.msg_id,
                    },
                };
                writeln!(&mut self.mouth, "{:}", json!(answer)).unwrap();
                self.message_counter += 1;
                eprintln!("SENT TOPO_OK {}", json!(answer))
            }
            SpecificBodyFields::TopologyOk => unreachable!(),

            SpecificBodyFields::MultiBroadcast { .. } => {}

            SpecificBodyFields::MultiBroadcastOk => todo!(),
        }
    }
}

fn main() -> io::Result<()> {
    let mut node: Node<StdinLock, Stdout> = Node::default();
    node.run()?;
    // test_serde();
    Ok(())
}

#[allow(unused)]
fn test_serde() {
    let mess = Message {
        src: String::from("pierre"),
        dest: String::from("n1"),
        body: Body {
            specific_fields: SpecificBodyFields::Init {
                node_id: String::from("n1"),
                node_ids: vec![String::from("n1"), String::from("n2")],
            },
            msg_id: Some(123),
            in_reply_to: Some(345),
        },
    };

    let json = serde_json::to_string_pretty(&mess).unwrap();
    println!("{json}");
}
