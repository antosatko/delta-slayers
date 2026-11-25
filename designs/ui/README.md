# UI prototype

Open `index.html` in a browser. The prototype uses one visual direction: **Field Folio — Carbon**.

## Screens

- **Contract** — long-form reading, summary panels, objectives, and contextual references.
- **Inventory** — filterable and searchable item grid with a persistent detail panel.
- **Character** — identity, status bars, switchable statistics, equipment, and effects.
- **Journal** — filterable entries beside a reading surface and objective list.
- **Map** — schematic regional canvas with selectable location records.
- **Trade** — merchant stock, a transaction basket, and player goods.
- **Workshop** — recipe selection, recipe details, and available resources.

## Shared implementation

- `styles.css` contains the Carbon palette, shared layout primitives, responsive rules, and reference-card styling.
- `interaction.js` contains the shared contextual reference graph and the small interactions used by the screen studies.
- Reference cards open after a 600 ms pointer dwell; leaving the term before the delay cancels the opening. Keyboard focus and click still open immediately. Cards support click-to-pin, linked entries, Back, Escape, and scrolling content. The card keeps a fixed viewport position while navigating between entries of different lengths.

The shared navigation and frequent controls use inline SVG icons while retaining visible labels. Spacing is intentionally compact to use more of the available screen without compressing long-form reading. These are layout and interaction studies, not connected game systems. Buttons that would change game state display preview feedback only.