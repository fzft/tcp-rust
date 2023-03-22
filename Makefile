build:
	carge build

build-linux:
	cargo build --target=x86_64-unknown-linux-musl

scp: build-linux
	scp target/debug/tcp-rust mos@192.168.64.8:/home/mos/tcp-rust
