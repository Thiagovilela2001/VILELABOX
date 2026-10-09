#!/usr/bin/env python3
"""Capture one window by id instead of grabbing the whole screen.

Why this exists: a full-screen `spectacle -f` grab shows whatever happens to be
stacked on top, so a floating terminal or an unfocused browser lands in the
middle of the skin - and a tooltip appears wherever the pointer was left. Two
"verification" screenshots were thrown away to exactly that before this.

This finds the top-level window whose WM_CLASS matches, raises it through EWMH,
and hands the id to ImageMagick's `import -window`, which reads that window's own
pixels. Obscured regions come back correct under a compositor because XWayland
redirects every window into an offscreen buffer.

usage:  python3 tools/grab_window.py out.png [wm_class_substring] [WxH]
"""
import ctypes
import ctypes.util
import os
import subprocess
import sys
import time

ClientMessage = 33
_NET_WM_STATE_ADD = 1
_SUBSTRUCTURE_MASK = 0x80000 | 0x100000


def x11():
    lib = ctypes.CDLL(ctypes.util.find_library("X11") or "libX11.so.6")
    lib.XOpenDisplay.restype = ctypes.c_void_p
    lib.XOpenDisplay.argtypes = [ctypes.c_char_p]
    lib.XDefaultRootWindow.restype = ctypes.c_ulong
    lib.XDefaultRootWindow.argtypes = [ctypes.c_void_p]
    lib.XInternAtom.restype = ctypes.c_ulong
    lib.XInternAtom.argtypes = [ctypes.c_void_p, ctypes.c_char_p, ctypes.c_int]
    lib.XSendEvent.restype = ctypes.c_int
    lib.XSendEvent.argtypes = [ctypes.c_void_p, ctypes.c_ulong, ctypes.c_int,
                               ctypes.c_long, ctypes.c_void_p]
    lib.XRaiseWindow.restype = ctypes.c_int
    lib.XRaiseWindow.argtypes = [ctypes.c_void_p, ctypes.c_ulong]
    lib.XSync.restype = ctypes.c_int
    lib.XSync.argtypes = [ctypes.c_void_p, ctypes.c_int]
    return lib


def send_above(lib, dpy, root, win):
    ev = ctypes.create_string_buffer(192)
    above = lib.XInternAtom(dpy, b"_NET_WM_STATE_ABOVE", 0)
    state = lib.XInternAtom(dpy, b"_NET_WM_STATE", 0)
    puts = [(0, ctypes.c_int, ClientMessage), (16, ctypes.c_int, 1),
            (32, ctypes.c_ulong, win), (40, ctypes.c_ulong, state),
            (48, ctypes.c_int, 32), (56, ctypes.c_long, _NET_WM_STATE_ADD),
            (64, ctypes.c_long, above), (72, ctypes.c_long, 0),
            (80, ctypes.c_long, 1)]
    for off, ctype, val in puts:
        c = ctype(val)
        ctypes.memmove(ctypes.byref(ev, off), ctypes.byref(c), ctypes.sizeof(c))
    lib.XSendEvent(dpy, root, False, _SUBSTRUCTURE_MASK, ctypes.byref(ev))


def wm_class(win):
    try:
        return subprocess.run(["xprop", "-id", hex(win), "WM_CLASS"],
                              capture_output=True, text=True, timeout=10).stdout
    except Exception:
        return ""


def clients():
    try:
        out = subprocess.run(["xprop", "-root", "_NET_CLIENT_LIST"],
                             capture_output=True, text=True, timeout=10).stdout
    except Exception:
        return []
    ids = []
    for tok in out.replace(",", " ").split():
        if tok.startswith("0x"):
            try:
                ids.append(int(tok, 16))
            except ValueError:
                pass
    return ids


def main():
    out = sys.argv[1] if len(sys.argv) > 1 else "/tmp/window.png"
    want = (sys.argv[2] if len(sys.argv) > 2 else "mixxx").lower()
    size = sys.argv[3] if len(sys.argv) > 3 else None

    lib = x11()
    dpy = lib.XOpenDisplay(None)
    if not dpy:
        print("grab_window: cannot open $DISPLAY", file=sys.stderr)
        return 2
    root = lib.XDefaultRootWindow(dpy)

    target = None
    for _ in range(20):
        for win in clients():
            if want in wm_class(win).lower():
                target = win
                break
        if target:
            break
        time.sleep(1)
    if not target:
        print("grab_window: no window with WM_CLASS containing %r" % want,
              file=sys.stderr)
        return 1

    send_above(lib, dpy, root, target)
    lib.XRaiseWindow(dpy, target)
    lib.XSync(dpy, 0)
    time.sleep(1.5)

    if size:
        subprocess.run(["python3",
                        os.path.join(os.path.dirname(os.path.abspath(__file__)),
                                     "winresize.py"),
                        size, want], capture_output=True)
        time.sleep(1.5)

    subprocess.run(["python3",
                    os.path.join(os.path.dirname(os.path.abspath(__file__)),
                                 "park_pointer.py"), "4x4"], capture_output=True)
    time.sleep(1.0)

    # -window takes the id; cropping to WxH is done by the caller when needed
    r = subprocess.run(["magick", "import", "-window", hex(target), out],
                       capture_output=True, text=True)
    if r.returncode != 0 or not os.path.exists(out):
        print("grab_window: import failed:", r.stderr.strip(), file=sys.stderr)
        return 1
    print("captured 0x%x -> %s" % (target, out))
    return 0


if __name__ == "__main__":
    sys.exit(main())
