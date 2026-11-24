(() => {
  const entries = {
    "brine-wight": {
      title: "Brine-wight",
      category: "Local folklore · Unconfirmed",
      icon: "spectre",
      description: [
        [
          { text: "A coastal name for something believed to return from the water. In Greybridge, accounts disagree about whether it describes a " },
          { ref: "wight", label: "wight" },
          { text: ", a condition of the " },
          { ref: "west-watercourse", label: "watercourse" },
          { text: ", or an old debt given a shape." }
        ],
        [
          { text: "The name appears in two apprentice statements. Neither account is detailed enough to establish what was encountered." }
        ]
      ],
      factsTitle: "Field record",
      facts: [["Reports", "2"], ["Classification", "Disputed"], ["Threat", "Unknown"]],
      related: ["wight", "brine", "west-watercourse"]
    },
    "tithe": {
      title: "Tithe",
      category: "Guild custom · Payment",
      icon: "ledger",
      description: [
        [{ text: "A portion of a harvest or income paid to an institution or authority. The amount and form depend on the local agreement." }],
        [
          { text: "In Greybridge, the " }, { ref: "guild", label: "guild" },
          { text: " collected its tithe in grain. The old records become unclear around the " },
          { ref: "famine-winter", label: "famine winter" }, { text: "." }
        ]
      ],
      factsTitle: "Local record",
      facts: [["Usual payment", "Harvest goods"], ["Recorded form", "Grain"], ["Rate", "Not recorded"]],
      related: ["guild", "famine-winter"]
    },
    "wight": {
      title: "Wight",
      category: "Bestiary term · Classification",
      icon: "spectre",
      description: [
        [{ text: "A term used for a person or creature believed to persist after death. It describes a broad category, not a reliable account of a particular being." }],
        [
          { text: "The word alone does not establish whether a " }, { ref: "brine-wight", label: "brine-wight" },
          { text: " is undead, or even whether it is a creature." }
        ]
      ],
      factsTitle: "Known properties",
      facts: [["Origin", "Death or burial rites"], ["Behaviour", "Varies by account"], ["Weakness", "Not established"]],
      related: ["brine", "brine-wight"]
    },
    "brine": {
      title: "Brine",
      category: "Material · Water",
      icon: "water",
      description: [
        [{ text: "Water containing a high concentration of dissolved salt. In coastal records, the word is also used for the pools left behind by a receding tide." }],
        [
          { text: "The old " }, { ref: "west-watercourse", label: "watercourse" },
          { text: " is fresh on most maps, though the miller's notes mention a taste of salt." }
        ]
      ],
      factsTitle: "Reference",
      facts: [["Composition", "Salt water"], ["Common source", "Coastal pools"], ["Mill sample", "Not collected"]],
      related: ["west-watercourse", "brine-wight"]
    },
    "west-watercourse": {
      title: "West watercourse",
      category: "Location · Greybridge",
      icon: "water",
      description: [
        [{ text: "An older channel that runs beneath the west foundation of the mill. It predates the current stonework and is no longer shown on the public village plan." }],
        [
          { text: "The miller's reference to " }, { ref: "brine", label: "brine" },
          { text: " has not been confirmed by a sample or inspection." }
        ]
      ],
      factsTitle: "Survey notes",
      facts: [["Known entrance", "Not established"], ["Last inspection", "Unknown"], ["Current flow", "Unsurveyed"]],
      related: ["brine", "brine-wight"]
    },
    "guild": {
      title: "Greybridge Guild",
      category: "Institution · Local chapter",
      icon: "guild",
      description: [
        [{ text: "The local chapter handles contracts, arbitration, and records for the surrounding settlements. Its authority comes from a series of agreements with the villages, not from a single charter." }],
        [
          { text: "Its collection of the " }, { ref: "tithe", label: "tithe" },
          { text: " was altered after the " }, { ref: "famine-winter", label: "famine winter" },
          { text: ", though the surviving ledger is incomplete." }
        ]
      ],
      factsTitle: "Chapter record",
      facts: [["Office", "North road, Greybridge"], ["Current clerk", "Mara Venn"], ["Ledger status", "Incomplete"]],
      related: ["tithe", "famine-winter"]
    },
    "famine-winter": {
      title: "The famine winter",
      category: "Historical event · Greybridge",
      icon: "harvest",
      description: [
        [{ text: "The name given to a season of crop failure and food shortage remembered in the older village records. The surviving accounts disagree about its exact duration." }],
        [
          { text: "The mill bell bears a date associated with this period. The inscription may connect the miller's ledger to the changes in the " },
          { ref: "tithe", label: "guild tithe" }, { text: "." }
        ]
      ],
      factsTitle: "Historical record",
      facts: [["Date", "Inscription incomplete"], ["Cause", "Not established"], ["Recorded impact", "Crop failure"]],
      related: ["guild", "tithe"]
    }
  };

  const icons = {
    spectre: '<path d="M5 20v-4a7 7 0 1 1 14 0v4l-3-2-2 2-2-2-2 2-2-2z"/><path d="M9 11h.01M15 11h.01M9 15c2 1 4 1 6 0"/>',
    ledger: '<path d="M6 4h12v16H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2z"/><path d="M8 8h7M8 12h7M8 16h4"/>',
    water: '<path d="M12 3c-2.5 3.5-6 7.4-6 11a6 6 0 0 0 12 0c0-3.6-3.5-7.5-6-11z"/><path d="M8.5 15c.5 1.3 1.6 2 3 2"/>',
    guild: '<path d="M12 3 20 6v5c0 5-3.2 8-8 10-4.8-2-8-5-8-10V6z"/><path d="M9 10h6M9 13h6"/>',
    harvest: '<path d="M12 21V4M12 9c-3 0-5-2-5-5 3 0 5 2 5 5zM12 13c3 0 5-2 5-5-3 0-5 2-5 5zM12 17c-3 0-5-2-5-5 3 0 5 2 5 5z"/>'
  };

  const card = document.querySelector("[data-reference-card]");
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
  const triggers = [...document.querySelectorAll("[data-entry-trigger]")];
  const states = new Map();
  let activeState = null;

  function makeElement(tag, className, text) {
    const node = document.createElement(tag);
    if (className) node.className = className;
    if (text !== undefined) node.textContent = text;
    return node;
  }

  function cancelClose(state) {
    if (state.closeTimer) {
      clearTimeout(state.closeTimer);
      state.closeTimer = null;
    }
  }

  function positionCard(state) {
    const rect = state.trigger.getBoundingClientRect();
    const margin = 12;
    const width = card.offsetWidth;
    const height = card.offsetHeight;
    const left = Math.max(margin, Math.min(rect.left, window.innerWidth - width - margin));
    let top = rect.bottom + 8;
    if (top + height > window.innerHeight - margin && rect.top > height + 8) {
      top = rect.top - height - 8;
    }
    top = Math.max(margin, Math.min(top, window.innerHeight - height - margin));
    card.style.left = left + "px";
    card.style.top = top + "px";
  }

  function appendParagraph(segments) {
    const paragraph = makeElement("p", "reference-paragraph");
    for (const segment of segments) {
      if (typeof segment === "string" || segment.text !== undefined) {
        paragraph.append(document.createTextNode(typeof segment === "string" ? segment : segment.text));
      } else if (segment.ref && entries[segment.ref]) {
        const link = makeElement("button", "reference-inline", segment.label || entries[segment.ref].title);
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
    icon.innerHTML = '<svg viewBox="0 0 24 24" aria-hidden="true">' + (icons[entry.icon] || icons.ledger) + '</svg>';

    description.replaceChildren();
    entry.description.forEach(appendParagraph);

    factsList.replaceChildren();
    if (entry.facts && entry.facts.length > 0) {
      facts.hidden = false;
      factsTitle.textContent = entry.factsTitle || "Details";
      for (const [label, value] of entry.facts) {
        const row = makeElement("div", "reference-fact");
        row.append(makeElement("dt", "", label), makeElement("dd", "", value));
        factsList.append(row);
      }
    } else {
      facts.hidden = true;
    }

    relatedList.replaceChildren();
    if (entry.related && entry.related.length > 0) {
      related.hidden = false;
      for (const relatedId of entry.related) {
        if (!entries[relatedId]) continue;
        const link = makeElement("button", "", entries[relatedId].title);
        link.type = "button";
        link.dataset.entryRef = relatedId;
        relatedList.append(link);
      }
    } else {
      related.hidden = true;
    }

    const parentId = state.history[state.history.length - 1];
    const backLabel = parentId ? "Return to " + entries[parentId].title : "Go back";
    back.setAttribute("aria-label", backLabel);
    back.title = backLabel;
    back.hidden = state.history.length === 0;
    card.hidden = false;
    positionCard(state);
    if (focusTitle) title.focus({ preventScroll: true });
  }

  function closeState(state, restoreFocus, suppressFocus) {
    cancelClose(state);
    card.hidden = true;
    state.trigger.setAttribute("aria-expanded", "false");
    state.pinned = false;
    state.history.length = 0;
    state.currentId = null;
    if (suppressFocus) state.suppressFocus = true;
    if (activeState === state) activeState = null;
    if (restoreFocus) state.trigger.focus({ preventScroll: true });
  }

  function openState(state, pin) {
    if (activeState && activeState !== state) closeState(activeState, false, false);
    activeState = state;
    cancelClose(state);
    if (!state.currentId) {
      state.currentId = state.trigger.dataset.entryTrigger;
      state.history.length = 0;
      renderEntry(state, state.currentId, false);
    } else {
      card.hidden = false;
      positionCard(state);
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
    if (!state.currentId) state.currentId = state.trigger.dataset.entryTrigger;
    state.history.push(state.currentId);
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

  for (const trigger of triggers) {
    const state = {
      trigger,
      currentId: null,
      history: [],
      pinned: false,
      suppressFocus: false,
      pointerOnTrigger: false,
      pointerOnCard: false,
      closeTimer: null
    };

    states.set(trigger, state);

    trigger.addEventListener("pointerenter", () => {
      state.pointerOnTrigger = true;
      openState(state, false);
    });
    trigger.addEventListener("pointerleave", () => {
      state.pointerOnTrigger = false;
      scheduleClose(state);
    });
    trigger.addEventListener("focus", () => {
      if (!state.suppressFocus) openState(state, false);
    });
    trigger.addEventListener("blur", () => {
      setTimeout(() => {
        if (document.activeElement !== trigger && !card.contains(document.activeElement)) {
          state.suppressFocus = false;
        }
      }, 0);
    });
    trigger.addEventListener("keydown", (event) => {
      if (event.key !== "Tab" || event.shiftKey || activeState !== state || card.hidden) return;
      const firstControl = card.querySelector("button:not([hidden])");
      if (firstControl) {
        event.preventDefault();
        firstControl.focus();
      }
    });

    trigger.addEventListener("click", (event) => {
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
    if (activeState) {
      activeState.pointerOnCard = true;
      cancelClose(activeState);
    }
  });
  card.addEventListener("pointerleave", () => {
    if (activeState) {
      activeState.pointerOnCard = false;
      scheduleClose(activeState);
    }
  });

  card.addEventListener("click", (event) => {
    const reference = event.target.closest("[data-entry-ref]");
    if (reference && card.contains(reference)) {
      navigate(activeState, reference.dataset.entryRef);
      return;
    }
    if (event.target.closest("[data-reference-back]")) {
      if (activeState) goBack(activeState);
      return;
    }
    if (event.target.closest("[data-reference-close]") && activeState) {
      closeState(activeState, true, true);
    }
  });

  document.addEventListener("focusin", (event) => {
    if (activeState && (event.target === activeState.trigger || card.contains(event.target))) {
      cancelClose(activeState);
      return;
    }
    const trigger = event.target.closest && event.target.closest("[data-entry-trigger]");
    if (trigger) {
      const state = states.get(trigger);
      if (state && !state.suppressFocus) openState(state, false);
    } else if (activeState) {
      const state = activeState;
      scheduleClose(state);
    }
  });

  document.addEventListener("pointerdown", (event) => {
    if (activeState && event.target !== activeState.trigger && !card.contains(event.target)) {
      closeState(activeState, false, false);
    }
  });

  card.addEventListener("keydown", (event) => {
    if (event.key === "Tab" && event.shiftKey) {
      const firstControl = card.querySelector("button:not([hidden])");
      if (event.target === title || event.target === firstControl) {
        event.preventDefault();
        if (activeState) activeState.trigger.focus();
        return;
      }
    }
    if (event.key !== "Escape" || !activeState) return;
    const state = activeState;
    if (state.history.length > 0) {
      goBack(state);
    } else {
      closeState(state, true, true);
    }
    event.preventDefault();
    event.stopPropagation();
  });

  document.addEventListener("keydown", (event) => {
    if (event.key !== "Escape" || !activeState) return;
    const state = activeState;
    if (state.history.length > 0) {
      goBack(state);
    } else {
      closeState(state, true, true);
    }
    event.preventDefault();
    event.stopPropagation();
  });

  window.addEventListener("resize", () => {
    if (activeState && !card.hidden) positionCard(activeState);
  });
  window.addEventListener("scroll", () => {
    if (activeState && !card.hidden) positionCard(activeState);
  }, true);

  document.querySelectorAll("[data-demo-action]").forEach((button) => {
    button.addEventListener("click", () => {
      const feedback = document.querySelector("#action-feedback");
      if (feedback) feedback.textContent = button.dataset.demoAction;
    });
  });
})();