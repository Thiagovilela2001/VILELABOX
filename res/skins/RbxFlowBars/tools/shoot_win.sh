#!/usr/bin/env bash
# Reliable window capture for skin inspection.
#
# Why not shoot3.sh: it grabs the whole screen and therefore photographs whatever
# window happens to be on top -- with the user reading the chat, that is the
# terminal, and a "layout measurement" quietly measures the wrong thing.
#
# This forces the X11 platform plugin (so the window is an XWayland client the
# X tools can see), resizes it deterministically with tools/winresize.py, and
# grabs the window itself with `import -window <id>` -- z-order independent.
#
#   usage: shoot_win.sh [WxH] [out.png]
set -u
SIZE=${1:-1920x1080}
SP=/home/thiago/.hermes/cache/scratch/rbxwin
SKIN=/home/thiago/.mixxx/skins/RbxFlow
OUT=${2:-/home/thiago/.hermes/cache/scratch/win_${SIZE}.png}
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

DISPLAY=${DISPLAY:-:1} QT_QPA_PLATFORM=xcb nohup env -u WAYLAND_DISPLAY \
  mixxx --settings-path "$SP" --log-level info --log-flush-level debug "$A" "$B" \
  > "$SP/mixxx.log" 2>&1 &
MIXPID=$!
for i in $(seq 1 24); do
  sleep 5
  grep -q 'Displaying main window' "$SP/mixxx.log" 2>/dev/null && break
done
# "Displaying main window" is logged before the window paints, and Mixxx's
# 500x204 splash stays in front meanwhile -- a grab here photographs the logo.
# Wait until only the main window is left.
count_windows() {
  python3 - <<'PY'
import subprocess
out = subprocess.run(['xprop', '-root', '_NET_CLIENT_LIST'],
                     capture_output=True, text=True).stdout
n = 0
for i in [t for t in out.replace(',', ' ').split() if t.startswith('0x')]:
    c = subprocess.run(['xprop', '-id', i, 'WM_CLASS'],
                       capture_output=True, text=True).stdout
    if 'mixxx' in c.lower():
        n += 1
print(n)
PY
}
for i in $(seq 1 12); do
  [ "$(count_windows)" -le 1 ] && break
  sleep 5
done
sleep 8

python3 "$SKIN/tools/winresize.py" "$SIZE" mixxx
sleep 4

ID=$(python3 -c "
import subprocess
out=subprocess.run(['xprop','-root','_NET_CLIENT_LIST'],capture_output=True,text=True).stdout
for i in [t for t in out.replace(',',' ').split() if t.startswith('0x')]:
    c=subprocess.run(['xprop','-id',i,'WM_CLASS'],capture_output=True,text=True).stdout
    if 'mixxx' in c.lower(): print(i); break
")
echo "window id: ${ID:-<none>}"
rm -f "$OUT"
if [ -n "$ID" ]; then
  timeout 60 import -window "$ID" "$OUT" 2>/dev/null
fi
if [ -s "$OUT" ]; then
  echo "captured -> $OUT ($(identify -format '%wx%h' "$OUT"))"
else
  echo "NO SCREENSHOT"
fi
echo "--- skin ---"
grep -iE 'Loaded skin|Skin parsing failed|Cannot open file' "$SP/mixxx.log" | head -4
kill $MIXPID 2>/dev/null
wait $MIXPID 2>/dev/null
exit 0
