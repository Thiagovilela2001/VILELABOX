#!/usr/bin/env python3
"""RbxOriginal: knobs, fader caps, fader grooves and the jog display.

The rekordbox installer ships no knob or fader art for Performance mode (that
UI is drawn by rekordbox.exe itself), so these are drawn here to match it:
flat #1e1e1e / #282828 greys, a white index line, rekordbox blue #1473eb for
value arcs. The other icons in rb/ are own drawings that keep the installer's
file names (commit 7b7d6fd0ab); none is a copy of the installer's file.

Sizes are the ones the XML already uses (viewBox == pixel size).
"""
import os
import struct
import zlib

SKIN = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

BLUE = "#1473eb"
WHITE = "#e6e6e6"


def write(rel, w, h, body):
    # newline="\n": the same bytes on Windows and Linux.
    with open(os.path.join(SKIN, rel), "w", newline="\n") as f:
        f.write('<svg xmlns="http://www.w3.org/2000/svg" width="%d" height="%d" '
                'viewBox="0 0 %d %d">%s</svg>\n' % (w, h, w, h, body))


def write_png(rel, w, h, pixel):
    """RGB PNG with the standard library only; pixel(x, y) -> (r, g, b)."""
    rows = b"".join(b"\x00" + bytes(c for x in range(w) for c in pixel(x, y)) for y in range(h))

    def chunk(kind, data):
        return (struct.pack(">I", len(data)) + kind + data
                + struct.pack(">I", zlib.crc32(kind + data) & 0xFFFFFFFF))

    with open(os.path.join(SKIN, rel), "wb") as f:
        f.write(b"\x89PNG\r\n\x1a\n"
                + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 2, 0, 0, 0))
                + chunk(b"IDAT", zlib.compress(rows, 9)) + chunk(b"IEND", b""))


# ------------------------------------------------------------- knobs
# Mixxx draws the value arc at ArcRadius; the cap stays inside it.
for name, d in (("main", 48), ("regular", 40), ("small", 32)):
    c = d / 2
    write("bases/knob_bg_%s.svg" % name, d, d,
          '<circle cx="%g" cy="%g" r="%g" fill="#141414"/>'
          '<circle cx="%g" cy="%g" r="%g" fill="#2d2d2d" stroke="#4b4b4b" stroke-width="1"/>'
          % (c, c, c - 0.5, c, c, d * 0.30))
    for col in ("amber", "blue", "green", "grey", "red"):
        write("bases/knob_indicator_%s_%s.svg" % (name, col), d, d,
              '<line x1="%g" y1="%g" x2="%g" y2="%g" stroke="%s" stroke-width="2" '
              'stroke-linecap="round"/>' % (c, c - d * 0.08, c, c - d * 0.27, WHITE))
for col in ("amber", "blue", "green", "grey", "red"):
    write("bases/knob_indicator_%s.svg" % col, 40, 40,
          '<line x1="20" y1="16.8" x2="20" y2="9.2" stroke="%s" stroke-width="2" '
          'stroke-linecap="round"/>' % WHITE)


# ------------------------------------------------------- fader caps
def cap_v(w, h, cw):
    x = (w - cw) / 2
    return ('<rect x="%g" y="1" width="%g" height="%g" rx="2" fill="#3a3a3a" stroke="#141414"/>'
            '<rect x="%g" y="2" width="%g" height="%g" rx="1.5" fill="#2a2a2a"/>'
            '<rect x="%g" y="%g" width="%g" height="2" fill="#f0f0f0"/>'
            % (x, cw, h - 2, x + 1, cw - 2, h / 2 - 2, x + 2, h / 2 - 1, cw - 4))


write("bases/knob_volume.svg", 62, 26, cap_v(62, 26, 34))
write("bases/knob_pitch.svg", 84, 24, cap_v(84, 24, 38))
write("bases/knob_crossfader.svg", 24, 34,
      '<rect x="4" y="1" width="16" height="32" rx="2" fill="#3a3a3a" stroke="#141414"/>'
      '<rect x="5" y="2" width="14" height="15" rx="1.5" fill="#2a2a2a"/>'
      '<rect x="11" y="4" width="2" height="26" fill="#f0f0f0"/>')


# ------------------------------------------------------- fader grooves
def groove_v(w, h, ticks, centre=False):
    c = w / 2
    b = '<rect x="%g" y="6" width="4" height="%g" rx="2" fill="#000" stroke="#3c3c3c"/>' % (c - 2, h - 12)
    for i in range(ticks + 1):
        y = 10 + (h - 20) * i / ticks
        b += '<rect x="%g" y="%g" width="7" height="1" fill="#4b4b4b"/>' % (c - 17, y)
        b += '<rect x="%g" y="%g" width="7" height="1" fill="#4b4b4b"/>' % (c + 10, y)
    if centre:
        b += '<rect x="%g" y="%g" width="14" height="2" fill="#8a8a8a"/>' % (c - 19, h / 2 - 1)
        b += '<rect x="%g" y="%g" width="14" height="2" fill="#8a8a8a"/>' % (c + 5, h / 2 - 1)
    return b


write("bases/slider_volume.svg", 62, 164, groove_v(62, 164, 10))
write("bases/slider_pitch.svg", 84, 260, groove_v(84, 260, 8, centre=True))
xf = '<rect x="8" y="16" width="544" height="4" rx="2" fill="#000" stroke="#3c3c3c"/>'
for i in range(9):
    x = 14 + 532 * i / 8
    xf += '<rect x="%g" y="5" width="1" height="7" fill="#4b4b4b"/>' % x
    xf += '<rect x="%g" y="24" width="1" height="7" fill="#4b4b4b"/>' % x
write("bases/slider_crossfader.svg", 560, 36, xf)


# ------------------------------------------------------------ jog display
# KnobComposed bound to playposition: the background is the black display
# with rekordbox's thin blue inner ring; the "knob" is the red position mark
# riding the outer ring, the grey arc (ArcBgColor) is the ring itself.
J = 150
c = J / 2
write("rb/jog_bg.svg", J, J,
      '<circle cx="%g" cy="%g" r="%g" fill="#000"/>'
      '<circle cx="%g" cy="%g" r="%g" fill="none" stroke="%s" stroke-width="1.6"/>'
      % (c, c, c - 13, c, c, c - 17, BLUE))
write("rb/jog_marker.svg", J, J,
      '<rect x="%g" y="1" width="8" height="11" fill="#e01e1e"/>' % (c - 4))


# ------------------------------------------------------------------ mixer
# rekordbox's mixer, as in the installer's tutorial screenshots
# (tutorial/12_17.png, tutorial/DDJ200/mix_10.png): no knob body, just the
# ring (Mixxx's ArcBg, -135..135) and a short white index; its knob.svg is a
# 2 x 5 white bar. Faders are a hairline track with a blue level line and a
# line handle; the crossfader is a hairline with three ticks.
write("bases/knob_bg_none.svg", 32, 32, '<rect width="32" height="32" fill="none"/>')
write("bases/knob_indicator_rb.svg", 32, 32,
      '<rect x="15" y="5" width="2" height="6" fill="#f0f0f0"/>')
write("rb/blank.svg", 4, 4, '<rect width="4" height="4" fill="none"/>')


# EQ band label = kill button, like rekordbox's eqHighOff/eqHighOn (36 x 16):
# a #282828 plate; when the band is killed, a blue outline and a diagonal.
def eq_kill(on):
    b = '<rect x="0.5" y="0.5" width="37" height="13" rx="2" fill="#282828"%s/>' % (
        ' stroke="%s"' % BLUE if on else "")
    if on:
        b += '<line x1="1" y1="1" x2="37" y2="13" stroke="%s" stroke-width="1"/>' % BLUE
    return b


write("rb/eq_kill_off.svg", 38, 14, eq_kill(False))
write("rb/eq_kill_on.svg", 38, 14, eq_kill(True))

# Channel fader handle: 20 px wide slider, 8 px handle -> BarMargins 4,4.
write("rb/fader_handle.svg", 20, 8, '<rect x="2" y="3" width="16" height="2" fill="#d2d2d2"/>')
# Horizontal handle for the master level slider in the top bar (14 px high).
write("rb/slider_handle_h.svg", 6, 14, '<rect x="2" y="1" width="2" height="12" fill="#d2d2d2"/>')

XW, XH = 155, 20
xf = '<rect x="4" y="9" width="%d" height="2" fill="#3c3c3c"/>' % (XW - 8)
for x in (4, (XW - 1) / 2, XW - 5):
    xf += '<rect x="%g" y="5" width="1" height="10" fill="#4b4b4b"/>' % x
write("rb/xfader_track.svg", XW, XH, xf)
write("rb/xfader_handle.svg", 8, XH, '<rect x="3" y="2" width="2" height="16" fill="#d2d2d2"/>')

write("rb/headphones.svg", 14, 14,
      '<path d="M2.5 9V7a4.5 4.5 0 0 1 9 0v2" fill="none" stroke="#aaaaaa" stroke-width="1.4"/>'
      '<rect x="1.5" y="8" width="3" height="5" rx="1" fill="#aaaaaa"/>'
      '<rect x="9.5" y="8" width="3" height="5" rx="1" fill="#aaaaaa"/>')


# Level meters: flat green / yellow / red zones instead of LED segments, so the
# channel meter can stretch with the fader (scalemode STRETCH) without blurring.
# PNG because the GL VuMeter cannot texture an SVG.
def zone(frac):
    if frac > 0.86:
        return (0xFF, 0x3B, 0x30)
    if frac > 0.70:
        return (0xFF, 0xD2, 0x3F)
    return (0x36, 0xD8, 0x60)


def dim(rgb):
    return tuple(int(10 + (c - 10) * 0.16) for c in rgb)


VW, VH = 4, 120
write_png("bases/vu_rb_v_active.png", VW, VH, lambda x, y: zone(1 - (y + 0.5) / VH))
write_png("bases/vu_rb_v_bg.png", VW, VH, lambda x, y: dim(zone(1 - (y + 0.5) / VH)))
HW, HH = 90, 3
write_png("bases/vu_rb_h_active.png", HW, HH, lambda x, y: zone((x + 0.5) / HW))
write_png("bases/vu_rb_h_bg.png", HW, HH, lambda x, y: dim(zone((x + 0.5) / HW)))
