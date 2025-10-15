#!/bin/bash
set -euo pipefail

# Function to generate a session ID. Use uuidgen if available, otherwise fall back
# to a simple timestamp-based ID.
generate_session_id() {
  if command -v uuidgen >/dev/null 2>&1; then
    uuidgen
  else
    echo "no-uuid-$(date +%s%N)"
  fi
}

KIND="${1:-}"
if [ -z "$KIND" ]; then
  echo "Usage: $0 <event-kind> [args...]"
  echo "Kinds:"
  echo "  - audio.session_started <session_id> <bpm> <routing>"
  echo "  - audio.session_ended <session_id> <duration_ms>"
  echo "  - audio.latency_ms <latency_ms>"
  exit 1
fi

TS=$(date +%s)
EVENT=""

case "$KIND" in
  "audio.session_started")
    SESSION_ID="${2:-$(generate_session_id)}"
    BPM="${3:-120}"
    ROUTING="${4:-default}"
    EVENT=$(cat <<-JSON
      {"ts": $TS, "kind":"audio.session_started","session_id":"$SESSION_ID", "bpm":$BPM, "routing":"$ROUTING"}
JSON
    )
    ;;
  "audio.session_ended")
    SESSION_ID="${2:-$(generate_session_id)}"
    DURATION_MS="${3:-5000}"
    EVENT=$(cat <<-JSON
      {"ts": $TS, "kind":"audio.session_ended","session_id":"$SESSION_ID","duration_ms":$DURATION_MS}
JSON
    )
    ;;
  "audio.latency_ms")
    LATENCY_MS="${2:-10}"
    EVENT=$(cat <<-JSON
      {"ts": $TS, "kind":"audio.latency_ms","latency_ms":$LATENCY_MS}
JSON
    )
    ;;
  *)
    echo "Unknown event kind: $KIND"
    exit 1
    ;;
esac

echo "$EVENT" | tee -a export/audio.events.jsonl
