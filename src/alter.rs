use std::io::{self, BufRead, StdinLock, Stdout, Write};

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
    #[serde(flatten)]
    common_fields: CommonBodyFields,
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
}

#[derive(Serialize, Deserialize, Debug)]
struct CommonBodyFields {
    #[serde(skip_serializing_if = "Option::is_none")]
    msg_id: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    in_reply_to: Option<usize>,
}

// Generic over any BufRead to allow for different input sources like a TcpStream
// might be dumb
struct Node<R: BufRead, W: Write> {
    id: String,
    ears: R,
    mouth: W,
    message_counter: usize,
}

impl<'a> Default for Node<StdinLock<'a>, Stdout> {
    fn default() -> Self {
        Self {
            id: String::from("NO_ID"),
            ears: io::stdin().lock(),
            mouth: io::stdout(),
            message_counter: 0,
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

    // {"src":  "c1", "dest": "n1", "body": { "type": "init", "msg_id":   1, "node_id":  "n3", "node_ids": ["n1", "n2", "n3"] }}

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
                        common_fields: CommonBodyFields {
                            msg_id: Some(self.message_counter),
                            in_reply_to: message.body.common_fields.msg_id,
                        },
                    },
                };
                let json = serde_json::to_string(&answer).unwrap();
                writeln!(&mut self.mouth, "{:}", json).unwrap();
                self.message_counter += self.message_counter;
            }
            SpecificBodyFields::InitOk => unreachable!(),
            SpecificBodyFields::Echo { echo } => {
                let answer = Message {
                    src: self.id.clone(),
                    dest: message.src,
                    body: Body {
                        specific_fields: SpecificBodyFields::EchoOk { echo },
                        common_fields: CommonBodyFields {
                            msg_id: Some(self.message_counter),
                            in_reply_to: message.body.common_fields.msg_id,
                        },
                    },
                };
                writeln!(&mut self.mouth, "{:}", json!(answer)).unwrap();
                self.message_counter += self.message_counter;
            }
            SpecificBodyFields::EchoOk { echo } => {
                let _ = echo;
            }
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
            common_fields: CommonBodyFields {
                msg_id: Some(123),
                in_reply_to: Some(345),
            },
        },
    };

    let json = serde_json::to_string_pretty(&mess).unwrap();
    println!("{json}");
}
