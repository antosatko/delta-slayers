# Delta Slayers

## About this project

Delta Slayers is a **non-commercial** experiment. Its only purpose is to try out
**agentic coding** — building a game by collaborating with an AI coding agent.
There is no long-term vision or roadmap; features are implemented on the fly as
ideas come up.

All assets currently in the repository are **placeholders**. They will be
replaced with original artwork over time.

## Project overview

A small Bevy 0.20 (Rust) game prototype combining 3D movement with 2D
billboard entities.

Current features:

- Main menu with Quick Load, New Game, Load Save, Credits, and Exit
- Loading screen while the Blender-authored world is instantiated
- Overhead follow camera with click-to-move and camera-relative WASD / arrow movement
- Shift sprint and a moving destination-path indicator
- 2D billboard entities that face the camera
- Oriented Blender-authored trigger boxes for Shop and Guild interactions
- Inventory overlay with equipment, item artwork, resources, and fast-travel choices
- Player-oriented minimap
- Save/load of player position, camera orientation, and visited zones

## Source layout

The source is grouped by responsibility rather than by individual file type:

- `app/` owns application startup and top-level application states.
- `gameplay/` owns gameplay data, world setup, simulation systems, and
  persistence. Its modules are split by domain (player, zones, inventory).
- `ui/` owns overlay navigation, BSN screens, input handling, and projection
  systems. Screen definitions live in `ui/screens/`; systems that synchronize
  state into presentation components live in `ui/systems/`.
- `crates/delta-slayers-ui/` owns the reusable Carbon palette, BSN components,
  and presentation-only hover/selection behavior. It does not own game state.

Gameplay data is authoritative. The UI reads it and sends requests; it does not
own gameplay rules or construct gameplay entities.

## Blender level authoring

The project uses [Bevy Skein](https://bevy-skein.netlify.app/) to author
gameplay components in Blender and export them through glTF. The main level is
loaded from `assets/worlds/town-map.glb` during the Loading state. This local
GLB is required; the game no longer creates a procedural test world.

See [assets/worlds/README.md](assets/worlds/README.md) for the component
reference, spawn-point rules, and export workflow.

## Reusable UI library

The Carbon UI components live in the workspace crate `crates/delta-slayers-ui`.
It provides a shared theme and composable BSN scenes for panels, headings,
buttons, badges, item tiles, resource bars, and delayed reference popovers.

Run the standalone component showcase:

```sh
cargo run -p delta-slayers-ui --example carbon_showcase
```

See [the crate README](crates/delta-slayers-ui/README.md) for its public API
and composition examples.

## Tech stack

- [Rust](https://www.rust-lang.org/)
- [Bevy 0.20](https://bevy.org/)

## Status

Experimental. The project is developed on the fly with no stable roadmap. Expect
frequent breaking changes. **Project abandonment is very likely.**

## Controls

- `W` `A` `S` `D` or arrow keys — move relative to the camera
- Left click — set a movement destination
- Hold left click — continuously retarget after a short delay
- `Shift` — sprint
- Mouse wheel — adjust camera distance and pitch
- Right click — open inventory or close the topmost overlay
- `E` — interact with a nearby zone
- `P` — pause
- `Esc` — open the in-game menu, or close the topmost overlay
- `Backspace` — close the topmost overlay, or save and return to the main menu

## Running

```sh
cargo run
```

## Contributing

Contributions are welcome, but due to the experimental nature of this project
they are **heavily discouraged**. The codebase changes frequently and without
a stable direction, so external PRs may not be accepted or may be overwritten.

If you still want to contribute, please open an issue first to discuss.

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.
