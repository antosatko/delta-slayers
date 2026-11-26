# Inventory — Initial Implementation

**Branch:** `refactor/exploration-architecture`  
**Status:** Initial implementation; automated and runtime verification pending.

## Gameplay rules

- Resources and items remain separate inventory categories.
- Resources are quantity stacks. Items are individual instances with stable IDs.
- Both categories contribute to one shared capacity budget.
- Capacity is displayed in fictional capacity units, not kilograms or physical weight.
- The initial maximum is 100.00 capacity units.
- Capacity is stored internally as integer hundredths to avoid floating-point rounding in limit checks.
- The inventory may exceed capacity. Nothing is discarded automatically.
- Manual additions and inventory management are allowed while overloaded.
- Automatic-acquisition APIs reject a pickup if the resulting inventory would exceed capacity.
- Overcapacity blocks player movement and portal/fast-travel use. Combat does not exist yet; the model exposes the same eligibility check for future combat entry points.
- Opening the inventory pauses gameplay simulation, so NPC/world simulation waits while the player reorganizes their belongings.

## Initial content and tuning

The existing placeholder screen showed Gold, Wood, Stone and five example item artworks (Evil pup, Maurice, Salad, Scooby, Theo). Those examples are now represented by actual model data rather than static UI tiles.

Initial capacity costs are deliberately provisional:

| Entry | Cost |
| --- | ---: |
| Gold | 0.01 per coin |
| Wood | 1.00 per unit |
| Stone | 1.50 per unit |
| Evil pup | 4.50 |
| Maurice | 3.00 |
| Salad | 1.00 |
| Scooby | 4.50 |
| Theo | 3.00 |

The five starter items therefore use 16.00 capacity. These values are balancing placeholders and should be tuned once real loot sources and item definitions exist.

## Architecture

- `src/gameplay/inventory.rs` owns the authoritative model, capacity calculation, manual inventory operations, automatic-pickup acceptance checks, and queued inventory actions.
- `src/ui/inventory.rs` owns the presentation ViewModel and projects capacity, overload state, resources, and item entries.
- `src/ui/screens/gameplay.rs` authors the inventory screen with BSN.
- UI observers enqueue `InventoryActionRequest`; gameplay applies the requested action before UI projection.
- Movement checks the model before accepting click destinations and before applying keyboard/click movement.
- Fast travel and exploration portal processing both reject requests while overloaded.
- Opening the inventory is now a simulation-pausing overlay. The player can still use inventory controls because UI input remains active.

## Current limitations

- The project does not yet contain world resource nodes, item pickup entities, automatic pickup systems, combat, crafting, or camp-building mechanics. The inventory model exposes operations for future systems; it does not invent these gameplay systems.
- Equipment slots remain presentation placeholders. There is not yet an equipment taxonomy or stat/effect system, so no item is equipable in this initial pass.
- The visible test controls are temporary scaffolding to exercise manual additions and capacity overflow before real loot sources are implemented.
- Item and resource capacity costs are provisional and currently defined alongside their types. A data-driven item-definition registry may replace this once the item catalogue grows.

## Test checklist

### Inventory contents and capacity

- [ ] Start a new game and open inventory with right-click. The five starter items appear in the list.
- [ ] Capacity starts at 16.00 / 100.00 with the current starter loadout.
- [ ] Add 10 wood; capacity increases by 10.00.
- [ ] Add 10 stone; capacity increases by 15.00.
- [ ] Add 100 gold; capacity increases by 1.00.
- [ ] Add an item; the item list and capacity update.
- [ ] Automatic pickup test buttons add loot when it fits.
- [ ] Automatic pickup test buttons leave the inventory unchanged when the pickup would exceed capacity.
- [ ] The Overfill capacity test button creates an overflow in one action.
- [ ] Reorder items; item order changes but capacity does not.
- [ ] Leave the last item; item list and capacity decrease.
- [ ] Add resources until capacity is exceeded; overload amount is displayed.
- [ ] Remove resources or items until under capacity; overload state clears immediately.

### Overcapacity restrictions

- [ ] While overloaded, keyboard movement does nothing.
- [ ] While overloaded, click-to-move does not create or follow a destination.
- [ ] While overloaded, fast travel does not move the player.
- [ ] While overloaded, entering or returning through an exploration portal is rejected.
- [ ] While overloaded, inventory controls still work and can bring capacity back under the limit.
- [ ] Once under capacity, movement and portal usage work again without closing/reopening inventory.
- [ ] Opening inventory pauses NPC movement and other gameplay simulation; closing it resumes simulation.

### Persistence and compatibility

- [ ] Exit to the main menu using Backspace and quick-load. Inventory resources, item order, and capacity are restored.
- [ ] A legacy versionless save still loads and receives the starter inventory.
- [ ] A version 2 save still loads with its previous camera pitch and receives the starter inventory.
- [ ] New Game resets inventory to the starter loadout.

### Existing behavior regression

- [ ] Shop/Guild panels, minimap, exploration setup, and pause/menu overlays still work.
- [ ] Inventory fits the current window size and remains scrollable on smaller windows.

## Automated checks

Run from the repository root:

```sh
cargo fmt --check
cargo check
cargo test
```

Record actual results in the change log. Runtime checklist items require a local game session and must not be marked complete from source inspection alone.
