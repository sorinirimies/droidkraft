#!/usr/bin/env bash
# Sample the GUI's RSS (MB) every 10s. Starts the GUI if it isn't running.
# usage: soak_gui.sh [secs]
set -euo pipefail
secs=${1:-60}
started=0
if ! pgrep -x droidkraft >/dev/null; then
  target/debug/droidkraft >/tmp/droidkraft-soak.log 2>&1 &
  started=1; sleep 6
fi
pid=$(pgrep -x droidkraft | head -1)
first=""; last=""
for ((t=0; t<secs; t+=10)); do
  mb=$(ps -o rss= -p "$pid" | awk '{printf "%.1f", $1/1024}')
  echo "${t}s  ${mb} MB  (children: $(pgrep -P "$pid" | wc -l | tr -d ' '))"
  [ -z "$first" ] && first=$mb; last=$mb
  sleep 10
done
echo "growth: $(awk -v a="$first" -v b="$last" 'BEGIN{printf "%+.1f MB", b-a}') over ${secs}s"
[ "$started" = 1 ] && kill "$pid" || true
