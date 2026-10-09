#!/usr/bin/env bash
# Render the skin at a resolution LARGER than the physical screen.
#
# A window cannot be bigger than the output (the compositor clamps it, verified),
# so start a second, nested Xwayland with its own big root window. Mixxx runs
# there with no window manager: the saved geometry still applies, and the window
# is then resized to fill the root and grabbed with `import -window root`.
set -u
SIZE=${1:-2560x1440}
DISP=${2:-:2}
SP=/home/thiago/.hermes/cache/scratch/rbxbig
SKIN=/home/thiago/.mixxx/skins/RbxFlow
OUT=${3:-/home/thiago/.hermes/cache/scratch/big_${SIZE}.png}

pkill -9 -f 'settings-path /home/thiago/.hermes/cache/scratch/rbx' 2>/dev/null
sleep 2
rm -rf "$SP"; mkdir -p "$SP/skins"
ln -s "$SKIN" "$SP/skins/RbxFlow"
cp "$SKIN/tools/seed.cfg" "$SP/mixxx.cfg"
for f in mixxxdb.sqlite samplers.xml effects.xml soundconfig.xml; do
  cp "$HOME/.mixxx/$f" "$SP/" 2>/dev/null
done

Xwayland "$DISP" -geometry "$SIZE" -noreset > "$SP/xwayland.log" 2>&1 &
XPID=$!
sleep 6
if ! DISPLAY="$DISP" xdpyinfo >/dev/null 2>&1; then
  echo "nested Xwayland $DISP failed to start"; kill $XPID 2>/dev/null; exit 1
fi

DISPLAY="$DISP" QT_QPA_PLATFORM=xcb nohup env -u WAYLAND_DISPLAY \
  mixxx --settings-path "$SP" --log-level info \
  "/home/thiago/Músicas/sophierefix.mp3" \
  "/home/thiago/Músicas/musicas/SOPHIE - MSMSMSM.mp3" > "$SP/mixxx.log" 2>&1 &
MIXPID=$!
for i in $(seq 1 40); do
  sleep 5
  grep -q 'Displaying main window' "$SP/mixxx.log" 2>/dev/null && break
done
sleep 6

for i in $(seq 1 12); do
  if DISPLAY="$DISP" python3 "$SKIN/tools/winresize.py" "$SIZE" mixxx; then
    break
  fi
  sleep 10          # no WM on this display, so wait for Mixxx to map its window
done
sleep 6

DISPLAY="$DISP" import -window root "$OUT" 2>&1 | head -3
if [ -s "$OUT" ]; then
  echo "captured -> $OUT ($(identify -format '%wx%h' "$OUT"))"
else
  echo "NO SCREENSHOT"
fi
grep -iE 'Loaded skin|Skin parsing failed|Cannot open file|GL|OpenGL' "$SP/mixxx.log" | head -6
echo "LOG=$SP/mixxx.log"
kill $MIXPID 2>/dev/null
wait $MIXPID 2>/dev/null
kill $XPID 2>/dev/null
wait $XPID 2>/dev/null
exit 0
