window.addEventListener("load", () => {
  const changeEvent = (event, id, candidate) => {
    const el = document.getElementById(id);
    el.hidden = false;
    const appender = [
      `${candidate ? "candidates" : "voters"}`,
      `${candidate ? event.target.value.split("_")[1] : event.target.value}`,
    ].join("/");
    const newSrc = [
      document.getElementById("voting-table-hook").src,
      appender,
    ].join("/");
    if (el.localName === "iframe" && newSrc !== el.src) {
      el.src = newSrc;
    }
    if (el.localName === "a" && newSrc !== el.src) {
      el.href = newSrc;
    }
  };
  const userHooks = ["user-table-hook", "user-link-hook"];
  const canditeHooks = ["candidate-table-hook", "candidate-link-hook"];
  for (const id of userHooks.concat(canditeHooks)) {
    const foundEl = document.getElementById(id);
    foundEl.hidden = true;
    if (id.startsWith("user")) {
      const userEl = document.getElementById("name");
      userEl.addEventListener("change", (event) =>
        changeEvent(event, id, false),
      );
    } else {
      const candidateEl = document.getElementById("candidate");
      candidateEl.addEventListener("change", (event) =>
        changeEvent(event, id, true),
      );
    }
  }
});
