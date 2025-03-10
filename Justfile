default: run

run: build
    maelstrom test -w lin-kv --bin target/debug/crakv --log-stderr --time-limit 10 --concurrency 2n --node-count 2

build:
    cargo build
