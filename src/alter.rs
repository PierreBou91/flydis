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
}

// Generic over any BufRead to allow for different input sources like a TcpStream
// might be dumb
struct Node<R: BufRead, W: Write> {
    id: String,
    ears: R,
    mouth: W,
    message_counter: usize,
    store: HashSet<usize>,
    topo: HashMap<String, Vec<String>>,
}

impl<'a> Default for Node<StdinLock<'a>, Stdout> {
    fn default() -> Self {
        Self {
            id: String::from("NO_ID"),
            ears: io::stdin().lock(),
            mouth: io::stdout(),
            message_counter: 0,
            store: HashSet::new(),
            topo: HashMap::new(),
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
            match serde_json::from_str::<Message>(line) {
                Ok(msg) => self.handle_message(msg),
                Err(e) => writeln!(io::stderr().lock(), "Error deserializing input: {e}")?,
            }
        }
        Ok(())
    }

    fn handle_message(&mut self, message: Message) {
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
                self.message_counter += self.message_counter;
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
                self.message_counter += self.message_counter;
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
                self.message_counter += self.message_counter;
            }
            SpecificBodyFields::GenerateOk { .. } => unreachable!(),
            SpecificBodyFields::Broadcast { broadcast_message } => {
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
                self.message_counter += self.message_counter;

                // Broadcast any new value to my neighbor if he's not the one who sent it
                if let Some(neighbours) = self.topo.get(&self.id) {
                    for nei in neighbours {
                        if nei != &message.src {
                            let answer = Message {
                                src: self.id.clone(),
                                dest: nei.into(),
                                body: Body {
                                    specific_fields: SpecificBodyFields::Broadcast {
                                        broadcast_message,
                                    },
                                    msg_id: Some(self.message_counter),
                                    in_reply_to: message.body.msg_id,
                                },
                            };
                            writeln!(&mut self.mouth, "{:}", json!(answer)).unwrap();
                            self.message_counter += self.message_counter;
                        }
                    }
                }
            }
            SpecificBodyFields::BroadcastOk => {}
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
                self.message_counter += self.message_counter;
            }
            SpecificBodyFields::ReadOk { .. } => unreachable!(),
            SpecificBodyFields::Topology { topology } => {
                self.topo = topology;
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
                self.message_counter += self.message_counter;
            }
            SpecificBodyFields::TopologyOk => unreachable!(),
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
        src: String::from("n1"),
        dest: String::from("n2"),
        body: Body {
            specific_fields: SpecificBodyFields::Init {
                node_id: String::from("n1"),
                node_ids: vec![String::from("qwer"), String::from("Yo")],
            },
            msg_id: Some(123),
            in_reply_to: Some(345),
        },
    };

    let json = serde_json::to_string_pretty(&mess).unwrap();
    println!("{json}");
}
