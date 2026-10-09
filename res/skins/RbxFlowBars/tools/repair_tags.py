#!/usr/bin/env python3
"""Repair closers wrongly rewritten to </SetVariable>.

Scans with a tag stack. Whenever a closer does not match the open tag on top
of the stack, it is rewritten to close that tag instead. This undoes a bad
regex rewrite while preserving every byte of text content.
"""
import glob
import os
import re

SKIN = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

TAG = re.compile(r"<(/?)([A-Za-z_][A-Za-z0-9_]*)\b((?:\"[^\"]*\"|[^>\"])*?)(/?)>")


def repair(src):
    stack = []
    pieces = []
    for m in TAG.finditer(src):
        closing, name, _attrs, selfclose = m.groups()
        if selfclose:
            continue
        if closing:
            if not stack:
                continue
            opener = stack.pop()
            if opener != name:
                pieces.append((m.start(), m.end(), "</%s>" % opener))
        else:
            stack.append(name)
    if not pieces:
        return None
    out, last = [], 0
    for s, e, t in pieces:
        out.append(src[last:s])
        out.append(t)
        last = e
    out.append(src[last:])
    return "".join(out)


fixed = []
for f in sorted(glob.glob(os.path.join(SKIN, "**", "*.xml"), recursive=True)):
    src = open(f).read()
    res = repair(src)
    if res and res != src:
        open(f, "w").write(res)
        fixed.append(os.path.relpath(f, SKIN))

print("\n".join(fixed) if fixed else "nothing to repair")