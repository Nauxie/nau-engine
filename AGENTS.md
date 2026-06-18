# AGENTS.md

## Project

The NAU Engine is a Mac-first Rust/Bevy flight traversal sandbox. The early goal is not to build a full general-purpose engine; it is to create a small, well-instrumented 3D playground where glide, dive, lift, camera feel, and large-world architecture can be tuned deliberately.

## Stack

- Language: Rust
- Engine layer: Bevy
- GPU path: Bevy -> wgpu -> Metal on macOS
- Package manager/build tool: Cargo

## Commands

- `cargo run` - run the sandbox.
- `cargo check` - type-check the project.
- `cargo fmt --check` - verify Rust formatting.
- `cargo clippy --all-targets --all-features -- -D warnings` - lint with warnings treated as errors.

## Working Rules

- Keep the first playable loop small and measurable before adding world scale.
- Prefer Bevy-native systems and components until a real limitation appears.
- Do not introduce raw Metal code unless a specific measured hotspot requires it.
- Keep traversal constants visible and easy to tune.
- Add debug visualization before complex behavior, especially for movement and streaming.
- Avoid adding engine abstractions until at least two concrete systems need the same shape.
