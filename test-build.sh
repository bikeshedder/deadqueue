#!/bin/bash

set -xe

cargo build

cargo build --no-default-features --features unlimited
cargo build --no-default-features --features resizable
cargo build --no-default-features --features limited

cargo test --no-default-features --features unlimited
cargo test --no-default-features --features resizable
cargo test --no-default-features --features limited
