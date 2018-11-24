all: build

build:
	cargo build --release
	strip target/release/echo-socket
