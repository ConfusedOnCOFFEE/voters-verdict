function post(formId, route) {
  let body;
  switch (formId) {
    case "criteria":
      const min = this.votersVerdict.getValueByElementId("criteria-minimum");
      const max = this.votersVerdict.getValueByElementId("criteria-maximum");
      if (min > max || min == max) {
        alert("Minimum needs to be atleast 2 less then maximum.");
        return;
      }
      body = {
        name: this.votersVerdict.getValueByElementId("criteria-name"),
        min,
        max,
        weight: this.votersVerdict.getValueByElementId("criteria-weight"),
      };
      break;
    case "user":
      const userName = this.votersVerdict.getValueByElementId("user-name");
      if (userName) {
        body = {
          label: userName,
          id: userName.toLowerCase().replace("_", "-"),
          voter:
            this.votersVerdict.getValueByElementId("user-type") === "candidate"
              ? false
              : true,
        };
      }
      break;
  }
  return this.votersVerdict.ajax(
    this.rxjs.ajax.ajax({
      url: route,
      method: "POST",
      body,
      headers: {
        "Content-Type": "application/json",
        "X-Concafectl": "admin",
        "x-user": "admin",
      },
    }),
  );
}
window.addEventListener("load", () => {
  for (const formName of ["criteria", "user"]) {
    const form = document.getElementById(formName + "-form");
    const fromForm = this.votersVerdict.fromEvent(form, "submit", () =>
      post(formName, form.getAttribute("data-route")),
    );
  }
});
