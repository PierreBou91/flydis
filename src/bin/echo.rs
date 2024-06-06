use std::io::{self, BufRead, Write};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    src: String,
    dest: String,
    body: Body,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct Body {
    r#type: r#Type,
    #[serde(skip_serializing_if = "Option::is_none")]
    msg_id: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    in_reply_to: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    node_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    node_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    echo: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum r#Type {
    #[default]
    Init,
    InitOk,
    Echo,
    EchoOk,
}

fn main() {
    let stdin = io::stdin().lock();
    let mut stdout = io::stdout().lock();

    for line in stdin.lines() {
        match line {
            Ok(input) => {
                eprintln!("Received input: {}", input);
                let message: Message = match serde_json::from_str(&input) {
                    Ok(message) => {
                        eprintln!("Deserialized input: {:?}", message);
                        message
                    }
                    Err(error) => {
                        eprintln!("Error deserializing message: {}", error);
                        continue;
                    }
                };
                match message.body.r#type {
                    r#Type::Init => {
                        let res = Message {
                            src: message.dest,
                            dest: message.src,
                            body: Body {
                                r#type: r#Type::InitOk,
                                msg_id: message.body.msg_id,
                                in_reply_to: Some(message.body.msg_id.unwrap()),
                                ..Default::default()
                            },
                        };
                        eprintln!("Serialized output: {:?}", res);
                        serde_json::to_writer(&mut stdout, &res).unwrap();
                        stdout.write_all(b"\n").unwrap();
                        stdout.flush().unwrap();
                    }
                    r#Type::InitOk => {
                        unimplemented!("InitOk message handling")
                    }
                    r#Type::Echo => {
                        let res = Message {
                            src: message.dest,
                            dest: message.src,
                            body: Body {
                                r#type: r#Type::EchoOk,
                                msg_id: message.body.msg_id,
                                in_reply_to: Some(message.body.msg_id.unwrap()),
                                echo: message.body.echo,
                                ..Default::default()
                            },
                        };
                        eprintln!("Serialized output: {:?}", res);
                        serde_json::to_writer(&mut stdout, &res).unwrap();
                        stdout.write_all(b"\n").unwrap();
                        stdout.flush().unwrap();
                    }
                    r#Type::EchoOk => {
                        unimplemented!("EchoOk message handling")
                    }
                }
            }
            Err(error) => {
                eprintln!("Error reading line: {}", error);
                break;
            }
        }
    }
}
