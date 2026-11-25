# Delta Slayers — Gameplay Design

This document records the intended player experience and gameplay rules for
Delta Slayers. It is separate from [DESIGN.md](DESIGN.md), which describes
technical architecture, implementation boundaries, UI conventions, and
authoring workflows.

Use the following status distinctions while developing the design:

- **Decided** — explicitly agreed direction or rule.
- **Proposed** — an idea to discuss; not an implementation requirement.
- **Open** — a question that still needs a decision.

Do not silently promote proposals or assumptions into decided mechanics.

## Core direction

### World and movement

**Decided**

- The game combines a continuous real-time character game with a hex-based
  world layout.
- The broader world is made from procedurally generated, mostly flat hexagonal
  terrain chunks.
- Hexes describe terrain chunks or regions. They are not a turn-based board
  and do not constrain entities to tile centers or discrete movement steps.
- The player and other entities move continuously and can roam freely across
  the terrain.
- Characters and other suitable entities are represented as camera-facing
  billboards.
- The town is currently handmade and Blender-authored. It remains separate
  from the procedural hex terrain for now.

### Current prototype context

**Existing implementation, not necessarily final gameplay design**

- The current playable town is a Blender-authored scene.
- The player can move continuously using ground clicks or camera-relative
  keyboard controls.
- NPCs can move through the town; the current Godo prototype follows the
  player.
- Blender-authored movement blockers define obstacles in the town.

These are useful prototype foundations. They should not be mistaken for a
complete or final gameplay loop.

## Action-driven simulation

**Proposed**

The game could use a real-time simulation that advances only when the player
performs an action. Instead of taking turns, each player action opens a
temporary window during which the whole world progresses.

- While no action is advancing time, the simulation is frozen.
- Starting an action unfreezes the simulation for a duration associated with
  that action.
- All entities advance together during that window: the player, NPCs, enemies,
  projectiles, animations, and physics.
- When the action's time window expires, the simulation freezes again until
  another action advances it.
- The player does not necessarily lose control during this window. Each action
  defines which controls remain available while it is executing.

An action therefore has at least two distinct properties:

- **Duration:** How much simulation time the action advances.
- **Control:** What the player is allowed to do while that time passes.

For example, a heavy spell might advance one second of simulation while
allowing the player to aim or move, depending on the spell. Another spell of
the same duration might keep the player stationary. A quick movement or dodge
could advance a much shorter window.

This makes action duration part of the combat balance. A powerful, slow action
gives enemies more time to reposition or attack; a quick action gives them less
time to respond. Enemies can use ordinary real-time behaviour, but that
behaviour only progresses when the player advances the simulation.

**Open questions**

- Does holding a movement input continuously advance time, or does each
  movement command create a separate time window?
- Are action durations fixed, or can they depend on distance, charge time,
  animation state, or other conditions?
- Can a new action replace or interrupt an action that is still executing?
- Can actions be queued or buffered while the simulation is frozen?
- Does every action advance exactly its full duration, or can the player end
  some action windows early?
- How should passive effects, hazards, and status durations behave while time
  is frozen?

## Core gameplay loop

**Open**

Describe the repeated sequence of activities that gives the player a reason to
explore, interact, fight, collect, and return to safe locations.

## Exploration

**Open**

Questions to resolve:
- What makes locations worth visiting?
- How are points of interest discovered and communicated?
- What role do the hex regions play beyond organizing terrain?
- How does the handmade town connect to the wider procedural world?

## Encounters and combat

**Open**

No combat rules, enemy roles, or encounter structure have been committed to
this document yet. The proposed action-driven simulation above is a possible
foundation for combat, not a finalized combat system.

## Progression and rewards

**Open**

Define what the player gains from exploration and encounters, how those gains
change their capabilities, and what provides longer-term goals.

## Items and equipment

**Open**

The current inventory interface is a presentation prototype. Its displayed
item artwork and equipment positions should not be treated as a finalized
itemization or equipment system.

## World generation

**Decided**

- The broader world uses procedural, mostly flat hexagonal terrain chunks.
- The hex layout does not make the game turn-based or grid-locked.
- The handmade town remains outside this procedural terrain approach for now.

**Open**

Generation rules, region connectivity, terrain variation, resource placement,
points of interest, and how generated terrain meets authored locations.

## Decisions and changes

Record meaningful gameplay decisions here as they are made. When a decision
changes, update the relevant section rather than leaving contradictory rules
in place.
