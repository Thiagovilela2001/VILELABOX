#!/usr/bin/env python3
"""Generate the flat/minimal SVG asset set for the RbxFlow skin.

All geometry here is original: flat rectangles, rings and simple gradients. No
domed/beveled button art, nothing copied from LateNight or any other skin.

Two rules learned the hard way and encoded below:

  1. Mixxx has no 9-slice scaling. `scalemode="STRETCH"` scales x and y
     independently, so a generic 80x20 button base dropped into a 28x17 slot
     turns its rx=3 corners into ellipses. Every button base is therefore
     generated at the exact size of the slot that uses it, discovered by
     scanning the skin's own XML for `btn_<W>x<H>.svg` references.

  2. A near-black asset on a near-black panel is an invisible asset. The meter
     wells and slider rails used to sit at 1.0-1.6:1 contrast against the
     surfaces they are drawn on. They now carry a real outline and a
     distinguishable fill.

Run from the skin root:  python3 tools/make_assets.py
"""
import os
import re

SKIN = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BTN = os.path.join(SKIN, "bases")
os.makedirs(BTN, exist_ok=True)

# --- palette (Rekordbox-like dark neutral, own values) -----------------------
BG_APP = "#0d0f11"
BG_PANEL = "#15181b"
BG_PANEL_HI = "#1c2024"
BG_WELL = "#12161a"        # meter housing: visible against #0d0f11
BORDER = "#252a30"
RAIL = "#343b43"           # slider groove: 2.8:1 against the app background
RAIL_EDGE = "#4a545f"
RAIL_CORE = "#191d21"
TICK = "#8b959e"
BG_BTN = "#23282e"
BORDER_HI = "#4a545f"
TEXT = "#c8cfd6"
TEXT_DIM = "#78828c"
ACCENT = "#0a84ff"
ACCENT_DIM = "#0a5fb0"
AMBER = "#ffa600"
GREEN = "#2bd67c"
GREEN_DIM = "#1f9e57"
RED = "#ff4b4b"
RED_DIM = "#3a1a1d"


def svg(w, h, body):
    return (
        '<svg xmlns="http://www.w3.org/2000/svg" width="%d" height="%d" '
        'viewBox="0 0 %d %d">%s</svg>\n' % (w, h, w, h, body)
    )


def write(relpath, w, h, body):
    p = os.path.join(SKIN, relpath)
    os.makedirs(os.path.dirname(p), exist_ok=True)
    with open(p, "w") as f:
        f.write(svg(w, h, body))
    return p


def shades(x, y, w, h, radius, top_op=0.10, bot_op=0.30):
    """A 1 px lit line under the top edge and a 1 px shade above the bottom edge.

    Deliberately no gradients: Qt's SVG rasteriser is the only renderer that
    matters here, and a solid pair of hairlines gives the same "lit from above"
    read with zero risk of a fill silently failing to resolve.
    """
    out = ""
    inner = max(1, w - 2 * (radius + 1))
    if h >= 8 and inner >= 2:
        out += ('<rect x="%d" y="%d" width="%d" height="1" fill="#ffffff" '
                'fill-opacity="%.2f"/>' % (radius + 1, y + 1, inner, top_op))
        out += ('<rect x="%d" y="%d" width="%d" height="1" fill="#000000" '
                'fill-opacity="%.2f"/>' % (radius + 1, y + h - 2, inner, bot_op))
    return out


def btn(w, h, active, radius=3):
    """One flat button: recessed panel, 1 px border, lit top / shaded bottom.

    `active` swaps the base to a blue so an engaged control reads at a glance
    instead of relying on a 34%-opacity wash.
    """
    r = min(radius, max(1, min(w, h) // 2))
    base = "#1b3f66" if active else BG_BTN
    edge = ACCENT if active else BORDER_HI
    out = ('<rect x="0.5" y="0.5" width="%d" height="%d" rx="%d" fill="%s" '
           'stroke="%s"/>' % (w - 1, h - 1, r, base, edge))
    out += shades(0.5, 0.5, w - 1, h - 1, r,
                  top_op=0.16 if active else 0.10, bot_op=0.28)
    return out


# ---------------------------------------------------------------- push buttons
# Every size actually referenced by the skin's XML is discovered and generated,
# so no button base is ever anisotropically scaled.
BTN_REF = re.compile(r"btn_(\d+)x(\d+)(_active)?\.svg")
# Template-based buttons reference their base through a variable, so the size is
# only visible in the caller's BtnSize set variable.
BTNSIZE_REF = re.compile(r'<SetVariable name="BtnSize">(\d+)x(\d+)</SetVariable>')


def referenced_button_sizes():
    sizes = set()
    for root, dirs, files in os.walk(SKIN):
        dirs[:] = [d for d in dirs if d != "tools"]
        for f in files:
            if not f.endswith(".xml"):
                continue
            with open(os.path.join(root, f), errors="ignore") as fh:
                text = fh.read()
            for rx in (BTN_REF, BTNSIZE_REF):
                for m in rx.finditer(text):
                    sizes.add((int(m.group(1)), int(m.group(2))))
    return sorted(sizes)


_sizes = referenced_button_sizes()
for w, h in _sizes:
    write("bases/btn_%dx%d.svg" % (w, h), w, h, btn(w, h, False))
    write("bases/btn_%dx%d_active.svg" % (w, h), w, h, btn(w, h, True))
print("buttons: %d distinct sizes -> %d files"
      % (len(_sizes), len(_sizes) * 2))

# StatusLight bases used by the AutoDJ light and the clip indicators
for nm, w, h in (("sm", 18, 18),):
    write("bases/btn_%s.svg" % nm, w, h, btn(w, h, False))
    write("bases/btn_%s_set.svg" % nm, w, h,
          '<rect x="0.5" y="0.5" width="%d" height="%d" rx="3" fill="%s" '
          'stroke="%s"/><rect x="1.5" y="1.5" width="%d" height="%d" rx="2" '
          'fill="%s" fill-opacity="0.55"/>'
          % (w - 1, h - 1, BG_BTN, AMBER, w - 3, h - 3, AMBER))

# Transparent dummies. `btn_blank` is used for the invisible seek overlay; the
# `blank`/`blank_active` pair is also the art for the segmented layout switcher,
# whose look comes entirely from QSS - WPushButton::paintOnDevice() calls
# drawPrimitive(QE_Widget) before the state pixmap, so a QSS background-color
# does paint, and a transparent pixmap lets it show. Same trick as the hot-cue
# pads.
for _n in ("btn_blank.svg", "btn_blank_active.svg"):
    write("bases/" + _n, 1, 1,
          '<rect x="0" y="0" width="1" height="1" fill="none"/>')

# Record button: its own red theme so an armed recorder reads instantly.
for nm, active in (("btn_rec.svg", False), ("btn_rec_active.svg", True)):
    w, h = 60, 18
    fill = "#5c1f22" if active else BG_BTN
    edge = RED if active else BORDER_HI
    body = ('<rect x="0.5" y="0.5" width="%d" height="%d" rx="3" fill="%s" '
            'stroke="%s"/>' % (w - 1, h - 1, fill, edge))
    body += shades(0.5, 0.5, w - 1, h - 1, 3, top_op=0.14, bot_op=0.26)
    body += '<circle cx="9" cy="9" r="3.2" fill="%s"/>' % (RED if active else "#6b3a3d")
    write("bases/" + nm, w, h, body)

# Faint beat grid for the empty-deck state. Tiled by QSS background-image, so it
# must be seamless: lines sit exactly on the tile edges.
# The BPM dial's disc and its ring. The live tempo arc is drawn by
# WKnobComposed on top of this, so the disc stays plain and the inner ring is
# only there to give the arc a track to travel in.
write("bases/dial_ring.svg", 62, 62,
      # the disc is filled: the readout is a sibling BELOW it now, so nothing
      # can be covered, and a filled disc is what makes it read as a dial
      '<circle cx="31" cy="31" r="31" fill="#171717"/>'
      '<circle cx="31" cy="31" r="30.5" fill="none" stroke="#2e2e2e" stroke-width="1"/>'
      '<circle cx="31" cy="31" r="21" fill="none" stroke="#242424" stroke-width="1"/>')

write("bases/empty_grid.svg", 40, 40,
      '<rect x="0" y="0" width="40" height="40" fill="#090b0d"/>'
      # one faint bar line and one fainter beat division per tile; no brighter
      # intersection, which turned the whole surface into graph paper
      '<rect x="0" y="19" width="40" height="1" fill="#141a20"/>'
      '<rect x="19" y="0" width="1" height="40" fill="#10151a"/>')

# ------------------------------------------------------------------- knobs
# KnobComposed draws <BackPath> (the body, not rotated) and then <Knob> (the
# indicator, rotated about the pixmap centre) on top. The body is therefore a
# fixed housing and the indicator must be a radial element centred on the canvas.
#
# The old indicator was a filled wedge (M r,3 L r,d-7 L r-3,d-7 Z) - a 3 px
# knife blade that tapered to a point and poked past the rim. It is replaced by
# a uniform rounded bar with a dark separation hairline underneath, which is
# what makes it read as a pointer instead of a needle.
KNOB_C = "#2d343c"        # cap
KNOB_C_TOP = "#4e5964"    # lit crescent at the top of the cap
KNOB_C_BOT = "#12161a"    # shaded crescent at the bottom
KNOB_BEZEL = "#39424c"    # the ring the cap sits in
KNOB_RECESS = "#0d1013"   # the recess the whole knob sits in


def knob_body(rad):
    """Housing + cap, with a lit top and a shaded bottom.

    KnobComposed defaults <BackPath> and <Knob> to STRETCH, so one body per
    knob *kind* covers every widget size it is used at (the same
    knob_bg_regular.svg serves the 34, 38 and 40 px knobs).

    Layout, from the outside in: a machined well (the value arc is drawn by
    KnobComposed at ArcRadius, i.e. inside this well), then the cap. The cap's
    rim lighting is made by drawing two equal circles offset by +-1 px: the one
    moved up peeks out at the top, the one moved down peeks out at the bottom.
    """
    c = rad
    well = rad - 0.5
    deep = rad - 2.0
    cap = rad - 3.2
    out = ('<circle cx="%g" cy="%g" r="%g" fill="#101418"/>'
           '<circle cx="%g" cy="%g" r="%g" fill="none" stroke="#2b333b"/>'
           '<circle cx="%g" cy="%g" r="%g" fill="#0b0e11"/>'
           % (c, c, well, c, c, well, c, c, deep))
    out += '<circle cx="%g" cy="%g" r="%g" fill="%s"/>' % (c, c - 1.2, cap, KNOB_C_TOP)
    out += '<circle cx="%g" cy="%g" r="%g" fill="%s"/>' % (c, c + 1.2, cap, KNOB_C_BOT)
    out += '<circle cx="%g" cy="%g" r="%g" fill="%s"/>' % (c, c, cap, KNOB_C)
    out += ('<circle cx="%g" cy="%g" r="%g" fill="none" stroke="#0f1316"/>'
            % (c, c, cap))
    return out


def knob_pointer(color, rad):
    """A uniform rounded pointer from 0.42R to 0.76R, with a dark hairline."""
    c = rad
    y_far = c - 0.80 * rad
    y_near = c - 0.42 * rad
    return ('<line x1="%g" y1="%g" x2="%g" y2="%g" stroke="#0d1013" '
            'stroke-width="4.2" stroke-linecap="round" stroke-opacity="0.9"/>'
            '<line x1="%g" y1="%g" x2="%g" y2="%g" stroke="%s" '
            'stroke-width="2.4" stroke-linecap="round"/>'
            % (c, y_near, c, y_far, c, y_near, c, y_far, color))


for kind, r in (("regular", 17), ("small", 14), ("main", 20)):
    d = r * 2
    write("bases/knob_bg_%s.svg" % kind, d, d, knob_body(r))
    for color, cname in ((TEXT, "grey"), (ACCENT, "blue"), (AMBER, "amber"),
                         (GREEN, "green"), (RED, "red")):
        write("bases/knob_indicator_%s_%s.svg" % (kind, cname), d, d,
              knob_pointer(color, r))

# Size-agnostic aliases used by controls/knob.xml and mixer/eq_knob.xml.
# KnobComposed scales <Knob> to the widget, so one 40 px canvas serves the
# 30/34/38/40 px knobs without re-deriving the geometry.
KNOB_ALIAS = 20
for color, cname in ((TEXT, "grey"), (ACCENT, "blue"), (AMBER, "amber"),
                     (GREEN, "green"), (RED, "red")):
    write("bases/knob_indicator_%s.svg" % cname, KNOB_ALIAS * 2, KNOB_ALIAS * 2,
          knob_pointer(color, KNOB_ALIAS))
# alias for the two other backgrounds (kept so knob.xml can ask for small/main)
for kind, r in (("regular", 17), ("small", 14), ("main", 20)):
    write("bases/knob_bg_%s.svg" % kind, r * 2, r * 2, knob_body(r))


# ------------------------------------------------------------------ sliders
def slider_v(w, h):
    """Vertical groove: visible rail, darker core, centre tick."""
    rail, core = 9, 3
    x = w // 2 - rail // 2
    c = w // 2 - core // 2
    return (
        '<rect x="%d" y="1" width="%d" height="%d" rx="4.5" fill="%s" '
        'stroke="%s"/>'
        '<rect x="%d" y="5" width="%d" height="%d" rx="1.5" fill="%s"/>'
        '<rect x="%d" y="%d" width="%d" height="1" fill="%s"/>'
        % (x, rail, h - 2, RAIL, RAIL_EDGE, c, core, h - 10, RAIL_CORE,
           x, h // 2, rail, TICK))


def slider_h(w, h):
    """Horizontal groove, same treatment."""
    rail, core = 9, 3
    y = h // 2 - rail // 2
    c = h // 2 - core // 2
    return (
        '<rect x="1" y="%d" width="%d" height="%d" rx="4.5" fill="%s" '
        'stroke="%s"/>'
        '<rect x="5" y="%d" width="%d" height="%d" rx="1.5" fill="%s"/>'
        '<rect x="%d" y="%d" width="1" height="%d" fill="%s"/>'
        % (y, w - 2, rail, RAIL, RAIL_EDGE, c, w - 10, core,
           RAIL_CORE, w // 2, y, rail, TICK))


def handle(w, h):
    """Flat fader knob: lighter than the panel so it stands off the groove."""
    return ('<rect x="1.5" y="1.5" width="%d" height="%d" rx="3" fill="%s" '
            'stroke="%s"/>'
            '<rect x="6" y="%d" width="%d" height="2" rx="1" fill="%s"/>'
            % (w - 3, h - 3, "#2f353c", "#8b959e", h // 2 - 1, max(1, w - 12),
               TICK))


# Generated TALL and drawn with scalemode="STRETCH": a vertical groove scales
# vertically without visible harm, and the widget size is then always honoured.
write("bases/slider_volume.svg", 62, 240, slider_v(62, 240))
write("bases/slider_pitch.svg", 74, 260, slider_v(74, 260))
write("bases/slider_crossfader.svg", 560, 36, slider_h(560, 36))
# Wide, shallow caps: the handle length comes from the widget, so a square cap
# would eat a large share of the crossfader's travel.
write("bases/knob_volume.svg", 62, 24, handle(62, 24))
write("bases/knob_pitch.svg", 74, 22, handle(74, 22))
write("bases/knob_crossfader.svg", 22, 30, handle(22, 30))


# ----------------------------------------------------------------- VU meter
def vu_scale(w, h, ticks=8, tick_h=1, color="#2f3941"):
    """Evenly spaced reference ticks, so a meter reads as a meter."""
    out = []
    for i in range(1, ticks):
        y = int(round(h * i / float(ticks)))
        out.append('<rect x="3" y="%d" width="%d" height="%d" fill="%s"/>'
                   % (y, w - 6, tick_h, color))
    return "".join(out)


def vu_bg(w, h):
    """Visible housing: mid-dark well, brighter outline, reference ticks."""
    return ('<rect x="0.5" y="0.5" width="%d" height="%d" fill="%s" '
            'stroke="%s"/>%s'
            % (w - 1, h - 1, BG_WELL, "#303840", vu_scale(w, h)))


def vu_body(w, h, top, bottom):
    """Level bar with a vertical gradient (Mixxx stretches it to the level)."""
    return ('<defs><linearGradient id="g" x1="0" y1="1" x2="0" y2="0">'
            '<stop offset="0" stop-color="%s"/>'
            '<stop offset="1" stop-color="%s"/></linearGradient></defs>'
            '<rect x="0" y="0" width="%d" height="%d" fill="url(#g)"/>'
            % (bottom, top, w, h))


def clip_bg(w, h):
    return ('<rect x="0.5" y="0.5" width="%d" height="%d" fill="%s" '
            'stroke="%s"/>' % (w - 1, h - 1, RED_DIM, "#5a2a2e"))


# IMPORTANT: the meter art is generated at its final on-screen pixel size. In
# Mixxx an element with no `scalemode` is Fixed to the pixmap's size, so the
# pixmap decides the widget size - which is how LateNight does it and why
# declaring <Size> on a VuMeter achieves nothing. Deck meters are 34 x 230 with
# a separate 34 x 10 clip housing.
for size, w, h, cw, ch in (("deck", 34, 230, 34, 10),
                           ("small", 12, 30, 12, 10),
                           ("sampler", 22, 30, 22, 10)):
    write("bases/vu_%s_level_bg.svg" % size, w, h, vu_bg(w, h))
    write("bases/vu_%s_level_active.svg" % size, w, h,
          vu_body(w, h, GREEN, GREEN_DIM))
    write("bases/vu_%s_clipping_bg.svg" % size, cw, ch, clip_bg(cw, ch))
    write("bases/vu_%s_clipping_active.svg" % size, cw, ch,
          '<rect x="0" y="0" width="%d" height="%d" fill="%s"/>' % (cw, ch, RED))

# --------------------------------------------------------- misc style pixmaps
# CPU / latency meter (horizontal bar) inside #RbxCpuBox, painted #15181b.
write("bases/latency_bg.svg", 70, 11,
      '<rect x="0.5" y="1.5" width="69" height="8" rx="2" fill="%s" '
      'stroke="%s"/>%s'
      % ("#2b323a", "#59636e",
         "".join('<rect x="%d" y="2" width="1" height="7" fill="#1b1f24"/>'
                 % (7 * i) for i in range(1, 10))))
write("bases/latency_over.svg", 70, 11,
      '<rect x="0.5" y="1.5" width="69" height="8" rx="2" fill="%s"/>' % ACCENT)

# default cover art: neutral placeholder, own design (no vendor logo).
# The glyph is centred with equal margins: the old version sat at +6+12 with a
# 1px bottom margin, which read as a mis-aligned image rather than a placeholder.
cd = 46
write("styles/cover_default.svg", cd, cd,
      '<rect x="0" y="0" width="%d" height="%d" fill="%s"/>'
      '<rect x="0.5" y="0.5" width="%d" height="%d" fill="none" stroke="%s"/>'
      # a record seen face-on: outer disc, groove ring, label, spindle
      '<circle cx="23" cy="23" r="15" fill="#191e23" stroke="#2f3841"/>'
      '<circle cx="23" cy="23" r="11.5" fill="none" stroke="#242c33"/>'
      '<circle cx="23" cy="23" r="8.5" fill="none" stroke="#242c33"/>'
      '<circle cx="23" cy="23" r="5.5" fill="#2b333c"/>'
      '<circle cx="23" cy="23" r="1.6" fill="#0d0f11"/>'
      # a soft top-left highlight so the disc reads as a disc
      '<path d="M12.5 16 A13.5 13.5 0 0 1 24 8.6" fill="none" '
      'stroke="#48535e" stroke-width="1.4" stroke-linecap="round"/>'
      % (cd, cd, BG_PANEL, cd - 1, cd - 1, BORDER_HI))
