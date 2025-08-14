# Makefile for LeoNTP_QUERY
# Automatically detects binary name from Cargo.toml

# Extract binary name from Cargo.toml
BIN_NAME := $(shell grep -A1 '\[bin\]' Cargo.toml | grep name | cut -d'"' -f2)

# If no bin section found, fallback to package name
ifeq ($(BIN_NAME),)
    BIN_NAME := $(shell grep '^name' Cargo.toml | head -n1 | cut -d'"' -f2)
endif

# Default target
all: build

# Compile in release mode
build:
	cargo build --release --bin $(BIN_NAME)

# Run the program (after build)
run: build
	./target/release/$(BIN_NAME)

# Clean build artifacts
clean:
	cargo clean

# Build and run in debug mode
debug:
	cargo run --bin $(BIN_NAME)

# Rebuild from scratch
rebuild: clean build
