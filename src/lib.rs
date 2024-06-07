use std::{
    collections::{HashMap, HashSet},
    io::{stdin, stdout, Write},
    time::SystemTime,
};

use serde::{Deserialize, Serialize};

pub struct Node {
    pub id: String,
    pub messages: Option<HashSet<usize>>,
    pub topo: Option<HashMap<String, Vec<String>>>,
    pub ears: std::io::Stdin,
    pub mouth: std::io::Stdout,
}

impl Node {
    pub fn new() -> Self {
        Node {
            id: "NO_ID_YET".to_string(),
            messages: Some(HashSet::new()),
            topo: None,
            ears: stdin(),
            mouth: stdout(),
        }
    }

    pub fn speak(&mut self, message: Message) {
        if let Err(e) = serde_json::to_writer(&mut self.mouth, &message)
            .and_then(|_| writeln!(self.mouth).map_err(serde_json::Error::io))
        {
            eprintln!("Error writing response: {}", e);
        }
        self.mouth.flush().unwrap();
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn push_message(&mut self, message: usize) {
        self.messages.as_mut().unwrap().insert(message);
    }

    pub fn create_topo(&mut self, topo: HashMap<String, Vec<String>>) {
        self.topo = Some(topo)
    }

    pub fn handle_init(&mut self, message: Message) {
        self.id = message.body.node_id.unwrap();
        let response = Message {
            src: self.id().to_string(),
            dest: message.src,
            body: Body {
                r#type: r#Type::InitOk,
                msg_id: message.body.msg_id,
                in_reply_to: message.body.msg_id,
                ..Default::default()
            },
        };
        eprintln!("Serialized output: {:?}", response);
        self.speak(response);
    }

    pub fn handle_echo(&mut self, message: Message) {
        let response = Message {
            src: self.id.to_string(),
            dest: message.src,
            body: Body {
                r#type: r#Type::EchoOk,
                msg_id: message.body.msg_id,
                in_reply_to: message.body.msg_id,
                echo: message.body.echo,
                ..Default::default()
            },
        };
        eprintln!("Serialized output: {:?}", response);
        self.speak(response);
    }

    pub fn handle_generate(&mut self, message: Message) {
        let response = Message {
            src: self.id().to_string(),
            dest: message.src,
            body: Body {
                r#type: r#Type::GenerateOk,
                id: Some(format!("{}{:?}", message.dest, SystemTime::now())),
                msg_id: message.body.msg_id,
                in_reply_to: message.body.msg_id,
                ..Default::default()
            },
        };
        eprintln!("Serialized output: {:?}", response);
        self.speak(response);
    }

    pub fn handle_broadcast(&mut self, message: Message) {
        self.push_message(message.body.message.unwrap());
        let neighbors = self.topo.as_ref().unwrap().get(self.id()).unwrap();
        let mut props = Vec::new();

        // first broadcast to every neighboring node
        for neighbor in neighbors {
            // except for the one who sent
            if *neighbor == message.src {
                continue;
            }
            let propagate = Message {
                src: self.id().to_string(),
                dest: neighbor.to_string(),
                body: Body {
                    r#type: r#Type::Broadcast,
                    message: message.body.message,
                    ..Default::default()
                },
            };
            // self.speak(propagate);
            props.push(propagate)
        }

        for p in props {
            self.speak(p);
        }

        // then answer the boradcast_ok
        let response = Message {
            src: self.id().to_string(),
            dest: message.src,
            body: Body {
                r#type: r#Type::BroadcastOk,
                msg_id: message.body.msg_id,
                in_reply_to: message.body.msg_id,
                ..Default::default()
            },
        };
        eprintln!("Serialized output: {:?}", response);
        self.speak(response);
    }

    pub fn handle_read(&mut self, message: Message) {
        let response = Message {
            src: self.id().to_string(),
            dest: message.src,
            body: Body {
                r#type: r#Type::ReadOk,
                msg_id: message.body.msg_id,
                in_reply_to: message.body.msg_id,
                messages: self.messages.clone(),
                ..Default::default()
            },
        };
        eprintln!("Serialized output: {:?}", response);
        self.speak(response);
    }

    pub fn handle_topology(&mut self, message: Message) {
        self.create_topo(message.body.topology.unwrap());
        let response = Message {
            src: self.id().to_string(),
            dest: message.src,
            body: Body {
                r#type: r#Type::TopologyOk,
                msg_id: message.body.msg_id,
                in_reply_to: message.body.msg_id,
                ..Default::default()
            },
        };
        eprintln!("Serialized output: {:?}", response);
        self.speak(response);
    }
}

impl Default for Node {
    fn default() -> Self {
        Node::new()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub src: String,
    pub dest: String,
    pub body: Body,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Body {
    pub r#type: r#Type,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msg_id: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_reply_to: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub echo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub messages: Option<HashSet<usize>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topology: Option<HashMap<String, Vec<String>>>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum r#Type {
    #[default]
    Init,
    InitOk,
    Echo,
    EchoOk,
    Generate,
    GenerateOk,
    Broadcast,
    BroadcastOk,
    Read,
    ReadOk,
    Topology,
    TopologyOk,
}
