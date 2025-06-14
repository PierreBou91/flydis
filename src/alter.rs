use std::io::{self, BufRead, StdinLock, Stdout, Write};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    src: String,
    dest: String,
    body: Body,
}

#[derive(Serialize, Deserialize, Debug)]
struct Body {
    r#type: Type,
    #[serde(skip_serializing_if = "Option::is_none")]
    msg_id: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    in_reply_to: Option<usize>,
    #[serde(flatten)]
    specific_fields: BodyKind,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum BodyKind {
    BodyInit {
        node_id: String,
        node_ids: Vec<String>,
    },
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
enum Type {
    Init,
}

// Generic over any BufRead to allow for different input sources like a TcpStream
// might be dumb
struct Node<R: BufRead, W: Write> {
    id: String,
    ears: R,
    mouth: W,
}

impl<'a> Default for Node<StdinLock<'a>, Stdout> {
    fn default() -> Self {
        Self {
            id: String::from("NO_ID"),
            ears: io::stdin().lock(),
            mouth: io::stdout(),
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
                Err(e) => writeln!(&mut self.mouth, "Error deserializing input: {e}")?,
            }
        }
        Ok(())
    }

    fn handle_message(&mut self, message: Message) {
        writeln!(&mut self.mouth, "{:?}", message).unwrap();
        match message.body.r#type {
            Type::Init => todo!(),
        }
    }
}

fn main() -> io::Result<()> {
    let mut node: Node<StdinLock, Stdout> = Node::default();
    node.run()?;
    Ok(())
}

#[allow(unused)]
fn test_serde() {
    let mess = Message {
        src: String::from("n1"),
        dest: String::from("n2"),
        body: Body {
            r#type: Type::Init,
            msg_id: Some(4),
            in_reply_to: Some(5),
            specific_fields: BodyKind::BodyInit {
                node_id: String::from("n3"),
                node_ids: vec![String::from("n1"), String::from("n2")],
            },
        },
    };

    let json = serde_json::to_string_pretty(&mess).unwrap();
    println!("{json}");
}
