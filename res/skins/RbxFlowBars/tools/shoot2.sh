#!/usr/bin/env bash
# Launch Mixxx with RbxFlow, wait until the MAIN window is actually up and has
# been resized, then capture. Reliable on KWin 6 / Wayland: `spectacle -f`
# grabs the whole screen (no window racing), and we verify the window exists via
# the Mixxx log before shooting.
#
# usage: shoot2.sh [out.png] [crop WxH+X+Y]
set -u
SP=/home/thiago/.hermes/cache/scratch/rbxshot
SKIN=/home/thiago/.mixxx/skins/RbxFlow
OUT=${1:-/home/thiago/.hermes/cache/scratch/rbx_shot.png}
CROP=${2:-}
SEED=$SKIN/tools/seed.cfg

pkill -9 -f 'settings-path /home/thiago/.hermes/cache/scratch/rbx' 2>/dev/null
sleep 2

rm -rf "$SP"
mkdir -p "$SP/skins" "$SP/controllers" "$SP/effects" "$SP/analysis"
ln -s "$SKIN" "$SP/skins/RbxFlow"
cp "$SEED" "$SP/mixxx.cfg"
for f in mixxxdb.sqlite samplers.xml effects.xml soundconfig.xml; do
  [ -e "$HOME/.mixxx/$f" ] && cp "$HOME/.mixxx/$f" "$SP/" 2>/dev/null
done

mixxx --settings-path "$SP" --log-level debug > "$SP/mixxx.log" 2>&1 &
MIXPID=$!
echo "mixxx pid $MIXPID"

# Wait for the main window, not just "process started".
for i in $(seq 1 40); do
  sleep 5
  if grep -q 'Displaying main window' "$SP/mixxx.log" 2>/dev/null; then
    echo "main window up after $((i * 5))s"
    break
  fi
done

if grep -q 'A device is unavailable\|configured to use a sound device' "$SP/mixxx.log" 2>/dev/null; then
  echo "WARNING: audio device dialog appeared"
fi

# maximize the window, then let the skin re-lay-out
cat > "$SP/max.js" <<'JS'
{
  "org.kde.kwin.Scripting": {
    "jsCode": "workspace().windowList().forEach(function(w){ if (w.caption && w.caption.indexOf('ixxx') >= 0) { w.minimized = false; w.keepAbove = true; w.maximized = true; } });"
  }
}
JS
qdbus6 org.kde.KWin /Scripting org.kde.kwin.Scripting.loadDeclarativeScript "$SP/max.js" >/dev/null 2>&1
sleep 6

rm -f "$OUT"
if [ -n "$CROP" ]; then
  W=${CROP%%+*}; rest=${CROP#*+}; H=${rest%%+*}; X=${rest#*+}; Y=${CROP##*+}
  Y=${CROP##*+}
  spectacle -b -n -f -o "$SP/full.png" 2>/dev/null
  sleep 1
  magick "$SP/full.png" -crop "${W}x${H}+${X}+${Y}" +repage -resize 1500x "$OUT"
else
  spectacle -b -n -f -o "$OUT" 2>/dev/null
fi
sleep 1

if [ -s "$OUT" ]; then
  echo "captured -> $OUT ($(identify -format '%wx%h' "$OUT"))"
else
  echo "NO SCREENSHOT"
fi
grep -iE 'Loaded skin|Skin parsing failed' "$SP/mixxx.log" | head -3
kill $MIXPID 2>/dev/null
wait $MIXPID 2>/dev/null
exit 0