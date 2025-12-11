#!/bin/bash

# Micro:bit V2
echo "Adding thumbv7em-none-eabihf target..."
rustup target add thumbv7em-none-eabihf

echo "Adding llvm-tools-preview for objcopy..."
rustup component add llvm-tools-preview

#cargo install cargo-embed   # part of probe-rs now
#cargo install probe-rs  # part of libary, no need install now

echo "Installing probe-rs-tools and flip-link..."
cargo install probe-rs-tools flip-link

echo "Setup complete!"
