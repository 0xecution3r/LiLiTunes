#!/bin/bash
#this is for legacy ios devices, mainly iPod toouch 1st gen
#test script for debugging purposes
# Path to your custom legacy iOS stack
export PREFIX="$HOME/ios-legacy"

# Use legacy binaries first
export PATH="$PREFIX/bin:$PATH"

# Ensure pkg-config finds legacy .pc files
export PKG_CONFIG_PATH="$PREFIX/lib/pkgconfig:$PREFIX/share/pkgconfig"

# Force runtime linker to use legacy OpenSSL + libs
export LD_LIBRARY_PATH="$PREFIX/lib:$PREFIX/lib64:$LD_LIBRARY_PATH"
