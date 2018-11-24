all: build

build:
	cargo build --release
	strip target/release/socket-to-stdout
