#!/bin/bash
# Start the WAS backend server

cd "$(dirname "$0")/.."

echo "Starting WAS Server..."
echo "API running at http://localhost:3000"
echo ""

./target/release/was
