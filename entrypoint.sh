#!/bin/bash
set -e

# Start PulseAudio daemon in system-wide mode for headless recording
pulseaudio --system --daemonize --disallow-exit --exit-idle-time=-1 2>/dev/null || true

# Wait briefly for PulseAudio socket to become available
sleep 0.5

exec "$@"
