#!/usr/bin/env python3
"""Fix `<SetVariable name="X"><Variable .../></X>` -> `</SetVariable>` in the skin."""
import glob
import os
import re

SKIN = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

TAGS = ("ArcRadius", "ArcThickness", "ArcColor", "ArcBgColor", "ArcBgThickness",
        "ArcUnipolar", "ArcReversed", "KnobColor", "KnobBg", "ObjectName",
        "TooltipId", "Size", "MinimumSize", "MaximumSize", "BtnSize", "Control",
        "Group", "Align", "ConfigKey", "ConfigKeyRight", "ConfigKeyDisp",
        "State", "Label", "number", "ChanNum", "CoverConfigKey", "fxaderAssign")

CLOSERS = "|".join(TAGS)
pat = re.compile(r"(</(?:%s)>)" % CLOSERS)

fixed = []
for f in sorted(glob.glob(os.path.join(SKIN, "**", "*.xml"), recursive=True)):
    src = open(f).read()
    out, pos, changed = [], 0, False
    # Only rewrite closers that sit inside a SetVariable open tag.
    for m in re.finditer(r'<SetVariable name="([^"]+)">(.*?)</\1>', src, re.S):
        inner = m.group(2)
        new_inner = pat.sub("</SetVariable>", inner)
        if new_inner != inner:
            out.append(src[pos:m.start(2)])
            out.append(new_inner)
            pos = m.end(2)
            changed = True
    if changed:
        out.append(src[pos:])
        open(f, "w").write("".join(out))
        fixed.append(os.path.relpath(f, SKIN))

print("\n".join(fixed) if fixed else "nothing to fix")