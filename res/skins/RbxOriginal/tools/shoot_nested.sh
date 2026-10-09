#!/usr/bin/env bash
# Render RbxOriginal with the blackmesa build in a nested, headless KWin.
# Nothing appears on the desktop and ~/.mixxx is only read, never written.
#   usage: tools/shoot_nested.sh out.png [W H] [trackA trackB]
set -u
SKIN=$(cd "$(dirname "$0")/.."; pwd)
NAME=$(basename "$SKIN")
SRC=$HOME/.mixxx/mixxx-source
BIN=${MIXXX_BIN:-$SRC/build/mixxx}
OUT=${1:?out.png}; W=${2:-1920}; H=${3:-1080}
A=${4:-}; B=${5:-}
TRACKS=""
for t in "$A" "$B"; do [ -n "$t" ] && TRACKS="$TRACKS $(printf '%q' "$t")"; done
WORK=$(mktemp -d); SP=$WORK/sp
mkdir -p "$SP/skins" "$SP/controllers" "$SP/effects" "$SP/analysis"
ln -s "$SKIN" "$SP/skins/$NAME"
cp "$SKIN/tools/seed.cfg" "$SP/mixxx.cfg"
for f in mixxxdb.sqlite samplers.xml effects.xml soundconfig.xml; do
  cp "$HOME/.mixxx/$f" "$SP/" 2>/dev/null
done
cp -r "$HOME/.mixxx/analysis/." "$SP/analysis/" 2>/dev/null
cp -r "$HOME/.mixxx/effects/." "$SP/effects/" 2>/dev/null
cat > "$WORK/inner.sh" <<INNER
#!/usr/bin/env bash
QT_QPA_PLATFORM=xcb "$BIN" --resource-path "$SRC/res" --settings-path "$SP" --log-level debug --developer $TRACKS > "$SP/mixxx.log" 2>&1 &
P=\$!
for i in \$(seq 1 60); do sleep 1; grep -q 'Displaying main window' "$SP/mixxx.log" && break; done
sleep \${WAIT:-12}
python3 "$SKIN/tools/grab_window.py" "$OUT" mixxx \${SIZE:-}
kill \$P; sleep 2; kill -9 \$P 2>/dev/null
INNER
rm -f "$OUT"
timeout 150 dbus-run-session -- kwin_wayland --virtual --width "$W" --height "$H" \
  --xwayland --no-lockscreen --exit-with-session "bash $WORK/inner.sh" >/dev/null 2>&1
[ -s "$OUT" ] && echo "captured -> $OUT" || echo "NO SCREENSHOT"
echo "log: $SP/mixxx.log"
grep -iE 'Skin parsing failed|Failed to find|Loaded skin' "$SP/mixxx.log" | head -5
grep -c 'Creating skin control object' "$SP/mixxx.log"
