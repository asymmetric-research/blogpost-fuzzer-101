
.phony: all clean

all:
	cargo build --release --target x86_64-unknown-linux-gnu

clean:
	cargo clean
