default: run

# https://github.com/jepsen-io/maelstrom/blob/main/README.md
# --time-limit SECONDS: How long to run tests for
# --rate FLOAT: Approximate number of requests per second
# --concurrency INT: Number of clients to run concurrently. Use 4n for 4 times the number of nodes.
# --latency MILLIS: Approximate simulated network latency, during normal operations.
# --latency-dist DIST: What latency distribution should Maelstrom use?
# --nemesis FAULT_TYPE: A comma-separated list of faults to inject
# --nemesis-interval SECONDS: How long between nemesis operations, on average

run: build
    maelstrom test -w lin-kv --bin target/debug/crakv --log-stderr --rate 100 --time-limit 10 --concurrency 2n --node-count 2

build:
    cargo build
