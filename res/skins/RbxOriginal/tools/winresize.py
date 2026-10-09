#!/usr/bin/env python3
"""Resize/move another application's window without xdotool or wmctrl.

Why this exists: KWin 6 declarative scripting cannot resize a client (assigning
frameGeometry/geometry silently does nothing), Mixxx restores its saved
[MainWindow] geometry so Qt's "-geometry" is overridden, and there is no Xvfb on
this box. libX11 needs no extra packages and works through XWayland: the request
reaches the compositor as an ordinary ConfigureRequest.

Two traps found the hard way:
  * a MAXIMIZED window ignores XResizeWindow -- the compositor keeps enforcing
    the maximized geometry. The EWMH _NET_WM_STATE message below removes
    MAXIMIZED_HORZ/VERT first (_NET_WM_STATE_REMOVE = 0).
  * do not walk the X tree by hand with ctypes to find the window; that
    corrupted the heap. `xprop` (x11-utils) does it safely.

usage:  winresize.py <W>x<H> [wm_class_substring]
        DISPLAY must be set (XWayland is fine).
"""
import ctypes
import ctypes.util
import subprocess
import sys
import time

ClientMessage = 33
_NET_WM_STATE_REMOVE = 0
_NET_WM_STATE_ADD = 1
_SUBSTRUCTURE_MASK = 0x80000 | 0x100000  # SubstructureNotify | SubstructureRedirect


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
    lib.XMoveWindow.restype = ctypes.c_int
    lib.XMoveWindow.argtypes = [ctypes.c_void_p, ctypes.c_ulong,
                                ctypes.c_int, ctypes.c_int]
    lib.XResizeWindow.restype = ctypes.c_int
    lib.XResizeWindow.argtypes = [ctypes.c_void_p, ctypes.c_ulong,
                                  ctypes.c_uint, ctypes.c_uint]
    lib.XRaiseWindow.restype = ctypes.c_int
    lib.XRaiseWindow.argtypes = [ctypes.c_void_p, ctypes.c_ulong]
    lib.XSync.restype = ctypes.c_int
    lib.XSync.argtypes = [ctypes.c_void_p, ctypes.c_int]
    lib.XGetGeometry.restype = ctypes.c_int
    lib.XGetGeometry.argtypes = [ctypes.c_void_p, ctypes.c_ulong,
                                 ctypes.POINTER(ctypes.c_ulong),
                                 ctypes.POINTER(ctypes.c_int),
                                 ctypes.POINTER(ctypes.c_int),
                                 ctypes.POINTER(ctypes.c_uint),
                                 ctypes.POINTER(ctypes.c_uint),
                                 ctypes.POINTER(ctypes.c_uint),
                                 ctypes.POINTER(ctypes.c_uint)]
    return lib


def geometry(lib, dpy, win):
    r, x, y = ctypes.c_ulong(), ctypes.c_int(), ctypes.c_int()
    w, h, bw, d = (ctypes.c_uint() for _ in range(4))
    lib.XGetGeometry(dpy, win, ctypes.byref(r), ctypes.byref(x), ctypes.byref(y),
                     ctypes.byref(w), ctypes.byref(h), ctypes.byref(bw),
                     ctypes.byref(d))
    return w.value, h.value, x.value, y.value


def send_wm_state(lib, dpy, root, win, action, atom1, atom2):
    """XClientMessage to the root: change a window's EWMH state."""
    ev = ctypes.create_string_buffer(192)
    puts = [(0, ctypes.c_int, ClientMessage),
            (16, ctypes.c_int, 1),                     # send_event = True
            (32, ctypes.c_ulong, win),
            (40, ctypes.c_ulong, lib.XInternAtom(dpy, b"_NET_WM_STATE", 0)),
            (48, ctypes.c_int, 32),                    # format
            (56, ctypes.c_long, action),
            (64, ctypes.c_long, atom1),
            (72, ctypes.c_long, atom2),
            (80, ctypes.c_long, 1)]                    # source: application
    for off, ctype, val in puts:
        c = ctype(val)
        ctypes.memmove(ctypes.byref(ev, off), ctypes.byref(c), ctypes.sizeof(c))
    lib.XSendEvent(dpy, root, False, _SUBSTRUCTURE_MASK, ctypes.byref(ev))


def clients():
    """Window ids: EWMH client list, else the root's children.

    There is no _NET_CLIENT_LIST on a bare X server with no window manager
    (e.g. the nested Xwayland used to render resolutions larger than the
    physical screen), so fall back to walking the tree. The walk uses
    XQueryTree only -- the class-hint call is what corrupted the heap before.
    """
    ids = []
    try:
        out = subprocess.run(["xprop", "-root", "_NET_CLIENT_LIST"],
                             capture_output=True, text=True, timeout=10).stdout
    except Exception:
        out = ""
    for tok in out.replace(",", " ").split():
        if tok.startswith("0x"):
            try:
                ids.append(int(tok, 16))
            except ValueError:
                pass
    if ids:
        return ids
    return tree_children()


def tree_children():
    lib = x11()
    dpy = lib.XOpenDisplay(None)
    if not dpy:
        return []
    lib.XQueryTree.restype = ctypes.c_int
    lib.XQueryTree.argtypes = [ctypes.c_void_p, ctypes.c_ulong,
                               ctypes.POINTER(ctypes.c_ulong),
                               ctypes.POINTER(ctypes.c_ulong),
                               ctypes.POINTER(ctypes.POINTER(ctypes.c_ulong)),
                               ctypes.POINTER(ctypes.c_uint)]
    lib.XDefaultRootWindow.restype = ctypes.c_ulong
    lib.XDefaultRootWindow.argtypes = [ctypes.c_void_p]
    lib.XFree.restype = ctypes.c_int
    lib.XFree.argtypes = [ctypes.c_void_p]

    out = []

    def walk(win, depth):
        if depth > 3:
            return
        r = ctypes.c_ulong()
        p = ctypes.c_ulong()
        kids = ctypes.POINTER(ctypes.c_ulong)()
        n = ctypes.c_uint()
        ok = lib.XQueryTree(dpy, win, ctypes.byref(r), ctypes.byref(p),
                            ctypes.byref(kids), ctypes.byref(n))
        if not ok:
            return
        for i in range(n.value):
            kid = kids[i]
            out.append(kid)
            walk(kid, depth + 1)
        if kids:
            lib.XFree(kids)

    walk(lib.XDefaultRootWindow(dpy), 0)
    return out


def wm_class(win):
    try:
        return subprocess.run(["xprop", "-id", hex(win), "WM_CLASS"],
                              capture_output=True, text=True,
                              timeout=10).stdout.strip()
    except Exception:
        return ""


def is_top_level(win, root):
    """True only for a child of the root window.

    Qt gives OpenGL widgets (the waveforms) their own X windows, and they carry
    the same WM_CLASS. Resizing those to the full screen wrecks the layout --
    the waveform ends up covering the whole window.
    """
    lib = x11()
    dpy = lib.XOpenDisplay(None)
    if not dpy:
        return False
    lib.XQueryTree.restype = ctypes.c_int
    lib.XQueryTree.argtypes = [ctypes.c_void_p, ctypes.c_ulong,
                               ctypes.POINTER(ctypes.c_ulong),
                               ctypes.POINTER(ctypes.c_ulong),
                               ctypes.POINTER(ctypes.POINTER(ctypes.c_ulong)),
                               ctypes.POINTER(ctypes.c_uint)]
    lib.XFree.restype = ctypes.c_int
    lib.XFree.argtypes = [ctypes.c_void_p]
    r, p = ctypes.c_ulong(), ctypes.c_ulong()
    kids = ctypes.POINTER(ctypes.c_ulong)()
    n = ctypes.c_uint()
    if not lib.XQueryTree(dpy, win, ctypes.byref(r), ctypes.byref(p),
                          ctypes.byref(kids), ctypes.byref(n)):
        return False
    if kids:
        lib.XFree(kids)
    return p.value == root


def main():
    size = sys.argv[1] if len(sys.argv) > 1 else "1366x768"
    want = (sys.argv[2] if len(sys.argv) > 2 else "mixxx").lower()
    w, h = (int(v) for v in size.lower().split("x"))

    lib = x11()
    dpy = lib.XOpenDisplay(None)
    if not dpy:
        print("winresize: cannot open $DISPLAY", file=sys.stderr)
        return 2
    root = lib.XDefaultRootWindow(dpy)

    targets = []
    for _ in range(20):
        cand = [win for win in clients() if want in wm_class(win).lower()]
        targets = [win for win in cand if is_top_level(win, root)]
        if not targets and cand:
            targets = cand          # fall back if the parent check is unavailable
        if targets:
            break
        time.sleep(1)
    if not targets:
        print("winresize: no window with WM_CLASS containing %r" % want,
              file=sys.stderr)
        return 1

    horz = lib.XInternAtom(dpy, b"_NET_WM_STATE_MAXIMIZED_HORZ", 0)
    vert = lib.XInternAtom(dpy, b"_NET_WM_STATE_MAXIMIZED_VERT", 0)
    above = lib.XInternAtom(dpy, b"_NET_WM_STATE_ABOVE", 0)
    for win in targets:
        print("winresize: %s before %dx%d+%d+%d"
              % (hex(win), *geometry(lib, dpy, win)))
        send_wm_state(lib, dpy, root, win, _NET_WM_STATE_REMOVE, horz, vert)
        lib.XSync(dpy, 0)
        time.sleep(1)                      # let the compositor un-maximize
        lib.XMoveWindow(dpy, win, 0, 0)
        lib.XResizeWindow(dpy, win, w, h)
        # stack it on top, or a screenshot grabs whatever window happens to be
        # in front (which is how a "resolution sweep" silently captures Konsole)
        send_wm_state(lib, dpy, root, win, _NET_WM_STATE_ADD, above, 0)
        lib.XRaiseWindow(dpy, win)
        lib.XSync(dpy, 0)
    time.sleep(2)
    for win in targets:
        gw, gh, gx, gy = geometry(lib, dpy, win)
        print("winresize: %s after  %dx%d+%d+%d (requested %dx%d)"
              % (hex(win), gw, gh, gx, gy, w, h))
    return 0


if __name__ == "__main__":
    sys.exit(main())
