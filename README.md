This is my Rust solution to the excellent [fly.io distributed challenge](https://fly.io/dist-sys/3a/).

To test my solution:
1. Clone this repository `git clone https://github.com/PierreBou91/flydis.git`
2. Install [just](https://github.com/casey/just) with `cargo install just`
3. Follow the [prerequisites](https://github.com/jepsen-io/maelstrom/blob/main/doc/01-getting-ready/index.md) from Maelstrom. Specifically, make sure you have JDK, Graphviz, and Gnuplot installed then download the maelstrom tarball and extract it in the sorce directory of this cloned repository.
4. Then run `just <challenge>` to run the challenge you want to test. For example, `just t1` will run the echo challenge.