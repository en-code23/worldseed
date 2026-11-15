// Desktop-only calls into the Tauri backend. On the web every call reports "unavailable".
import type { Install } from "./types";

export const isDesktop = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

async function tauri() {
  const core = await import("@tauri-apps/api/core");
  const event = await import("@tauri-apps/api/event");
  return { invoke: core.invoke, listen: event.listen };
}

export async function detect(): Promise<Install> {
  return (await tauri()).invoke<Install>("detect");
}
export async function setInstall(path: string): Promise<Install> {
  return (await tauri()).invoke<Install>("set_install", { path });
}
export async function pickFolder(): Promise<string | null> {
  const d = await import("@tauri-apps/plugin-dialog");
  const r = await d.open({ directory: true, title: "Choose your Terraria folder" });
  return typeof r === "string" ? r : null;
}
export async function generate(req: { size: number; difficulty: number; evil: number; name: string; seed: string; job: string; keep?: boolean }): Promise<string> {
  return (await tauri()).invoke<string>("generate", { req });
}
export async function readWorld(path: string): Promise<Uint8Array> {
  const buf = await (await tauri()).invoke<ArrayBuffer>("read_world", { path });
  return new Uint8Array(buf);
}
export async function installWorld(path: string, name: string): Promise<string> {
  return (await tauri()).invoke<string>("install_world", { path, name });
}
export interface LibEntry {
  path: string;
  source: "terraria" | "worldseed";
  modified: number;
  bytes: number;
  header: import("./types").Header | null;
  error: string | null;
}
export async function listWorlds(): Promise<LibEntry[]> {
  return (await tauri()).invoke<LibEntry[]>("list_worlds");
}
export async function removeWorld(path: string): Promise<void> {
  return (await tauri()).invoke("remove_world", { path });
}
export async function revealPath(p: string) {
  const o = await import("@tauri-apps/plugin-opener");
  return o.revealItemInDir(p);
}
export async function cancel() {
  return (await tauri()).invoke("cancel");
}
export async function find(req: unknown): Promise<number> {
  return (await tauri()).invoke<number>("find", { req });
}
export async function on<T>(name: string, fn: (p: T) => void): Promise<() => void> {
  return (await tauri()).listen<T>(name, (e) => fn(e.payload));
}
export async function openPath(p: string) {
  const o = await import("@tauri-apps/plugin-opener");
  return o.openPath(p);
}

/* ---------- in-app updates ---------- */
export interface Update {
  version: string;
  notes: string;
  install: (progress: (pct: number) => void) => Promise<void>;
}

/** Desktop: Tauri's signed updater, fed by latest.json on GitHub Releases. */
export async function checkDesktopUpdate(): Promise<Update | null> {
  const { check } = await import("@tauri-apps/plugin-updater");
  const u = await check();
  if (!u) return null;
  return {
    version: u.version,
    notes: u.body ?? "",
    install: async (progress) => {
      let total = 0;
      let got = 0;
      await u.downloadAndInstall((e) => {
        if (e.event === "Started") total = e.data.contentLength ?? 0;
        else if (e.event === "Progress") {
          got += e.data.chunkLength;
          if (total) progress(Math.min(99, Math.round((got / total) * 100)));
        } else progress(100);
      });
      const { relaunch } = await import("@tauri-apps/plugin-process");
      await relaunch();
    },
  };
}

export const isNativeMobile = typeof window !== "undefined" && "Capacitor" in window && !!(window as any).Capacitor?.isNativePlatform?.();

const newer = (a: string, b: string) => {
  const pa = a.replace(/^v/, "").split(".").map(Number);
  const pb = b.replace(/^v/, "").split(".").map(Number);
  for (let i = 0; i < Math.max(pa.length, pb.length); i++) {
    if ((pa[i] ?? 0) !== (pb[i] ?? 0)) return (pa[i] ?? 0) > (pb[i] ?? 0);
  }
  return false;
};

/** Android: compare with the latest GitHub release and point at the new APK. */
export async function checkMobileUpdate(): Promise<{ version: string; url: string } | null> {
  const r = await fetch("https://api.github.com/repos/en-code23/worldseed/releases/latest", { headers: { Accept: "application/vnd.github+json" } });
  if (!r.ok) return null;
  const rel = (await r.json()) as { tag_name: string; assets: { name: string; browser_download_url: string }[] };
  if (!newer(rel.tag_name, __APP_VERSION__)) return null;
  const apk = rel.assets.find((a) => a.name.endsWith(".apk"));
  return apk ? { version: rel.tag_name.replace(/^v/, ""), url: apk.browser_download_url } : null;
}
