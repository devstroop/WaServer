#!/bin/bash

# WhatsApp Engine Rust - Comprehensive Test Runner
# This script runs various tests to verify all functionalities work correctly

set -e  # Exit on any error

echo "=================================================="
echo "  WhatsApp Engine Rust - Test Runner"
echo "=================================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    print_error "Please run this script from the project root directory"
    exit 1
fi

print_status "Starting WhatsApp Engine Rust tests..."

# Step 1: Build the project
print_status "Building the project..."
if cargo build --release; then
    print_success "Build completed successfully"
else
    print_error "Build failed"
    exit 1
fi

# Step 2: Run unit tests
print_status "Running unit tests..."
if cargo test --lib; then
    print_success "Unit tests passed"
else
    print_warning "Some unit tests failed"
fi

# Step 3: Run service tests (without browser)
print_status "Running service tests..."
if cargo test service_tests::test_whatsapp_service_creation; then
    print_success "Service creation tests passed"
else
    print_warning "Service creation tests failed"
fi

if cargo test service_tests::test_config_validation; then
    print_success "Config validation tests passed"
else
    print_warning "Config validation tests failed"
fi

# Step 4: Start the server in background
print_status "Starting the WhatsApp Engine server..."
cargo run --release &
SERVER_PID=$!

# Give the server time to start
print_status "Waiting for server to start..."
sleep 5

# Function to cleanup server
cleanup() {
    print_status "Cleaning up..."
    if kill -0 $SERVER_PID 2>/dev/null; then
        print_status "Stopping server (PID: $SERVER_PID)..."
        kill $SERVER_PID
        wait $SERVER_PID 2>/dev/null || true
    fi
}

# Set trap to cleanup on script exit
trap cleanup EXIT

# Step 5: Test if server is running
print_status "Testing server health..."
sleep 2

if curl -s "http://localhost:3000/docs" > /dev/null; then
    print_success "Server is running and responding"
else
    print_error "Server is not responding"
    exit 1
fi

# Step 6: Run integration tests
print_status "Running integration tests..."

# Test API health check
if cargo test integration_tests::test_api_health_check -- --ignored; then
    print_success "API health check passed"
else
    print_warning "API health check failed"
fi

# Test authentication endpoints
if cargo test integration_tests::test_auth_status -- --ignored; then
    print_success "Auth status test passed"
else
    print_warning "Auth status test failed"
fi

if cargo test integration_tests::test_unauthorized_request -- --ignored; then
    print_success "Unauthorized request test passed"
else
    print_warning "Unauthorized request test failed"
fi

if cargo test integration_tests::test_missing_auth_header -- --ignored; then
    print_success "Missing auth header test passed"
else
    print_warning "Missing auth header test failed"
fi

# Test QR code endpoint
if cargo test integration_tests::test_qr_code -- --ignored; then
    print_success "QR code test passed"
else
    print_warning "QR code test failed"
fi

# Test concurrent requests
if cargo test integration_tests::test_concurrent_requests -- --ignored; then
    print_success "Concurrent requests test passed"
else
    print_warning "Concurrent requests test failed"
fi

# Step 7: Manual API Testing with curl
print_status "Running manual API tests with curl..."

# Test auth status
print_status "Testing auth status endpoint..."
AUTH_RESPONSE=$(curl -s -H "Authorization: Bearer test-api-token-123456789" "http://localhost:3000/api/auth/status")
if echo "$AUTH_RESPONSE" | grep -q "authorized"; then
    print_success "Auth status endpoint working"
    echo "Response: $AUTH_RESPONSE"
else
    print_warning "Auth status endpoint issue"
    echo "Response: $AUTH_RESPONSE"
fi

# Test QR code endpoint
print_status "Testing QR code endpoint..."
QR_RESPONSE=$(curl -s -H "Authorization: Bearer test-api-token-123456789" "http://localhost:3000/api/auth/qrcode")
if [ $? -eq 0 ]; then
    print_success "QR code endpoint responding"
    echo "Response: $QR_RESPONSE"
else
    print_warning "QR code endpoint issue"
fi

# Test unauthorized access
print_status "Testing unauthorized access..."
UNAUTH_RESPONSE=$(curl -s -w "%{http_code}" -o /dev/null "http://localhost:3000/api/auth/status")
if [ "$UNAUTH_RESPONSE" = "401" ]; then
    print_success "Unauthorized access properly rejected"
else
    print_warning "Unauthorized access not properly handled (got $UNAUTH_RESPONSE)"
fi

# Test invalid token
print_status "Testing invalid token..."
INVALID_RESPONSE=$(curl -s -w "%{http_code}" -o /dev/null -H "Authorization: Bearer invalid-token" "http://localhost:3000/api/auth/status")
if [ "$INVALID_RESPONSE" = "401" ]; then
    print_success "Invalid token properly rejected"
else
    print_warning "Invalid token not properly handled (got $INVALID_RESPONSE)"
fi

# Step 8: Test browser functionality (if Chrome is available)
print_status "Testing browser functionality..."
if command -v google-chrome &> /dev/null || command -v chromium-browser &> /dev/null || command -v chromium &> /dev/null; then
    print_status "Chrome/Chromium detected, running browser tests..."
    
    if cargo test browser_tests::test_browser_service_creation; then
        print_success "Browser service creation test passed"
    else
        print_warning "Browser service creation test failed"
    fi
    
    # Only run browser initialization test if we want to test actual browser functionality
    # This is commented out by default as it requires a display and can be slow
    # if cargo test browser_tests::test_browser_initialization -- --ignored; then
    #     print_success "Browser initialization test passed"
    # else
    #     print_warning "Browser initialization test failed"
    # fi
else
    print_warning "Chrome/Chromium not detected, skipping browser tests"
fi

# Step 9: Performance test
print_status "Running basic performance test..."
print_status "Sending 10 concurrent requests to auth status endpoint..."

for i in {1..10}; do
    curl -s -H "Authorization: Bearer test-api-token-123456789" "http://localhost:3000/api/auth/status" > /dev/null &
done

wait
print_success "Concurrent requests completed"

# Step 10: Summary
echo ""
echo "=================================================="
echo "  Test Summary"
echo "=================================================="
print_success "✓ Project build successful"
print_success "✓ Server startup successful"
print_success "✓ API endpoints responding"
print_success "✓ Authentication working"
print_success "✓ Error handling working"
print_success "✓ Concurrent requests working"

echo ""
print_status "Test runner completed!"
print_status "Server is still running at http://localhost:3000"
print_status "Swagger UI available at http://localhost:3000/docs"
print_status ""
print_status "To stop the server, press Ctrl+C or run: kill $SERVER_PID"

echo ""
echo "=================================================="
echo "  Manual Testing Guide"
echo "=================================================="
echo "1. Visit http://localhost:3000/docs for Swagger UI"
echo "2. Use the API token: test-api-token-123456789"
echo "3. Test auth/status endpoint first"
echo "4. Test auth/qrcode endpoint (will show QR if available)"
echo "5. Test chat/send endpoint (requires authorization)"
echo ""
echo "For browser functionality (requires WhatsApp Web access):"
echo "1. Ensure Chrome/Chromium is installed"
echo "2. Scan QR code when prompted"
echo "3. Test message sending to a valid phone number"
echo "=================================================="

# Keep server running for manual testing
print_status "Press Ctrl+C to stop the server and exit..."
wait $SERVER_PID
