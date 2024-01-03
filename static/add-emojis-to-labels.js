function insertEmojis(body) {
  const selectOptions = document.querySelectorAll("label");
  for (const s of selectOptions) {
    const v = body.emojis.find((e) => e.name == s.getAttribute("value"));
    if (v) {
      let prefix = "";
      for (const emoji of v.emojis) {
        if (parseInt(emoji, 10)) {
          prefix += "&#" + emoji + "; ";
        } else {
          prefix += emoji + " ";
        }
      }
      s.innerHTML = prefix + " " + s.innerText;
    }
  }
}
window.addEventListener("load", () => {
  this.rxjs.fetch
    .fromFetch("/static/assets/emojis.json")
    .pipe(
      this.rxjs.first(),
      this.rxjs.switchMap((res) => res.json()),
      this.rxjs.tap((json) => insertEmojis(json)),
    )
    .subscribe();
});
