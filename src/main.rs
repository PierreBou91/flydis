use flydis::*;
use std::{
    io::{self, BufRead, BufWriter, Write},
    time::SystemTime,
};

fn main() {
    let stdin = io::stdin().lock();
    let stdout = io::stdout();
    let mut stdout = BufWriter::new(stdout.lock());

    for line in stdin.lines() {
        match process_line(line) {
            Ok(Some(response)) => {
                if let Err(e) = serde_json::to_writer(&mut stdout, &response)
                    .and_then(|_| writeln!(stdout).map_err(serde_json::Error::io))
                {
                    eprintln!("Error writing response: {}", e);
                }
                stdout.flush().unwrap();
            }
            Ok(None) => {}
            Err(e) => eprintln!("Error: {}", e),
        }
    }
}

fn process_line(line: Result<String, io::Error>) -> Result<Option<Message>, String> {
    let input = line.map_err(|e| format!("Error reading line: {}", e))?;
    eprintln!("Received input: {}", input);

    let message: Message =
        serde_json::from_str(&input).map_err(|e| format!("Error deserializing message: {}", e))?;
    eprintln!("Deserialized input: {:?}", message);

    match message.body.r#type {
        r#Type::Init => handle_init(message),
        r#Type::Echo => handle_echo(message),
        r#Type::Generate => handle_generate(message),
        r#Type::InitOk | r#Type::EchoOk | r#Type::GenerateOk => {
            eprintln!("Unimplemented message type: {:?}", message.body.r#type);
            Ok(None)
        }
    }
}

fn handle_init(message: Message) -> Result<Option<Message>, String> {
    let response = Message {
        src: message.dest,
        dest: message.src,
        body: Body {
            r#type: r#Type::InitOk,
            msg_id: message.body.msg_id,
            in_reply_to: message.body.msg_id,
            ..Default::default()
        },
    };
    eprintln!("Serialized output: {:?}", response);
    Ok(Some(response))
}

fn handle_echo(message: Message) -> Result<Option<Message>, String> {
    let response = Message {
        src: message.dest,
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
    Ok(Some(response))
}

fn handle_generate(message: Message) -> Result<Option<Message>, String> {
    let response = Message {
        src: message.dest.clone(),
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
    Ok(Some(response))
}
