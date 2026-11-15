// Interactive world map: pan, zoom, pinch, hover and marker picking on a canvas.
import type { Poi } from "./types";

const CHUNK = 2048;

const KIND_COLOR: Record<string, string> = {
  spawn: "--map-spawn", dungeon: "--map-dungeon", temple: "--map-temple", lihzahrd_altar: "--map-temple",
  pyramid: "--map-pyramid", sky_island: "--map-sky", shimmer: "--map-shimmer", orb: "--map-orb",
  heart: "--map-heart", altar: "--map-altar", life_crystal: "--map-crystal", sword_shrine: "--map-sword",
  chest: "--map-chest", living_tree: "--map-cave", marble: "--map-cave", granite: "--map-cave",
  mushroom: "--map-cave", hive: "--map-pyramid", larva: "--map-pyramid", hellforge: "--map-hell",
  underworld: "--map-hell", ocean: "--map-sky",
};

/** Kinds that are areas or big landmarks: always labelled. */
const LANDMARK = new Set(["spawn", "dungeon", "temple", "pyramid", "shimmer", "underworld", "ocean", "sky_island"]);

export function kindColorVar(kind: string): string {
  return KIND_COLOR[kind] ?? "--map-chest";
}

export class MapView {
  private ctx: CanvasRenderingContext2D;
  private chunks: HTMLCanvasElement[] = [];
  private W = 0;
  private H = 0;
  private scale = 1;
  private ox = 0;
  private oy = 0;
  private dpr = 1;
  private pois: Poi[] = [];
  private colors: Record<string, string> = {};
  private raf = 0;
  private pointers = new Map<number, { x: number; y: number }>();
  private dragStart: { x: number; y: number; ox: number; oy: number; moved: boolean } | null = null;
  private pinch: { d: number; scale: number; cx: number; cy: number } | null = null;
  visible = new Set<string>();
  selected: Poi | null = null;
  onHover: (wx: number, wy: number, poi: Poi | null, px: number, py: number) => void = () => {};
  onLeave: () => void = () => {};
  onPick: (poi: Poi | null) => void = () => {};
  onEscape: () => void = () => {};
  private focusX = 0;
  private focusY = 0;

  constructor(private canvas: HTMLCanvasElement) {
    this.ctx = canvas.getContext("2d")!;
    new ResizeObserver(() => this.resize()).observe(canvas);
    canvas.addEventListener("wheel", (e) => this.wheel(e), { passive: false });
    canvas.addEventListener("pointerdown", (e) => this.down(e));
    canvas.addEventListener("pointermove", (e) => this.move(e));
    canvas.addEventListener("pointerup", (e) => this.up(e));
    canvas.addEventListener("pointercancel", (e) => this.up(e));
    canvas.addEventListener("pointerleave", () => this.onLeave());
    canvas.tabIndex = 0;
    canvas.addEventListener("keydown", (e) => this.key(e));
    this.readColors();
  }

  private readColors() {
    const cs = getComputedStyle(document.documentElement);
    for (const v of new Set([...Object.values(KIND_COLOR), "--map-rare", "--map-ink", "--map-label"])) {
      this.colors[v] = cs.getPropertyValue(v).trim() || "#fff";
    }
  }

  get loaded() {
    return this.W > 0;
  }

  setWorld(rgba: Uint8Array, w: number, h: number, pois: Poi[], focusX = w / 2, focusY = h / 3) {
    this.W = w;
    this.H = h;
    this.focusX = focusX;
    this.focusY = focusY;
    this.pois = pois;
    this.chunks = [];
    for (let x0 = 0; x0 < w; x0 += CHUNK) {
      const cw = Math.min(CHUNK, w - x0);
      const c = document.createElement("canvas");
      c.width = cw;
      c.height = h;
      const img = new ImageData(cw, h);
      for (let y = 0; y < h; y++) {
        const src = (y * w + x0) * 4;
        img.data.set(rgba.subarray(src, src + cw * 4), y * cw * 4);
      }
      c.getContext("2d")!.putImageData(img, 0, 0);
      this.chunks.push(c);
    }
    this.selected = null;
    this.fit();
  }

  setPois(p: Poi[]) {
    this.pois = p;
    this.draw();
  }

  clear() {
    this.W = 0;
    this.chunks = [];
    this.pois = [];
    this.draw();
  }

  private resize() {
    this.dpr = window.devicePixelRatio || 1;
    const r = this.canvas.getBoundingClientRect();
    this.canvas.width = Math.max(1, Math.round(r.width * this.dpr));
    this.canvas.height = Math.max(1, Math.round(r.height * this.dpr));
    if (this.W && !this.scaleSet) this.fit();
    this.draw();
  }
  private scaleSet = false;

  private get cw() {
    return this.canvas.width / this.dpr;
  }
  private get ch() {
    return this.canvas.height / this.dpr;
  }

  /** Whole world on wide screens; on narrow ones, fill the height and centre on spawn so the map is never a thin strip. */
  fit() {
    if (!this.W) return;
    const whole = Math.min(this.cw / this.W, this.ch / this.H);
    const narrow = this.cw < 720 && this.ch / this.H > whole * 1.6;
    this.scale = narrow ? this.ch / this.H : whole;
    this.ox = narrow
      ? Math.min(Math.max(this.focusX - this.cw / this.scale / 2, 0), this.W - this.cw / this.scale)
      : -(this.cw / this.scale - this.W) / 2;
    this.oy = -(this.ch / this.scale - this.H) / 2;
    this.scaleSet = false;
    this.draw();
  }

  private minScale() {
    return Math.min(this.cw / this.W, this.ch / this.H) * 0.5;
  }

  zoomAt(f: number, px = this.cw / 2, py = this.ch / 2) {
    if (!this.W) return;
    const wx = this.ox + px / this.scale;
    const wy = this.oy + py / this.scale;
    this.scale = Math.min(24, Math.max(this.minScale(), this.scale * f));
    this.ox = wx - px / this.scale;
    this.oy = wy - py / this.scale;
    this.scaleSet = true;
    this.draw();
  }

  flyTo(x: number, y: number, scale = 4) {
    this.scale = Math.max(this.scale, scale);
    this.ox = x - this.cw / 2 / this.scale;
    this.oy = y - this.ch / 2 / this.scale;
    this.scaleSet = true;
    this.draw();
  }

  private wheel(e: WheelEvent) {
    e.preventDefault();
    const r = this.canvas.getBoundingClientRect();
    const f = Math.exp(-e.deltaY * (e.ctrlKey ? 0.01 : 0.0015));
    this.zoomAt(f, e.clientX - r.left, e.clientY - r.top);
  }

  private key(e: KeyboardEvent) {
    const step = 80 / this.scale;
    const map: Record<string, () => void> = {
      ArrowLeft: () => (this.ox -= step), ArrowRight: () => (this.ox += step),
      ArrowUp: () => (this.oy -= step), ArrowDown: () => (this.oy += step),
      "+": () => this.zoomAt(1.4), "=": () => this.zoomAt(1.4), "-": () => this.zoomAt(1 / 1.4), "0": () => this.fit(),
      Enter: () => this.pickNearestCentre(), " ": () => this.pickNearestCentre(), Escape: () => this.onEscape(),
    };
    const fn = map[e.key];
    if (fn) {
      e.preventDefault();
      fn();
      this.draw();
    }
  }

  private local(e: PointerEvent) {
    const r = this.canvas.getBoundingClientRect();
    return { x: e.clientX - r.left, y: e.clientY - r.top };
  }

  private down(e: PointerEvent) {
    this.canvas.setPointerCapture(e.pointerId);
    const p = this.local(e);
    this.pointers.set(e.pointerId, p);
    if (this.pointers.size === 1) {
      this.dragStart = { x: p.x, y: p.y, ox: this.ox, oy: this.oy, moved: false };
    } else if (this.pointers.size === 2) {
      const [a, b] = [...this.pointers.values()];
      this.pinch = { d: Math.hypot(a.x - b.x, a.y - b.y), scale: this.scale, cx: (a.x + b.x) / 2, cy: (a.y + b.y) / 2 };
      this.dragStart = null;
    }
  }

  private move(e: PointerEvent) {
    const p = this.local(e);
    if (this.pointers.has(e.pointerId)) this.pointers.set(e.pointerId, p);
    if (this.pinch && this.pointers.size === 2) {
      const [a, b] = [...this.pointers.values()];
      const d = Math.hypot(a.x - b.x, a.y - b.y);
      const target = this.pinch.scale * (d / this.pinch.d);
      this.zoomAt(target / this.scale, this.pinch.cx, this.pinch.cy);
      return;
    }
    if (this.dragStart) {
      const dx = p.x - this.dragStart.x;
      const dy = p.y - this.dragStart.y;
      if (Math.abs(dx) + Math.abs(dy) > 4) this.dragStart.moved = true;
      if (this.dragStart.moved) {
        this.canvas.classList.add("dragging");
        this.ox = this.dragStart.ox - dx / this.scale;
        this.oy = this.dragStart.oy - dy / this.scale;
        this.scaleSet = true;
        this.draw();
      }
      return;
    }
    if (!this.W) return;
    const wx = Math.floor(this.ox + p.x / this.scale);
    const wy = Math.floor(this.oy + p.y / this.scale);
    this.onHover(wx, wy, this.hit(p.x, p.y), p.x, p.y);
  }

  private up(e: PointerEvent) {
    const p = this.local(e);
    const ds = this.dragStart;
    this.pointers.delete(e.pointerId);
    this.canvas.classList.remove("dragging");
    if (this.pointers.size < 2) this.pinch = null;
    if (ds && !ds.moved && this.W) {
      const hit = this.hit(p.x, p.y);
      this.selected = hit;
      this.onPick(hit);
      this.draw();
    }
    this.dragStart = null;
  }

  /** Keyboard picking: the visible marker closest to the middle of the view. */
  private pickNearestCentre() {
    if (!this.W) return;
    const cx = this.ox + this.cw / 2 / this.scale;
    const cy = this.oy + this.ch / 2 / this.scale;
    let best: Poi | null = null;
    let bd = Infinity;
    for (const p of this.pois) {
      if (!this.shown(p) || this.scale < this.minZoomFor(p)) continue;
      const d = Math.hypot(p.x - cx, p.y - cy);
      if (d < bd) {
        bd = d;
        best = p;
      }
    }
    if (!best) return;
    this.selected = best;
    this.ox = best.x - this.cw / 2 / this.scale;
    this.oy = best.y - this.ch / 2 / this.scale;
    this.onPick(best);
  }

  private shown(p: Poi): boolean {
    if (!this.visible.has(p.kind)) return false;
    if (p.kind === "chest" && this.visible.has("rare_only") && p.tier < 2) return false;
    return true;
  }

  private minZoomFor(p: Poi): number {
    if (LANDMARK.has(p.kind) || p.kind === "sword_shrine" || p.tier >= 2) return 0;
    if (p.kind === "chest") return p.tier > 0 ? 0.3 : 0.6;
    return 0.35;
  }

  private hit(px: number, py: number): Poi | null {
    let best: Poi | null = null;
    let bd = 12;
    for (const p of this.pois) {
      if (!this.shown(p) || this.scale < this.minZoomFor(p)) continue;
      const sx = (p.x - this.ox) * this.scale;
      const sy = (p.y - this.oy) * this.scale;
      const d = Math.hypot(sx - px, sy - py);
      if (d < bd) {
        bd = d;
        best = p;
      }
    }
    return best;
  }

  draw() {
    cancelAnimationFrame(this.raf);
    this.raf = requestAnimationFrame(() => this.paint());
  }

  private paint() {
    const { ctx } = this;
    ctx.setTransform(this.dpr, 0, 0, this.dpr, 0, 0);
    ctx.clearRect(0, 0, this.cw, this.ch);
    if (!this.W) return;
    ctx.imageSmoothingEnabled = this.scale < 1;
    ctx.imageSmoothingQuality = "medium";
    this.chunks.forEach((c, i) => {
      const x = (i * CHUNK - this.ox) * this.scale;
      const w = c.width * this.scale;
      if (x + w < 0 || x > this.cw) return;
      // overlap by a pixel so smoothed chunks never show a seam
      ctx.drawImage(c, Math.floor(x), -this.oy * this.scale, Math.ceil(w) + 1, this.H * this.scale);
    });
    const col = (k: string) => this.colors[kindColorVar(k)];
    const ink = this.colors["--map-ink"];
    const rare = this.colors["--map-rare"];
    ctx.font = "600 12px Manrope, system-ui, sans-serif";
    ctx.textBaseline = "middle";
    // areas first
    for (const p of this.pois) {
      if (!this.shown(p) || p.w <= 0 || p.kind === "underworld" || p.kind === "ocean") continue;
      const x = (p.x - p.w / 2 - this.ox) * this.scale;
      const y = (p.y - p.h / 2 - this.oy) * this.scale;
      ctx.strokeStyle = col(p.kind);
      ctx.lineWidth = 1.2;
      ctx.setLineDash(p.kind === "sky_island" || p.kind === "temple" || p.kind === "pyramid" ? [] : [4, 3]);
      ctx.strokeRect(x, y, p.w * this.scale, p.h * this.scale);
    }
    ctx.setLineDash([]);
    for (const p of this.pois) {
      if (!this.shown(p) || this.scale < this.minZoomFor(p)) continue;
      const sx = (p.x - this.ox) * this.scale;
      const sy = (p.y - this.oy) * this.scale;
      if (sx < -40 || sy < -40 || sx > this.cw + 40 || sy > this.ch + 40) continue;
      const big = LANDMARK.has(p.kind);
      const r = big ? 6 : p.kind === "chest" ? 3.5 : 4;
      ctx.fillStyle = col(p.kind);
      ctx.strokeStyle = ink;
      ctx.lineWidth = 1.5;
      ctx.beginPath();
      if (p.kind === "chest") ctx.rect(sx - r, sy - r, r * 2, r * 2);
      else if (p.kind === "orb" || p.kind === "heart" || p.kind === "life_crystal") ctx.arc(sx, sy, r, 0, Math.PI * 2);
      else {
        ctx.moveTo(sx, sy - r - 1);
        ctx.lineTo(sx + r + 1, sy);
        ctx.lineTo(sx, sy + r + 1);
        ctx.lineTo(sx - r - 1, sy);
        ctx.closePath();
      }
      ctx.fill();
      ctx.stroke();
      if (p.tier >= 2 || p.kind === "sword_shrine") {
        ctx.strokeStyle = rare;
        ctx.lineWidth = 2;
        ctx.beginPath();
        ctx.arc(sx, sy, r + 5, 0, Math.PI * 2);
        ctx.stroke();
      }
      if (this.selected === p) {
        ctx.strokeStyle = this.colors["--map-label"];
        ctx.lineWidth = 2;
        ctx.strokeRect(sx - r - 6, sy - r - 6, (r + 6) * 2, (r + 6) * 2);
      }
      const labelled = big && !(p.kind === "sky_island" && this.scale < 0.45);
      if (labelled || this.scale > 1.5 || this.selected === p) {
        if (p.kind === "chest" && this.selected !== p && this.scale < 3) continue;
        const label = p.label;
        ctx.lineWidth = 3;
        ctx.strokeStyle = ink;
        ctx.fillStyle = this.colors["--map-label"];
        ctx.strokeText(label, sx + r + 5, sy);
        ctx.fillText(label, sx + r + 5, sy);
      }
    }
  }
}
