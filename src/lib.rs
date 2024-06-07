use std::{
    collections::HashMap,
    io::{stdin, stdout, BufWriter, Write},
    time::SystemTime,
};

use serde::{Deserialize, Serialize};

pub struct Node<'a> {
    pub id: String,
    pub messages: Option<Vec<usize>>,
    pub topo: Option<HashMap<String, Vec<String>>>,
    pub ears: std::io::Stdin,
    pub mouth: BufWriter<std::io::StdoutLock<'a>>,
}

impl<'a> Node<'a> {
    pub fn new() -> Self {
        Node {
            id: "NO_ID_YET".to_string(),
            messages: Some(Vec::new()),
            topo: None,
            ears: stdin(),
            mouth: BufWriter::new(stdout().lock()),
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
        self.messages.as_mut().unwrap().push(message);
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

impl<'a> Default for Node<'a> {
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
    pub messages: Option<Vec<usize>>,
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
