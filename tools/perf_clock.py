#!/usr/bin/env python3
"""Identify the eval clock; optionally instrument a disposable legacy baseline."""

import hashlib
import json
from pathlib import Path
import subprocess
import sys


def git(*args):
    return subprocess.check_output(["git", *args])


def main():
    if sys.argv[1:] not in ([], ["--prepare-baseline"]):
        raise ValueError("usage: perf_clock.py [--prepare-baseline]")
    prepare = bool(sys.argv[1:])
    path = "src/eval_app_runtime/metrics.rs"
    legacy = b"pub(crate) fn collect_eval_frame_time(time: Res<Time>, mut run: ResMut<EvalRun>) {"
    real = legacy.replace(b"Res<Time>", b"Res<Time<bevy::time::Real>>")
    head = git("show", f"HEAD:{path}")
    current = Path(path).read_bytes()

    def clock(source):
        if source.count(real) == 1 and source.count(legacy) == 0:
            return "real"
        if source.count(legacy) == 1 and source.count(real) == 0:
            return "virtual"
        raise ValueError("unrecognized eval frame-time collector; refusing to guess its clock")

    head_clock = clock(head)
    measured_clock = clock(current)
    expected = head.replace(legacy, real) if head_clock == "virtual" else head
    if prepare:
        changed = git("diff", "--name-only", "HEAD", "--").decode().splitlines()
        if git("ls-files", "--others", "--exclude-standard").strip() or any(
            name != path for name in changed
        ):
            raise ValueError("baseline has unrelated changes; use a clean disposable worktree")
        if current not in (head, expected):
            raise ValueError("baseline collector has changes beyond the exact clock patch")
        if current != expected:
            Path(path).write_bytes(expected)
            current = expected
            measured_clock = "real"
            print("Applied eval wall-clock measurement patch to baseline only", file=sys.stderr)

    print(json.dumps({
        "frame_time_clock": measured_clock,
        "measurement_patch": "eval_real_clock_v1" if current == expected and current != head else None,
        "source_file": path,
        "source_head_sha256": hashlib.sha256(head).hexdigest(),
        "source_measured_sha256": hashlib.sha256(current).hexdigest(),
    }))


if __name__ == "__main__":
    try:
        main()
    except (ValueError, OSError, subprocess.CalledProcessError) as error:
        sys.exit(str(error))
