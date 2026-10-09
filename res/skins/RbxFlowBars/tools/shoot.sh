#!/usr/bin/env bash
# Launch Mixxx with the RbxFlow skin, force the main window to a real
# resolution, and screenshot it.
#
# This machine runs KWin 6 on Wayland: window control must go through
# loadDeclarativeScript. The older loadScript + workspace() API fails with
# "Property 'workspace' of object [null] is not a function". Declarative
# scripts run once at load time, so Mixxx must already be running before we
# load the resize script -- hence the retry loop below.
#
# usage: shoot.sh [out.png] [WxH] [seed.cfg]
set -u
SP=/home/thiago/.hermes/cache/scratch/rbxshot
SKIN=/home/thiago/.mixxx/skins/RbxFlow
OUT=${1:-/home/thiago/.hermes/cache/scratch/rbxflow_shot.png}
GEOM=${2:-1920x1080}
SEED=${3:-$SKIN/tools/seed.cfg}

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

W=${GEOM%x*}
H=${GEOM#*x}

apply_geometry() {
  cat > "$SP/resize.js" <<JS
{
  "org.kde.kwin.Scripting": {
    "jsCode": "const W=${W}, H=${H}; workspace().windowList().forEach(function(w){ if (w.caption && w.caption.indexOf('ixxx') >= 0) { w.minimized = false; w.keepAbove = true; w.frameGeometry = { x: 0, y: 0, width: W, height: H }; print('RBXRESIZE:' + w.frameGeometry.width + 'x' + w.frameGeometry.height); } });"
  }
}
JS
  qdbus6 org.kde.KWin /Scripting org.kde.kwin.Scripting.loadDeclarativeScript "$SP/resize.js" >/dev/null 2>&1
}

# Mixxx idles a long time before the window appears; keep re-applying geometry
# until the journal confirms the window actually reached the requested size.
want="${W}x${H}"
MARK=$(date '+%s')
ok=0
for i in $(seq 1 24); do
  sleep 5
  apply_geometry
  if journalctl --user --since "@$MARK" --no-pager 2>/dev/null | grep -q "RBXRESIZE:${want}"; then
    ok=1
    break
  fi
done

apply_geometry
sleep 3
rm -f "$OUT"
spectacle -b -n -a -o "$OUT" 2>/dev/null
sleep 1

if [ -s "$OUT" ]; then
  echo "captured -> $OUT  ($(identify -format '%wx%h' "$OUT"))  geometry_ok=$ok"
else
  echo "NO SCREENSHOT  geometry_ok=$ok"
fi
grep -iE 'Failed to find|Loaded skin|Skin parsing failed' "$SP/mixxx.log" | head -5
kill $MIXPID 2>/dev/null
wait $MIXPID 2>/dev/null
exit 0