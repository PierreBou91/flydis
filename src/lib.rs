use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub struct Node {
    pub id: String,
    pub messages: Option<Vec<usize>>,
    pub topo: Option<HashMap<String, Vec<String>>>,
}

impl Node {
    pub fn new() -> Self {
        Node {
            id: "NO_ID_YET".to_string(),
            messages: Some(Vec::new()),
            topo: None,
        }
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
