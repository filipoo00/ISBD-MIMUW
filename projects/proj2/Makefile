SAMPLE ?= sample.bin
ROWS ?= 10000

.PHONY: all build test generate metrics clean

all: build

build:
	cargo build --release

test:
	cargo test

generate: build
	./target/release/generate $(SAMPLE) $(ROWS)

metrics: build
	./target/release/metrics $(SAMPLE)

clean:
	cargo clean