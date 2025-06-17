#!/bin/bash
# Production Readiness Test Suite for WhatsApp Engine
#
# Comprehensive testing script to validate production deployment

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
BASE_URL="${BASE_URL:-http://localhost:3000}"
API_TOKEN="${API_TOKEN:-your-secure-api-token-change-this}"
PROMETHEUS_URL="${PROMETHEUS_URL:-http://localhost:9090}"
GRAFANA_URL="${GRAFANA_URL:-http://localhost:3001}"

# Test counters
TESTS_TOTAL=0
TESTS_PASSED=0
TESTS_FAILED=0

# Functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[PASS]${NC} $1"
}

log_error() {
    echo -e "${RED}[FAIL]${NC} $1"
}

test_start() {
    echo -e "${YELLOW}[TEST]${NC} $1"
    ((TESTS_TOTAL++))
}

test_pass() {
    log_success "$1"
    ((TESTS_PASSED++))
}

test_fail() {
    log_error "$1"
    ((TESTS_FAILED++))
}

# HTTP test helper
http_test() {
    local url="$1"
    local expected_status="${2:-200}"
    local method="${3:-GET}"
    local headers="${4:-}"
    
    if [ -n "$headers" ]; then
        response=$(curl -s -w "%{http_code}" -X "$method" -H "$headers" "$url" || echo "000")
    else
        response=$(curl -s -w "%{http_code}" -X "$method" "$url" || echo "000")
    fi
    
    status_code="${response: -3}"
    body="${response%???}"
    
    if [ "$status_code" -eq "$expected_status" ]; then
        return 0
    else
        echo "Expected: $expected_status, Got: $status_code"
        return 1
    fi
}

# Health and Readiness Tests
test_health_endpoints() {
    log_info "Testing health and readiness endpoints..."
    
    test_start "Health check endpoint"
    if http_test "$BASE_URL/health" 200; then
        test_pass "Health endpoint returns 200"
    else
        test_fail "Health endpoint failed"
    fi
    
    test_start "Readiness check endpoint"
    if http_test "$BASE_URL/ready" 200; then
        test_pass "Readiness endpoint returns 200"
    else
        test_fail "Readiness endpoint failed"
    fi
    
    test_start "Liveness check endpoint"
    if http_test "$BASE_URL/live" 200; then
        test_pass "Liveness endpoint returns 200"
    else
        test_fail "Liveness endpoint failed"
    fi
    
    test_start "Metrics endpoint"
    if http_test "$BASE_URL/metrics" 200; then
        test_pass "Metrics endpoint returns 200"
    else
        test_fail "Metrics endpoint failed"
    fi
}

# API Authentication Tests
test_api_authentication() {
    log_info "Testing API authentication..."
    
    test_start "Unauthenticated API access"
    if http_test "$BASE_URL/api/auth/status" 401; then
        test_pass "Unauthenticated access properly rejected"
    else
        test_fail "Unauthenticated access should return 401"
    fi
    
    test_start "Invalid token"
    if http_test "$BASE_URL/api/auth/status" 401 "GET" "Authorization: Bearer invalid-token"; then
        test_pass "Invalid token properly rejected"
    else
        test_fail "Invalid token should return 401"
    fi
    
    test_start "Valid token access"
    if http_test "$BASE_URL/api/auth/status" 200 "GET" "Authorization: Bearer $API_TOKEN"; then
        test_pass "Valid token accepted"
    else
        test_fail "Valid token should be accepted"
    fi
}

# API Endpoints Tests
test_api_endpoints() {
    log_info "Testing API endpoints..."
    
    local auth_header="Authorization: Bearer $API_TOKEN"
    
    test_start "Auth status endpoint"
    if http_test "$BASE_URL/api/auth/status" 200 "GET" "$auth_header"; then
        test_pass "Auth status endpoint accessible"
    else
        test_fail "Auth status endpoint failed"
    fi
    
    test_start "QR code endpoint"
    if http_test "$BASE_URL/api/auth/qrcode" 200 "GET" "$auth_header"; then
        test_pass "QR code endpoint accessible"
    else
        test_fail "QR code endpoint failed"
    fi
    
    # Note: Phone auth and chat endpoints require actual WhatsApp connection
    # In production, you'd want more sophisticated integration tests
}

# Documentation Tests
test_documentation() {
    log_info "Testing API documentation..."
    
    test_start "Swagger UI"
    if http_test "$BASE_URL/swagger-ui/" 200; then
        test_pass "Swagger UI accessible"
    else
        test_fail "Swagger UI failed"
    fi
    
    test_start "OpenAPI spec"
    if http_test "$BASE_URL/api-docs/openapi.json" 200; then
        test_pass "OpenAPI spec accessible"
    else
        test_fail "OpenAPI spec failed"
    fi
}

# Security Headers Tests
test_security_headers() {
    log_info "Testing security headers..."
    
    local headers=$(curl -s -I "$BASE_URL/health" || echo "")
    
    test_start "X-Content-Type-Options header"
    if echo "$headers" | grep -qi "x-content-type-options: nosniff"; then
        test_pass "X-Content-Type-Options header present"
    else
        test_fail "X-Content-Type-Options header missing"
    fi
    
    test_start "X-Frame-Options header"
    if echo "$headers" | grep -qi "x-frame-options"; then
        test_pass "X-Frame-Options header present"
    else
        test_fail "X-Frame-Options header missing"
    fi
    
    test_start "X-XSS-Protection header"
    if echo "$headers" | grep -qi "x-xss-protection"; then
        test_pass "X-XSS-Protection header present"
    else
        test_fail "X-XSS-Protection header missing"
    fi
}

# Performance Tests
test_performance() {
    log_info "Testing performance..."
    
    test_start "Response time test"
    local start_time=$(date +%s%N)
    if http_test "$BASE_URL/health" 200; then
        local end_time=$(date +%s%N)
        local duration=$(( (end_time - start_time) / 1000000 ))  # Convert to milliseconds
        
        if [ "$duration" -lt 500 ]; then
            test_pass "Response time: ${duration}ms (< 500ms)"
        else
            test_fail "Response time: ${duration}ms (> 500ms)"
        fi
    else
        test_fail "Performance test failed - endpoint not accessible"
    fi
}

# Monitoring Tests
test_monitoring() {
    log_info "Testing monitoring stack..."
    
    test_start "Prometheus accessibility"
    if http_test "$PROMETHEUS_URL" 200; then
        test_pass "Prometheus accessible"
    else
        test_fail "Prometheus not accessible"
    fi
    
    test_start "Grafana accessibility"
    if http_test "$GRAFANA_URL" 200; then
        test_pass "Grafana accessible"
    else
        test_fail "Grafana not accessible"
    fi
    
    test_start "Prometheus metrics collection"
    if curl -s "$PROMETHEUS_URL/api/v1/query?query=up" | grep -q '"status":"success"'; then
        test_pass "Prometheus collecting metrics"
    else
        test_fail "Prometheus not collecting metrics properly"
    fi
}

# Load Test (basic)
test_load() {
    log_info "Running basic load test..."
    
    test_start "Concurrent requests test"
    local concurrent_requests=10
    local temp_dir=$(mktemp -d)
    
    for i in $(seq 1 $concurrent_requests); do
        (
            if http_test "$BASE_URL/health" 200; then
                echo "success" > "$temp_dir/result_$i"
            else
                echo "failure" > "$temp_dir/result_$i"
            fi
        ) &
    done
    
    wait
    
    local success_count=$(ls "$temp_dir"/result_* 2>/dev/null | xargs grep -l "success" | wc -l)
    rm -rf "$temp_dir"
    
    if [ "$success_count" -eq "$concurrent_requests" ]; then
        test_pass "Handled $concurrent_requests concurrent requests"
    else
        test_fail "Only $success_count/$concurrent_requests requests succeeded"
    fi
}

# Configuration Validation
test_configuration() {
    log_info "Testing configuration validation..."
    
    test_start "Environment variable override"
    local health_response=$(curl -s "$BASE_URL/health" || echo "{}")
    
    if echo "$health_response" | grep -q '"version"'; then
        test_pass "Configuration properly loaded"
    else
        test_fail "Configuration validation failed"
    fi
}

# Main test execution
main() {
    echo "=================================================="
    echo "   WhatsApp Engine Production Readiness Tests"
    echo "=================================================="
    echo
    echo "Base URL: $BASE_URL"
    echo "Testing against: $(curl -s "$BASE_URL/health" | grep -o '"version":"[^"]*"' | cut -d'"' -f4 2>/dev/null || echo "Unknown")"
    echo
    
    # Wait for service to be ready
    log_info "Waiting for service to be ready..."
    local attempts=0
    while [ $attempts -lt 10 ]; do
        if curl -s "$BASE_URL/health" &> /dev/null; then
            break
        fi
        sleep 2
        ((attempts++))
    done
    
    if [ $attempts -eq 10 ]; then
        log_error "Service not ready after 20 seconds"
        exit 1
    fi
    
    # Run all tests
    test_health_endpoints
    test_api_authentication
    test_api_endpoints
    test_documentation
    test_security_headers
    test_performance
    test_configuration
    test_load
    
    # Optional monitoring tests (only if URLs are accessible)
    if curl -s "$PROMETHEUS_URL" &> /dev/null; then
        test_monitoring
    else
        log_info "Skipping monitoring tests (services not accessible)"
    fi
    
    # Results summary
    echo
    echo "=================================================="
    echo "                  Test Results"
    echo "=================================================="
    echo "Total Tests: $TESTS_TOTAL"
    echo -e "Passed: ${GREEN}$TESTS_PASSED${NC}"
    echo -e "Failed: ${RED}$TESTS_FAILED${NC}"
    echo
    
    if [ $TESTS_FAILED -eq 0 ]; then
        echo -e "${GREEN}🚀 All tests passed! WhatsApp Engine is production ready!${NC}"
        exit 0
    else
        echo -e "${RED}❌ Some tests failed. Please review the issues above.${NC}"
        exit 1
    fi
}

# Handle script arguments
case "${1:-all}" in
    "health")
        test_health_endpoints
        ;;
    "auth")
        test_api_authentication
        ;;
    "api")
        test_api_endpoints
        ;;
    "security")
        test_security_headers
        ;;
    "performance")
        test_performance
        ;;
    "monitoring")
        test_monitoring
        ;;
    "load")
        test_load
        ;;
    "all")
        main
        ;;
    *)
        echo "Usage: $0 {all|health|auth|api|security|performance|monitoring|load}"
        echo
        echo "Test categories:"
        echo "  all         - Run all tests (default)"
        echo "  health      - Health and readiness endpoints"
        echo "  auth        - API authentication"
        echo "  api         - API endpoints"
        echo "  security    - Security headers"
        echo "  performance - Response time and performance"
        echo "  monitoring  - Monitoring stack"
        echo "  load        - Basic load testing"
        echo
        echo "Environment variables:"
        echo "  BASE_URL     - Base URL for testing (default: http://localhost:3000)"
        echo "  API_TOKEN    - API token for authentication"
        echo "  PROMETHEUS_URL - Prometheus URL (default: http://localhost:9090)"
        echo "  GRAFANA_URL  - Grafana URL (default: http://localhost:3001)"
        exit 1
        ;;
esac
