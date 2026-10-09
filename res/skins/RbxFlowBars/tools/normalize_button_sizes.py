#!/usr/bin/env python3
"""Rewrite button pixmap references so every button base matches its slot.

Why: Mixxx has no 9-slice scaling. `scalemode="STRETCH"` scales x and y
independently, so a generic 80x20 base dropped into a 28x17 slot deforms its
rx=3 corners into ellipses and makes the 1px border 2.5x thicker vertically
than horizontally. Generating one base per (width, height) used removes the
distortion completely.

Two passes:
  1. template calls (controls/btn_*.xml): BtnSize := "<W>x<H>" taken from the
     call's own Size set variable, when that Size is a concrete "Wf,Hf".
  2. raw <PushButton> blocks: btn_<name>.svg -> btn_<W>x<H>.svg using the
     <Size> inside the same element.

Idempotent; prints what it changed.
"""
import os
import re
import sys

SKIN = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

# Art that is deliberately NOT part of the generic per-size family and must not
# be "normalised" back to btn_<W>x<H>:
#   blank      the transparent pair used for the segmented switcher and the
#              invisible seek overlay - its look comes from QSS
#   rec        the record button's own red theme
#   hotcue*    the numbered pad art
KEEP_BASES = ("blank", "rec", "hotcue")
KEEP_BTNSIZE = ("blank",)

SIZE_RE = re.compile(
    r'<SetVariable name="Size">(-?\d+)f,(-?\d+)f</SetVariable>'
    r"|<Size>(-?\d+)f,(-?\d+)f</Size>")


def size_of(body):
    """Return (w, h) from either a SetVariable Size or a plain <Size> tag."""
    m = SIZE_RE.search(body)
    if not m:
        return None
    g = m.groups()
    return (g[0], g[1]) if g[0] is not None else (g[2], g[3])

BTN_REF = re.compile(r"btn_(\d+)x(\d+)(_active)?\.svg")
BTNSIZE_REF = re.compile(r'<SetVariable name="BtnSize">(\d+)x(\d+)</SetVariable>')
BLOCK_RE = re.compile(
    r'<Template src="skin:controls/(btn_[a-z0-9_]+)\.xml">(.*?)</Template>', re.S)
PUSHB_RE = re.compile(r"<PushButton>(.*?)</PushButton>", re.S)
BASEPIX_RE = re.compile(r"skin:bases/btn_([A-Za-z0-9_]+?)(_active)?\.svg")


def xml_files():
    for root, _dirs, files in os.walk(SKIN):
        if os.sep + "tools" in root:
            continue
        for f in files:
            if f.endswith(".xml"):
                yield os.path.join(root, f)


def convert(text):
    changed = []

    def block_sub(m):
        name, body = m.group(1), m.group(2)
        wh = size_of(body)
        if not wh:
            return m.group(0)
        want = "%sx%s" % wh
        bm = BTNSIZE_REF.search(body)
        if not bm:
            return m.group(0)
        if bm.group(1) == want or bm.group(1).lower() in KEEP_BTNSIZE:
            return m.group(0)
        changed.append("    %s: BtnSize %s -> %s" % (name, bm.group(1), want))
        newbody = body[:bm.start()] + '<SetVariable name="BtnSize">%s</SetVariable>' % want + body[bm.end():]
        return m.group(0).replace(body, newbody)

    text = BLOCK_RE.sub(block_sub, text)

    def push_sub(m):
        body = m.group(1)
        wh = size_of(body)
        if not wh:
            return m.group(0)
        want = "%sx%s" % wh

        def pix(pm):
            base, act = pm.group(1), pm.group(2) or ""
            if base.lower().startswith(KEEP_BASES):
                return pm.group(0)
            if base == want:
                return pm.group(0)
            changed.append("    raw PushButton: btn_%s%s -> btn_%s%s"
                           % (base, act, want, act))
            return "skin:bases/btn_%s%s.svg" % (want, act)

        return "<PushButton>" + BASEPIX_RE.sub(pix, body) + "</PushButton>"

    text = PUSHB_RE.sub(push_sub, text)
    return text, changed


def main():
    total = 0
    for path in sorted(xml_files()):
        with open(path) as fh:
            src = fh.read()
        new, changed = convert(src)
        if changed and new != src:
            rel = os.path.relpath(path, SKIN)
            print(rel)
            for line in sorted(set(changed)):
                print(line)
            total += len(changed)
            if "--dry-run" not in sys.argv:
                with open(path, "w") as fh:
                    fh.write(new)
    print("\n%d reference(s) rewritten%s" % (total, " (dry run)" if "--dry-run" in sys.argv else ""))


if __name__ == "__main__":
    main()
