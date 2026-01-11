#!/bin/bash

echo "=== Checking Rust Build ==="
echo ""

cd pacs-server

echo "Running cargo check..."
cargo check 2>&1 | tail -20

echo ""
echo "=== Build check completed ==="

