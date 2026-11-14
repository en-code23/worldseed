import { loadCore, WorldHandle, build_seed, seed_catalog } from "./core";
import * as desk from "./bridge";
import { MapView, kindColorVar } from "./viewer";
import type { Analysis, Goals, Hint, Install, Poi, Secret, Special } from "./types";

const $ = <T extends HTMLElement = HTMLElement>(id: string) => document.getElementById(id) as T;
const el = <K extends keyof HTMLElementTagNameMap>(tag: K, props: Partial<HTMLElementTagNameMap[K]> & Record<string, unknown> = {}, ...kids: (Node | string)[]) => {
  const e = document.createElement(tag);
  for (const [k, v] of Object.entries(props)) {
    if (k.startsWith("data-") || k.startsWith("aria-")) e.setAttribute(k, String(v));
    else (e as any)[k] = v;
  }
  e.append(...kids);
  return e;
};

if (desk.isDesktop) document.body.classList.add("is-desktop");

/* ---------- versions ---------- */
const VERSIONS = ["1.4.5.8", "1.4.5.6", "1.4.5.0", "1.4.4.9", "1.4.3.6", "1.4.2.3", "1.4.1.2", "1.4.0.5"];
const vnum = (v: string) => v.split(".").map(Number).reduce((a, n, i) => a + n * Math.pow(1000, 4 - i), 0);
const state = {
  version: VERSIONS[0],
  install: null as Install | null,
  size: 2,
  difficulty: 1,
  evil: 1,
  biasOn: false,
  bias: 70,
  rolledEvil: 0,
  specials: new Set<string>(),
  secrets: new Set<string>(),
  catalog: { specials: [] as Special[], secrets: [] as Secret[] },
};
const fullFormat = () => vnum(state.version) >= vnum("1.4.5.0");
const availableSince = (since: string) => vnum(since) <= vnum(state.version);

function toast(msg: string) {
  const t = $("toast");
  t.textContent = msg;
  t.hidden = false;
  clearTimeout((t as any)._h);
  (t as any)._h = setTimeout(() => (t.hidden = true), 3200);
}

/* ---------- tabs ---------- */
const tabs = [...document.querySelectorAll<HTMLButtonElement>(".tabs [role=tab]")];
function showTab(id: string) {
  for (const t of tabs) {
    const on = t.id === id;
    t.setAttribute("aria-selected", String(on));
    t.tabIndex = on ? 0 : -1;
    $(t.getAttribute("aria-controls")!).hidden = !on;
  }
  if (id === "tab-map") view.draw();
}
tabs.forEach((t, i) => {
  t.addEventListener("click", () => showTab(t.id));
  t.addEventListener("keydown", (e) => {
    const d = e.key === "ArrowRight" ? 1 : e.key === "ArrowLeft" ? -1 : 0;
    if (!d) return;
    const n = tabs[(i + d + tabs.length) % tabs.length];
    n.focus();
    showTab(n.id);
  });
});

/* ---------- map ---------- */
const view = new MapView($<HTMLCanvasElement>("map"));
(window as any).worldseed = { view }; // handy for debugging and screenshots
let world: WorldHandle | null = null;
let analysis: Analysis | null = null;
let currentPath: string | null = null;

const KIND_LABEL: Record<string, string> = {
  spawn: "Spawn", dungeon: "Dungeon", temple: "Jungle temple", lihzahrd_altar: "Lihzahrd altar", pyramid: "Pyramids",
  sky_island: "Sky islands", shimmer: "Shimmer", underworld: "Underworld", ocean: "Oceans", sword_shrine: "Sword shrines",
  chest: "Chests", orb: "Shadow orbs", heart: "Crimson hearts", altar: "Altars", life_crystal: "Life crystals",
  larva: "Queen Bee larvae", hive: "Bee hives", living_tree: "Living trees", marble: "Marble caves",
  granite: "Granite caves", mushroom: "Mushroom biomes", hellforge: "Hellforges",
};
const DEFAULT_OFF = new Set(["hellforge", "marble", "granite", "mushroom", "hive"]);
const MODES = ["Classic", "Expert", "Master", "Journey"];

function feet(p: { x: number; y: number }) {
  if (!analysis) return "";
  const h = analysis.header;
  const ex = Math.round((p.x - h.width / 2) * 2);
  const dy = Math.round((p.y - h.surface) * 2);
  const horiz = ex === 0 ? "Center" : `${Math.abs(ex)} ft ${ex > 0 ? "east" : "west"}`;
  const vert = dy <= 0 ? `${Math.abs(dy)} ft above surface` : `${dy} ft below`;
  return `${horiz}, ${vert}`;
}

async function openBytes(bytes: Uint8Array, path: string | null = null, shield = false) {
  $("loading").hidden = false;
  $("loading-text").textContent = "Reading the world…";
  await new Promise((r) => setTimeout(r, 30));
  try {
    await loadCore();
    world?.free();
    world = new WorldHandle(bytes);
    analysis = JSON.parse(world.analysis()) as Analysis;
    currentPath = path;
    $("loading-text").textContent = "Drawing the map…";
    await new Promise((r) => setTimeout(r, 10));
    view.setWorld(world.render(15), world.width(), world.height(), analysis.pois);
    fillSide();
    ($<HTMLInputElement>("shield")).checked = shield;
    applyShield();
    $("map-empty").hidden = true;
    showTab("tab-map");
  } catch (e) {
    toast(String((e as Error)?.message ?? e));
  } finally {
    $("loading").hidden = true;
  }
}

function fillSide() {
  if (!analysis || !world) return;
  const { header: h, summary: s } = analysis;
  $("empty-side").hidden = true;
  $("world-side").hidden = false;
  $("w-name").textContent = h.name;
  $("w-seed").textContent = h.seed || "unknown";
  const sizeName = h.width <= 4200 ? "Small" : h.width <= 6400 ? "Medium" : "Large";
  const flags = [h.drunk && "Drunk", h.not_the_bees && "Not the Bees", h.for_the_worthy && "For the Worthy", h.celebration && "Celebration",
    h.the_constant && "The Constant", h.remix && "Remix", h.no_traps && "No Traps", h.zenith && "Zenith", h.skyblock && "Skyblock"].filter(Boolean);
  const facts: [string, string][] = [
    ["Size", `${sizeName}, ${h.width} × ${h.height}`],
    ["Mode", MODES[h.game_mode] ?? "Classic"],
    ["Evil", h.drunk ? "Both" : h.crimson ? "Crimson" : "Corruption"],
    ["Made in", h.game_version || `save v${h.version}`],
    ["Dungeon", `${s.dungeon_side}, ${s.dungeon_distance} tiles from spawn`],
    ["Jungle", s.jungle_side],
    ["Sky islands", String(s.sky_islands)],
    ["Sword shrines", String(s.sword_shrines)],
  ];
  if (flags.length) facts.splice(2, 0, ["Seeds", flags.join(", ")]);
  const dl = $("w-facts");
  dl.replaceChildren(...facts.flatMap(([k, v]) => [el("dt", {}, k), el("dd", {}, v)]));

  // layers
  const counts = new Map<string, number>();
  for (const p of analysis.pois) counts.set(p.kind, (counts.get(p.kind) ?? 0) + 1);
  view.visible = new Set([...counts.keys()].filter((k) => !DEFAULT_OFF.has(k)));
  const layers = $("layers");
  layers.replaceChildren();
  for (const k of Object.keys(KIND_LABEL)) {
    const n = counts.get(k);
    if (!n) continue;
    const cb = el("input", { type: "checkbox", checked: view.visible.has(k) });
    cb.addEventListener("change", () => {
      cb.checked ? view.visible.add(k) : view.visible.delete(k);
      view.draw();
    });
    const sw = el("span", { className: "sw" });
    sw.style.setProperty("--sw", `var(${kindColorVar(k)})`);
    layers.append(el("label", { className: "layer" }, cb, sw, el("span", {}, KIND_LABEL[k]), el("span", { className: "n" }, String(n))));
  }
  const rareOnly = el("input", { type: "checkbox" });
  rareOnly.addEventListener("change", () => {
    rareOnly.checked ? view.visible.add("rare_only") : view.visible.delete("rare_only");
    view.draw();
  });
  layers.append(el("label", { className: "layer" }, rareOnly, el("span", {}, "Only chests with rare loot")));
  renderLoot("");
  renderHints();
}

function renderLoot(q: string) {
  if (!analysis) return;
  const list = $("loot");
  list.replaceChildren();
  const h = analysis.header;
  let rows: { item: string; tier: number; chest: number; place: string; x: number; y: number }[];
  if (q.trim()) {
    const ql = q.trim().toLowerCase();
    rows = [];
    analysis.chests.forEach((c, i) =>
      c.items.forEach((it) => {
        if (it.name.toLowerCase().includes(ql)) rows.push({ item: it.stack > 1 ? `${it.name} × ${it.stack}` : it.name, tier: 0, chest: i, place: c.place, x: c.x, y: c.y });
      }),
    );
    rows = rows.slice(0, 200);
  } else rows = analysis.loot;
  if (!rows.length) {
    list.append(el("li", {}, el("p", { className: "note" }, q ? "No chest holds that item." : "No notable loot in this world.")));
    return;
  }
  for (const r of rows) {
    const b = el("button", {},
      el("span", { className: "it" }, r.item),
      el("span", { className: "dist" }, `${Math.abs(r.x - h.spawn_x)} tiles ${r.x < h.spawn_x ? "W" : "E"}`),
      el("span", { className: "pl" }, `${r.place}, ${analysis!.chests[r.chest]?.kind ?? "chest"}`));
    b.addEventListener("click", () => {
      const poi = analysis!.pois.find((p) => p.kind === "chest" && p.chest === r.chest) ?? null;
      view.visible.add("chest");
      view.flyTo(r.x, r.y, 5);
      view.selected = poi;
      showDetail(poi);
      view.draw();
    });
    list.append(el("li", { className: r.tier >= 2 ? "rare" : "" }, b));
  }
}
$<HTMLInputElement>("loot-search").addEventListener("input", (e) => renderLoot((e.target as HTMLInputElement).value));

function renderHints() {
  if (!world) return;
  const hints = JSON.parse(world.hints()) as Hint[];
  const ol = $("hints");
  ol.replaceChildren();
  for (const h of hints) {
    let shown = 0;
    const li = el("li", {}, el("span", { className: "topic" }, h.topic));
    const btn = el("button", { className: "link" }, "Show a hint");
    btn.addEventListener("click", () => {
      li.insertBefore(el("p", {}, h.levels[shown]), btn);
      shown++;
      if (shown >= h.levels.length) btn.remove();
      else btn.textContent = shown === h.levels.length - 1 ? "Tell me exactly" : "A bit more";
    });
    li.append(btn);
    ol.append(li);
  }
}

function showDetail(p: Poi | null) {
  const d = $("detail");
  if (!p || !analysis) {
    d.hidden = true;
    return;
  }
  const close = el("button", { className: "icon-btn close", "aria-label": "Close" }, "×");
  close.addEventListener("click", () => {
    d.hidden = true;
    view.selected = null;
    view.draw();
  });
  const kids: Node[] = [close, el("h3", {}, p.label), el("p", { className: "note" }, feet(p))];
  if (p.chest >= 0) {
    const c = analysis.chests[p.chest];
    kids.push(el("p", {}, c.place + (c.name ? `, named “${c.name}”` : "")));
    const ul = el("ul");
    for (const it of c.items) {
      const tier = it.name && analysis.loot.some((l) => l.chest === p.chest && l.item === it.name && l.tier >= 2);
      ul.append(el("li", { className: tier ? "rare" : "" }, el("span", {}, it.name), el("span", { className: "num" }, `× ${it.stack}`)));
    }
    if (!c.items.length) ul.append(el("li", {}, "Empty"));
    kids.push(ul);
  } else kids.push(el("p", {}, p.detail));
  d.replaceChildren(...kids);
  d.hidden = false;
}

view.onPick = (p) => showDetail(p);
view.onHover = (wx, wy, poi, px, py) => {
  if (!world) return;
  const tip = $("tip");
  tip.textContent = poi ? `${poi.label}${poi.chest >= 0 && poi.tier ? " (notable loot)" : ""}` : world.describe(wx, wy);
  tip.hidden = !tip.textContent;
  tip.style.left = `${px + 14}px`;
  tip.style.top = `${py + 14}px`;
};
view.onLeave = () => ($("tip").hidden = true);
$("zoom-in").addEventListener("click", () => view.zoomAt(1.5));
$("zoom-out").addEventListener("click", () => view.zoomAt(1 / 1.5));
$("zoom-fit").addEventListener("click", () => view.fit());

function applyShield() {
  const on = $<HTMLInputElement>("shield").checked;
  $("shielded").hidden = !on || !world;
  for (const id of ["layers-block", "loot-block"]) $(id).hidden = on;
  if (on && world) {
    const v = JSON.parse(world.judge("{}")) as { teaser: string[] };
    $("teaser").replaceChildren(...v.teaser.map((t) => el("li", {}, t)));
    $("detail").hidden = true;
  }
}
$("shield").addEventListener("change", applyShield);
$("unshield").addEventListener("click", () => {
  $<HTMLInputElement>("shield").checked = false;
  applyShield();
});

$<HTMLInputElement>("file").addEventListener("change", async (e) => {
  const f = (e.target as HTMLInputElement).files?.[0];
  if (f) openBytes(new Uint8Array(await f.arrayBuffer()));
});
$("sample").addEventListener("click", async () => {
  $("loading").hidden = false;
  try {
    const r = await fetch(new URL("../samples/sample-medium.wld", location.href));
    if (!r.ok) throw new Error("The sample world could not be loaded.");
    await openBytes(new Uint8Array(await r.arrayBuffer()));
  } catch (e) {
    toast(String((e as Error).message ?? e));
    $("loading").hidden = true;
  }
});
if (new URLSearchParams(location.search).has("sample")) queueMicrotask(() => $("sample").click());

let dragDepth = 0;
window.addEventListener("dragenter", (e) => {
  e.preventDefault();
  dragDepth++;
  $("drop").hidden = false;
});
window.addEventListener("dragleave", () => {
  if (--dragDepth <= 0) $("drop").hidden = true;
});
window.addEventListener("dragover", (e) => e.preventDefault());
window.addEventListener("drop", async (e) => {
  e.preventDefault();
  dragDepth = 0;
  $("drop").hidden = true;
  const f = e.dataTransfer?.files?.[0];
  if (f) openBytes(new Uint8Array(await f.arrayBuffer()));
});

/* ---------- create ---------- */
function seg(id: string, options: [number, string][], get: () => number, set: (v: number) => void) {
  const box = $(id);
  const render = () => {
    box.replaceChildren(
      ...options.map(([v, label]) => {
        const b = el("button", { type: "button", "aria-pressed": String(get() === v) }, label);
        b.addEventListener("click", () => {
          set(v);
          render();
          updateSeed();
        });
        return b;
      }),
    );
  };
  render();
  return render;
}
seg("size", [[1, "Small"], [2, "Medium"], [3, "Large"]], () => state.size, (v) => (state.size = v));
seg("difficulty", [[1, "Classic"], [2, "Expert"], [3, "Master"], [4, "Journey"]], () => state.difficulty, (v) => (state.difficulty = v));
const renderEvil = seg("evil", [[1, "Random"], [2, "Corruption"], [3, "Crimson"]], () => state.evil, (v) => {
  state.evil = v;
  if (v !== 1) {
    state.biasOn = false;
    $<HTMLInputElement>("bias-on").checked = false;
    $("bias-row").hidden = true;
  }
});

function rollEvil() {
  state.rolledEvil = Math.random() * 100 < state.bias ? 3 : 2;
}
$("bias-on").addEventListener("change", (e) => {
  state.biasOn = (e.target as HTMLInputElement).checked;
  $("bias-row").hidden = !state.biasOn;
  if (state.biasOn) {
    state.evil = 1;
    renderEvil();
    rollEvil();
  }
  updateSeed();
});
$("bias").addEventListener("input", (e) => {
  state.bias = Number((e.target as HTMLInputElement).value);
  $("bias-out").textContent = state.bias >= 50 ? `${state.bias}% Crimson` : `${100 - state.bias}% Corruption`;
  rollEvil();
  updateSeed();
});

const effectiveEvil = () => (state.biasOn ? state.rolledEvil || 1 : state.evil);

function renderChips() {
  const sp = $("specials");
  sp.replaceChildren();
  for (const s of state.catalog.specials) {
    const ok = availableSince(s.since);
    if (!ok) state.specials.delete(s.key);
    const b = el("button", { type: "button", className: "chip", disabled: !ok, title: ok ? s.about : `Needs ${s.since}`, "aria-pressed": String(state.specials.has(s.key)) }, s.name);
    b.addEventListener("click", () => {
      if (state.specials.has(s.key)) state.specials.delete(s.key);
      else {
        if (!fullFormat()) state.specials.clear(); // older versions: one special seed at a time
        state.specials.add(s.key);
      }
      renderChips();
      updateSeed();
    });
    sp.append(b);
  }
  const se = $("secrets");
  se.replaceChildren();
  $("secrets-set").hidden = !fullFormat();
  for (const s of state.catalog.secrets) {
    const ok = availableSince(s.since);
    if (!ok) state.secrets.delete(s.key);
    const b = el("button", { type: "button", className: "chip", disabled: !ok, title: ok ? s.about : `Needs ${s.since}`, "aria-pressed": String(state.secrets.has(s.key)) }, s.name);
    b.addEventListener("click", () => {
      state.secrets.has(s.key) ? state.secrets.delete(s.key) : state.secrets.add(s.key);
      renderChips();
      updateSeed();
    });
    se.append(b);
  }
}

function randomSeed() {
  return String(1 + Math.floor(Math.random() * 2147483646));
}
$("dice").addEventListener("click", () => {
  $<HTMLInputElement>("seed").value = randomSeed();
  if (state.biasOn) rollEvil();
  updateSeed();
});
$("seed").addEventListener("input", () => updateSeed());

function seedFor(number: string, evil = effectiveEvil()): string {
  if (fullFormat()) {
    return build_seed(JSON.stringify({ size: state.size, difficulty: state.difficulty, evil, specials: [...state.specials], secrets: [...state.secrets], seed: number }));
  }
  const sp = state.catalog.specials.find((s) => state.specials.has(s.key));
  return sp ? sp.code : number;
}

function updateSeed() {
  const n = $<HTMLInputElement>("seed").value.trim() || "random";
  $("seed-out").textContent = seedFor(n);
  const evilName = ["", "Random", "Corruption", "Crimson"][effectiveEvil()];
  $("seed-howto").textContent = fullFormat()
    ? "Paste it into the seed field when creating a world. It carries size, difficulty, evil and every seed you ticked."
    : `In Terraria ${state.version}, pick ${["", "Small", "Medium", "Large"][state.size]}, ${MODES[state.difficulty - 1]} and ${evilName}, then type this seed.`;
}
$("copy-seed").addEventListener("click", async () => {
  if (!$<HTMLInputElement>("seed").value.trim()) {
    $<HTMLInputElement>("seed").value = randomSeed();
    updateSeed();
  }
  await navigator.clipboard.writeText($("seed-out").textContent ?? "");
  toast("Seed copied");
});

/* ---------- desktop: install + generate ---------- */
function installChip() {
  const c = $("install-chip");
  if (!desk.isDesktop) return;
  c.hidden = false;
  const i = state.install;
  c.dataset.ok = String(!!i?.found);
  c.textContent = i?.found ? `Terraria ${i.version || ""} found` : "Find Terraria";
  c.title = i?.found ? i.root : "Choose the folder Terraria is installed in";
  if (i?.found) $("worlds-hint").textContent = `Your worlds live in ${i.worlds_dir}`;
}
$("install-chip").addEventListener("click", async () => {
  const p = await desk.pickFolder();
  if (!p) return;
  const i = await desk.setInstall(p);
  if (!i.found) toast("No TerrariaServer in that folder");
  else state.install = i;
  installChip();
});

function versionMismatch(): boolean {
  const iv = state.install?.version;
  if (iv && iv !== state.version) {
    toast(`Your Terraria is ${iv}. Switch it to ${state.version} in Steam (Properties, Betas), then pick the folder again.`);
    return true;
  }
  return false;
}

async function genOnce(install: boolean) {
  if (!state.install?.found) return toast("Terraria was not found. Choose its folder first.");
  if (versionMismatch()) return;
  const seedInput = $<HTMLInputElement>("seed");
  if (!seedInput.value.trim()) seedInput.value = randomSeed();
  if (state.biasOn) rollEvil();
  updateSeed();
  const name = $<HTMLInputElement>("wname").value.trim() || "My World";
  const prog = $("gen-progress");
  prog.hidden = false;
  const buttons = [$<HTMLButtonElement>("gen-preview"), $<HTMLButtonElement>("gen-install")];
  buttons.forEach((b) => (b.disabled = true));
  const off = await desk.on<{ pct: number; step: string }>("gen-progress", (p) => {
    $<HTMLProgressElement>("gen-bar").value = p.pct;
    $("gen-step").textContent = `${p.step} (${p.pct.toFixed(0)}%)`;
  });
  try {
    const path = await desk.generate({ size: state.size, difficulty: state.difficulty, evil: effectiveEvil(), name, seed: $("seed-out").textContent ?? "", job: "one" });
    if (install) {
      const dest = await desk.installWorld(path, name);
      toast(`Saved to your Terraria worlds: ${dest.split(/[\\/]/).pop()}`);
    } else {
      await openBytes(await desk.readWorld(path), path);
      if (currentPath) toast("World ready. Put it in Terraria from the Map tab whenever you like.");
    }
  } catch (e) {
    if (String(e) !== "cancelled") toast(String(e));
  } finally {
    off();
    prog.hidden = true;
    buttons.forEach((b) => (b.disabled = false));
  }
}
$("gen-preview").addEventListener("click", () => genOnce(false));
$("gen-install").addEventListener("click", () => genOnce(true));
$("gen-cancel").addEventListener("click", () => desk.cancel());

/* ---------- find ---------- */
const RARE_PICKS = ["Fledgling Wings", "Enchanted Sword", "Seaweed", "Flower Boots", "Living Loom", "Lava Charm", "Sandstorm in a Bottle",
  "Flying Carpet", "Pharaoh's Mask", "Fiberglass Fishing Pole", "Snowball Cannon", "Bast Statue", "Sky Mill", "Water Walking Boots",
  "Shoe Spikes", "Band of Regeneration", "Extractinator", "Ice Skates", "Blizzard in a Bottle", "Flurry Boots", "Feral Claws"];
const SKY_PICKS = ["Fledgling Wings", "Starfury", "Lucky Horseshoe", "Shiny Red Balloon", "Celestial Magnet", "Sky Mill"];
const SHARED_PICKS = ["Hermes Boots", "Cloud in a Bottle", "Magic Mirror", "Ice Mirror", "Band of Regeneration", "Aglet", "Climbing Claws", "Shoe Spikes", "Lucky Horseshoe", "Shiny Red Balloon"];

interface Preset { key: string; name: string; about: string; goals: Goals; suggest?: string }
const PRESETS: Preset[] = [
  { key: "lucky", name: "Lucky explorer", about: "A world with treasure worth finding. You learn what, never where.", goals: { items: ["Fledgling Wings"], sword_shrine: true } },
  { key: "speed", name: "Speedrun", about: "Short trips: sword shrine and dungeon near spawn.", goals: { sword_shrine: true, max_sword_distance: 500, max_dungeon_distance: 1000 } },
  { key: "challenge", name: "Challenge", about: "Evil at your doorstep, the dungeon far away, no free sword.", goals: { max_evil_distance: 200, min_dungeon_distance: 1300 }, suggest: "Try it with For the Worthy and Master." },
  { key: "multi", name: "Multiplayer", about: "Enough good loot for everyone on the server.", goals: { players: 4, shared_items: ["Hermes Boots", "Cloud in a Bottle", "Magic Mirror"], min_sword_shrines: 2, min_sky_islands: 4 } },
  { key: "custom", name: "Custom", about: "Pick exactly what the world must have.", goals: {} },
];
let goals: Goals = structuredClone(PRESETS[0].goals);
let presetKey = "lucky";

function renderPresets() {
  const box = $("presets");
  box.replaceChildren(
    ...PRESETS.map((p) => {
      const b = el("button", { type: "button", className: "preset", "aria-pressed": String(p.key === presetKey) }, el("strong", {}, p.name), el("span", {}, p.about));
      b.addEventListener("click", () => {
        presetKey = p.key;
        goals = structuredClone(p.goals);
        if (p.suggest) toast(p.suggest);
        renderPresets();
        renderGoals();
      });
      return b;
    }),
  );
}

function chipList(title: string, picks: string[], arr: string[]) {
  const set = el("fieldset", {}, el("legend", {}, title));
  const box = el("div", { className: "chips" });
  for (const n of picks) {
    const b = el("button", { type: "button", className: "chip", "aria-pressed": String(arr.includes(n)) }, n);
    b.addEventListener("click", () => {
      const i = arr.indexOf(n);
      i >= 0 ? arr.splice(i, 1) : arr.push(n);
      b.setAttribute("aria-pressed", String(i < 0));
    });
    box.append(b);
  }
  set.append(box);
  return set;
}
function numField(label: string, key: keyof Goals, hint = "") {
  const inp = el("input", { type: "number", min: "0", value: String((goals[key] as number) || ""), placeholder: "Any" });
  inp.addEventListener("input", () => ((goals as any)[key] = Number(inp.value) || 0));
  return el("label", { className: "field" }, el("span", {}, label), inp, ...(hint ? [el("span", { className: "note" }, hint)] : []));
}
function checkField(label: string, key: keyof Goals) {
  const cb = el("input", { type: "checkbox", checked: !!goals[key] });
  cb.addEventListener("change", () => ((goals as any)[key] = cb.checked));
  return el("label", { className: "switch" }, cb, el("span", {}, label));
}

function renderGoals() {
  goals.items ??= [];
  goals.sky_items ??= [];
  goals.shared_items ??= [];
  const box = el("div", { className: "goals" });
  box.append(
    chipList("Must be in some chest", RARE_PICKS, goals.items),
    chipList("Must be on a sky island", SKY_PICKS, goals.sky_items),
    el("div", { className: "row" }, checkField("Enchanted Sword shrine", "sword_shrine"), checkField("Desert pyramid", "pyramid")),
    el("div", { className: "grid2" },
      numField("At least this many sky islands", "min_sky_islands"),
      numField("Sword shrine within (tiles of spawn)", "max_sword_distance"),
      numField("Dungeon within (tiles)", "max_dungeon_distance"),
      numField("Dungeon at least (tiles)", "min_dungeon_distance"),
      numField("Evil within (tiles)", "max_evil_distance", "Small numbers make a harder start."),
      numField("Evil at least (tiles)", "min_evil_distance"),
    ),
  );
  if (presetKey === "multi" || presetKey === "custom") {
    box.append(
      el("div", { className: "grid2" }, numField("Players", "players", "Each item below must exist once per player."), numField("Sword shrines at least", "min_sword_shrines")),
      chipList("One for every player", SHARED_PICKS, goals.shared_items),
    );
  }
  $("goal-editor").replaceChildren(box);
}

async function runFind() {
  if (!state.install?.found) return toast("Terraria was not found. Choose its folder first.");
  if (versionMismatch()) return;
  const go = $<HTMLButtonElement>("find-go");
  go.disabled = true;
  $("hits").replaceChildren();
  $("find-progress").hidden = false;
  const attempts = Number($<HTMLInputElement>("attempts").value) || 40;
  $<HTMLProgressElement>("find-bar").value = 0;
  $("find-text").textContent = "Starting Terraria's generator…";
  let found = 0;
  const offs = [
    await desk.on<{ tried: number; hits: number; attempts: number; last_ms: number }>("find-tick", (t) => {
      $<HTMLProgressElement>("find-bar").value = (t.tried / t.attempts) * 100;
      $("find-text").textContent = `${t.tried} of ${t.attempts} worlds checked, ${t.hits} match${t.hits === 1 ? "" : "es"}. About ${(t.last_ms / 1000).toFixed(0)} s per world.`;
    }),
    await desk.on<{ seed: string; path: string; score: number; met: string[]; teaser: string[] }>("find-hit", (h) => {
      found++;
      addHit(found, h);
    }),
  ];
  try {
    const n = await desk.find({
      size: state.size, difficulty: state.difficulty, evil: state.evil,
      crimson_chance: state.biasOn ? state.bias / 100 : null,
      specials: [...state.specials], secrets: [...state.secrets], goals,
      attempts, workers: Number($<HTMLInputElement>("workers").value) || 2, want: Number($<HTMLInputElement>("want").value) || 3,
      name: $<HTMLInputElement>("wname").value.trim() || "Lucky World", full_format: fullFormat(),
    });
    $("find-text").textContent = n ? `Done. ${n} world${n === 1 ? "" : "s"} matched.` : "Done. Nothing matched; try fewer goals or more worlds.";
  } catch (e) {
    toast(String(e));
  } finally {
    offs.forEach((o) => o());
    go.disabled = false;
    $("find-cancel").hidden = true;
    setTimeout(() => ($("find-cancel").hidden = false), 0);
  }
}

function addHit(n: number, h: { seed: string; path: string; met: string[]; teaser: string[] }) {
  const li = el("li");
  const name = $<HTMLInputElement>("wname").value.trim() || "Lucky World";
  const install = el("button", { className: "btn btn--primary" }, "Put in Terraria");
  install.addEventListener("click", async () => {
    try {
      const dest = await desk.installWorld(h.path, n > 1 ? `${name} ${n}` : name);
      install.textContent = "In your world list";
      install.disabled = true;
      toast(`Saved as ${dest.split(/[\\/]/).pop()}`);
    } catch (e) {
      toast(String(e));
    }
  });
  const reveal = el("button", { className: "btn btn--secondary" }, "Show the map");
  reveal.addEventListener("click", async () => {
    if (!confirm("This shows the whole world, including where everything is. Continue?")) return;
    openBytes(await desk.readWorld(h.path), h.path);
  });
  const seedBtn = el("button", { className: "link" }, "Show seed");
  seedBtn.addEventListener("click", () => seedBtn.replaceWith(el("code", {}, h.seed)));
  li.append(
    el("strong", {}, `Match ${n}`),
    el("ul", {}, ...[...h.met.filter((m) => !m.startsWith("Bonus")), ...h.teaser].map((t) => el("li", {}, t))),
    el("div", { className: "row" }, install, reveal),
    seedBtn,
  );
  $("hits").append(li);
}
$("find-go").addEventListener("click", runFind);
$("find-cancel").addEventListener("click", () => desk.cancel());

/* ---------- seeds catalog ---------- */
function renderCatalog() {
  const fill = (id: string, rows: (Special | Secret)[]) => {
    const tb = $(id).querySelector("tbody")!;
    tb.replaceChildren(
      ...rows.map((r) => {
        const copy = el("button", { className: "copy", title: "Copy code" }, r.code);
        copy.addEventListener("click", async () => {
          await navigator.clipboard.writeText(r.code);
          toast(`Copied “${r.code}”`);
        });
        return el("tr", {}, el("td", {}, r.name), el("td", {}, copy), el("td", {}, r.since), el("td", {}, r.about));
      }),
    );
  };
  fill("cat-specials", state.catalog.specials);
  fill("cat-secrets", state.catalog.secrets);
}

/* ---------- boot ---------- */
async function boot() {
  const vsel = $<HTMLSelectElement>("version");
  vsel.replaceChildren(...VERSIONS.map((v, i) => el("option", { value: v }, i === 0 ? `${v} (latest)` : v)));
  vsel.addEventListener("change", () => {
    state.version = vsel.value;
    renderChips();
    updateSeed();
  });
  await loadCore();
  state.catalog = JSON.parse(seed_catalog());
  $<HTMLInputElement>("seed").value = randomSeed();
  renderChips();
  updateSeed();
  renderCatalog();
  renderPresets();
  renderGoals();
  const hashTab = ({ "#seeds": "tab-seeds", "#create": "tab-create", "#find": "tab-find" } as Record<string, string>)[location.hash];
  if (hashTab) showTab(hashTab);
  if (desk.isDesktop) {
    try {
      state.install = await desk.detect();
      if (state.install.found && VERSIONS.includes(state.install.version)) {
        state.version = state.install.version;
        vsel.value = state.version;
        renderChips();
        updateSeed();
      }
    } catch (e) {
      toast(String(e));
    }
    installChip();
  } else {
    $("worlds-hint").textContent = "Worlds are in Documents/My Games/Terraria/Worlds on Windows, and ~/Library/Application Support/Terraria/Worlds on Mac.";
  }
}
boot();
