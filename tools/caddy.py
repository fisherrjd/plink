#!/usr/bin/env python3
"""CLI driver for the in-game Caddy bridge (game must run with PLINK_CADDY=1).

Usage:
  caddy.py state                        print current game state
  caddy.py putt <target_x> <target_y> <power>   stroke aimed at a point (power 0..1)
  caddy.py shot <dx> <dy> <power>       stroke with a raw direction vector
  caddy.py screenshot <abs_path>        save a viewport screenshot
"""
import json
import os
import sys
import time

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CADDY = os.path.join(ROOT, ".caddy")
STATE = os.path.join(CADDY, "state.json")
COMMAND = os.path.join(CADDY, "command.json")


def read_state(timeout=5.0):
    deadline = time.time() + timeout
    while time.time() < deadline:
        try:
            with open(STATE) as f:
                return json.load(f)
        except (OSError, json.JSONDecodeError):
            time.sleep(0.05)
    sys.exit("no readable state.json — is the game running with PLINK_CADDY=1?")


def write_command(cmd):
    tmp = COMMAND + ".tmp"
    with open(tmp, "w") as f:
        json.dump(cmd, f)
    os.rename(tmp, COMMAND)


def fmt(s):
    ball = s.get("ball")
    hole_ix = min(s["hole"], s["total_holes"]) - 1
    lines = [
        f"hole {s['hole']}/{s['total_holes']}  par {s['pars'][hole_ix]}  "
        f"strokes {s['strokes_this_hole']}  finished={s['finished']}"
    ]
    if ball:
        lines.append(
            f"ball ({ball['x']:.0f},{ball['y']:.0f}) v=({ball['vx']:.0f},{ball['vy']:.0f}) "
            f"stopped={ball['stopped']} sunk={ball['sunk']}"
        )
        cup = s["cup"]
        lines.append(f"cup  ({cup['x']:.0f},{cup['y']:.0f})")
    lines.append(f"scorecard {s['strokes']} (pars {s['pars']})")
    return "\n".join(lines)


def wait_outcome(before):
    """After sending a putt: wait for the ball to move, then to settle."""
    hole0 = before["hole"]
    strokes0 = before["strokes_this_hole"]
    # phase 1: stroke registered (strokes bump) or command evidently rejected
    deadline = time.time() + 4.0
    taken = False
    while time.time() < deadline:
        s = read_state()
        if s["hole"] != hole0 or s["finished"]:
            taken = True
            break
        if s["strokes_this_hole"] > strokes0:
            taken = True
            break
        time.sleep(0.1)
    if not taken:
        print("NO STROKE TAKEN (ball still moving, or already sunk?)")
        print(fmt(read_state()))
        return
    # phase 2: ball settles / sinks / hole advances
    deadline = time.time() + 30.0
    last = None
    while time.time() < deadline:
        s = read_state()
        last = s
        if s["finished"]:
            print(f"ROUND FINISHED! final scorecard: {s['strokes']}")
            print(fmt(s))
            return
        if s["hole"] != hole0:
            print(f"SUNK hole {hole0} in {s['strokes'][hole0 - 1]} strokes "
                  f"(par {s['pars'][hole0 - 1]}) -> now on hole {s['hole']}")
            print(fmt(s))
            return
        ball = s.get("ball")
        if ball and ball["sunk"]:
            print(f"SUNK hole {hole0} in {s['strokes'][hole0 - 1]} strokes "
                  f"(par {s['pars'][hole0 - 1]}) — advancing…")
            time.sleep(1.2)
            print(fmt(read_state()))
            return
        if ball and ball["stopped"] and s["strokes_this_hole"] > strokes0:
            print("ball stopped")
            print(fmt(s))
            return
        time.sleep(0.1)
    print("TIMEOUT waiting for outcome")
    if last:
        print(fmt(last))


def main():
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    cmd = sys.argv[1]

    if cmd == "state":
        print(fmt(read_state()))
    elif cmd in ("putt", "chip"):
        tx, ty, power = map(float, sys.argv[2:5])
        s = read_state()
        ball = s.get("ball")
        if not ball:
            sys.exit("no ball in state (hole transition?) — retry in a second")
        dx, dy = tx - ball["x"], ty - ball["y"]
        write_command({cmd: {"dx": dx, "dy": dy, "power": power}})
        wait_outcome(s)
    elif cmd == "shot":
        dx, dy, power = map(float, sys.argv[2:5])
        s = read_state()
        write_command({"putt": {"dx": dx, "dy": dy, "power": power}})
        wait_outcome(s)
    elif cmd == "screenshot":
        path = os.path.abspath(sys.argv[2])
        write_command({"screenshot": path})
        deadline = time.time() + 5.0
        while time.time() < deadline:
            if os.path.exists(path):
                print(f"saved {path}")
                return
            time.sleep(0.1)
        sys.exit("screenshot did not appear")
    else:
        sys.exit(__doc__)


if __name__ == "__main__":
    main()
