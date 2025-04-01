RUSTFLAGS+=-g
RUSTFLAGS+=-Cpasses=sancov-module
RUSTFLAGS+=-Cllvm-args=-sanitizer-coverage-trace-pc-guard
RUSTFLAGS+=-Clink-dead-code
RUSTFLAGS+=-Cforce-frame-pointers=yes
RUSTFLAGS+=-Ctarget-feature=-crt-static
RUSTFLAGS+=-Cinstrument-coverage

.phony: all clean

all:
	RUSTFLAGS="$(RUSTFLAGS)" cargo build --release --target aarch64-apple-darwin

clean:
	cargo clean
