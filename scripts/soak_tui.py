#!/usr/bin/env python3
"""Memory soak for the TUI: run it in a pty, drive it, sample RSS (MB).

usage: soak_tui.py [secs] [tab-keys] [binary]
  tab-keys: keys sent after opening device 1 (default: ESC s = Screen tab)
Reports RSS of the process tree every 5s and the growth slope.
"""
import os, pty, sys, time, subprocess, select, struct, fcntl, termios

secs = int(sys.argv[1]) if len(sys.argv) > 1 else 60
keys = (sys.argv[2] if len(sys.argv) > 2 else "\x1bs").encode()
binary = sys.argv[3] if len(sys.argv) > 3 else "target/release/droidkraft-tui"

pid, fd = pty.fork()
if pid == 0:
    os.environ["TERM"] = "xterm-256color"
    os.execv(binary, [binary])
fcntl.ioctl(fd, termios.TIOCSWINSZ, struct.pack("HHHH", 50, 160, 0, 0))

def tree_rss(root):
    out = subprocess.run(["ps", "-axo", "pid=,ppid=,rss="], capture_output=True, text=True).stdout
    rows = [tuple(map(int, l.split())) for l in out.splitlines() if l.strip()]
    kids, total = {root}, 0
    changed = True
    while changed:
        changed = False
        for p, pp, _ in rows:
            if pp in kids and p not in kids:
                kids.add(p); changed = True
    for p, _, rss in rows:
        if p in kids:
            total += rss
    return total / 1024

seen = b""
def drain(t=0.2):
    global seen
    end = time.time() + t
    while time.time() < end:
        r, _, _ = select.select([fd], [], [], 0.05)
        if r:
            try: seen = (seen + os.read(fd, 1 << 20))[-200000:]
            except OSError: return

time.sleep(2); drain()
os.write(fd, b"1"); drain(1)
os.write(fd, keys); drain(1)
samples = []
t0 = time.time()
while time.time() - t0 < secs:
    drain(5)
    rss = tree_rss(pid)
    samples.append((time.time() - t0, rss))
    print(f"{samples[-1][0]:5.0f}s  {rss:8.1f} MB", flush=True)
print("fps seen:", b"fps" in seen, "| Screen seen:", b"Screen" in seen)
import re as _re
print(_re.sub(rb"\x1b\[[0-9;?]*[A-Za-z]", b" ", seen)[-600:].decode("utf8","replace"))
os.write(fd, b"q")
time.sleep(1)
try: os.kill(pid, 9)
except OSError: pass
(a, b) = samples[len(samples)//4], samples[-1]
print(f"slope: {(b[1]-a[1])/max(b[0]-a[0],1)*60:+.1f} MB/min")
