default: run

run: build
    maelstrom test -w lin-kv --bin target/debug/crakv --time-limit 10 --concurrency 2n --nodes 1

build:
    cargo build
