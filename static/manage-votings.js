class ManageVotings {
  #xhrRequest(method, url, loadFn) {
    const XHR = new XMLHttpRequest();
    XHR.addEventListener("load", loadFn);
    XHR.addEventListener("error", (event) => {
      alert("Could not be clsoed sorry.");
    });
    XHR.open(method, url);
    XHR.send();
  }
  get_votings() {
    for (const button of Array.from(document.querySelectorAll("button"))) {
      const form = document.getElementById(button.id);
      form.addEventListener("click", (event) => {
        event.preventDefault();
        event.stopPropagation();
        const actOnSuccess = () => alert("Voting is now closed");
        const route = [
          "/api",
          "v1",
          "votings",
          event.target.id,
          "close",
          window.location.search
        ].join("/");
        this.#xhrRequest("PUT", route, actOnSuccess);
      });
    }
  }
}
window.addEventListener("load", () => {
  const b = new ManageVotings();
  b.get_votings();
});
