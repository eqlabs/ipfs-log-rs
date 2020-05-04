all: build

bench:
	rustup run nightly cargo bench

docs:
	cargo doc --no-deps -p cid -p ipfs_log

watch:
	cargo watch -x "check && cargo doc --no-deps && cargo test -- --nocapture"
