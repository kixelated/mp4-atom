#!/usr/bin/env just --justfile

# Using Just: https://github.com/casey/just?tab=readme-ov-file#installation

export RUST_BACKTRACE := "1"
export RUST_LOG := "info"

# List all of the available commands.
default:
  just --list

# Run the CI checks
check:
	cargo check --all-targets --all-features
	cargo clippy --all-targets --all-features -- -D warnings
	cargo fmt -- --check
	cargo shear # requires: cargo binstall cargo-shear

# Run any CI tests
test:
	cargo test --all-targets
	cargo test --all-targets --features strict

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
