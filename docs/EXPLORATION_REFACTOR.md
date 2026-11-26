# Exploration Refactor — Change Log and Test Plan

**Branch:** `refactor/exploration-architecture`  
**Base:** `main` at `4642290193c99d99b9e378df929eb0a97cc852b3`  
**Status:** Work plan created; implementation and verification are pending.

This document is the running record for the exploration refactor. Update it in the same commit as each meaningful code change. Record what changed, why it changed, affected files, behavior that must remain intact, tests added or run, and anything that still needs manual verification. Do not mark a check complete unless it was actually performed.

## 1. Scope

Refactor the existing exploration implementation in `src/gameplay/exploration*` and its directly related UI integration where necessary.

The goal is to make the code easier to understand and maintain, remove obsolete or duplicated code only when its lack of use is established, and clarify ownership between exploration domain logic, map generation, terrain, travel, and presentation.

This is primarily a structural/refactoring task. Preserve existing gameplay and authoring behavior unless a specific bug is identified and recorded before changing it. Avoid unrelated gameplay, inventory, general UI-library, or visual redesign work.

### Known starting structure

The current implementation is already split into:

- `src/gameplay/exploration.rs`: public/module façade, reflected `ExplorationPortal` authoring component, exploration resources and shared types.
- `src/gameplay/exploration/generation.rs`: tile generation, reachability flood fill, imported tile spawning, biome tinting, boundary blockers, and return beacon geometry.
- `src/gameplay/exploration/terrain.rs`: seeded noise, biome sampling, smooth terrain surface interpolation, and player/NPC ground-height synchronization.
- `src/gameplay/exploration/travel.rs`: portal detection, entering and leaving maps, map root lifecycle, and restoration of the town-side travel anchor.
- `src/gameplay/exploration/hex.rs`: axial coordinates, hex layout, centers/corners, and coordinate conversion.
- `src/gameplay/exploration/tests.rs`: existing exploration tests.
- `src/ui/systems/projection.rs`: exploration-related minimap and setup/debug projections.
- `src/ui/screens/gameplay.rs`: exploration setup and debug scene definitions.

The file split above is a description of the current code, not a requirement to preserve every current boundary. Inspect call sites and responsibilities before moving or deleting anything.

## 2. Architectural constraints

- Follow `AGENTS.md` and `DESIGN.md`; preserve the gameplay/UI MVVM boundary.
- Gameplay owns exploration state, generation, travel, and world entities. UI may read gameplay state and project it, but must not become the owner of gameplay data.
- Keep the reflected `ExplorationPortal` type and its serialized field names compatible with existing Blender/Skein-authored data unless an intentional migration is explicitly justified and documented.
- Preserve the existing `.blend` / exported `assets/worlds/town-map.glb` authoring contract. Do not casually rename reflected types, fields, or enum variants.
- Preserve the imported tile assets and nested Skein-authored components, including authored movement blockers.
- Do not add dependencies without a concrete requirement.
- Keep changes cohesive; do not turn this into a broad rewrite of unrelated systems.
- Any discovered behavior change or bug fix must be called out separately from pure refactoring.

## 3. Behavior that must be preserved

### Portal and travel lifecycle

- Entering an authored exploration portal opens the existing setup flow and uses the portal's sanitized configuration.
- A fixed seed reproduces the same map. An unset/random seed produces a new map seed; the selected seed shown to the player must correspond to the generated map.
- Entering a map captures the town-side player/camera travel state needed for returning.
- Returning through the generated return portal restores the captured town-side state and cleans up the generated map.
- Generated map entities must not leak into the town or survive an invalid lifecycle transition.
- Portal detection must continue to respect the authored trigger volume's world transform, including nested GLB hierarchy where applicable.

### Generation and terrain

- The map remains a bounded regular hex grid. For radius `r`, expected tile count is `1 + 3r(r + 1)`.
- Generation remains deterministic for the same configuration and seed.
- Biome selection and tile variants remain deterministic.
- Reachability is flood-filled from the intended spawn tile using the configured maximum jump height.
- Unreachable tiles retain the gameplay marker used by other systems and the minimap.
- Steep neighboring edges and the outer boundary remain blocked; traversable edges remain traversable.
- Imported tile models continue to load correctly, including nested mesh children and Skein-authored components.
- Biome tinting must affect the intended renderable descendants without destroying authored data.
- Terrain height remains continuous across tile boundaries. Player and NPC ground-height synchronization must remain consistent with the generated terrain.
- The return portal's interaction volume remains independent of the beacon's visual dimensions.

### UI and minimap integration

- Exploration setup controls continue to edit the same parameters, show a correct tile-count/size summary, and synchronize with the selected portal configuration.
- The generated-map debug panel continues to show the actual generated seed and parameters and remains hidden outside the active exploration map.
- Exploration minimap dots continue to represent nearby generated tiles, biome colors, and unreachable tiles correctly.
- Town minimap dots for the Shop and Guild must remain visible when appropriate. A recent regression report said these markers disappeared; explicitly investigate and verify this rather than assuming it is fixed by the refactor.
- Existing overlay navigation, cursor behavior, and gameplay input handling must not regress.

## 4. Refactor approach

Work through these steps, updating the progress log below as each is completed.

1. **Inventory responsibilities and call sites.** Trace every exploration type, function, resource, system, and event/request from registration through use. Identify stale code with evidence (no references, obsolete compatibility path, duplicate source of truth, or superseded implementation).
2. **Define ownership and module interfaces.** Keep domain/configuration/state separate from generation algorithms, terrain sampling, travel lifecycle, and UI projection. Make module visibility intentional; avoid widening APIs merely to make refactoring convenient.
3. **Refactor in small steps.** Move or reshape one responsibility at a time. Preserve behavior and update imports/tests together. Avoid mixing cosmetic formatting with semantic changes.
4. **Address confirmed defects only.** If the inspection exposes a real bug (including the Shop/Guild minimap regression), record the reproduction and fix separately from structural cleanup.
5. **Update architecture documentation.** If the final module boundaries or authoring contracts change, update `DESIGN.md` and any relevant comments alongside the implementation.
6. **Verify and review.** Run the automated checks, inspect the final diff for unrelated changes and accidental API/asset changes, and complete the manual test checklist below.

## 5. Manual test checklist

Run these in a built game after the refactor. Record the build/commit used and mark each item only after checking it.

### Town and portal

- [ ] Town scene loads from `assets/worlds/town-map.glb` without missing assets or Skein/reflection warnings.
- [ ] Shop and Guild markers appear at their expected positions on the town minimap.
- [ ] Shop/Guild proximity and interaction behavior still work.
- [ ] An authored exploration portal is detected at the correct world-space location, including when nested in the GLB hierarchy.
- [ ] The portal setup modal opens and closes correctly; its controls do not leak input to gameplay.
- [ ] Setup values are clamped/sanitized as intended and the displayed tile count matches `1 + 3r(r + 1)`.

### Generated map

- [ ] Entering exploration creates exactly the expected number of hex tiles for the selected radius.
- [ ] Re-entering with the same explicit seed produces the same tile layout, biomes, terrain, and tile variants.
- [ ] Random seed mode generates a valid map and displays the actual generated seed.
- [ ] Imported tile GLBs render with biome tinting; nested meshes are tinted too.
- [ ] Skein-authored components on imported tile content remain present and functional.
- [ ] The center/spawn area and return beacon are correctly positioned.
- [ ] Reachable/unreachable tiles match the configured jump-height constraint.
- [ ] Boundary and steep-edge blockers prevent crossing; valid neighboring tiles remain traversable.
- [ ] Player and NPC remain grounded while moving across tile centers and seams; no visible vertical snapping at boundaries.
- [ ] Exploration minimap follows the player, uses the correct orientation, and displays biome/unreachable markers.

### Return and lifecycle

- [ ] Returning through the beacon restores the town-side player position and camera state.
- [ ] The generated map is removed after returning; no generated tiles/blockers remain in town.
- [ ] Re-entering exploration after returning works.
- [ ] Leaving gameplay or changing app state during/after exploration does not leave stale exploration entities or resources.
- [ ] Town interactions and minimap markers still work after returning.

### UI and input regression checks

- [ ] Exploration setup/debug UI visibility follows the active state and overlay stack.
- [ ] Closing setup/debug UI leaves no stale modal or input capture.
- [ ] Minimap dots stay within the minimap bounds and hide when the relevant map/zone is inactive.
- [ ] Inventory, pause, and other overlays remain functional; inventory pause behavior is covered by `docs/INVENTORY_IMPLEMENTATION.md`.

## 6. Automated verification

Run from the repository root:

```sh
cargo fmt --check
cargo check
cargo test
```

Also run focused tests for exploration generation, reachability, deterministic sampling, hex-coordinate conversion, terrain continuity, travel lifecycle, and minimap projection wherever the implementation exposes suitable test seams.

Record exact command results below. If a command cannot be run, state why; do not imply it passed.

## 7. Change log

### Initial setup

- Created branch `refactor/exploration-architecture` from `main` at `4642290193c99d99b9e378df929eb0a97cc852b3`.
- Added this document before implementation work.
- Inspected the current exploration module layout and project architecture instructions.
- No gameplay or UI implementation files have been changed yet.
- No build, test, or runtime verification has been performed in this work session.

### Subsequent changes

_Add an entry for every implementation commit. Include commit SHA, purpose, files changed, behavioral impact, tests added/run, and remaining manual checks._

| Commit | Change | Files | Verification |
| --- | --- | --- | --- |
| Pending | No implementation changes yet | — | Not run |

## 8. Verification results

| Check | Result | Notes |
| --- | --- | --- |
| `cargo fmt --check` | Pending | |
| `cargo check` | Pending | |
| `cargo test` | Pending | |
| Runtime/manual checklist | Pending | Requires running the game locally |

## 9. Known risks / compatibility notes

- `ExplorationPortal` is a reflected Skein authoring component. Treat its name and serialized fields as an external data contract; inspect existing exports before changing it.
- `vegetation_density` is documented in the current implementation as a legacy compatibility field. Do not remove it merely because generation no longer uses it; old GLB exports may still serialize it.
- `max_jump_height` and other portal parameters affect both generation and collision/reachability behavior. Keep their sanitization and interpretation consistent.
- Imported tile assets contain nested entities and may include authored components. Avoid replacing the hierarchy with a flattened or manually reconstructed version.
- The current Shop/Guild minimap disappearance is a reported regression, not a confirmed root cause. Reproduce it and trace the town-zone query/projection before changing the implementation.
