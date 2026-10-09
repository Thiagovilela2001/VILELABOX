#!/usr/bin/env python3
"""Verify that every `skin:<path>` reference in the skin tree resolves to a file.

Mixxx silently skips a pixmap it cannot find: a button just renders blank and
there is no fatal error, so a typo in an asset name survives a smoke test and
only shows up as an empty control. This catches that statically.

It also verifies, for every generated SVG, that viewBox equals width/height -
Mixxx rasterises an SVG at its VIEWBOX size, so a mismatch silently renders the
asset at the wrong size (this is what once drew hot-cue pads as small brackets).

usage:  python3 tools/check_refs.py
"""
import glob
import os
import re
import sys

SKIN = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

REF = re.compile(r"skin:([A-Za-z0-9_./-]+)")
SVG_HEAD = re.compile(
    r'width="([\d.]+)"\s+height="([\d.]+)"\s+viewBox="0 0 ([\d.]+) ([\d.]+)"')


def main():
    problems = []

    # 1. every skin: reference must exist
    refs = set()
    for path in glob.glob(os.path.join(SKIN, "**", "*.xml"), recursive=True):
        with open(path, errors="ignore") as fh:
            text = fh.read()
        for m in REF.finditer(text):
            # `skin:bases/btn_<Variable name="BtnSize"/>.svg` is a templated
            # path, not a literal one - it is resolved per call site, and the
            # concrete names are checked by the BtnSize pass below.
            if text[m.end():m.end() + 1] == "<":
                continue
            refs.add((m.group(1), os.path.relpath(path, SKIN)))
    for target, src in sorted(refs):
        full = os.path.join(SKIN, target)
        if not os.path.isfile(full):
            problems.append("missing asset: skin:%s  (referenced by %s)" % (target, src))
    print("%d distinct skin: references checked" % len(refs))

    # 2. templates must exist too
    tmpl = set()
    for path in glob.glob(os.path.join(SKIN, "**", "*.xml"), recursive=True):
        with open(path, errors="ignore") as fh:
            for m in re.finditer(r'<Template src="skin:([^"]+)"', fh.read()):
                tmpl.add(m.group(1))
    for t in sorted(tmpl):
        if not os.path.isfile(os.path.join(SKIN, t)):
            problems.append("missing template: skin:%s" % t)
    print("%d distinct templates checked" % len(tmpl))

    # 3. viewBox must equal pixel size in every generated SVG
    svgs = (glob.glob(os.path.join(SKIN, "bases", "*.svg")) +
            glob.glob(os.path.join(SKIN, "styles", "*.svg")))
    for f in sorted(svgs):
        head = open(f).read(600)
        m = SVG_HEAD.search(head)
        if not m:
            problems.append("no width/height/viewBox triple: %s" % os.path.relpath(f, SKIN))
            continue
        w, h, vw, vh = (float(x) for x in m.groups())
        if (w, h) != (vw, vh):
            problems.append("viewBox != pixel size in %s: %gx%g vs %gx%g"
                            % (os.path.relpath(f, SKIN), w, h, vw, vh))
    print("%d SVGs checked for viewBox/pixel agreement" % len(svgs))

    # 4. every button base the XML asks for must have been generated
    wanted = set()
    for path in glob.glob(os.path.join(SKIN, "**", "*.xml"), recursive=True):
        with open(path, errors="ignore") as fh:
            text = fh.read()
        for m in re.finditer(r"btn_(\d+)x(\d+)(_active)?\.svg", text):
            wanted.add("btn_%sx%s%s.svg" % (m.group(1), m.group(2), m.group(3) or ""))
        for m in re.finditer(r'<SetVariable name="BtnSize">(\d+)x(\d+)</SetVariable>', text):
            wanted.add("btn_%sx%s.svg" % (m.group(1), m.group(2)))
            wanted.add("btn_%sx%s_active.svg" % (m.group(1), m.group(2)))
    missing = [n for n in sorted(wanted)
               if not os.path.isfile(os.path.join(SKIN, "bases", n))]
    for n in missing:
        problems.append("button base not generated: bases/%s" % n)
    print("%d button bases required" % len(wanted))

    if problems:
        print("\n%d problem(s):" % len(problems))
        for p in problems:
            print("  -", p)
        return 1
    print("\nall references resolve")
    return 0


if __name__ == "__main__":
    sys.exit(main())
