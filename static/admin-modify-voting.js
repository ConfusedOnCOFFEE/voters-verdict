function put(formId, route) {
  const criterias = this.votersVerdict.getValuesByFieldSetId("voting-criteria");
  const candidates =
    this.votersVerdict.getValuesByFieldSetId("voting-candidates");
  const body = {
    criterias,
    candidates,
  };
  const ajaxRequest = this.rxjs.ajax.ajax({
    url: route + window.location.search,
    method: "PUT",
    body,
    headers: {
      "Content-Type": "application/json",
      "X-Concafectl": "admin",
      "x-user": "admin",
    },
  });
  return this.votersVerdict.ajax(ajaxRequest, () =>
    alert("Modification done."),
  );
}
window.addEventListener("load", () => {
  const form = document.getElementById("voting-form");
  this.votersVerdict.fromEvent(form, "submit", () =>
    put("voting-form", form.getAttribute("data-route")),
  );
  const btn = document.getElementById("voting-btn");
  const voting = document.getElementById("voting-form").getAttribute("name");
  const inviteCode = document
    .getElementById("voting-btn")
    .getAttribute("data-invite-code");
  const handleClick = () => {
    const params = new URLSearchParams(window.location.search);
    window.location.assign(
      window.location.origin +
        "/votings/" +
        voting +
        "?invite_code=" +
        inviteCode,
    );
  };
  this.votersVerdict.fromEventIntoTap(btn, "click", handleClick);
});
