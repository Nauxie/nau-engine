# Apple Silicon Frame Hitch Evidence

Status as of July 14, 2026: a periodic presentation hitch was reproduced and
localized on one host running macOS Sequoia 15.7.3, then was not observed on
the same hardware and display after upgrading to macOS Tahoe 26.5.2. No NAU
gameplay code fix is claimed, and no local Bevy/wgpu workaround is justified
by this evidence.

## Scope

Validated hardware and display:

- Model: Mac16,6.
- SoC: Apple M4 Max, 16 CPU cores, 64 GiB memory.
- Display: built-in Retina display at 120 Hz for the strict comparison.
- Before: macOS Sequoia 15.7.3, build 24G419, kernel 24.6.0.
- After: macOS Tahoe 26.5.2, build 25F84, kernel 25.5.0.

The result applies to this host and display path. It does not establish that
all Apple Silicon Macs, external displays, refresh rates, or later OS builds
behave the same way.

## Investigation History

### July 11: Isolate the symptom

The visible approximately five-second flick first appeared during NAU release
play. Short gameplay-side experiments removed or varied asset scanning, audio,
gamepad input, vsync behavior, frame pacing, and foreground helpers without
eliminating the recurring cadence.

An independent Bevy crate then reproduced the cadence with only a window,
`Camera2d`, frame-interval measurement, and focus tracking. That removed NAU
traversal, world streaming, assets, camera follow, and gameplay profiling from
the reproducer.

### July 12: Localize the block

Version, window-mode, presentation-mode, and raw-Metal controls showed that
the symptom was below NAU gameplay and was not isolated to a recent Bevy
release. Process samples and targeted `wgpu-hal 27.0.4` instrumentation placed
the long synchronous wait in `CAMetalLayer.nextDrawable`.

### July 14: Repeat after the OS upgrade

The same Mac and built-in display were rerun on Tahoe 26.5.2. Two normal Bevy
captures and the preserved instrumented executable had no post-startup frame
over 50 ms. The instrumented executable also emitted no acquisition record
over its 20 ms logging threshold. A scripted five-minute diagnostic and a
final focused five-minute agent-driven traversal were then clean; a separate
human playtest also reported no recurring five-second hitch.

## Sequoia Reproduction

Strict foreground evidence requires `focused_ratio >= 0.95`. Valid captures
on Bevy 0.15.3, 0.16.1, 0.17.3, 0.18.1, and 0.19.0 reproduced the cadence.
Older 0.12.1, 0.13.2, and 0.14.2 sweep artifacts also showed it, but those
captures do not meet the current focus threshold and are supporting history,
not strict foreground validation. The sweep therefore argues against a recent
Bevy/wgpu introducing release without proving an exact lower version bound.

The strongest minimal capture is:

`target/eval/frame_hitch_probe/bevy-0.18.1-instrumented-no-audio-gilrs.json`

It ran for 30.005 seconds with `focused_ratio = 0.999701`. After startup, the
periodic frame gaps occurred at 6.173, 11.287, 16.401, 21.523, and 26.641
seconds.

The preserved patch wraps only Metal surface acquisition:

`tools/frame_hitch_probe/patches/wgpu-hal-27.0.4-next-drawable-timing.patch`

```text
bevy_render::view::window::prepare_windows
  wgpu::Surface::get_current_texture
    wgpu_core::present::Surface::get_current_texture
      wgpu_hal::metal::Surface::acquire_texture
        -[CAMetalLayer nextDrawable]
          CAMetalLayerPrivateNextDrawableLocked
            semaphore_timedwait_trap
```

The Bevy and instrumentation clocks began 279 ms apart. With that offset, each
focused frame gap aligns with a long acquisition:

|Frame elapsed|Frame gap|Acquire elapsed|`nextDrawable`|
|-|-|-|-|
|6.173 s|77.07 ms|5.894 s|76.07 ms|
|11.287 s|81.65 ms|11.007 s|80.47 ms|
|16.401 s|88.38 ms|16.122 s|87.32 ms|
|21.523 s|93.38 ms|21.244 s|92.03 ms|
|26.641 s|86.43 ms|26.361 s|84.64 ms|

For this capture, the periodic visible gap was dominated by synchronous
`nextDrawable` acquisition. This localizes the measured wait; it does not
identify why macOS delayed drawable delivery.

## Side-Experiment Ledger

These experiments were useful rejection screens on Sequoia. A "not removed"
outcome means the recurring cadence remained in at least one admissible
foreground capture; it does not prove that the tested setting can never affect
frame pacing.

|Question|Experiment|Observed outcome on this host|
|-|-|-|
|NAU-specific work?|Empty Bevy window with no NAU systems|Cadence reproduced|
|Default subsystems?|Default, no audio, no gilrs, no audio plus gilrs, no pipelined rendering|No variant established removal|
|Recent Bevy regression?|Bevy 0.12.1 through 0.19.0 sweep|Cadence spans the sweep; strict focus-valid evidence starts at 0.15.3|
|Presentation mode?|FIFO and immediate|Cadence reproduced|
|Window mode?|Windowed, borderless, and fullscreen|Cadence reproduced|
|Display refresh?|Matched 60 Hz test|Did not establish removal|
|Presentation scheduling?|Conditional/unconditional present and wait-for-scheduled variants|Did not establish removal|
|Presentation callback?|Raw Metal without the per-drawable presented callback|Six over-50 ms gaps remained|
|Core Animation transaction?|`presentsWithTransaction`|Destroyed useful pacing/timestamps; not a viable mitigation|
|Display-link driver?|CVDisplayLink, CADisplayLink, and CAMetalDisplayLink variants|No valid focused clean mitigation established|
|Display-link thread?|Main run loop and dedicated user-interactive render thread|No valid focused clean mitigation established|
|Frame latency?|CAMetalDisplayLink preferred latency 1, 2, and 3|No valid focused clean mitigation established|
|Instrumentation artifact?|Controls with and without presentation-completion instrumentation|Cadence remained|

Signed, focused raw-Metal controls quantified the presentation-path behavior:

|Mode|Focused|max gap|Frames over 50 ms|
|-|-|-|-|
|Windowed FIFO A|100%|98.19 ms|6|
|Windowed FIFO B|100%|88.82 ms|6|
|Fullscreen FIFO|100%|91.69 ms|6|
|Windowed immediate|100%|82.54 ms|6|
|Fullscreen immediate|100%|83.47 ms|6|

The windowed FIFO control without a presented callback remained 100% focused
and recorded six gaps from 70.72 to 87.71 ms in 30.257 seconds.

Background-only or low-focus runs are diagnostic screens only. They must not
be cited as foreground acceptance, even when their frame-time result looks
clean.

## Same-Host Sequoia/Tahoe Result

The Tahoe comparison reused the Mac16,6, built-in 120 Hz display, Bevy 0.18.1
FIFO path, and, for the strongest control, the preserved July 11 instrumented
executable.

For these legacy-schema captures, "post-startup" below means elapsed time at
or after 3.0 seconds, calculated from the event list. The raw instrumented
Tahoe report contains two startup frames over 50 ms at 1.064 and 1.526
seconds; they are not represented as steady-state evidence.

|Capture|Duration|Focused|Post-startup frames >=50 ms|Post-startup frames >=25 ms|
|-|-|-|-|-|
|`bevy-0.18.1-macos-26.5.2-repeat-a.json`|60.006 s|99.986%|0|0|
|`bevy-0.18.1-macos-26.5.2-default.json`|60.000 s|100%|0|0|
|`bevy-0.18.1-macos-26.5.2-instrumented-preserved.json`|60.005 s|100%|0|0|

The preserved instrumented Tahoe run emitted no
`NAU_WGPU_METAL_ACQUIRE` record. The patch logs every `nextDrawable`
acquisition lasting at least 20 ms. The 76-92 ms periodic acquisitions seen
with that instrumentation on Sequoia were therefore absent during this
60-second Tahoe run.

The same-hardware, same-display, preserved-executable comparison is consistent
with a change in the macOS presentation path being the effective resolution
on this host. It does not isolate a specific Apple change, and normal host
activity was not held identical across OS installations.

## Five-Minute Gameplay Evidence

The first Tahoe full-game diagnostic was scripted:

- Artifact:
  `target/eval/play_profile/macos-26.5.2-freeflight-5min-busy-host-diagnostic.json`
- Duration: 300.008 seconds.
- Focus: 100% over 35,415 samples.
- Horizontal travel: 4,019.106 m.
- Steady average/p95/p99/max: `8.453/8.659/16.579/41.734 ms`.
- Steady frames over 50/100 ms: `0/0`.
- Context: post-upgrade media indexing was still documented, so this is a
  diagnostic rather than the final acceptance.

The final focused five-minute agent-driven acceptance was:

- Artifact:
  `target/eval/play_profile/macos-26.5.2-agent-focused-5min.json`
- Date: July 14, 2026.
- Duration: 300.005 seconds after arming.
- Focus: 100% over 35,891 samples and 300.005 focused seconds.
- Horizontal travel: 7,461.534 m.
- Coverage: launch, glide, steering, dive, braking, reset, and sustained world
  streaming changes.
- Steady average/p95/p99/max: `8.351/8.668/8.839/33.568 ms`.
- Steady frames over 50/100 ms: `0/0`.
- Result: every built-in profile check passed.
- Source identity: commit `1799ba0dd77bf72924fdf6bbbcc7d9e586e1b2ac`,
  dirty worktree, fingerprint
  `3e76c594a139302e8e64dc6632796c5a8bbb3b2801e4b95a1cb0481a9b92fade`.
- Host gate: before/after snapshots passed with `cmux|codex` explicitly
  excluded from process CPU evaluation.

The snapshots still recorded excluded `cmux` and Codex activity. This result
must be called a focused acceptance that passed its configured host gate, not
an unqualified idle- or quiet-host capture. A separate human traversal on the
same Tahoe host also reported no recurring five-second hitch.

## Acceptance Decision

For this host, acceptance required:

- A focus-valid minimal probe with no recurring post-startup frame over 50 ms.
- A preserved-instrumentation run with no `nextDrawable` acquisition over
  20 ms.
- Five minutes of focused NAU traversal with no steady frame over 50 ms.
- No recurring approximately five-second hitch during separate human play.
- No gameplay, world, camera, traversal, or visual-quality change made to hide
  the symptom.

Those conditions passed on Tahoe 26.5.2. This closes the host-specific
investigation; it does not declare a gameplay code fix.

## Limitations

- One Mac model, GPU, built-in display, and refresh path were strictly
  validated.
- The A/B crossed an OS upgrade, so it cannot identify the responsible Apple
  component or change.
- Sixty-second minimal captures and a five-minute gameplay capture cannot
  prove indefinite absence.
- The final gameplay artifact came from a dirty source tree; its recorded
  fingerprint, not the commit alone, identifies the tested source state.
- The instrumentation applies exactly to `wgpu-hal 27.0.4`; other versions
  require a reviewed port.
- Local `target/eval` artifacts may not exist in every checkout. The
  measurements and artifact names are preserved here so future runs can be
  compared without overstating provenance.
- Focus and host-load gates reject obvious invalid captures but cannot remove
  all scheduler, WindowServer, thermal, or background-process effects.

## Rerun Procedure

Run the focused minimal probe:

```sh
NAU_HITCH_PROBE_VARIANT=no-audio-gilrs \
NAU_HITCH_PROBE_OUTPUT=target/eval/frame_hitch_probe/rerun.json \
NAU_HITCH_PROBE_DURATION_SECS=60 \
cargo run --release --manifest-path tools/frame_hitch_probe/Cargo.toml
```

Focus the window manually. Require `focused_ratio >= 0.95`, inspect
`steady_frames_over_50ms`, and inspect event timing rather than relying only on
percentiles. Use the disposable instrumentation procedure in
[README.md](README.md) when acquisition timing is needed.

Then repeat the final full-game acceptance:

```sh
NAU_PLAY_PROFILE_DURATION_SECS=300 \
NAU_MANUAL_PROFILE_IGNORE_PROCESS_PATTERN='cmux|codex' \
NAU_MANUAL_PROFILE_HOST_WAIT_SECS=600 \
./tools/manual_play_profile.sh \
  target/eval/play_profile/frame-hitch-focused-5min-rerun.json
```

Keep the game foregrounded and traverse continuously. Preserve the JSON, run
log, before/after host snapshots, OS build, Mac model, display, refresh rate,
source commit/state/fingerprint, and configured process exclusions with any
new conclusion.

If the cadence returns, first reproduce it in a focus-valid minimal capture,
then collect instrumented acquisition timing. Prepare Apple Feedback and wgpu
tracking evidence before considering a local dependency workaround.
