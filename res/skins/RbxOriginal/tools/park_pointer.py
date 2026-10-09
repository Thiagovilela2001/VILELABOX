#!/usr/bin/env python3
"""Park the X pointer in a corner so it cannot raise a tooltip in a screenshot.

Mixxx shows a tooltip under the pointer after a short hover. The screenshot
harnesses never move the mouse, so the pointer sits wherever the session left it
- usually the middle of the screen, over a waveform - and every capture comes out
with a tooltip covering the very zone being inspected. This warps the pointer
somewhere harmless first.

usage:  python3 tools/park_pointer.py [XxY]     (default 4x4)
        DISPLAY must be set (XWayland is fine).
"""
import ctypes
import ctypes.util
import sys


def main():
    lib = ctypes.CDLL(ctypes.util.find_library("X11") or "libX11.so.6")
    lib.XOpenDisplay.restype = ctypes.c_void_p
    lib.XOpenDisplay.argtypes = [ctypes.c_char_p]
    lib.XDefaultRootWindow.restype = ctypes.c_ulong
    lib.XDefaultRootWindow.argtypes = [ctypes.c_void_p]
    lib.XWarpPointer.restype = ctypes.c_int
    lib.XWarpPointer.argtypes = [
        ctypes.c_void_p, ctypes.c_ulong, ctypes.c_ulong,
        ctypes.c_int, ctypes.c_int, ctypes.c_uint, ctypes.c_uint,
        ctypes.c_int, ctypes.c_int]
    lib.XSync.restype = ctypes.c_int
    lib.XSync.argtypes = [ctypes.c_void_p, ctypes.c_int]

    dpy = lib.XOpenDisplay(None)
    if not dpy:
        print("park_pointer: cannot open $DISPLAY", file=sys.stderr)
        return 2

    spec = sys.argv[1] if len(sys.argv) > 1 else "4x4"
    x, y = (int(v) for v in spec.lower().split("x"))
    root = lib.XDefaultRootWindow(dpy)
    # dest_w = root makes dest_x/dest_y absolute; a null dest_w would be relative
    lib.XWarpPointer(dpy, 0, root, 0, 0, 0, 0, x, y)
    lib.XSync(dpy, 0)
    print("pointer parked at %d,%d" % (x, y))
    return 0


if __name__ == "__main__":
    sys.exit(main())
