alias t1 := test_echo

default:
  @just --list

build:
    cargo b --release

test_echo: build
    /Users/pierre/RustProjects/flydis/maelstrom/maelstrom test -w echo --bin target/release/flydis --node-count 1 --time-limit 10