(() => {
  const palettes = new Set(["slate","carbon","indigo","jade","ocean","copper","plum"]);
  const key = "delta-slayers-ui-palette";
  const root = document.documentElement;
  const selectors = [...document.querySelectorAll("[data-palette-select]")];
  let selected = "slate";
  try { const saved = localStorage.getItem(key); if (palettes.has(saved)) selected = saved; } catch (_) {}
  function apply(name, persist = false) {
    if (!palettes.has(name)) return;
    root.dataset.palette = name;
    selectors.forEach(select => { select.value = name; });
    const meta = document.querySelector('meta[name="theme-color"]');
    if (meta) {
      const page = getComputedStyle(root).getPropertyValue("--page").trim();
      if (page) meta.content = page;
    }
    if (persist) { try { localStorage.setItem(key, name); } catch (_) {} }
  }
  selectors.forEach(select => select.addEventListener("change", () => apply(select.value, true)));
  apply(selected);
})();
