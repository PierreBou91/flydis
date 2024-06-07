alias t1 := test_echo
alias t2 := test_id
alias t3 := test_broadcast1
alias t4 := test_broadcast2

bin := "target/release/flydis"

default:
  @just --list

build:
  cargo b --release

test_echo: build
  ./maelstrom/maelstrom test -w echo --bin {{bin}} --node-count 1 --time-limit 10

test_id: build
  ./maelstrom/maelstrom test -w unique-ids --bin {{bin}} --time-limit 30 --rate 1000 --node-count 3 --availability total --nemesis partition

test_broadcast1: build
  ./maelstrom/maelstrom test -w broadcast --bin {{bin}} --node-count 1 --time-limit 20 --rate 10

test_broadcast2: build
  ./maelstrom/maelstrom test -w broadcast --bin {{bin}} --node-count 5 --time-limit 20 --rate 10
