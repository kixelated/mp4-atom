#!/usr/bin/env just --justfile

# Using Just: https://github.com/casey/just?tab=readme-ov-file#installation

export RUST_BACKTRACE := "1"
export RUST_LOG := "info"

# List all of the available commands.
default:
  just --list

# Run the checks.
check:
	cargo check --all-targets --all-features
	cargo clippy --all-targets --all-features -- -D warnings
	cargo fmt -- --check
	cargo shear # requires: cargo binstall cargo-shear

# Run the tests.
test:
	cargo test --all-targets

# Check and test on the MSRV, Cargo.toml's rust-version, when a change might need newer Rust.
msrv:
	cargo +1.88.0 check --all-targets --all-features
	cargo +1.88.0 test --all-targets

# Test on 32-bit, when a change converts sizes or counts to usize. Needs a 32-bit C toolchain.
i686:
	cargo test --all-targets --target i686-unknown-linux-gnu

# Automatically fix some issues.
fix:
	cargo fix --allow-staged --all-targets --all-features
	cargo clippy --fix --allow-staged --all-targets --all-features
	cargo fmt --all
	cargo shear --fix

# Upgrade any tooling
upgrade:
	rustup upgrade

	# Install cargo-upgrades if needed.
	cargo install cargo-upgrades cargo-edit
	cargo upgrade
