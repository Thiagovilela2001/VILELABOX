#!/usr/bin/env bash
# Resolution sweep for RbxFlow.
#
#   usage: shoot4.sh 1366x768 [out.png]
#
# Why it works this way:
#  * KWin 6 scripting cannot resize a client: assigning frameGeometry/geometry
#    from a declarative script silently does nothing (verified — the window
#    stayed 1920x1080). There is no Xvfb/Xephyr on this box either.
#  * Qt DOES honour "-geometry WxH+X+Y", but only on the X11 platform plugin.
#    DISPLAY=:1 is XWayland, so force QT_QPA_PLATFORM=xcb and the request lands.
#    Qt strips the option before Mixxx's own QCommandLineParser sees it.
#  * The compositor may reposition the window, so the window box is measured by
#    diffing a desktop-only screenshot against the one with Mixxx in it. The
#    bottom 70px are dropped from the diff because the panel clock changes.
set -u
SIZE=${1:?usage: shoot4.sh WxH [out.png]}
W=${SIZE%x*}; H=${SIZE#*x}
SP=/home/thiago/.hermes/cache/scratch/rbxsweep
SKIN=/home/thiago/.mixxx/skins/RbxFlow
OUT=${2:-/home/thiago/.hermes/cache/scratch/sweep_${SIZE}.png}
A="/home/thiago/Músicas/sophierefix.mp3"
B="/home/thiago/Músicas/musicas/SOPHIE - MSMSMSM.mp3"

pkill -9 -f 'settings-path /home/thiago/.hermes/cache/scratch/rbx' 2>/dev/null
sleep 3
rm -rf "$SP"
mkdir -p "$SP/skins" "$SP/controllers" "$SP/effects" "$SP/analysis"
ln -s "$SKIN" "$SP/skins/RbxFlow"
cp "$SKIN/tools/seed.cfg" "$SP/mixxx.cfg"
for f in mixxxdb.sqlite samplers.xml effects.xml soundconfig.xml; do
  [ -e "$HOME/.mixxx/$f" ] && cp "$HOME/.mixxx/$f" "$SP/" 2>/dev/null
done

spectacle -b -n -f -o "$SP/base.png" 2>/dev/null
SW=$(identify -format '%w' "$SP/base.png"); SH=$(identify -format '%h' "$SP/base.png")
KEEP=$((SH - 90))   # drop the panel strip; its clock makes the diff wider

QT_QPA_PLATFORM=xcb mixxx --settings-path "$SP" --log-level info \
  "$A" "$B" > "$SP/mixxx.log" 2>&1 &
MIXPID=$!
for i in $(seq 1 40); do
  sleep 5
  grep -q 'Displaying main window' "$SP/mixxx.log" 2>/dev/null && break
done
sleep 5

# resize the real X11 window and report what the server accepted
python3 "$SKIN/tools/winresize.py" "${W}x${H}" mixxx || echo "resize failed"
sleep 8
spectacle -b -n -f -o "$SP/full.png" 2>/dev/null

BOX=""
if [ -s "$SP/full.png" ]; then
  magick "$SP/base.png" -crop "${SW}x${KEEP}+0+0" +repage "$SP/b.png"
  magick "$SP/full.png" -crop "${SW}x${KEEP}+0+0" +repage "$SP/f.png"
  BOX=$(magick "$SP/b.png" "$SP/f.png" -compose difference -composite \
        -threshold 12% -format '%@' info: 2>/dev/null)
fi
echo "window box: ${BOX:-<none>} (screen ${SW}x${SH}, requested ${W}x${H})"

if [ -n "$BOX" ] && [ -s "$SP/full.png" ]; then
  magick "$SP/full.png" -crop "$BOX" +repage "$OUT"
  echo "captured -> $OUT ($(identify -format '%wx%h' "$OUT"))"
else
  echo "NO SCREENSHOT"
fi
echo "--- skin ---"
grep -iE 'Loaded skin|Skin parsing failed|Cannot open file' "$SP/mixxx.log" | head -5
echo "LOG=$SP/mixxx.log"
kill $MIXPID 2>/dev/null
wait $MIXPID 2>/dev/null
exit 0
