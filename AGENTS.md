# Agent instructions

Read DESIGN.md before making architectural changes.

## Project constraints

- Rust 2024, Bevy 0.20.1.
- All game UI must be authored with Bevy Scene Notation (BSN).
- Use current BSN syntax: @scene() for scene composition and -- between
  sibling entities in Children [ ... ] and bsn_list! { ... }.
- Do not generate UI trees with Commands::spawn, with_children,
  ChildSpawnerCommands, children!, or manual entity construction.
- World/gameplay entities may still be spawned imperatively from gameplay setup.

## Source structure

- Keep `main.rs` limited to declaring root modules and starting the app.
- Put application setup and top-level states in `app/`.
- Put authoritative gameplay models, simulation, world setup, and persistence
  in `gameplay/`.
- Put UI navigation, BSN scenes, input, and presentation projection in `ui/`.
- Keep screen definitions in `ui/screens/` and state-to-view systems in
  `ui/systems/`.
- Do not move gameplay data into UI modules to make it easier for a screen to
  access. UI should depend on gameplay models, not the reverse.

## Blender / Skein authoring

- Use reflected components as a stable authoring API for Skein.
- When changing reflected type names, field names, or enum variants, document
  migration implications for existing Blender files and exported glTF.
- Keep level-authored markers in gameplay modules, not UI modules.
- glTF world entities can be nested; systems that consume authored points or
  zones should query GlobalTransform when world-space position is required.
- Keep the procedural world as a fallback until the Blender export is present.
- TriggerBox is authored on a default Blender cube. Its local bounds are
  [-1, 1] on each axis; use the object's location, rotation, and unapplied
  scale to define the world-space volume.

## Architecture

- Follow the MVVM separation described in DESIGN.md.
- Keep gameplay/domain state out of UI components.
- Keep display-only derived values in a ViewModel resource.
- UiStack is the single source of truth for in-game overlay navigation. Never
  add a second boolean for a state already represented by a UI layer.
- UI observers should mutate state or enqueue requests, not edit Visibility,
  CursorOptions, layout, or text components directly.
- Projection systems own synchronization from state/ViewModel to UI.
- UI requests gameplay actions through request resources such as
  FastTravelRequest; gameplay systems carry them out.
- Preserve explicit system ordering using UiSet::Input and UiSet::Projection when introducing UI-related systems.

## Code quality

- Prefer small, cohesive systems and scene functions over a monolithic module.
- Avoid hidden ordering assumptions and duplicated sources of truth.
- Do not swallow new errors with unwrap() unless the invariant is explicit
  and local.
- Do not add dependencies without a concrete requirement.
- Keep comments factual and useful.
- Update DESIGN.md when architecture or conventions change.

## Verification

Run:

    cargo fmt --check
    cargo check
    cargo test

Report commands that could not be run and distinguish verified results from
reasoned expectations.
