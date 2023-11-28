class EmojiInserter {
  #xhrRequest(method, url, loadFn, body, headers) {
    const XHR = new XMLHttpRequest();
    XHR.addEventListener("load", loadFn);
    XHR.addEventListener("error", (event) => {
      alert("Thanks for voting!");
    });
    XHR.open(method, url);
    XHR.send();
  }
  getEmojis(url) {
    this.#xhrRequest("GET", url, (event) => {
      this.#insertEmojis(event.target.response);
    });
  }
  #insertEmojis(resp) {
    const emojiMap = JSON.parse(resp);
    const selectOptions = document.querySelectorAll("label");
    for (const s of selectOptions) {
      // const v = emojiMap.get(s.getAttribute("value"));
      const v = emojiMap.emojis.find((e) => e.name == s.getAttribute("value"));
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
}
window.addEventListener("load", () => {
  const e = new EmojiInserter();
  e.getEmojis("/static/emojis.json");
});
