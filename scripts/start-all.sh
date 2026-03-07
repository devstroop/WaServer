#!/bin/bash
# Start both WAS server and client

cd "$(dirname "$0")/.."

echo "======================================"
echo "  WAS - WhatsApp Automation Service   "
echo "======================================"
echo ""
echo "Starting backend server..."
./target/release/was &
SERVER_PID=$!

sleep 2

echo "Starting frontend client..."
cd client && npm run dev &
CLIENT_PID=$!

echo ""
echo "======================================"
echo "  Services Running:"
echo "  - Backend:  http://localhost:3000"
echo "  - Frontend: http://localhost:5173"
echo "======================================"
echo ""
echo "Press Ctrl+C to stop all services"

trap "kill $SERVER_PID $CLIENT_PID 2>/dev/null" EXIT
wait
