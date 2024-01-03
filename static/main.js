function postVote() {
  const selections = document.querySelectorAll("select");
  const selected_values = Array.from(selections)
    .filter((s) => s.id !== "name" && s.id !== "candidate")
    .filter((s) => !isNaN(parseInt(s.value, 10)))
    .map((s) => ({
      name: s.id,
      point: parseInt(s.value, 10),
    }));
  const user = document.getElementById("name").value;
  const body = {
    voter: user,
    candidate: Array.from(selections)
      .find((s) => s.id === "candidate")
      .value.split("_")[1],
    votes: selected_values,
    notes: null,
  };
  if (document.getElementById("notes").value != "") {
    body.notes = document.getElementById("notes").value;
  }
  let votingName = document
    .getElementsByTagName("body")[0]
    .getAttribute("data-voting");
  const searchParams = new URLSearchParams(window.location.search);
  let invite_code;
  if (searchParams.has("invite_code")) {
    invite_code = searchParams.get("invite_code");
  } else {
    alert(
      "Messed up invite_code. Please restart the window and provide a new code",
    );
    return;
  }

  return this.votersVerdict.ajax(
    this.rxjs.ajax.ajax({
      url: [window.location.origin, "api/v1/ballots", votingName].join("/"),
      method: "POST",
      body,
      headers: {
        "Content-Type": "application/json",
        "X-concafe": document.querySelector("h1").innerText,
        "x-concafe-user": user,
        "x-concafe-invite-code": invite_code,
      },
    }),
    () =>
      window.location.assign(
        [window.location.origin, "ballots", votingName.toLowerCase()].join("/"),
      ),
  );
}
window.addEventListener("load", () => {
  const form = document.querySelector("form");
  this.votersVerdict.fromEvent(form, "submit", postVote);
});
