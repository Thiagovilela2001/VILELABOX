#!/usr/bin/env python3
"""Restore widget size tags that were wrongly turned into <SetVariable>.

Mixxx only honours <SetVariable> as a DIRECT child of <Template>, where it
assigns a template variable. Inside widget content -- <PushButton>,
<KnobComposed>, <SliderComposed>, <Label> -- a size must be a real <Size> tag.
A <SetVariable> there is ignored: the widget is built with no size and collapses
to nothing, which is why whole rows of buttons render blank.

Walks the XML with a tag stack and rewrites

    <SetVariable name="Size">X</SetVariable>   ->   <Size>X</Size>

whenever the SetVariable's parent element is NOT <Template>.
"""
import glob
import os
import re

SKIN = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

TAG = re.compile(r"<(/?)([A-Za-z_][A-Za-z0-9_]*)\b((?:\"[^\"]*\"|[^>\"])*?)(/?)>")
SIZE_TAGS = {"Size", "MinimumSize", "MaximumSize"}


def restore(src):
    edits = []
    stack = []      # names of currently open elements
    open_tag = None  # (start, end, name) of the most recent open tag

    for m in TAG.finditer(src):
        closing, name, attrs, selfclose = m.groups()
        if selfclose:
            continue

        if not closing:
            if name == "SetVariable":
                nm = re.search(r'name="([^"]+)"', attrs or "")
                val = nm.group(1) if nm else ""
                parent = stack[-1] if stack else None
                if val in SIZE_TAGS and parent != "Template":
                    edits.append([m.start(), m.end(), val, None])
            stack.append(name)
            continue

        # closing tag: if it closes the SetVariable we recorded, complete it
        if edits and edits[-1][3] is None and stack and stack[-1] == "SetVariable":
            edits[-1][3] = (m.start(), m.end())
        if stack:
            stack.pop()

    edits = [e for e in edits if e[3] is not None]
    if not edits:
        return None

    out, last = [], 0
    for o_start, o_end, val, (c_start, c_end) in sorted(edits):
        out.append(src[last:o_start])
        out.append("<%s>" % val)
        out.append(src[o_end:c_start])
        out.append("</%s>" % val)
        last = c_end
    out.append(src[last:])
    return "".join(out)


changed = []
for f in sorted(glob.glob(os.path.join(SKIN, "**", "*.xml"), recursive=True)):
    if os.sep + "tools" + os.sep in f:
        continue
    src = open(f).read()
    res = restore(src)
    if res and res != src:
        open(f, "w").write(res)
        changed.append(os.path.relpath(f, SKIN))

print("\n".join(changed) if changed else "nothing to restore")