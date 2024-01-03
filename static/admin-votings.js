function post(formId, route) {
  let body;
  let actOnSuccess;
  const expires_at = new Date();
  const stringDate = document.getElementById("ends").value;
  if (!stringDate) {
    return;
  }
  const stringDateArray = stringDate.split("-");
  expires_at.setFullYear(stringDateArray[0]);
  expires_at.setMonth(stringDateArray[1]);
  expires_at.setDate(stringDateArray[2]);
  const criterias = this.votersVerdict.getValuesByFieldSetId("voting-criteria");
  if (criterias.length < 2) {
    alert("Please add criterias!");
    return;
  }
  const candidates =
    this.votersVerdict.getValuesByFieldSetId("voting-candidates");
  if (candidates.length < 2) {
    alert("Please add candidates!");
    return;
  }
  body = {
    name: document.getElementById("voting-id").value,
    criterias,
    candidates,
    expires_at: expires_at.toISOString(),
    invite_code: document.getElementById("voting-invite").value,
    styles: {
      background: this.votersVerdict.getValueByElementId("voting-color-bg"),
      font: this.votersVerdict.getValueByElementId("voting-color-font"),
      fields: this.votersVerdict.getValueByElementId("voting-field-bg"),
      selection: this.votersVerdict.getValueByElementId("voting-field-font"),
    },
  };
  actOnSuccess = () => {
    window.location.assign(
      window.location.origin +
        "/votings/" +
        document.getElementById("voting-id").value +
        "?invite_code=" +
        body.invite_code,
    );
  };
  const ajax = this.rxjs.ajax.ajax({
    method: "POST",
    url: route,
    body,
    headers: {
      "Content-Type": "application/json",
      "X-Concafectl": "admin",
      "x-user": "admin",
    },
  });
  return this.votersVerdict.ajax(ajax, actOnSuccess);
}
window.addEventListener("load", () => {
  const form = document.getElementById("voting-form");
  this.votersVerdict.fromEvent(form, "submit", () =>
    post("voting-form", form.getAttribute("data-route")),
  );
  for (const colorField of ["bg", "font"]) {
    const bg = document.getElementById("voting-color-" + colorField);
    bg.value = Math.floor(Math.random() * 16777215).toString(16);
    bg.addEventListener("change", (event) => {
      const visualizer = document.getElementById(
        "template-test-visualiser",
      ).style;
      const value = event.target.value;
      if (event.target.id.indexOf("bg") != -1) {
        visualizer.backgroundColor = value;
      } else {
        visualizer.color = value;
      }
    });
    const font = document.getElementById("voting-field-" + colorField);
    font.value = Math.floor(Math.random() * 16777215).toString(16);
    font.addEventListener("change", (event) => {
      const value = event.target.value;
      const criteriaTestStyle = document.getElementById("criteria-test").style;
      const inputTestStyle = document.getElementById("input-test").style;
      if (event.target.id.indexOf("font") != -1) {
        criteriaTestStyle.color = value;
        inputTestStyle.color = value;
      } else {
        inputTestStyle.color = value;
        criteriaTestStyle.backgroundColor = value;
      }
    });
  }
});
