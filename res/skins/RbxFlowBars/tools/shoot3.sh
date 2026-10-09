#!/usr/bin/env bash
# Launch Mixxx with RbxFlow, LOAD TRACKS INTO THE DECKS, wait for the main
# window, then capture. Unlike shoot2.sh this exercises the skin with real
# library content: waveform, beatgrid marks, track info, hot-cue colours.
#
#   mixxx <file...>  loads each file into a deck at start-up (deck 1, deck 2).
#
# usage: shoot3.sh [out.png] [crop WxH+X+Y] [extra mixxx args...]
set -u
SP=/home/thiago/.hermes/cache/scratch/rbxshot
SKIN=/home/thiago/.mixxx/skins/RbxFlow
OUT=${1:-/home/thiago/.hermes/cache/scratch/rbx_shot.png}
CROP=${2:-}
shift 2 2>/dev/null || shift $# 
SEED=$SKIN/tools/seed.cfg
A="/home/thiago/Músicas/sophierefix.mp3"
B="/home/thiago/Músicas/musicas/SOPHIE - MSMSMSM.mp3"

pkill -9 -f 'settings-path /home/thiago/.hermes/cache/scratch/rbx' 2>/dev/null
sleep 2

rm -rf "$SP"
mkdir -p "$SP/skins" "$SP/controllers" "$SP/effects" "$SP/analysis"
ln -s "$SKIN" "$SP/skins/RbxFlow"
cp "$SEED" "$SP/mixxx.cfg"
for f in mixxxdb.sqlite samplers.xml effects.xml soundconfig.xml; do
  [ -e "$HOME/.mixxx/$f" ] && cp "$HOME/.mixxx/$f" "$SP/" 2>/dev/null
done

mixxx --settings-path "$SP" --log-level debug "$A" "$B" "$@" > "$SP/mixxx.log" 2>&1 &
MIXPID=$!
echo "mixxx pid $MIXPID"

for i in $(seq 1 40); do
  sleep 5
  if grep -q 'Displaying main window' "$SP/mixxx.log" 2>/dev/null; then
    echo "main window up after $((i * 5))s"
    break
  fi
done

# maximize so the skin lays out at full desktop size
cat > "$SP/max.js" <<'JS'
{
  "org.kde.kwin.Scripting": {
    "jsCode": "workspace().windowList().forEach(function(w){ if (w.caption && w.caption.indexOf('ixxx') >= 0) { w.minimized = false; w.keepAbove = true; w.maximized = true; } });"
  }
}
JS
qdbus6 org.kde.KWin /Scripting org.kde.kwin.Scripting.loadDeclarativeScript "$SP/max.js" >/dev/null 2>&1
sleep 8

rm -f "$OUT"
if [ -n "$CROP" ]; then
  W=${CROP%%+*}; rest=${CROP#*+}; H=${rest%%+*}; X=${CROP##*+}; Y=${CROP##*+}
  Y=${X#*+}; X=${CROP#*+}; X=${X%%+*}
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
  spectacle -b -n -f -o "$SP/full.png" 2>/dev/null
  [ -s "$SP/full.png" ] && echo "fallback full -> $SP/full.png ($(identify -format '%wx%h' "$SP/full.png"))"
fi

echo "--- skin ---"
grep -iE 'Loaded skin|Skin parsing failed|Failed to find' "$SP/mixxx.log" | head -5
echo "--- track load ---"
grep -iE 'Loading track|Loaded track|track_loaded|Analyzer' "$SP/mixxx.log" | head -8
echo "--- errors ---"
grep -iE '\[error\]|critical|assert' "$SP/mixxx.log" | head -10
echo "LOG=$SP/mixxx.log PID=$MIXPID"
