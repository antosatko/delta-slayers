(() => {
  const tips = [...document.querySelectorAll("[data-tip]")];

  const closeTips = (except = null) => {
    for (const tip of tips) {
      if (tip !== except) {
        tip.setAttribute("aria-expanded", "false");
        tip.classList.remove("is-dismissed");
      }
    }
  };

  for (const tip of tips) {
    tip.addEventListener("click", () => {
      const willOpen = tip.getAttribute("aria-expanded") !== "true";
      closeTips(tip);
      tip.setAttribute("aria-expanded", String(willOpen));
      tip.classList.remove("is-dismissed");
    });

    tip.addEventListener("focus", () => tip.classList.remove("is-dismissed"));
    tip.addEventListener("mouseleave", () => tip.classList.remove("is-dismissed"));
    tip.addEventListener("blur", () => {
      tip.setAttribute("aria-expanded", "false");
      tip.classList.remove("is-dismissed");
    });

    tip.addEventListener("keydown", (event) => {
      if (event.key === "Escape") {
        tip.setAttribute("aria-expanded", "false");
        tip.classList.add("is-dismissed");
        tip.blur();
        event.stopPropagation();
      }
    });
  }

  document.addEventListener("click", (event) => {
    if (!event.target.closest("[data-tip]")) closeTips();
  });

  document.querySelectorAll("[data-demo-action]").forEach((button) => {
    button.addEventListener("click", () => {
      const feedback = document.querySelector("#action-feedback");
      if (feedback) feedback.textContent = button.dataset.demoAction;
    });
  });
})();
