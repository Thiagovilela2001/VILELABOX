#!/usr/bin/env python3
"""RbxOriginal: knobs, fader caps, fader grooves and the jog display.

The rekordbox installer ships no knob or fader art for Performance mode (that
UI is drawn by rekordbox.exe itself), so these are drawn here to match it:
flat #1e1e1e / #282828 greys, a white index line, rekordbox blue #1473eb for
value arcs. The rest of the icons in rb/ are the installer's own SVGs.

Sizes are the ones the XML already uses (viewBox == pixel size).
"""
import os

SKIN = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

BLUE = "#1473eb"
WHITE = "#e6e6e6"


def write(rel, w, h, body):
    with open(os.path.join(SKIN, rel), "w") as f:
        f.write('<svg xmlns="http://www.w3.org/2000/svg" width="%d" height="%d" '
                'viewBox="0 0 %d %d">%s</svg>\n' % (w, h, w, h, body))


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
