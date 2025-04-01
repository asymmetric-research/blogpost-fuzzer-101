RUSTFLAGS_COVERAGE := -C instrument-coverage

.phony: all clean

all:
	RUSTFLAGS="$(RUSTFLAGS_COVERAGE)" cargo build --release -Z build-std=std,panic_abort

clean:
	cargo clean
