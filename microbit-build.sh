#!/bin/bash
#
# Build script for micro:bit-snake
#
# Usage:
#   ./microbit-build.sh           # Build and auto-flash to micro:bit
#   ./microbit-build.sh --no-flash  # Build only, no flashing
#

# Parse command line arguments
AUTO_FLASH=true
if [ "$1" == "--no-flash" ]; then
    AUTO_FLASH=false
fi

pgname=$( cargo metadata --no-deps --format-version 1|jq -r '.packages[0] | .name')

# Build the project
echo "Building project..."
cargo build --release --target thumbv7em-none-eabihf

# Check if build was successful
if [ $? -ne 0 ]; then
    echo "Build failed!"
    exit 1
fi

# Convert ELF to HEX using llvm-objcopy
echo "Converting to HEX format..."

# Find llvm-objcopy from rustup's llvm-tools
LLVM_TOOLS=$(rustc --print sysroot)/lib/rustlib/x86_64-pc-windows-gnu/bin
if [ ! -d "$LLVM_TOOLS" ]; then
    LLVM_TOOLS=$(rustc --print sysroot)/lib/rustlib/x86_64-pc-windows-msvc/bin
fi

if [ -f "$LLVM_TOOLS/llvm-objcopy.exe" ]; then
    "$LLVM_TOOLS/llvm-objcopy.exe" -O ihex \
        target/thumbv7em-none-eabihf/release/${pgname} \
        target/thumbv7em-none-eabihf/release/${pgname}.hex
    echo "Success! HEX file generated at: target/thumbv7em-none-eabihf/release/${pgname}.hex"
else
    echo "Error: llvm-objcopy not found!"
    echo "Make sure llvm-tools-preview is installed: rustup component add llvm-tools-preview"
    exit 1
fi

# Prompt to flash to micro:bit
if [ "$AUTO_FLASH" = false ]; then
    echo ""
    echo "Build complete! HEX file is ready at:"
    echo "  target/thumbv7em-none-eabihf/release/${pgname}.hex"
    echo "You can manually copy it to your micro:bit."
    exit 0
fi

# Find micro:bit drive
MICROBIT_DRIVE=""
for drive in {D..Z}; do
    # Check both /d/ style (MSYS) and D:/ style (Windows)
    if [ -d "/$drive/" ] && [ -f "/$drive/MICROBIT.HTM" ]; then
        MICROBIT_DRIVE="/$drive"
        break
    elif [ -d "$drive:/" ] && [ -f "$drive:/MICROBIT.HTM" ]; then
        MICROBIT_DRIVE="$drive:"
        break
    fi
done

if [ -z "$MICROBIT_DRIVE" ]; then
    echo "Error: micro:bit not found!"
    echo "Please make sure your micro:bit is connected and shows up as a USB drive."
    echo "You can manually copy the file from: target/thumbv7em-none-eabihf/release/${pgname}.hex"
    exit 1
fi

echo "Found micro:bit at: $MICROBIT_DRIVE"

echo ""
echo "========================================"
echo "Ready to flash to micro:bit"
echo "========================================"
echo "Please connect your micro:bit to USB and press Enter..."
echo "(Or press Ctrl+C to cancel)"
read -p ""

echo "Copying hex file to micro:bit..."

cp -f target/thumbv7em-none-eabihf/release/${pgname}.hex "$MICROBIT_DRIVE/${pgname}.hex"

if [ $? -eq 0 ]; then
    echo "Success! File copied to micro:bit."
    echo "The micro:bit LED should blink as it flashes the new program."
    echo "Wait for the flashing to complete before disconnecting."
else
    echo "Error: Failed to copy file to micro:bit."
    exit 1
fi
