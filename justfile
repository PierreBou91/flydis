alias t1 := test_echo
alias t2 := test_id

default:
  @just --list

build:
  cargo b --release

test_echo: build
  ./maelstrom/maelstrom test -w echo --bin target/release/flydis --node-count 1 --time-limit 10

test_id: build
  ./maelstrom/maelstrom test -w unique-ids --bin target/release/flydis --time-limit 30 --rate 1000 --node-count 3 --availability total --nemesis partition