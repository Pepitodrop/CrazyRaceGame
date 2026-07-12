from pathlib import Path
import re


def replace_once(path: str, old: str, new: str) -> None:
    file = Path(path)
    text = file.read_text()
    if text.count(old) != 1:
        raise SystemExit(f"expected exactly one match in {path}: {old[:80]!r}; got {text.count(old)}")
    file.write_text(text.replace(old, new, 1))


def replace_regex(path: str, pattern: str, replacement: str) -> None:
    file = Path(path)
    text = file.read_text()
    updated, count = re.subn(pattern, replacement, text, count=1, flags=re.S)
    if count != 1:
        raise SystemExit(f"expected one regex match in {path}: {pattern!r}; got {count}")
    file.write_text(updated)


# --- Default Compose compatibility with Snap Docker -----------------------------
replace_once(
    "docker-compose.yml",
    '''    security_opt:
      - no-new-privileges:true
''',
    '''    # The local profile intentionally omits no-new-privileges because Snap Docker's
    # AppArmor policy rejects container exec with that flag. The production profile
    # keeps the stronger restriction and should run on Docker Engine or Docker Desktop.
''',
)

# --- Smoke tests ----------------------------------------------------------------
SMOKE = r'''#!/usr/bin/env bash
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

track_signature() {
  grep -o 'data-track-signature="[^"]*"' "$1" | head -1 | cut -d'"' -f2
}

curl --fail --silent --show-error \
  --dump-header "$temp_dir/landing.headers" \
  --output "$temp_dir/landing.html" \
  "$base_url/"

grep -q 'Local 1v1' "$temp_dir/landing.html"
grep -q 'rel="icon"' "$temp_dir/landing.html"
grep -qi '^content-security-policy:' "$temp_dir/landing.headers"
grep -qi '^cache-control: no-store' "$temp_dir/landing.headers"
grep -qi '^referrer-policy: no-referrer' "$temp_dir/landing.headers"
grep -qi '^x-frame-options: DENY' "$temp_dir/landing.headers"

curl --fail --silent --show-error --output "$temp_dir/favicon.svg" "$base_url/favicon.svg"
grep -q '<svg' "$temp_dir/favicon.svg"
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

curl --fail --silent --show-error --output "$temp_dir/game.html" "$base_url$player_one_location"
grep -q 'Segment 0 of 20' "$temp_dir/game.html"
grep -q 'Energy' "$temp_dir/game.html"
grep -q 'Straight' "$temp_dir/game.html"
grep -q 'Curve' "$temp_dir/game.html"
grep -q 'Mud' "$temp_dir/game.html"
grep -q 'Jump' "$temp_dir/game.html"
grep -q 'Reveal the Piet boost program' "$temp_dir/game.html"

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
grep -q 'Segment ' "$temp_dir/resolved.html"

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
first_signature="$(track_signature "$temp_dir/local-one.html")"
test "${#first_signature}" -eq 20

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

curl --fail --silent --show-error \
  --request POST \
  --data-urlencode "room=$local_room" \
  --data-urlencode "token=$local_token" \
  --data-urlencode 'action=drift' \
  --output /dev/null \
  "$base_url/local-action"

invalid_boost_status="$(curl --silent --output "$temp_dir/invalid-boost.html" --write-out '%{http_code}' \
  --request POST \
  --data-urlencode "room=$local_room" \
  --data-urlencode "token=$local_token" \
  --data-urlencode 'action=boost' \
  "$base_url/local-action")"
test "$invalid_boost_status" = "409"
grep -q 'Piet Boost requires 3 energy' "$temp_dir/invalid-boost.html"

second_local_location="$(request_location \
  --request POST \
  --data-urlencode 'player_one=Erin' \
  --data-urlencode 'player_two=Frank' \
  "$base_url/local")"
curl --fail --silent --show-error --output "$temp_dir/local-second-room.html" "$base_url$second_local_location"
second_signature="$(track_signature "$temp_dir/local-second-room.html")"
test "${#second_signature}" -eq 20
test "$first_signature" != "$second_signature"

large_name="$(head -c 9000 /dev/zero | tr '\0' 'a')"
status_code="$(curl --silent --output /dev/null --write-out '%{http_code}' \
  --request POST \
  --data-urlencode "name=$large_name" \
  "$base_url/create")"
test "$status_code" = "413"

echo 'Production smoke test passed.'
'''
Path("scripts/smoke-test.sh").write_text(SMOKE)

# --- CI policy ------------------------------------------------------------------
replace_once(
    ".github/workflows/ci.yml",
    '''          ruby -e 'require "yaml"; Dir[".github/**/*.{yml,yaml}"].sort.each { |file| YAML.load_file(file); puts "valid yaml: #{file}" }'
''',
    '''          ruby -e 'require "yaml"; Dir[".github/**/*.{yml,yaml}"].sort.each { |file| YAML.load_file(file); puts "valid yaml: #{file}" }'

          if grep -q 'no-new-privileges' docker-compose.yml; then
            echo 'The local Compose profile must remain compatible with Snap Docker.'
            exit 1
          fi
          grep -q 'no-new-privileges:true' compose.production.yml
''',
)

# --- Version metadata -----------------------------------------------------------
for path in ["Cargo.toml", "Cargo.lock"]:
    replace_once(path, 'version = "1.0.1"', 'version = "1.1.0"')
replace_once("Dockerfile", "ARG APP_VERSION=1.0.1", "ARG APP_VERSION=1.1.0")
