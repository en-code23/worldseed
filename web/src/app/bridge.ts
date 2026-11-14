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
export async function generate(req: { size: number; difficulty: number; evil: number; name: string; seed: string; job: string }): Promise<string> {
  return (await tauri()).invoke<string>("generate", { req });
}
export async function readWorld(path: string): Promise<Uint8Array> {
  const buf = await (await tauri()).invoke<ArrayBuffer>("read_world", { path });
  return new Uint8Array(buf);
}
export async function installWorld(path: string, name: string): Promise<string> {
  return (await tauri()).invoke<string>("install_world", { path, name });
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
