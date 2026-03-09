#!/bin/bash
# Start the WAS frontend client

cd "$(dirname "$0")/../client"

echo "Starting WAS Client..."
echo "Frontend running at http://localhost:5173"
echo ""

npm run dev
