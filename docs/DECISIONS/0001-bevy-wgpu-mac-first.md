# ADR 0001: Bevy And wgpu, Mac-First

Status: accepted

Date: 2026-06-18

## Context

The project targets a Mac-first development environment on Apple Silicon. The long-term vision includes large outdoor traversal spaces, high-fidelity islands, humanoid characters, wind, atmosphere, and physics-driven immersion.

Raw Metal would match macOS directly, but it would also force the project to learn and maintain a low-level Apple-specific renderer before the traversal game has proven its core feel.

## Decision

Use Rust and Bevy as the main game stack.

Use Bevy's renderer through wgpu. On macOS, wgpu uses Metal as the backend. The project stays Mac-first while avoiding an Apple-only renderer boundary at the start.

## Consequences

Positive:

- The project can iterate on gameplay faster than raw Metal would allow.
- Engine internals are open source and inspectable.
- Rust code and crate dependencies keep the stack transparent.
- wgpu keeps the renderer portable while still using Metal on macOS.
- Bevy gives ECS, rendering, input, windowing, assets, cameras, scenes, and app scheduling.

Negative:

- Bevy is younger than Unity, Unreal, and Godot.
- APIs may change across Bevy releases.
- Some engine/editor workflows will need to be built or adapted.
- Extremely custom rendering may eventually require lower-level wgpu work.

## Rules

- Prefer Bevy-native systems until a measured limitation appears.
- Prefer wgpu-level customization before any raw Metal-specific code.
- Keep raw Metal out of the codebase unless profiling identifies a narrow hotspot and the renderer boundary is explicit.
- Keep movement/camera/animation behavior testable outside a native window.

## Alternatives Considered

Raw Metal:

- Rejected for now. Too much low-level renderer work before traversal feel is proven.

Unity or Unreal:

- Not chosen for this project because the desired stack should be open-source, transparent, and Rust-native.

Godot:

- Plausible for open-source game development, but less aligned with the desire to build and understand engine systems in Rust.

Direct wgpu without Bevy:

- Plausible later, but slower for the early game loop because we would need to build app, input, scene, camera, and asset infrastructure ourselves.
