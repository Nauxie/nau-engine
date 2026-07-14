# NAU Bevy Frame Hitch Probe

This independent crate isolates frame presentation from NAU gameplay. It opens
a 1280x720 Bevy window with an empty 2D camera, records main-loop frame
intervals and focus state, and writes a JSON report.

The validated result is deliberately narrow: on one Mac16,6 with an Apple M4
Max and its built-in 120 Hz Retina display, periodic 76-92 ms
`CAMetalLayer.nextDrawable` stalls reproduced on macOS Sequoia 15.7.3
(24G419) and were absent after that same host was upgraded to macOS Tahoe
26.5.2 (25F84). This is evidence of an OS/display-presentation-path difference
on that host. It is not a gameplay code fix, a universal macOS result, or proof
of a specific Apple change.

See [EVIDENCE.md](EVIDENCE.md) for the exact A/B result, side-experiment
history, final five-minute acceptance, limitations, and artifact names.

## Run the Probe

From the repository root:

```sh
NAU_HITCH_PROBE_VARIANT=no-audio-gilrs \
NAU_HITCH_PROBE_OUTPUT=target/eval/frame_hitch_probe/rerun.json \
NAU_HITCH_PROBE_DURATION_SECS=60 \
cargo run --release --manifest-path tools/frame_hitch_probe/Cargo.toml
```

Focus the probe window manually immediately after launch. The probe never
requests focus or raises its window.

`NAU_HITCH_PROBE_VARIANT` accepts:

- `default`
- `no-audio`
- `no-gilrs`
- `no-audio-gilrs`
- `no-pipelined-rendering`

The selected variant is recorded in the report. Invalid duration values fail
instead of silently using a default.

Treat a run as foreground evidence only when `focused_ratio >= 0.95`. Schema 2
reports exclude the first three seconds from `steady_*` statistics and mark
each hitch event with `steady`. Low-focus reports are still written for
diagnosis, but the process exits nonzero.

## Instrument `nextDrawable`

The preserved patch targets exactly `wgpu-hal 27.0.4`:

`patches/wgpu-hal-27.0.4-next-drawable-timing.patch`

Apply it in a disposable workspace so the probe's checked-in manifest remains
unchanged:

```sh
mkdir -p target/eval/frame_hitch_probe
work_dir="$(mktemp -d)"
cp -R tools/frame_hitch_probe "${work_dir}/probe"
wgpu_hal_src="$(
  find "${CARGO_HOME:-$HOME/.cargo}/registry/src" \
    -type d -name 'wgpu-hal-27.0.4' -print -quit
)"
test -n "${wgpu_hal_src}"
cp -R "${wgpu_hal_src}" "${work_dir}/wgpu-hal-27.0.4"
patch -d "${work_dir}/wgpu-hal-27.0.4" -p1 \
  < tools/frame_hitch_probe/patches/wgpu-hal-27.0.4-next-drawable-timing.patch
printf '\n[patch.crates-io]\nwgpu-hal = { path = "%s" }\n' \
  "${work_dir}/wgpu-hal-27.0.4" >> "${work_dir}/probe/Cargo.toml"

NAU_HITCH_PROBE_VARIANT=no-audio-gilrs \
NAU_HITCH_PROBE_OUTPUT=target/eval/frame_hitch_probe/instrumented-rerun.json \
NAU_HITCH_PROBE_DURATION_SECS=60 \
cargo run --release --manifest-path "${work_dir}/probe/Cargo.toml" \
  2> target/eval/frame_hitch_probe/instrumented-rerun.log
```

The patch emits `NAU_WGPU_METAL_ACQUIRE` when
`CAMetalLayer.nextDrawable` takes at least 20 ms, including elapsed time and
window visibility/occlusion state. Compare those records with focused steady
hitch events in the JSON report.

## Repeat Five-Minute Acceptance

Use the full game only after the minimal probe is focused and clean:

```sh
NAU_PLAY_PROFILE_DURATION_SECS=300 \
NAU_MANUAL_PROFILE_IGNORE_PROCESS_PATTERN='cmux|codex' \
NAU_MANUAL_PROFILE_HOST_WAIT_SECS=600 \
./tools/manual_play_profile.sh \
  target/eval/play_profile/frame-hitch-focused-5min-rerun.json
```

Keep the game foregrounded and traverse continuously. Measurement arms after
focused horizontal movement. Acceptance requires:

- Both host snapshots pass the configured process/CPU gate.
- `window_focus.focused_ratio >= 0.95`.
- The profile's built-in checks pass.
- Raw steady hitch events show no recurring approximately five-second cadence
  over 50 ms.

The `cmux|codex` exclusion reproduces the final July 14, 2026 acceptance
configuration; it must be recorded with the result rather than described as an
unqualified quiet-host run.
