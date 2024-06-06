use std::io::{self, Read};

pub struct Node {}

impl Node {
    pub fn new() -> Self {
        Node {}
    }

    pub fn listen(&self) {
        let mut buf = vec![0; 1024];
        let n = io::stdin().read_to_end(&mut buf).unwrap();
        println!("buf: {:?}", &buf[..n]);
    }
}

impl Default for Node {
    fn default() -> Self {
        Self::new()
    }
}
