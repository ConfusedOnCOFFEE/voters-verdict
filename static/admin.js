class AdminPanel {
  #xhrRequest(method, url, loadFn, body, headers) {
    const XHR = new XMLHttpRequest();
    XHR.addEventListener("load", loadFn);
    XHR.addEventListener("error", (event) => {
      alert("Created!");
    });
    XHR.open(method, url);
    if (headers) {
      const headersKeys = Object.keys(headers);
      for (const h of headersKeys) {
        XHR.setRequestHeader(h, headers[h]);
      }
    }
    if (body) {
      XHR.send(JSON.stringify(body));
    } else {
      XHR.send();
    }
  }
  #getValueByElementId(name) {
    const value = document.getElementById(name).value;
    const triedNumber = parseInt(value, 10);
    if (isNaN(triedNumber)) {
      return value;
    } else {
      return triedNumber;
    }
  }
  #getValuesByFieldSetId(name) {
    const checkedValues = [];
    const inputs = document.getElementById(name).querySelectorAll("input");
    for (const input of Array.from(inputs)) {
      if (input.checked) {
        checkedValues.push(input.value.trim());
      }
    }
    return checkedValues;
  }
  post(formId, route) {
    let body;
    let actOnSuccess;
    switch (formId) {
    case "criteria":
      const min = this.#getValueByElementId("criteria-minimum");
      const max = this.#getValueByElementId("criteria-maximum");
      if (min > max || min == max) {
        alert("Minimum needs to be atleast 2 less then maximum.");
        return;
      }
      body = {
        name: this.#getValueByElementId("criteria-name"),
        min,
        max,
        weight: this.#getValueByElementId("criteria-weight"),
      };
      break;
    case "user":
      const userName = this.#getValueByElementId("user-name");
      if (userName) {
      body = {
        label: userName,
        id: userName.toLowerCase().replace('_', '-') ,
        voter:
        this.#getValueByElementId("user-type") === "candidate"
          ? false
          : true,
      };
      }
      break;
    case "voting":
      const expires_at = new Date();
      const stringDate = document.getElementById("ends").value;
      if (!stringDate) {
        return;
      }
        const stringDateArray = stringDate.split("-");
        expires_at.setFullYear(stringDateArray[0]);
        expires_at.setMonth(stringDateArray[1]);
        expires_at.setDate(stringDateArray[2]);
      const criterias = this.#getValuesByFieldSetId("voting-criteria");
      if (criterias.length < 2) {
        alert("Please add criterias!");
        return
      }
      const candidates = this.#getValuesByFieldSetId("voting-candidates");
      if (candidates.length < 2) {
        alert("Please add candidates!");
        return
      }
      body = {
        name: document.getElementById("voting-id").value,
        criterias,
        candidates,
        expires_at: expires_at.toISOString(),
        invite_code: document.getElementById("voting-invite").value,
        styles: {
          background: this.#getValueByElementId("voting-color-bg"),
          font: this.#getValueByElementId("voting-color-font"),
          fields: this.#getValueByElementId("voting-field-bg"),
          selection: this.#getValueByElementId("voting-field-font"),

        }
      };
      actOnSuccess = () => {
        window.location.assign(window.location.origin + "/votings/" + document.getElementById("voting-id").value + "?invite_code=" +  body.invite_code);
      };
      break;
    }
    this.#xhrRequest("POST", route, actOnSuccess, body, {
      "Content-Type": "application/json",
      "X-Concafectl": "admin",
      "x-user": "admin",
    });
  }
}
window.addEventListener("load", () => {
  // Listen on the forms.
  for (const formName of ["criteria", "user", "voting"]) {
    const form = document.getElementById(formName + "-form");
    form.addEventListener("submit", (event) => {
      event.preventDefault();
      event.stopPropagation();
      const panel = new AdminPanel();
      panel.post(formName, form.getAttribute("data-route"));
    });
  }
  // Preview choosen colors
  for (const colorField of ["bg", "font"]) {
    const bg = document.getElementById("voting-color-" + colorField);
    bg.value = Math.floor(Math.random()*16777215).toString(16);
    bg.addEventListener("change", (event) => {
      const visualizer  = document.getElementById("template-test-visualiser").style;
      const value = event.target.value;
      if (event.target.id.indexOf("bg") != -1) {
        visualizer.backgroundColor = value;
      } else {
        visualizer.color = value;
      }
    });
    const font = document.getElementById("voting-field-" + colorField);
    font.value = Math.floor(Math.random()*16777215).toString(16);
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
