#!/usr/bin/env python3
"""Audit (and optionally repair) size tags in the RbxFlow skin.

Mixxx's LegacySkinParser rule:
  * <SetVariable name="Size"> is only meaningful as a DIRECT child of <Template>
    (it assigns a template variable, later referenced as <Variable name="Size"/>).
  * Inside widget content -- <PushButton>, <KnobComposed>, <SliderComposed>,
    <Label>, ... -- a size must be a real <Size> tag. A <SetVariable> there is
    silently ignored: the widget is built with no size and collapses to nothing,
    which shows up as blank rows of buttons.

A "pure" template file is one whose <Template> root directly contains the whole
document (the controls/*.xml helpers). There every size IS a template variable.
Files that build real widget trees (decks/, mixer/, waveforms_container.xml,
skin.xml) may contain both.

usage:
    python3 tools/check_sizes.py          # report only
    python3 tools/check_sizes.py --fix    # repair widget-content sizes
"""
import glob
import os
import re
import sys

SKIN = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

TAG = re.compile(r"<(/?)([A-Za-z_][A-Za-z0-9_]*)\b((?:\"[^\"]*\"|[^>\"])*?)(/?)>")
SIZE_TAGS = {"Size", "MinimumSize", "MaximumSize"}

# Files whose root <Template> wraps the whole document: all sizes are variables.
PURE_TEMPLATES = {
    "controls/btn_2state.xml",
    "controls/btn_1state_right.xml",
    "controls/btn_hotcue.xml",
    "controls/knob.xml",
    "helpers/setting_check.xml",
}

FIX = "--fix" in sys.argv


def scan(src):
    """Yield (open_start, open_end, close_start, close_end, tag, parent)."""
    stack = []
    pending = None
    for m in TAG.finditer(src):
        closing, name, attrs, selfclose = m.groups()
        if selfclose:
            continue
        if not closing:
            if name == "SetVariable":
                nm = re.search(r'name="([^"]+)"', attrs or "")
                val = nm.group(1) if nm else ""
                pending = [m.start(), m.end(), None, None, val,
                           stack[-1] if stack else None]
            stack.append(name)
            continue
        if pending and pending[4] in SIZE_TAGS:
            pending[2], pending[3] = m.start(), m.end()
            yield tuple(pending)
            pending = None
        if stack:
            stack.pop()


report = []
for f in sorted(glob.glob(os.path.join(SKIN, "**", "*.xml"), recursive=True)):
    rel = os.path.relpath(f, SKIN)
    if os.sep + "tools" + os.sep in f:
        continue
    src = open(f).read()
    items = []
    for o_start, o_end, c_start, c_end, tag, parent in scan(src):
        if rel in PURE_TEMPLATES:
            # every size must be a template variable
            if parent != "Template":
                report.append("%s: <%s> must be SetVariable (pure template)" % (rel, tag))
                items.append((o_start, o_end, c_start, c_end, tag))
        else:
            # inside widget content it must be a real tag
            if parent != "Template":
                report.append("%s: <%s> inside <%s> must be a real tag"
                              % (rel, tag, parent))
                items.append((o_start, o_end, c_start, c_end, tag))
    if FIX and items:
        out, last = [], 0
        for o_start, o_end, c_start, c_end, tag in sorted(items):
            out.append(src[last:o_start])
            out.append(("<SetVariable name=\"%s\">" % tag) if rel in PURE_TEMPLATES
                       else ("<%s>" % tag))
            out.append(src[o_end:c_start])
            out.append(("</SetVariable>") if rel in PURE_TEMPLATES
                       else ("</%s>" % tag))
            last = c_end
        out.append(src[last:])
        open(f, "w").write("".join(out))

print("\n".join(report) if report else "all sizes conform")
print("%d site(s) %s" % (len(report), "repaired" if FIX else "found (report only)"))