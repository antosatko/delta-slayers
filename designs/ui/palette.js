(() => {
  const paletteIds = new Set(["carbon","harbor","citrine","moss","cobalt","night-market"]);
  const storageKey = "delta-slayers-ui-palette";
  const root = document.documentElement;
  const selectors = [...document.querySelectorAll("[data-palette-select]")];
  const choices = [...document.querySelectorAll("[data-palette-choice]")];
  let selected = "carbon";
  try {
    const saved = localStorage.getItem(storageKey);
    if (paletteIds.has(saved)) selected = saved;
  } catch (_) {}

  function applyPalette(name, persist = false) {
    if (!paletteIds.has(name)) return;
    root.dataset.palette = name;
    selectors.forEach(select => { select.value = name; });
    choices.forEach(button => {
      const active = button.dataset.paletteChoice === name;
      button.setAttribute("aria-pressed", String(active));
    });
    const meta = document.querySelector('meta[name="theme-color"]');
    if (meta) {
      const page = getComputedStyle(root).getPropertyValue("--page").trim();
      if (page) meta.content = page;
    }
    if (persist) {
      try { localStorage.setItem(storageKey, name); } catch (_) {}
    }
  }

  selectors.forEach(select => select.addEventListener("change", () => applyPalette(select.value, true)));
  choices.forEach(button => button.addEventListener("click", () => applyPalette(button.dataset.paletteChoice, true)));
  applyPalette(selected);
})();
