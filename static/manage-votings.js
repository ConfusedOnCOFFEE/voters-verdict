function get_votings() {
  const buttons = Array.from(document.querySelectorAll("button"));
  for (const button of buttons) {
    const form = document.getElementById(button.id);
    const ajaxRequest = this.rxjs.ajax.ajax({
      url: [
        "/api",
        "v1",
        "votings",
        button.id,
        "close",
        window.location.search,
      ].join("/"),
      method: "PUT",
    });
    const handleClick = () =>
      this.votersVerdict.ajax(ajaxRequest, () =>
        window.alert("Voting is now closed."),
      );
    this.votersVerdict.fromEvent(form, "click", handleClick);
  }
}
window.addEventListener("load", () => {
  get_votings();
});
