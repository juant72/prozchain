#!/bin/bash

# Create the project
cargo new prozchain --bin

# Navigate to project directory
cd prozchain

# Create necessary directories
mkdir -p src/network
mkdir -p src/config
mkdir -p src/util
mkdir -p src/crypto
mkdir -p src/types
mkdir -p src/common
