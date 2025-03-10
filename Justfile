default: run

run: build
    maelstrom test -w lin-kv --bin target/debug/crakv --log-stderr --time-limit 10 --concurrency 8n --node-count 8

build:
    cargo build
