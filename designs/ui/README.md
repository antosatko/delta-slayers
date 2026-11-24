# UI design

Open index.html in a browser. The current primary direction is the revised
Field Folio prototype in field-folio/.

## Field Folio — revision 2

This pass keeps the original reading layout: a narrow navigation rail, a
single main reading surface, and a compact contract summary beside it. The
palette moves to warm charcoal and muted olive so it can sit comfortably in a
dark game without taking on the gothic styling of the earlier Archive draft.

The visual language is deliberately reduced:

- One system sans-serif family throughout.
- One clear page title; section headings are plain and consistent.
- No decorative icons, monospaced labels, numbered section furniture, stamps,
  or ornamental borders.
- Borders separate the reading surface and summary groups; nested cards and
  drop shadows are removed.
- Colour is restrained to the background, text, borders, and one muted accent.
- Contextual terms remain underlined and offer optional definitions on hover,
  keyboard focus, or click/tap. Escape closes them.
- The demo actions only display preview feedback and do not change game state.

The content is unchanged from the first prototype to make the visual revision
easy to judge. The earlier Surveyor Console and Ash Archive drafts remain
available from the index for reference, but are not presented as active
directions.

## Research

The sources and accessibility notes behind the first pass remain in
research.md. They are reference guidance, not evidence that any one theme is
best for this game. Test the final interface at actual game resolution and
with the input methods the game supports.


## Reference cards

The Field Folio contract demonstrates the expanded reference interaction.
Hover or focus brine-wight or tithe to open a card. Click the term to pin
the card open. The card includes an icon, title, category, description,
optional fact rows, and links to related entries.

Selecting a linked entry navigates inside the same card. The Back control
returns to the previous entry; Escape moves back through the history and
closes the card at its starting entry. Only one card is open at a time. This
is deliberately a navigable reference graph, not a stack of nested popups.

The example data lives in interaction.js. It is sample lore for the
prototype. Its optional structured facts are designed to support different
kinds of information, including gameplay statistics, item properties,
requirements, effects, resistances, and historical details.
