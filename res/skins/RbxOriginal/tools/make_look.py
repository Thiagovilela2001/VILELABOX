#!/usr/bin/env python3
"""RbxFlow look pass 4 (flat): knobs, faders, VU meters and the tempo dial.

Run AFTER tools/make_assets.py - this overwrites the files listed below with
the depth-shaded versions (make_assets.py still owns buttons, latency meter and
the small/sampler VU sizes).

Why gradients now: the previous pass avoided them out of caution, which is a big
part of why the skin read as "flat". Qt's SVG renderer (QtSvg, SVG Tiny 1.2)
handles <linearGradient> and <radialGradient> fine - verified in the running
binary - so the hardware gets real shading: lit caps, recessed grooves,
segmented meters.

Same size rules as make_assets.py: viewBox == pixel size, and anything Mixxx
draws Fixed (VU meters, clip lights) is generated at its on-screen size.
"""
import os

SKIN = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

ACCENT = "#2f8cff"
CUE = "#ff8c1a"
GREEN = "#36d860"
YELLOW = "#ffd23f"
RED = "#ff3b30"


def svg(w, h, body, defs=""):
    d = "<defs>%s</defs>" % defs if defs else ""
    return ('<svg xmlns="http://www.w3.org/2000/svg" width="%d" height="%d" '
            'viewBox="0 0 %d %d">%s%s</svg>\n' % (w, h, w, h, d, body))


def write(rel, w, h, body, defs=""):
    p = os.path.join(SKIN, rel)
    with open(p, "w") as f:
        f.write(svg(w, h, body, defs))


def lin(id_, stops, vertical=True):
    x2, y2 = ("0", "1") if vertical else ("1", "0")
    s = "".join('<stop offset="%s" stop-color="%s"/>' % st for st in stops)
    return ('<linearGradient id="%s" x1="0" y1="0" x2="%s" y2="%s">%s'
            '</linearGradient>' % (id_, x2, y2, s))


# ------------------------------------------------------------------ knobs
# The value arc is drawn by KnobComposed at ArcRadius (14 for a 40 px knob,
# i.e. 0.70R). The body therefore keeps a dark recess out to the rim and the
# cap stays inside 0.62R so the arc runs in the gutter between them.
def knob_body(r):
    """Flat: a dark ring the value arc runs on, and a solid cap. No gradients
    (pass 4 - the shaded cap read as plastic)."""
    c = r
    body = ('<circle cx="%g" cy="%g" r="%g" fill="#0d0e10"/>'
            '<circle cx="%g" cy="%g" r="%g" fill="#2a2d33"/>'
            % (c, c, r - 0.5, c, c, r * 0.60))
    return body, ""


def knob_pointer(color, r):
    c = r
    return ('<line x1="%g" y1="%g" x2="%g" y2="%g" stroke="%s" '
            'stroke-width="%g" stroke-linecap="round"/>'
            % (c, c - 0.18 * r, c, c - 0.55 * r, color, max(1.8, r * 0.11)))


for kind, r in (("regular", 20), ("small", 16), ("main", 24)):
    body, defs = knob_body(r)
    write("bases/knob_bg_%s.svg" % kind, 2 * r, 2 * r, body, defs)

COLORS = (("#eef0f2", "grey"), (ACCENT, "blue"), (CUE, "amber"),
          (GREEN, "green"), (RED, "red"))
for color, name in COLORS:
    write("bases/knob_indicator_%s.svg" % name, 40, 40, knob_pointer(color, 20))
    for kind, r in (("regular", 20), ("small", 16), ("main", 24)):
        write("bases/knob_indicator_%s_%s.svg" % (kind, name), 2 * r, 2 * r,
              knob_pointer(color, r))


# ---------------------------------------------------------------- faders
def groove_v(w, h, ticks=10):
    """A recessed slot with a printed scale either side of it."""
    sx = w / 2.0 - 3
    out = ['<rect x="%g" y="2" width="6" height="%d" rx="3" fill="#0d0e10"/>'
           % (sx, h - 4)]
    for i in range(ticks + 1):
        y = 6 + (h - 12) * i / float(ticks)
        long_ = i in (0, ticks // 2, ticks)
        tw = 7 if long_ else 4
        col = "#4a4e56" if long_ else "#2c2f35"
        out.append('<rect x="%g" y="%g" width="%d" height="1" fill="%s"/>'
                   % (sx - 4 - tw, y, tw, col))
        out.append('<rect x="%g" y="%g" width="%d" height="1" fill="%s"/>'
                   % (sx + 10, y, tw, col))
    return "".join(out)


def groove_h(w, h, ticks=16):
    sy = h / 2.0 - 3
    out = ['<rect x="2" y="%g" width="%d" height="6" rx="3" fill="#0d0e10"/>'
           % (sy, w - 4)]
    for i in range(ticks + 1):
        x = 10 + (w - 20) * i / float(ticks)
        long_ = i in (0, ticks // 2, ticks)
        th = 6 if long_ else 3
        col = "#4a4e56" if long_ else "#2c2f35"
        out.append('<rect x="%g" y="%g" width="1" height="%d" fill="%s"/>'
                   % (x, sy - 3 - th, th, col))
        out.append('<rect x="%g" y="%g" width="1" height="%d" fill="%s"/>'
                   % (x, sy + 9, th, col))
    return "".join(out)


def cap_v(w, h, cw, line="#ffffff"):
    """Flat fader cap: solid rounded body, one index line. No shine."""
    x = (w - cw) / 2.0
    body = ('<rect x="%g" y="1" width="%d" height="%d" rx="4" fill="#3a3d44"/>'
            '<rect x="%g" y="%g" width="%d" height="2" rx="1" fill="%s"/>'
            % (x, cw, h - 2, x + 4, h / 2.0 - 1, cw - 8, line))
    return body, ""


def cap_h(w, h, ch):
    y = (h - ch) / 2.0
    body = ('<rect x="1" y="%g" width="%d" height="%d" rx="4" fill="#3a3d44"/>'
            '<rect x="%g" y="%g" width="2" height="%d" rx="1" fill="#2f8cff"/>'
            % (y, w - 2, ch, w / 2.0 - 1, y + 5, ch - 10))
    return body, ""


write("bases/slider_volume.svg", 62, 164, groove_v(62, 164))
write("bases/slider_pitch.svg", 84, 260, groove_v(84, 260, ticks=8))
write("bases/slider_crossfader.svg", 560, 36, groove_h(560, 36))
b, d = cap_v(62, 26, 40)
write("bases/knob_volume.svg", 62, 26, b, d)
b, d = cap_v(84, 24, 48, line=CUE)
write("bases/knob_pitch.svg", 84, 24, b, d)
b, d = cap_h(24, 34, 34)
write("bases/knob_crossfader.svg", 24, 34, b, d)


# -------------------------------------------------------------- VU meters
# Segmented LED ladder. Mixxx reveals the "active" pixmap from the bottom up to
# the level, so colouring each segment by its HEIGHT gives the classic
# green -> yellow -> red ladder for free.
def vu(w, h, seg=3, gap=1, lit=True):
    out = []
    n = (h + gap) // (seg + gap)
    y0 = h - n * (seg + gap) + gap
    for i in range(n):
        y = y0 + i * (seg + gap)
        frac = 1.0 - (y + seg / 2.0) / h        # 0 bottom .. 1 top
        if frac > 0.86:
            col = RED
        elif frac > 0.70:
            col = YELLOW
        else:
            col = GREEN
        if lit:
            out.append('<rect x="0" y="%d" width="%d" height="%d" fill="%s"/>'
                       % (y, w, seg, col))
        else:
            out.append('<rect x="0" y="%d" width="%d" height="%d" fill="%s" '
                       'fill-opacity="0.13"/>' % (y, w, seg, col))
    return "".join(out)


VW, VH = 10, 114
write("bases/vu_deck_level_bg.svg", VW, VH,
      '<rect x="0" y="0" width="%d" height="%d" fill="#060607"/>%s'
      % (VW, VH, vu(VW, VH, lit=False)))
write("bases/vu_deck_level_active.svg", VW, VH, vu(VW, VH))
# The GL VuMeter cannot texture an SVG (Paintable::toImage() is empty for SVG
# sources), so the XML references PNG copies of the two level images.
import subprocess
for _n in ("bg", "active"):
    _b = os.path.join(SKIN, "bases/vu_deck_level_%s" % _n)
    subprocess.run(["rsvg-convert", _b + ".svg", "-o", _b + ".png"], check=True)
write("bases/vu_deck_clipping_bg.svg", VW, 6,
      '<rect x="0" y="0" width="%d" height="5" rx="1" fill="%s" '
      'fill-opacity="0.22"/>' % (VW, RED))
write("bases/vu_deck_clipping_active.svg", VW, 6,
      '<rect x="0" y="0" width="%d" height="5" rx="1" fill="%s"/>' % (VW, RED))


# ------------------------------------------------------------- tempo dial
# Bare housing for the KnobComposed tempo arc (ArcRadius 26 on a 64 px dial):
# a recessed bezel with a printed scale, transparent centre.
def dial(d):
    c = d / 2.0
    defs = ('<radialGradient id="dg" cx="0.5" cy="0.35" r="0.65">'
            '<stop offset="0" stop-color="#26282c"/>'
            '<stop offset="1" stop-color="#101113"/></radialGradient>')
    out = ['<circle cx="%g" cy="%g" r="%g" fill="url(#dg)" stroke="#2a2c31"/>'
           % (c, c, c - 0.5),
           '<circle cx="%g" cy="%g" r="%g" fill="#08090a" stroke="#1f2124"/>'
           % (c, c, c - 9)]
    return "".join(out), defs


b, d = dial(62)
write("bases/dial_ring.svg", 62, 62, b, d)

print("make_look: knobs, faders, VU, dial written")
