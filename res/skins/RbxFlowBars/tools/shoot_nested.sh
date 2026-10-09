#!/usr/bin/env bash
# Render RbxFlow in a NESTED, headless KWin session - nothing appears on the
# real desktop and the user's Mixxx config is never touched.
#
#   usage: tools/shoot_nested.sh out.png [W H] [trackA trackB]
#   e.g.   SIZE=1366x768 tools/shoot_nested.sh /tmp/x.png 1366 768
#
# kwin_wayland --virtual gives a private Wayland+Xwayland display; Mixxx runs
# there as an X11 client with a throwaway --settings-path whose skins/RbxFlow
# symlinks this skin, and tools/grab_window.py captures the window by id.
# Runs with --enable-legacy-vumeter so the meters are in the capture.
set -u
SKIN=$(cd "$(dirname "$0")/.."; pwd)
OUT=${1:?out.png}; W=${2:-1920}; H=${3:-1080}
A=${4:-}; B=${5:-}
TRACKS=""
for t in "$A" "$B"; do [ -n "$t" ] && TRACKS="$TRACKS $(printf '%q' "$t")"; done
WORK=$(mktemp -d); SP=$WORK/sp
mkdir -p "$SP/skins" "$SP/controllers" "$SP/effects" "$SP/analysis"
ln -s "$SKIN" "$SP/skins/RbxFlow"
cp "$SKIN/tools/seed.cfg" "$SP/mixxx.cfg"
for f in mixxxdb.sqlite samplers.xml effects.xml soundconfig.xml; do
  cp "$HOME/.mixxx/$f" "$SP/" 2>/dev/null
done
cp -r "$HOME/.mixxx/analysis/." "$SP/analysis/" 2>/dev/null
cat > "$WORK/inner.sh" <<INNER
#!/usr/bin/env bash
QT_QPA_PLATFORM=xcb mixxx --settings-path "$SP" --log-level debug --enable-legacy-vumeter $TRACKS > "$SP/mixxx.log" 2>&1 &
P=\$!
for i in \$(seq 1 60); do sleep 1; grep -q 'Displaying main window' "$SP/mixxx.log" && break; done
sleep \${WAIT:-12}
python3 "$SKIN/tools/grab_window.py" "$OUT" mixxx \${SIZE:-}
kill \$P; sleep 2; kill -9 \$P 2>/dev/null
INNER
rm -f "$OUT"
timeout 150 dbus-run-session -- kwin_wayland --virtual --width "$W" --height "$H" \
  --xwayland --no-lockscreen --exit-with-session "bash $WORK/inner.sh" >/dev/null 2>&1
[ -s "$OUT" ] && echo "captured -> $OUT" || echo "NO SCREENSHOT (log: $SP/mixxx.log)"
grep -iE 'Skin parsing failed|Failed to find' "$SP/mixxx.log" | head -5
