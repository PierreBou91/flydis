use flydis::*;
use std::io::BufRead;

fn main() {
    let mut me = Node::new();

    for line in me.ears.lock().lines() {
        let line = line.expect("Line is correctly read");
        eprintln!("Received input: {}", line);

        let message: Message = serde_json::from_str(&line).expect("Line is correctly deserialized");
        eprintln!("Deserialized input: {:?}", message);

        match message.body.r#type {
            Type::Init => me.handle_init(message),
            Type::Echo => me.handle_echo(message),
            Type::Generate => me.handle_generate(message),
            Type::Broadcast => me.handle_broadcast(message),
            Type::Read => me.handle_read(message),
            Type::Topology => me.handle_topology(message),
            r#Type::InitOk
            | r#Type::EchoOk
            | r#Type::GenerateOk
            | r#Type::BroadcastOk
            | r#Type::ReadOk
            | r#Type::TopologyOk => {
                eprintln!("Unimplemented message type: {:?}", message.body.r#type);
                panic!();
            }
        };
    }
}
