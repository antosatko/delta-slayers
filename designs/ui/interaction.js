(() => {
const entries = {
    "brine-wight": {
      title: "Brine-wight", category: "Local folklore · Unconfirmed", icon: "spectre",
      description: [
        [{ text: "A coastal name for something believed to return from the water. In Greybridge, accounts disagree about whether it describes a " }, { ref: "wight", label: "wight" }, { text: ", a condition of the " }, { ref: "west-watercourse", label: "watercourse" }, { text: ", or an old debt given a shape." }],
        [{ text: "The name appears in two apprentice statements. Neither account is detailed enough to establish what was encountered." }]
      ],
      factsTitle: "Field record", facts: [["Reports", "2"], ["Classification", "Disputed"], ["Threat", "Unknown"]],
      related: ["wight", "brine", "west-watercourse"]
    },
    "tithe": {
      title: "Tithe", category: "Guild custom · Payment", icon: "ledger",
      description: [
        [{ text: "A portion of a harvest or income paid to an institution or authority. The amount and form depend on the local agreement." }],
        [{ text: "In Greybridge, the " }, { ref: "guild", label: "guild" }, { text: " collected its tithe in grain. The old records become unclear around the " }, { ref: "famine-winter", label: "famine winter" }, { text: "." }]
      ],
      factsTitle: "Local record", facts: [["Usual payment", "Harvest goods"], ["Recorded form", "Grain"], ["Rate", "Not recorded"]],
      related: ["guild", "famine-winter"]
    },
    "wight": {
      title: "Wight", category: "Bestiary term · Classification", icon: "spectre",
      description: [
        [{ text: "A term used for a person or creature believed to persist after death. It describes a broad category, not a reliable account of a particular being." }],
        [{ text: "The word alone does not establish whether a " }, { ref: "brine-wight", label: "brine-wight" }, { text: " is undead, or even whether it is a creature." }]
      ],
      factsTitle: "Known properties", facts: [["Origin", "Death or burial rites"], ["Behaviour", "Varies by account"], ["Weakness", "Not established"]],
      related: ["brine", "brine-wight"]
    },
    "brine": {
      title: "Brine", category: "Material · Water", icon: "water",
      description: [
        [{ text: "Water containing a high concentration of dissolved salt. In coastal records, the word is also used for the pools left behind by a receding tide." }],
        [{ text: "The old " }, { ref: "west-watercourse", label: "watercourse" }, { text: " is fresh on most maps, though the miller's notes mention a taste of salt." }]
      ],
      factsTitle: "Reference", facts: [["Composition", "Salt water"], ["Common source", "Coastal pools"], ["Mill sample", "Not collected"]],
      related: ["west-watercourse", "brine-wight"]
    },
    "west-watercourse": {
      title: "West watercourse", category: "Location · Greybridge", icon: "water",
      description: [
        [{ text: "An older channel that runs beneath the west foundation of the mill. It predates the current stonework and is no longer shown on the public village plan." }],
        [{ text: "The miller's reference to " }, { ref: "brine", label: "brine" }, { text: " has not been confirmed by a sample or inspection." }]
      ],
      factsTitle: "Survey notes", facts: [["Known entrance", "Not established"], ["Last inspection", "Unknown"], ["Current flow", "Unsurveyed"]],
      related: ["brine", "brine-wight"]
    },
    "guild": {
      title: "Greybridge Guild", category: "Institution · Local chapter", icon: "guild",
      description: [
        [{ text: "The local chapter handles contracts, arbitration, and records for the surrounding settlements. Its authority comes from a series of agreements with the villages, not from a single charter." }],
        [{ text: "Its collection of the " }, { ref: "tithe", label: "tithe" }, { text: " was altered after the " }, { ref: "famine-winter", label: "famine winter" }, { text: ", though the surviving ledger is incomplete." }]
      ],
      factsTitle: "Chapter record", facts: [["Office", "North road, Greybridge"], ["Current clerk", "Mara Venn"], ["Ledger status", "Incomplete"]],
      related: ["tithe", "famine-winter"]
    },
    "famine-winter": {
      title: "The famine winter", category: "Historical event · Greybridge", icon: "harvest",
      description: [
        [{ text: "The name given to a season of crop failure and food shortage remembered in the older village records. The surviving accounts disagree about its exact duration." }],
        [{ text: "The mill bell bears a date associated with this period. The inscription may connect the miller's ledger to the changes in the " }, { ref: "tithe", label: "guild tithe" }, { text: "." }]
      ],
      factsTitle: "Historical record", facts: [["Date", "Inscription incomplete"], ["Cause", "Not established"], ["Recorded impact", "Crop failure"]],
      related: ["guild", "tithe"]
    },
    "screen-help-inventory": {
      title: "Using the inventory", category: "Screen guide", icon: "item",
      description: [
        [{ text: "The inventory groups everything carried by the party. Select an item to open its detail panel and review its condition, value, and other properties." }],
        [{ text: "Use the category filters or search field to narrow the grid. A number on an item tile is the quantity currently carried." }]
      ],
      factsTitle: "Inventory controls", facts: [["Filters", "Equipment, consumables, and materials"], ["Search", "Matches item names"], ["Selection", "Opens item details"]],
      related: []
    },
    "screen-help-character": {
      title: "Character record", category: "Screen guide", icon: "guild",
      description: [
        [{ text: "This page brings together the character's current condition, core statistics, proficiencies, resistances, and equipment." }],
        [{ text: "Use the tabs to switch between attributes, skills, and resistances. Item tiles provide additional information when hovered or focused." }]
      ],
      factsTitle: "Character sections", facts: [["Attributes", "Core values"], ["Skills", "Proficiency levels"], ["Resistances", "Defence against damage and effects"]],
      related: []
    },
    "screen-help-contract": {
      title: "Contract overview", category: "Screen guide", icon: "ledger",
      description: [
        [{ text: "The contract describes the assignment, its known circumstances, and the expected payment." }],
        [{ text: "Use the objective checklist to track completed steps. The checklist is local to this prototype and does not change the contract text." }]
      ],
      factsTitle: "Contract contents", facts: [["Brief", "Known details and background"], ["Objectives", "Track investigation steps"], ["Payment", "Expected reward"]],
      related: []
    },
    "screen-help-journal": {
      title: "Journal records", category: "Screen guide", icon: "ledger",
      description: [
        [{ text: "The journal collects contracts, clues, and other records gathered during the party's work." }],
        [{ text: "Filter the list by record type, then select an entry to read its details and related objectives." }]
      ],
      factsTitle: "Journal controls", facts: [["Filters", "All records, contracts, or clues"], ["Selection", "Opens the record"], ["Status", "Shows the current progress"]],
      related: []
    },
    "screen-help-map": {
      title: "Reading the map", category: "Screen guide", icon: "water",
      description: [
        [{ text: "The map shows known roads, landmarks, and places around Greybridge. Markers identify locations with a record available." }],
        [{ text: "Select a marker to reveal its description and current status in the location panel." }]
      ],
      factsTitle: "Map controls", facts: [["Markers", "Select to inspect a location"], ["North", "Up on the map"], ["Coverage", "Known locations only"]],
      related: []
    },
    "screen-help-trade": {
      title: "Trading with the provisioner", category: "Screen guide", icon: "ledger",
      description: [
        [{ text: "Switch between buying supplies and selling carried goods using the tabs below the screen title." }],
        [{ text: "Add items from the market to the transaction basket. Review the quantities and total before confirming; sale offers are estimates and may be refused." }]
      ],
      factsTitle: "Trading pages", facts: [["Buy supplies", "Add market stock to the basket"], ["Sell goods", "Offer carried items"], ["Transaction", "Review the basket and total"]],
      related: []
    },
    "screen-help-workshop": {
      title: "Workshop and recipes", category: "Screen guide", icon: "item",
      description: [
        [{ text: "The resource strip shows the materials currently available to the workshop." }],
        [{ text: "Select a known recipe to review its ingredients and requirements before starting the craft." }]
      ],
      factsTitle: "Workshop contents", facts: [["Materials", "Current available quantities"], ["Recipes", "Known craftable preparations"], ["Recipe details", "Ingredients and requirements"]],
      related: []
    }
  };

  const iconPaths = {
    spectre: '<path d="M5 20v-4a7 7 0 1 1 14 0v4l-3-2-2 2-2-2-2 2-2-2z"/><path d="M9 11h.01M15 11h.01M9 15c2 1 4 1 6 0"/>',
    ledger: '<path d="M6 4h12v16H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2z"/><path d="M8 8h7M8 12h7M8 16h4"/>',
    water: '<path d="M12 3c-2.5 3.5-6 7.4-6 11a6 6 0 0 0 12 0c0-3.6-3.5-7.5-6-11z"/><path d="M8.5 15c.5 1.3 1.6 2 3 2"/>',
    guild: '<path d="M12 3 20 6v5c0 5-3.2 8-8 10-4.8-2-8-5-8-10V6z"/><path d="M9 10h6M9 13h6"/>',
    harvest: '<path d="M12 21V4M12 9c-3 0-5-2-5-5 3 0 5 2 5 5zM12 13c3 0 5-2 5-5-3 0-5 2-5 5zM12 17c-3 0-5-2-5-5 3 0 5 2 5 5z"/>',
    item: '<path d="M5 7.5 12 4l7 3.5-7 3.5z"/><path d="M5 7.5V17l7 3 7-3V7.5M12 11v9"/>'
  };

  function el(tag, className, text) {
    const node = document.createElement(tag);
    if (className) node.className = className;
    if (text !== undefined) node.textContent = text;
    return node;
  }

  function createReferenceCard() {
    const node = el("section", "reference-card");
    node.setAttribute("role", "dialog");
    node.setAttribute("aria-modal", "false");
    node.setAttribute("aria-labelledby", "reference-title");
    node.setAttribute("aria-describedby", "reference-description");
    node.hidden = true;
    node.dataset.referenceCard = "";
    node.innerHTML =
      '<header class="reference-card-header">' +
        '<button class="reference-back" type="button" data-reference-back aria-label="Go back" title="Go back" hidden><svg viewBox="0 0 20 20" aria-hidden="true"><path d="M12.5 4.5 7 10l5.5 5.5M7.5 10H17"/></svg></button>' +
        '<span class="reference-icon" data-reference-icon aria-hidden="true"></span>' +
        '<div class="reference-heading"><p class="reference-category" data-reference-category></p><h2 class="reference-title" id="reference-title" data-reference-title tabindex="-1"></h2></div>' +
        '<button class="reference-close" type="button" data-reference-close aria-label="Close reference" title="Close"><svg viewBox="0 0 20 20" aria-hidden="true"><path d="m5 5 10 10M15 5 5 15"/></svg></button>' +
      '</header>' +
      '<div class="reference-content">' +
        '<div class="reference-description" id="reference-description" data-reference-description></div>' +
        '<section class="reference-facts" data-reference-facts hidden><h3 data-reference-facts-title>Details</h3><dl data-reference-facts-list></dl></section>' +
        '<section class="reference-related" data-reference-related hidden><h3>Related</h3><div class="reference-related-list" data-reference-related-list></div></section>' +
      '</div>';
    document.body.append(node);
    return node;
  }

  const card = document.querySelector("[data-reference-card]") || createReferenceCard();
  const title = card.querySelector("[data-reference-title]");
  const category = card.querySelector("[data-reference-category]");
  const icon = card.querySelector("[data-reference-icon]");
  const description = card.querySelector("[data-reference-description]");
  const facts = card.querySelector("[data-reference-facts]");
  const factsTitle = card.querySelector("[data-reference-facts-title]");
  const factsList = card.querySelector("[data-reference-facts-list]");
  const related = card.querySelector("[data-reference-related]");
  const relatedList = card.querySelector("[data-reference-related-list]");
  const back = card.querySelector("[data-reference-back]");
  const states = new Map();
  let activeState = null;

  function cancelClose(state) {
    if (state.closeTimer) {
      clearTimeout(state.closeTimer);
      state.closeTimer = null;
    }
  }

  const HOVER_OPEN_DELAY = 600;

  function cancelOpen(state) {
    if (state.openTimer) {
      clearTimeout(state.openTimer);
      state.openTimer = null;
    }
  }

  function scheduleOpen(state) {
    cancelOpen(state);
    state.openTimer = setTimeout(() => {
      state.openTimer = null;
      if (state.pointerOnTrigger) openState(state, false);
    }, HOVER_OPEN_DELAY);
  }

  function positionCard(state, chooseSide = false) {
    const rect = state.trigger.getBoundingClientRect();
    const margin = 12;
    const width = card.offsetWidth;
    const height = card.offsetHeight;
    const left = Math.max(margin, Math.min(rect.left, window.innerWidth - width - margin));
    if (chooseSide || !state.cardSide) {
      const belowFits = rect.bottom + 8 + height <= window.innerHeight - margin;
      const aboveFits = rect.top - 8 - height >= margin;
      state.cardSide = belowFits || !aboveFits ? "below" : "above";
    }
    const rawTop = state.cardSide === "above" ? rect.top - height - 8 : rect.bottom + 8;
    const top = Math.max(margin, Math.min(rawTop, window.innerHeight - height - margin));
    card.style.left = left + "px";
    card.style.top = top + "px";
  }

  function appendParagraph(segments) {
    const paragraph = el("p", "reference-paragraph");
    for (const segment of segments) {
      if (typeof segment === "string" || segment.text !== undefined) {
        paragraph.append(document.createTextNode(typeof segment === "string" ? segment : segment.text));
      } else if (segment.ref && entries[segment.ref]) {
        const link = el("button", "reference-inline", segment.label || entries[segment.ref].title);
        link.type = "button";
        link.dataset.entryRef = segment.ref;
        paragraph.append(link);
      }
    }
    description.append(paragraph);
  }

  function renderEntry(state, entryId, focusTitle) {
    const entry = entries[entryId];
    if (!entry) return;
    state.currentId = entryId;
    title.textContent = entry.title;
    category.textContent = entry.category;
    icon.innerHTML = '<svg viewBox="0 0 24 24" aria-hidden="true">' + (iconPaths[entry.icon] || iconPaths.ledger) + '</svg>';
    description.replaceChildren();
    entry.description.forEach(appendParagraph);

    factsList.replaceChildren();
    facts.hidden = !entry.facts || entry.facts.length === 0;
    if (!facts.hidden) {
      factsTitle.textContent = entry.factsTitle || "Details";
      for (const [label, value] of entry.facts) {
        const row = el("div", "reference-fact");
        row.append(el("dt", "", label), el("dd", "", value));
        factsList.append(row);
      }
    }

    relatedList.replaceChildren();
    const relatedEntries = (entry.related || []).filter(id => entries[id]);
    related.hidden = relatedEntries.length === 0;
    for (const relatedId of relatedEntries) {
      const link = el("button", "", entries[relatedId].title);
      link.type = "button";
      link.dataset.entryRef = relatedId;
      relatedList.append(link);
    }

    const parentId = state.history[state.history.length - 1];
    const backLabel = parentId ? "Return to " + entries[parentId].title : "Go back";
    back.setAttribute("aria-label", backLabel);
    back.title = backLabel;
    back.hidden = state.history.length === 0;
    card.hidden = false;
    positionCard(state, state.cardSide === null);
    if (focusTitle) title.focus({ preventScroll: true });
  }

  function closeState(state, restoreFocus, suppressFocus) {
    cancelOpen(state);
    cancelClose(state);
    card.hidden = true;
    state.trigger.setAttribute("aria-expanded", "false");
    state.pinned = false;
    state.history.length = 0;
    state.currentId = null;
    state.cardSide = null;
    if (suppressFocus) state.suppressFocus = true;
    if (activeState === state) activeState = null;
    if (restoreFocus) state.trigger.focus({ preventScroll: true });
  }

  function openState(state, pin) {
    cancelOpen(state);
    if (activeState && activeState !== state) closeState(activeState, false, false);
    activeState = state;
    cancelClose(state);
    if (!state.currentId) {
      state.history.length = 0;
      state.cardSide = null;
      renderEntry(state, state.trigger.dataset.entryTrigger, false);
    } else {
      card.hidden = false;
      positionCard(state, state.cardSide === null);
    }
    state.pinned = state.pinned || Boolean(pin);
    state.trigger.setAttribute("aria-expanded", "true");
  }

  function scheduleClose(state) {
    cancelClose(state);
    state.closeTimer = setTimeout(() => {
      const focusInside = document.activeElement === state.trigger || card.contains(document.activeElement);
      if (!state.pinned && !state.pointerOnTrigger && !state.pointerOnCard && !focusInside) {
        closeState(state, false, false);
      }
    }, 180);
  }

  function navigate(state, nextId) {
    if (!entries[nextId]) return;
    state.history.push(state.currentId || state.trigger.dataset.entryTrigger);
    state.pinned = true;
    renderEntry(state, nextId, true);
    state.trigger.setAttribute("aria-expanded", "true");
  }

  function goBack(state) {
    const previous = state.history.pop();
    if (!previous) return;
    renderEntry(state, previous, true);
    state.trigger.setAttribute("aria-expanded", "true");
  }

  // Item inspection reuses the reference card, including its 600 ms hover delay.
  document.querySelectorAll("[data-item-inspect]").forEach(trigger => {
    const itemId = "item-" + (trigger.dataset.itemKey || trigger.dataset.itemTitle || trigger.textContent)
      .toLowerCase().trim().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "");
    trigger.dataset.entryTrigger = itemId;
    const facts = (trigger.dataset.itemFacts || "").split(";").filter(Boolean).map(pair => {
      const separator = pair.indexOf("|");
      return separator < 0 ? [pair, ""] : [pair.slice(0, separator), pair.slice(separator + 1)];
    });
    entries[itemId] = {
      title: trigger.dataset.itemTitle || trigger.textContent.trim(),
      category: trigger.dataset.itemMeta || "Item",
      icon: "item",
      description: [[{ text: trigger.dataset.itemDescription || "No further information is available." }]],
      factsTitle: "Item details",
      facts,
      related: []
    };
  });

  const triggers = [...document.querySelectorAll("[data-entry-trigger]")];
  for (const trigger of triggers) {
    const state = {
      trigger, currentId: null, history: [], pinned: false, suppressFocus: false,
      pointerOnTrigger: false, pointerOnCard: false, openTimer: null, closeTimer: null, cardSide: null
    };
    states.set(trigger, state);
    trigger.setAttribute("aria-haspopup", "dialog");
    trigger.setAttribute("aria-expanded", "false");
    trigger.addEventListener("pointerenter", event => {
      if (event.pointerType === "touch") return;
      state.pointerOnTrigger = true;
      scheduleOpen(state);
    });
    trigger.addEventListener("pointerleave", () => {
      state.pointerOnTrigger = false;
      cancelOpen(state);
      scheduleClose(state);
    });
    trigger.addEventListener("focus", () => { if (!state.suppressFocus) openState(state, false); });
    trigger.addEventListener("blur", () => {
      setTimeout(() => {
        if (document.activeElement !== trigger && !card.contains(document.activeElement)) state.suppressFocus = false;
      }, 0);
    });
    trigger.addEventListener("keydown", event => {
      if (event.key !== "Tab" || event.shiftKey || activeState !== state || card.hidden) return;
      const firstControl = card.querySelector("button:not([hidden])");
      if (firstControl) { event.preventDefault(); firstControl.focus(); }
    });
    trigger.addEventListener("click", event => {
      // Item cards and screen-help icons are hover/focus references only.
      // Keep clicks available for the control's primary action.
      if (trigger.hasAttribute("data-item-inspect") || trigger.hasAttribute("data-hover-only")) return;
      if (activeState === state && !card.hidden && state.pinned) {
        closeState(state, false, true);
        return;
      }
      openState(state, true);
      state.pinned = true;
      if (event.detail === 0) title.focus({ preventScroll: true });
    });
  }

  card.addEventListener("pointerenter", () => {
    if (activeState) { activeState.pointerOnCard = true; cancelClose(activeState); }
  });
  card.addEventListener("pointerleave", () => {
    if (activeState) { activeState.pointerOnCard = false; scheduleClose(activeState); }
  });
  card.addEventListener("click", event => {
    const reference = event.target.closest("[data-entry-ref]");
    if (reference && card.contains(reference)) { navigate(activeState, reference.dataset.entryRef); return; }
    if (event.target.closest("[data-reference-back]")) { if (activeState) goBack(activeState); return; }
    if (event.target.closest("[data-reference-close]") && activeState) closeState(activeState, true, true);
  });
  document.addEventListener("focusin", event => {
    if (activeState && (event.target === activeState.trigger || card.contains(event.target))) {
      cancelClose(activeState);
      return;
    }
    const trigger = event.target.closest && event.target.closest("[data-entry-trigger]");
    if (trigger) {
      const state = states.get(trigger);
      if (state && !state.suppressFocus) openState(state, false);
    } else if (activeState) scheduleClose(activeState);
  });
  document.addEventListener("pointerdown", event => {
    if (activeState && !activeState.trigger.contains(event.target) && !card.contains(event.target)) {
      closeState(activeState, false, false);
    }
  });
  document.addEventListener("keydown", event => {
    if (event.key === "Escape") {
      const openDetail = document.querySelector("[data-detail-panel]:not([hidden])");
      if (openDetail) {
        closeDetail(openDetail);
        event.preventDefault();
        return;
      }
    }
    if (event.key !== "Escape" || !activeState) return;
    const state = activeState;
    if (state.history.length) goBack(state);
    else closeState(state, true, true);
    event.preventDefault();
    event.stopPropagation();
  });
  window.addEventListener("resize", () => { if (activeState && !card.hidden) positionCard(activeState, true); });
  window.addEventListener("scroll", () => { if (activeState && !card.hidden) positionCard(activeState, false); }, true);

  function applyFilter(scope) {
    const selected = scope.querySelector("[data-filter-value].active");
    const category = selected ? selected.dataset.filterValue : "all";
    const search = (scope.querySelector("[data-filter-search]")?.value || "").trim().toLowerCase();
    let visibleCount = 0;
    scope.querySelectorAll("[data-filter-item]").forEach(item => {
      const categoryMatch = category === "all" || item.dataset.category === category;
      const searchMatch = !search || (item.dataset.search || item.textContent).toLowerCase().includes(search);
      item.hidden = !(categoryMatch && searchMatch);
      if (!item.hidden) visibleCount += 1;
      if (item.hidden && item.classList.contains("selected")) {
        item.classList.remove("selected");
        item.setAttribute("aria-pressed", "false");
        const panel = document.querySelector(item.dataset.selectionTarget);
        if (panel) closeDetail(panel);
      }
    });
    const count = scope.querySelector("[data-filter-count]");
    if (count) count.textContent = visibleCount + " matching items";
    const empty = scope.querySelector("[data-filter-empty]");
    if (empty) empty.hidden = visibleCount !== 0;
  }
  document.querySelectorAll("[data-filter-scope]").forEach(scope => {
    scope.querySelectorAll("[data-filter-value]").forEach(button => {
      button.addEventListener("click", () => {
        scope.querySelectorAll("[data-filter-value]").forEach(other => other.classList.toggle("active", other === button));
        applyFilter(scope);
      });
    });
    scope.querySelector("[data-filter-search]")?.addEventListener("input", () => applyFilter(scope));
  });

  document.querySelectorAll("[data-tab-group]").forEach(group => {
    const name = group.dataset.tabGroup;
    group.querySelectorAll("[data-tab]").forEach(button => {
      button.addEventListener("click", () => {
        group.querySelectorAll("[data-tab]").forEach(other => {
          const active = other === button;
          other.classList.toggle("active", active);
          other.setAttribute("aria-selected", String(active));
        });
        document.querySelectorAll('[data-tab-panel-group="' + name + '"]').forEach(panel => {
          panel.hidden = panel.dataset.tabPanel !== button.dataset.tab;
        });
      });
    });
  });

  function closeDetail(panel) {
    if (!panel) return;
    panel.hidden = true;
    panel.closest(".inventory-layout, .workshop-layout")?.classList.remove("has-open-detail");
    panel.closest(".map-layout")?.classList.remove("has-open-detail");
    const scope = panel.closest("[data-selection-scope]") || document;
    const trigger = scope.querySelector('[aria-pressed="true"]');
    scope.querySelectorAll("[data-select-item], [data-map-place]").forEach(item => {
      item.classList.remove("selected");
      item.setAttribute("aria-pressed", "false");
    });
    if (trigger && !trigger.hidden) trigger.focus({ preventScroll: true });
  }

  document.querySelectorAll("[data-detail-close]").forEach(button => {
    button.addEventListener("click", () => closeDetail(button.closest("[data-detail-panel]")));
  });

  document.querySelectorAll("[data-select-item]").forEach(button => {
    button.setAttribute("aria-pressed", "false");
    button.addEventListener("click", () => {
      const target = document.querySelector(button.dataset.selectionTarget);
      if (!target) return;
      const scope = button.closest("[data-selection-scope]") || document;
      scope.querySelectorAll("[data-select-item]").forEach(other => {
        const selected = other === button;
        other.classList.toggle("selected", selected);
        other.setAttribute("aria-pressed", String(selected));
      });
      target.hidden = false;
      target.closest(".inventory-layout, .workshop-layout")?.classList.add("has-open-detail");
      const fields = {
        title: "[data-selection-title]", meta: "[data-selection-meta]",
        kicker: "[data-selection-kicker]", status: "[data-selection-status]", description: "[data-selection-description]", content: "[data-selection-content]", visual: "[data-selection-visual]"
      };
      for (const [key, selector] of Object.entries(fields)) {
        const node = target.querySelector(selector);
        if (!node) continue;
        if (key === "visual") node.textContent = button.dataset.itemVisual || "•";
        else node.textContent = button.dataset["item" + key[0].toUpperCase() + key.slice(1)] || "";
      }
      const factsNode = target.querySelector("[data-selection-facts]");
      if (factsNode) {
        factsNode.replaceChildren();
        const pairs = (button.dataset.itemFacts || "").split(";").filter(Boolean);
        for (const pair of pairs) {
          const separator = pair.indexOf("|");
          if (separator < 0) continue;
          const row = el("div", "fact-row");
          row.append(el("dt", "", pair.slice(0, separator)), el("dd", "", pair.slice(separator + 1)));
          factsNode.append(row);
        }
      }    });
  });

  document.querySelectorAll("[data-map-place]").forEach(button => {
    button.setAttribute("aria-pressed", "false");
    button.addEventListener("click", () => {
      const target = document.querySelector(button.dataset.mapTarget);
      if (!target) return;
      document.querySelectorAll("[data-map-place]").forEach(other => {
        const selected = other === button;
        other.classList.toggle("selected", selected);
        other.setAttribute("aria-pressed", String(selected));
      });
      target.hidden = false;
      target.closest(".map-layout")?.classList.add("has-open-detail");
      target.querySelector("[data-place-title]").textContent = button.dataset.placeTitle;
      target.querySelector("[data-place-description]").textContent = button.dataset.placeDescription;
      target.querySelector("[data-place-status]").textContent = button.dataset.placeStatus || "Mapped";
      target.querySelector("[data-place-coordinates]").textContent = button.dataset.placeCoordinates || "";
    });
  });

  document.querySelectorAll("[data-checklist]").forEach(list => {
    const update = () => {
      const checked = list.querySelectorAll("input[type=checkbox]:checked").length;
      const total = list.querySelectorAll("input[type=checkbox]").length;
      const count = list.querySelector("[data-check-count]");
      if (count) count.textContent = checked + " of " + total;
    };
    list.addEventListener("change", event => { if (event.target.matches("input[type=checkbox]")) update(); });
    update();
  });

  document.querySelectorAll("[data-trade-add]").forEach(button => {
    button.addEventListener("click", () => {
      const cart = document.querySelector("[data-trade-cart]");
      if (!cart) return;
      const name = button.dataset.tradeName;
      const price = Number(button.dataset.tradePrice);
      const existing = [...cart.querySelectorAll("[data-cart-name]")].find(row => row.dataset.cartName === name);
      if (existing) existing.dataset.quantity = String(Number(existing.dataset.quantity) + 1);
      else {
        const row = el("div", "cart-line");
        row.dataset.cartName = name; row.dataset.quantity = "1"; row.dataset.price = String(price);
        const label = el("span", "", name + " × 1");
        const amount = el("span", "", price + " crowns");
        const remove = el("button", "", "Remove");
        remove.type = "button";
        remove.addEventListener("click", () => { row.remove(); updateCart(); });
        row.append(label, amount, remove); cart.append(row);
      }
      updateCart();
    });
  });
  function updateCart() {
    const cart = document.querySelector("[data-trade-cart]");
    if (!cart) return;
    const rows = [...cart.querySelectorAll("[data-cart-name]")];
    const count = rows.reduce((sum, row) => sum + Number(row.dataset.quantity), 0);
    const total = rows.reduce((sum, row) => sum + Number(row.dataset.quantity) * Number(row.dataset.price), 0);
    rows.forEach(row => {
      const name = row.dataset.cartName, quantity = Number(row.dataset.quantity), price = Number(row.dataset.price);
      row.children[0].textContent = name + " × " + quantity;
      row.children[1].textContent = (price * quantity) + " crowns";
    });
    const countNode = document.querySelector("[data-cart-count]");
    const totalNode = document.querySelector("[data-cart-total]");
    if (countNode) countNode.textContent = String(count);
    if (totalNode) totalNode.textContent = total + " crowns";
    const empty = document.querySelector("[data-cart-empty]");
    if (empty) empty.hidden = rows.length > 0;
  }
  document.querySelector("[data-trade-clear]")?.addEventListener("click", () => {
    const cart = document.querySelector("[data-trade-cart]");
    cart?.replaceChildren();
    updateCart();
  });

  document.querySelectorAll("[data-demo-action]").forEach(button => {
    button.addEventListener("click", () => {
      const scope = button.closest("[data-action-scope]") || document;
      const feedback = scope.querySelector("[data-action-feedback]") || document.querySelector("[data-action-feedback]");
      if (feedback) feedback.textContent = button.dataset.demoAction;
    });
  });
})();