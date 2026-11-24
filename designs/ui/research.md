# Research notes: reading-first game UI

Research gathered for the first Delta Slayers UI exploration, 24 November 2026.
The sources are web design and accessibility references rather than game-specific
play-testing. Their guidance is useful as a baseline, but the final HUD and
interaction patterns still need to be tested at actual game resolution and
with controller / keyboard navigation.

## Findings that affect the prototypes

### 1. Give long text a deliberate reading measure

The U.S. Web Design System recommends 45–90 characters per line for most
reading, with about 66 characters as a useful target for long-form text. It
also recommends at least 1.5 line-height for longer passages. These prototypes
cap the prose measure around 66 characters and use approximately 1.6 line-height.

Source: [USWDS — Typography](https://designsystem.digital.gov/components/typography/)
Relevant section: Measure and line height.

### 2. Make the text contrast reliable

WCAG 2.2 specifies a contrast ratio of at least 4.5:1 for normal text and 3:1
for large text. The designs use high-contrast text colours and avoid using
low-contrast grey for meaningful prose. Borders are also strong enough to
separate adjacent panels, although border contrast is not the same criterion
as text contrast.

Source: [W3C — WCAG 2.2, Success Criterion 1.4.3](https://www.w3.org/TR/WCAG22/#contrast-minimum)

### 3. Do not use colour as the only signal

Digital.gov recommends reinforcing meaning through icons, labels, and other
visual cues rather than relying on colour alone. Accordingly, status labels
say AVAILABLE / UNVERIFIED, objective states have visible markers, and icons
sit beside written labels.

Source: [Digital.gov — Accessibility for visual designers](https://digital.gov/guides/accessibility-for-teams/visual-design/)

### 4. Use hierarchy to support scanning

Digital.gov recommends visually distinguishing heading levels through
consistent combinations of size, weight, typeface, or colour. The prototypes
separate the document title, section headers, metadata labels, paragraphs,
and optional notes rather than relying on a single text size or a coloured
background.

Source: [Digital.gov — Layout and hierarchy](https://digital.gov/guides/accessibility-for-teams/visual-design/)

### 5. Make tips optional and usable without a mouse

WCAG 2.2's "Content on Hover or Focus" criterion says additional content
triggered by hover or focus should be dismissible, hoverable, and persistent.
Carbon's tooltip guidance similarly describes tooltips as contextual,
nonessential information available on hover and focus, including keyboard
dismissal. This suggests a good in-game rule:

- Put essential information in the main text or a persistent panel.
- Use tips for definitions, lore context, abbreviations, and secondary
  explanations — not for an instruction the player must know to proceed.
- Support mouse hover, keyboard focus, controller focus, and click / tap.
- Keep the tooltip visible while the player moves the pointer onto it.
- Allow Escape or an equivalent back action to close it.

Sources:
- [W3C — WCAG 2.2, Success Criterion 1.4.13](https://www.w3.org/TR/WCAG22/#content-on-hover-or-focus)
- [Carbon Design System — Tooltip accessibility](https://www.carbondesignsystem.com/building-blocks/core/components/tooltip/accessibility)
- [Carbon Design System — Tooltip guidelines](https://www.carbondesignsystem.com/building-blocks/core/components/tooltip/guidelines)

### 6. Keep typography calm and left-aligned

The UK Home Office guidance recommends clear, readable fonts, left-aligned
content, headings and line breaks, and avoiding full paragraphs in italics or
all caps. Bold is recommended for inline emphasis. The prototypes use
uppercase only for short metadata labels, keep body text left-aligned, and
reserve serif faces for display / literary character rather than tiny UI labels.

Source: [Home Office User-Centred Design Manual — Layout and typography](https://design.homeoffice.gov.uk/accessibility/page-structure/layout-typography)

## Design principles derived for Delta Slayers

1. Body copy should remain the quietest, most readable element on the screen.
2. Long-form text should have a controlled line width, generous leading, and
   paragraph breaks based on ideas rather than screen height.
3. Give every section a visible boundary or unmistakable heading; use spacing
   to group related fields.
4. Use colour as an accent, not as the sole encoding of state or importance.
5. Always pair icons with labels in important navigation and metadata.
6. Use inline highlights sparingly. A highlighted term should signal that
   useful extra context exists, not that every sentence is interactive.
7. Make optional tips available by hover, focus, click / tap, and controller
   focus. Tips should never hide required instructions.
8. Avoid aggressive pulsing, flashing, auto-scrolling, moving text, or constant
   attention-grabbing highlights. Let the player's eyes settle on the prose.
9. Treat accessibility values as a measurable starting point; validate the
   finished game UI with real contrast checks and in-game testing.

## What these references do not establish

These sources do not prove that a particular font, dark theme, ornamental
border, or tooltip density is best for adult fantasy players. That is a matter
for prototype comparison and play-testing. The three visual directions are
intentionally hypotheses, not research-backed winners.
