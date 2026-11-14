// Landing page: feature switcher and an OS-aware primary download.
const tabs = [...document.querySelectorAll<HTMLButtonElement>(".switch__tabs [role=tab]")];
function select(t: HTMLButtonElement) {
  for (const o of tabs) {
    const on = o === t;
    o.setAttribute("aria-selected", String(on));
    o.tabIndex = on ? 0 : -1;
    document.getElementById(o.getAttribute("aria-controls")!)!.hidden = !on;
  }
}
tabs.forEach((t, i) => {
  t.addEventListener("click", () => select(t));
  t.addEventListener("keydown", (e) => {
    const d = e.key === "ArrowDown" || e.key === "ArrowRight" ? 1 : e.key === "ArrowUp" || e.key === "ArrowLeft" ? -1 : 0;
    if (!d) return;
    e.preventDefault();
    const n = tabs[(i + d + tabs.length) % tabs.length];
    n.focus();
    select(n);
  });
});

const ua = navigator.userAgent;
const os = /Android/i.test(ua) ? "android" : /iPhone|iPad|iPod/i.test(ua) ? "ios" : /Mac/i.test(ua) ? "mac" : /Win/i.test(ua) ? "win" : /Linux/i.test(ua) ? "linux" : "";
const names: Record<string, string> = { mac: "Download for Mac", win: "Download for Windows", linux: "Download for Linux", android: "Download for Android" };
const main = document.getElementById("dl-main") as HTMLAnchorElement | null;
if (main && os === "ios") {
  main.textContent = "Open the app";
  main.href = "app/index.html";
} else if (main && os && names[os]) {
  const link = document.querySelector<HTMLAnchorElement>(`.dl a[data-os="${os}"]`);
  if (link) {
    main.textContent = names[os];
    main.href = link.href;
  }
}
