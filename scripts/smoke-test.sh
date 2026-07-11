#!/usr/bin/env bash
set -euo pipefail

base_url="${1:-http://127.0.0.1:18080}"
temp_dir="$(mktemp -d)"
trap 'rm -rf "$temp_dir"' EXIT

request_location() {
  curl --fail --silent --show-error --dump-header - --output /dev/null "$@" \
    | tr -d '\r' \
    | awk 'tolower($1) == "location:" { print $2; exit }'
}

query_value() {
  local url="$1"
  local key="$2"
  printf '%s' "$url" | sed -n "s/.*[?&]${key}=\([^&]*\).*/\1/p"
}

curl --fail --silent --show-error \
  --dump-header "$temp_dir/landing.headers" \
  --output "$temp_dir/landing.html" \
  "$base_url/"

grep -q 'Local 1v1' "$temp_dir/landing.html"
grep -qi '^content-security-policy:' "$temp_dir/landing.headers"
grep -qi '^cache-control: no-store' "$temp_dir/landing.headers"
grep -qi '^referrer-policy: no-referrer' "$temp_dir/landing.headers"
grep -qi '^x-frame-options: DENY' "$temp_dir/landing.headers"

test "$(curl --fail --silent --show-error "$base_url/health")" = "ok"

player_one_location="$(request_location \
  --request POST \
  --data-urlencode 'name=Alice' \
  "$base_url/create")"
room_code="$(query_value "$player_one_location" room)"
player_one_token="$(query_value "$player_one_location" token)"
test "${#room_code}" -eq 6
test "${#player_one_token}" -eq 64

player_two_location="$(request_location \
  --request POST \
  --data-urlencode "room=$room_code" \
  --data-urlencode 'name=Bob' \
  "$base_url/join")"
player_two_token="$(query_value "$player_two_location" token)"
test "${#player_two_token}" -eq 64

curl --fail --silent --show-error \
  --request POST \
  --data-urlencode "room=$room_code" \
  --data-urlencode "token=$player_one_token" \
  --data-urlencode 'action=accelerate' \
  --output /dev/null \
  "$base_url/action"

curl --fail --silent --show-error \
  --output "$temp_dir/player-two.html" \
  "$base_url$player_two_location"
grep -q 'Alice locked in a move. Waiting for the rival.' "$temp_dir/player-two.html"
if grep -q 'Alice locked in Accelerate' "$temp_dir/player-two.html"; then
  echo 'Player one action leaked before player two submitted.' >&2
  exit 1
fi

curl --fail --silent --show-error \
  --request POST \
  --data-urlencode "room=$room_code" \
  --data-urlencode "token=$player_two_token" \
  --data-urlencode 'action=drift' \
  --output /dev/null \
  "$base_url/action"

curl --fail --silent --show-error \
  --output "$temp_dir/resolved.html" \
  "$base_url$player_one_location"
grep -q 'Round 1:' "$temp_dir/resolved.html"
grep -q 'Alice used Accelerate' "$temp_dir/resolved.html"
grep -q 'Bob used Drift' "$temp_dir/resolved.html"

local_location="$(request_location \
  --request POST \
  --data-urlencode 'player_one=Carol' \
  --data-urlencode 'player_two=Dave' \
  "$base_url/local")"
local_room="$(query_value "$local_location" room)"
local_token="$(query_value "$local_location" token)"
test "${#local_token}" -eq 64

curl --fail --silent --show-error \
  --output "$temp_dir/local-one.html" \
  "$base_url$local_location"
grep -q 'Player 1 move' "$temp_dir/local-one.html"

curl --fail --silent --show-error \
  --request POST \
  --data-urlencode "room=$local_room" \
  --data-urlencode "token=$local_token" \
  --data-urlencode 'action=boost' \
  --output /dev/null \
  "$base_url/local-action"

curl --fail --silent --show-error \
  --output "$temp_dir/local-two.html" \
  "$base_url$local_location"
grep -q 'Player 2 move' "$temp_dir/local-two.html"
grep -q 'Carol has chosen. Pass the device to Dave' "$temp_dir/local-two.html"

large_name="$(head -c 9000 /dev/zero | tr '\0' 'a')"
status_code="$(curl --silent --output /dev/null --write-out '%{http_code}' \
  --request POST \
  --data-urlencode "name=$large_name" \
  "$base_url/create")"
test "$status_code" = "413"

echo 'Production smoke test passed.'
