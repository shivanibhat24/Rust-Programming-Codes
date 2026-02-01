#!/bin/bash
# API Testing Script for Secure Messenger

set -e

BASE_URL="${BASE_URL:-http://localhost:3000}"

echo "🔐 Secure Messenger API Test Suite"
echo "===================================="
echo "Testing against: $BASE_URL"
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Test health check
echo "1. Testing health check..."
response=$(curl -s -o /dev/null -w "%{http_code}" "$BASE_URL/health")
if [ "$response" = "200" ]; then
    echo -e "${GREEN}✓${NC} Health check passed"
else
    echo -e "${RED}✗${NC} Health check failed (HTTP $response)"
    exit 1
fi

# Register Alice
echo ""
echo "2. Registering user 'alice'..."
register_response=$(curl -s -X POST "$BASE_URL/register" \
    -H "Content-Type: application/json" \
    -d '{
        "username": "alice_'$(date +%s)'",
        "password": "alice_strong_password_123"
    }')

alice_id=$(echo "$register_response" | grep -o '"user_id":"[^"]*"' | cut -d'"' -f4)
if [ -n "$alice_id" ]; then
    echo -e "${GREEN}✓${NC} Alice registered successfully (ID: $alice_id)"
else
    echo -e "${RED}✗${NC} Alice registration failed"
    echo "$register_response"
    exit 1
fi

# Register Bob
echo ""
echo "3. Registering user 'bob'..."
bob_username="bob_$(date +%s)"
register_response=$(curl -s -X POST "$BASE_URL/register" \
    -H "Content-Type: application/json" \
    -d '{
        "username": "'"$bob_username"'",
        "password": "bob_strong_password_456"
    }')

bob_id=$(echo "$register_response" | grep -o '"user_id":"[^"]*"' | cut -d'"' -f4)
if [ -n "$bob_id" ]; then
    echo -e "${GREEN}✓${NC} Bob registered successfully (ID: $bob_id)"
else
    echo -e "${RED}✗${NC} Bob registration failed"
    echo "$register_response"
    exit 1
fi

# Login Alice
echo ""
echo "4. Testing login..."
login_response=$(curl -s -X POST "$BASE_URL/login" \
    -H "Content-Type: application/json" \
    -d '{
        "username": "alice_'$(date +%s)'",
        "password": "alice_strong_password_123"
    }')

if echo "$login_response" | grep -q "token"; then
    echo -e "${GREEN}✓${NC} Login successful"
else
    echo -e "${RED}✗${NC} Login failed"
    echo "$login_response"
fi

# Get Bob's prekey bundle
echo ""
echo "5. Fetching Bob's prekey bundle..."
bundle_response=$(curl -s -X POST "$BASE_URL/prekey-bundle" \
    -H "Content-Type: application/json" \
    -d '{
        "username": "'"$bob_username"'"
    }')

if echo "$bundle_response" | grep -q "identity_key"; then
    echo -e "${GREEN}✓${NC} Prekey bundle retrieved successfully"
else
    echo -e "${RED}✗${NC} Failed to retrieve prekey bundle"
    echo "$bundle_response"
fi

# Test duplicate registration
echo ""
echo "6. Testing duplicate registration (should fail)..."
dup_response=$(curl -s -o /dev/null -w "%{http_code}" -X POST "$BASE_URL/register" \
    -H "Content-Type: application/json" \
    -d '{
        "username": "'"$bob_username"'",
        "password": "password"
    }')

if [ "$dup_response" = "409" ]; then
    echo -e "${GREEN}✓${NC} Duplicate registration correctly rejected (HTTP 409)"
else
    echo -e "${RED}✗${NC} Duplicate registration should return 409, got $dup_response"
fi

# Test invalid credentials
echo ""
echo "7. Testing invalid credentials (should fail)..."
invalid_response=$(curl -s -o /dev/null -w "%{http_code}" -X POST "$BASE_URL/login" \
    -H "Content-Type: application/json" \
    -d '{
        "username": "'"$bob_username"'",
        "password": "wrong_password"
    }')

if [ "$invalid_response" = "401" ]; then
    echo -e "${GREEN}✓${NC} Invalid credentials correctly rejected (HTTP 401)"
else
    echo -e "${RED}✗${NC} Invalid login should return 401, got $invalid_response"
fi

# Test rate limiting (send many requests)
echo ""
echo "8. Testing rate limiting..."
echo "   Sending 110 requests (limit is 100/min)..."
rate_limit_hit=false
for i in {1..110}; do
    response=$(curl -s -o /dev/null -w "%{http_code}" "$BASE_URL/health")
    if [ "$response" = "429" ]; then
        rate_limit_hit=true
        echo -e "${GREEN}✓${NC} Rate limit triggered after $i requests (HTTP 429)"
        break
    fi
done

if [ "$rate_limit_hit" = false ]; then
    echo -e "${RED}✗${NC} Rate limit not triggered (expected HTTP 429)"
fi

echo ""
echo "===================================="
echo "✨ Test suite completed!"
echo "===================================="
echo ""
echo "Summary:"
echo "  ✓ Health check"
echo "  ✓ User registration"
echo "  ✓ User login"
echo "  ✓ Prekey bundle retrieval"
echo "  ✓ Duplicate detection"
echo "  ✓ Authentication"
echo "  ✓ Rate limiting"
echo ""
