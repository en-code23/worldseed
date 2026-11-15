// Web and mobile library: worlds you opened, remembered in this browser (IndexedDB).
// Metadata and file bytes live in separate stores so listing never loads the big files.
import type { Header } from "./types";

export interface WebEntry {
  id: string;
  header: Header;
  added: number;
  bytes: number;
}

const MAX = 12;
let dbp: Promise<IDBDatabase> | null = null;

function db(): Promise<IDBDatabase> {
  dbp ??= new Promise((resolve, reject) => {
    const r = indexedDB.open("worldseed", 1);
    r.onupgradeneeded = () => {
      r.result.createObjectStore("meta", { keyPath: "id" });
      r.result.createObjectStore("data");
    };
    r.onsuccess = () => resolve(r.result);
    r.onerror = () => reject(r.error);
  });
  return dbp;
}

function req<T>(r: IDBRequest<T>): Promise<T> {
  return new Promise((resolve, reject) => {
    r.onsuccess = () => resolve(r.result);
    r.onerror = () => reject(r.error);
  });
}

export async function listWeb(): Promise<WebEntry[]> {
  try {
    const all = await req((await db()).transaction("meta").objectStore("meta").getAll());
    return (all as WebEntry[]).sort((a, b) => b.added - a.added);
  } catch {
    return [];
  }
}

/** Remembers a world. Returns false when the browser refuses (private mode, storage full). */
export async function saveWeb(bytes: Uint8Array, header: Header): Promise<boolean> {
  try {
    const id = `${header.name}|${header.seed}|${header.width}|${bytes.length}`;
    const d = await db();
    const tx = d.transaction(["meta", "data"], "readwrite");
    tx.objectStore("meta").put({ id, header, added: Date.now(), bytes: bytes.length } satisfies WebEntry);
    tx.objectStore("data").put(new Blob([bytes as BlobPart]), id);
    await new Promise<void>((res, rej) => {
      tx.oncomplete = () => res();
      tx.onerror = () => rej(tx.error);
    });
    const all = await listWeb();
    for (const old of all.slice(MAX)) await removeWeb(old.id);
    return true;
  } catch {
    return false;
  }
}

export async function loadWeb(id: string): Promise<Uint8Array | null> {
  try {
    const blob = (await req((await db()).transaction("data").objectStore("data").get(id))) as Blob | undefined;
    return blob ? new Uint8Array(await blob.arrayBuffer()) : null;
  } catch {
    return null;
  }
}

export async function removeWeb(id: string): Promise<void> {
  const tx = (await db()).transaction(["meta", "data"], "readwrite");
  tx.objectStore("meta").delete(id);
  tx.objectStore("data").delete(id);
  await new Promise<void>((res, rej) => {
    tx.oncomplete = () => res();
    tx.onerror = () => rej(tx.error);
  });
}
