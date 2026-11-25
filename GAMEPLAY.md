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

**Decided**

The game uses a real-time simulation that advances only while the player is
performing an action. There are no traditional turns: the player's actions
determine when and for how long the world progresses.

- When the player is not performing an action, the simulation is frozen.
- All entities advance together while the simulation is active: the player,
  NPCs, enemies, projectiles, animations, and physics.
- Continuous actions, especially movement, keep the simulation running for as
  long as the player maintains the input. Releasing the input stops the action
  and freezes the simulation; movement is not divided into artificial time
  windows and the player is not forced to pause at intervals.
- Other actions can advance the simulation for a defined duration. A heavy
  spell, for example, might advance one second of world time.
- The player may retain some control during a timed action. The action defines
  which controls remain available while it executes.

An action therefore has at least two distinct properties:

- **Duration:** Whether it runs continuously while maintained or advances a
  defined amount of simulation time.
- **Control:** What the player is allowed to do while that time passes.

For example, a heavy spell might advance one second of simulation while
allowing the player to aim or move, depending on the spell. Another spell of
the same duration might keep the player stationary.

This makes action duration part of the combat balance. A powerful, slow action
gives enemies more time to reposition or attack; a quick action gives them less
time to respond. Enemies can use ordinary real-time behaviour, but that
behaviour only progresses when the player advances the simulation.

**Open questions**

- Are timed action durations fixed, or can they depend on charge time,
  animation state, or other conditions?
- Can a new action replace or interrupt an action that is still executing?
- Can actions be queued or buffered while the simulation is frozen?
- Can the player end some timed action windows early?
- How should passive effects, hazards, and status durations behave while time
  is frozen?
- How should simultaneous or overlapping inputs combine, such as moving while
  casting a spell?

## Core gameplay loop

**Open**

Describe the repeated sequence of activities that gives the player a reason to
explore, interact, fight, collect, and return to safe locations.

**Proposed direction**

- Mainline quests may impose hard deadlines.
- The world should offer many other activities and opportunities competing for
  the player's available time. Choosing what to pursue, postpone, or abandon
  should matter.
- Time pressure should create meaningful prioritization rather than simply
  forcing the player to rush through every activity.

## Exploration

**Proposed direction**

- Exploration and combat happen in the same world and on the same map. Engaging
  an enemy does not load a separate combat arena or switch to a different
  encounter map.
- The player explores procedurally generated maps where enemies and other
  entities already inhabit the environment. Encounters can arise during
  exploration rather than only at predefined combat locations.
- The same action-driven simulation applies throughout the map. When the
  player acts, the rest of the world advances alongside them; when the player
  stops and the simulation freezes, the other entities stop as well.
- Maps should feel like places with their own ecosystems and activity, worth
  exploring beyond simply moving between objectives.

**Open questions**

- What makes locations worth visiting, beyond enemies and rewards?
- How are points of interest discovered and communicated?
- What role do the hex regions play beyond organizing terrain?
- How does the handmade town connect to the wider procedural world?
- Which entities and environmental processes make up a map's ecosystem, and
  how do they interact with one another?
- How does large-scale time passage work alongside the frozen/active
  simulation? For example, what advances quest deadlines, schedules, or
  opportunities when the player is not actively moving through the world?
- Should the player be able to deliberately spend time through activities such
  as resting, travelling, or interacting, and what consequences should those
  choices have?

## Time constraints

**Proposed direction**

Action-driven simulation and broader gameplay time constraints are related,
but they do not necessarily need identical rules.

- The action-driven simulation governs moment-to-moment activity on the map.
- Deadlines, limited opportunities, and other longer-term constraints can use
  a larger-scale notion of elapsed time.
- Different activities may advance that larger-scale time in ways appropriate
  to their purpose. This should not be assumed to mean that every interaction
  or system needs a real-time timer.

**Open questions**

- Are quest deadlines measured in elapsed world time, completed activities,
  milestones, or some combination?
- What actions or transitions advance large-scale time?
- Can the player intentionally advance time, and what risks or opportunities
  accompany doing so?
- How visible should deadlines and expiring opportunities be to the player?

## Encounters and combat

**Open**

The action-driven simulation is a decided foundation for gameplay, but combat
rules, enemy roles, and encounter structure have not yet been committed.
Combat is intended to take place directly in the explored environment rather
than in a separate arena or mode.

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
