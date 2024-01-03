(function (root, factory) {
  if (typeof define === "function" && define.amd) {
    define(["dependency"], factory);
  } else if (typeof module === "object" && module.exports) {
    module.exports = factory(require("dependency"));
  } else {
    root.votersVerdict = factory(root.dependency);
  }
})(typeof self !== "undefined" ? self : this, function (dependency) {
  "use strict";
  function getValueByElementId(name) {
    const value = document.getElementById(name).value;
    const triedNumber = parseInt(value, 10);
    if (isNaN(triedNumber)) {
      return value;
    } else {
      return triedNumber;
    }
  }
  function getValuesByFieldSetId(name) {
    const checkedValues = [];
    const inputs = document.getElementById(name).querySelectorAll("input");
    for (const input of Array.from(inputs)) {
      if (input.checked) {
        checkedValues.push(input.value.trim());
      }
    }
    return checkedValues;
  }
  function ajax(ajaxRequest, mapFn) {
    return ajaxRequest.pipe(
      globalThis.rxjs.tap(() => tap(mapFn)),
      globalThis.rxjs.catchError((err) => {
        console.error(JSON.stringify(err));
        window.alert("Something went wrong");
        return globalThis.rxjs.EMPTY;
      }),
    );
  }
  function fromEventWrapper(event, eventName, httpRequest) {
    const fromForm = globalThis.rxjs.fromEvent(event, eventName);
    fromForm
      .pipe(
        globalThis.rxjs.tap(cleanupEvent),
        globalThis.rxjs.switchMap(() => {
          if (httpRequest) {
            return httpRequest();
          }
          return globalThis.rxjs.EMPTY;
        }),
      )
      .subscribe();
  }

  function cleanupEvent(event) {
    event.preventDefault();
    event.stopPropagation();
  }
  function tap(tapFn) {
    if (tapFn) {
      tapFn();
    }
  }
  function fromEventIntoTap(event, eventName, sideEffectFn) {
    const fromForm = globalThis.rxjs.fromEvent(event, eventName);
    fromForm
      .pipe(
        globalThis.rxjs.first(),
        globalThis.rxjs.tap(cleanupEvent),
        globalThis.rxjs.tap(() => tap(sideEffectFn)),
      )
      .subscribe();
  }
  return {
    ajax,
    fromEvent: fromEventWrapper,
    fromEventIntoTap,
    getValuesByFieldSetId,
    getValueByElementId,
  };
});

window.addEventListener("load", () => {
  const link = document.getElementById("link-to-next-page");
  if (link) {
    link.href += window.location.search;
  }
});
