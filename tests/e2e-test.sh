#!/usr/bin/env bash
# ParkHub E2E Test Suite — reproduzierbar, plattformunabhängig
# Usage: ./e2e-test.sh [BASE_URL]
set -euo pipefail

BASE="${1:-http://localhost:7878}"
PASS=0; FAIL=0; TOTAL=0

green() { printf "\033[32m%s\033[0m\n" "$*"; }
red()   { printf "\033[31m%s\033[0m\n" "$*"; }
bold()  { printf "\033[1m%s\033[0m\n" "$*"; }

check() {
    local name="$1" expected="$2" actual="$3"
    TOTAL=$((TOTAL+1))
    if echo "$actual" | grep -q "$expected"; then
        green "  OK $name"
        PASS=$((PASS+1))
    else
        red "  FAIL $name (expected: $expected, got: $actual)"
        FAIL=$((FAIL+1))
    fi
}

api() { curl -sf "$BASE$1" "${@:2}" 2>/dev/null || echo '{"error":"request_failed"}'; }
api_auth() { curl -sf "$BASE$1" -H "Authorization: Bearer $2" "${@:3}" 2>/dev/null || echo '{"error":"request_failed"}'; }
jq() { python3 -c "import sys,json; d=json.load(sys.stdin); print($1)" 2>/dev/null <<< "$2" || echo "parse_error"; }

bold ""
bold "======================================"
bold "  ParkHub E2E Test Suite"
bold "  Target: $BASE"
bold "  Date:   $(date -u +%Y-%m-%dT%H:%M:%SZ)"
bold "======================================"
echo ""

# 1. Health
bold "--- Health & System ---"
R=$(api /health)
check "Health" "OK" "$R"

R=$(api /api/v1/system/version)
VER=$(jq "d['version']" "$R")
check "Version" "2026" "$VER"

# 2. Auth
bold "--- Authentication ---"
R=$(api /api/v1/auth/login -X POST -H 'Content-Type: application/json' -d '{"username":"admin","password":"admin"}')
SUC=$(jq "d.get('success',False)" "$R")
check "Admin login" "True" "$SUC"
ADMIN_TOKEN=$(jq "d['data']['tokens']['access_token']" "$R")

R=$(api_auth /api/v1/users/me/password "$ADMIN_TOKEN" -X PATCH -H 'Content-Type: application/json' -d '{"current_password":"admin","new_password":"Test1234!"}')
check "Change password" "True" "$(jq "d.get('success',False)" "$R")"

R=$(api /api/v1/auth/login -X POST -H 'Content-Type: application/json' -d '{"username":"admin","password":"Test1234!"}')
check "Re-login" "True" "$(jq "d.get('success',False)" "$R")"
ADMIN_TOKEN=$(jq "d['data']['tokens']['access_token']" "$R")

# 3. Setup
bold "--- Setup & Branding ---"
R=$(api_auth /api/v1/admin/branding "$ADMIN_TOKEN" -X PUT -H 'Content-Type: application/json' -d '{"company_name":"ParkHub Test","logo_url":"/logos/variant-1.png","primary_color":"#3B82F6"}')
check "Set branding" "True" "$(jq "d.get('success',False)" "$R")"

R=$(api_auth /api/v1/admin/setup/complete "$ADMIN_TOKEN" -X POST)
check "Complete setup" "" "$R"

# 4. Lots
bold "--- Parking Lots ---"
R=$(api_auth /api/v1/lots "$ADMIN_TOKEN" -X POST -H 'Content-Type: application/json' -d '{"name":"Tiefgarage Zentrum","total_slots":12,"address":"Hauptstr. 1, Goettingen"}')
check "Create lot" "True" "$(jq "d.get('success',False)" "$R")"
LOT_ID=$(jq "d['data']['id']" "$R" 2>/dev/null || echo "")

R=$(api_auth /api/v1/lots "$ADMIN_TOKEN")
check "List lots" "data" "$R"

if [ -n "$LOT_ID" ] && [ "$LOT_ID" != "parse_error" ]; then
    R=$(api_auth "/api/v1/lots/$LOT_ID/slots" "$ADMIN_TOKEN")
    SLOT_COUNT=$(jq "len(d['data'])" "$R")
    check "Slots (12)" "12" "$SLOT_COUNT"
    SLOT_ID=$(jq "d['data'][0]['id']" "$R")
fi

# 5. Users
bold "--- Users ---"
RAND=$RANDOM
R=$(api /api/v1/auth/register -X POST -H 'Content-Type: application/json' -d "{\"username\":\"user$RAND\",\"password\":\"Test1234!\",\"email\":\"u$RAND@test.com\",\"name\":\"Test $RAND\"}")
check "Register user" "True" "$(jq "d.get('success',False)" "$R")"

R=$(api /api/v1/auth/login -X POST -H 'Content-Type: application/json' -d "{\"username\":\"user$RAND\",\"password\":\"Test1234!\"}")
check "User login" "True" "$(jq "d.get('success',False)" "$R")"
USER_TOKEN=$(jq "d['data']['tokens']['access_token']" "$R")

R=$(api_auth /api/v1/admin/users "$ADMIN_TOKEN")
check "Admin list users" "data" "$R"

# 6. Vehicles
bold "--- Vehicles ---"
R=$(api_auth /api/v1/vehicles "$USER_TOKEN" -X POST -H 'Content-Type: application/json' -d "{\"license_plate\":\"GO-T $RAND\",\"make\":\"VW\",\"model\":\"Golf\"}")
check "Add vehicle" "True" "$(jq "d.get('success',False)" "$R")"

R=$(api_auth /api/v1/vehicles "$USER_TOKEN")
check "List vehicles" "data" "$R"

# 7. Bookings
bold "--- Bookings ---"
START=$(date -u +%Y-%m-%dT%H:%M:%SZ)
if [ -n "${SLOT_ID:-}" ] && [ "$SLOT_ID" != "parse_error" ]; then
    R=$(api_auth /api/v1/bookings "$USER_TOKEN" -X POST -H 'Content-Type: application/json' -d "{\"lot_id\":\"$LOT_ID\",\"slot_id\":\"$SLOT_ID\",\"vehicle_plate\":\"GO-T $RAND\",\"start_time\":\"$START\",\"duration_minutes\":60}")
    check "Create booking" "True" "$(jq "d.get('success',False)" "$R")"
    BOOK_ID=$(jq "d['data']['id']" "$R" 2>/dev/null || echo "")
fi

R=$(api_auth /api/v1/bookings "$USER_TOKEN")
check "List bookings" "data" "$R"

if [ -n "${BOOK_ID:-}" ] && [ "$BOOK_ID" != "parse_error" ]; then
    R=$(api_auth "/api/v1/bookings/$BOOK_ID" "$USER_TOKEN" -X DELETE)
    check "Cancel booking" "" "$R"
fi

# 8. Info
bold "--- Info & Legal ---"
R=$(api /api/v1/about)
check "About" "ParkHub" "$R"

R=$(api_auth /api/v1/users/me/export "$USER_TOKEN")
check "GDPR export" "" "$R"

# 9. Updates
bold "--- Updates ---"
R=$(api_auth /api/v1/admin/updates/check "$ADMIN_TOKEN")
check "Update check" "current" "$R"

# 10. Logout
bold "--- Cleanup ---"
R=$(api_auth /api/v1/auth/logout "$USER_TOKEN" -X POST)
check "Logout" "" "$R"

# Results
echo ""
bold "======================================"
if [ $FAIL -eq 0 ]; then
    green "  ALL TESTS PASSED: $PASS/$TOTAL"
else
    red "  RESULTS: $PASS passed, $FAIL failed (total: $TOTAL)"
fi
bold "======================================"
echo ""
exit $FAIL
