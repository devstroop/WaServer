#!/bin/bash

# Test script for phone authentication
# Uses the phone numbers provided by the user

SENDER_PHONE="919501005734"
CLIENT_PHONE="917488797047"
API_TOKEN="test-api-token-123456789"
BASE_URL="http://localhost:3000"

echo "🔧 Testing WhatsApp Phone Authentication"
echo "Sender: +$SENDER_PHONE"
echo "Client: +$CLIENT_PHONE"
echo ""

# Function to make API call and show response
make_api_call() {
    local method=$1
    local endpoint=$2
    local description=$3
    
    echo "📞 $description"
    echo "   $method $endpoint"
    
    if [ "$method" = "GET" ]; then
        response=$(curl -s -w "\nHTTP_STATUS:%{http_code}" \
            -H "Authorization: Bearer $API_TOKEN" \
            "$BASE_URL$endpoint")
    else
        response=$(curl -s -w "\nHTTP_STATUS:%{http_code}" \
            -X "$method" \
            -H "Authorization: Bearer $API_TOKEN" \
            "$BASE_URL$endpoint")
    fi
    
    # Extract HTTP status and body
    http_status=$(echo "$response" | grep "HTTP_STATUS:" | cut -d':' -f2)
    body=$(echo "$response" | sed '/HTTP_STATUS:/d')
    
    echo "   Status: $http_status"
    echo "   Response: $body"
    echo ""
}

# Test 1: Check initial auth status
make_api_call "GET" "/api/auth/status" "Checking initial authentication status"

# Test 2: Try phone authentication with sender number
make_api_call "POST" "/api/auth/phone/$SENDER_PHONE" "Authenticating with sender phone number"

# Wait a moment
sleep 2

# Test 3: Check auth status again
make_api_call "GET" "/api/auth/status" "Checking authentication status after phone auth"

echo "✅ Phone authentication test completed"
echo ""
echo "📋 Instructions:"
echo "1. Check the browser window for the link code"
echo "2. Open WhatsApp on your phone"
echo "3. Go to Settings > Linked Devices > Link a Device"
echo "4. Enter the code shown in the browser"
echo "5. Run: curl -H 'Authorization: Bearer $API_TOKEN' http://localhost:3000/api/auth/status"
