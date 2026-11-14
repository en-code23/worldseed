export interface Poi { kind: string; label: string; x: number; y: number; w: number; h: number; detail: string; chest: number; tier: number }
export interface ChestItem { id: number; name: string; stack: number; prefix: number }
export interface ChestInfo { x: number; y: number; kind: string; place: string; name: string; items: ChestItem[]; tier: number }
export interface LootHit { item: string; tier: number; chest: number; place: string; x: number; y: number }
export interface Header {
  version: number; name: string; seed: string; width: number; height: number; game_mode: number;
  drunk: boolean; for_the_worthy: boolean; celebration: boolean; the_constant: boolean; not_the_bees: boolean;
  remix: boolean; no_traps: boolean; zenith: boolean; skyblock: boolean; spawn_x: number; spawn_y: number;
  surface: number; rock: number; dungeon_x: number; dungeon_y: number; crimson: boolean; hardmode: boolean; game_version: string;
}
export interface Summary {
  sword_shrines: number; sky_islands: number; pyramids: number; living_trees: number; chests: number; orbs: number;
  hearts: number; altars: number; life_crystals: number; larvae: number; dungeon_side: string; jungle_side: string;
  snow_side: string; shimmer_side: string; dungeon_distance: number; evil_distance: number; sword_distance: number;
  temple_found: boolean; shimmer_found: boolean;
}
export interface Analysis { header: Header; pois: Poi[]; chests: ChestInfo[]; loot: LootHit[]; surface: { biome: string; x0: number; x1: number }[]; summary: Summary }
export interface Hint { topic: string; levels: string[] }
export interface Special { key: string; name: string; code: string; bit: number; since: string; about: string }
export interface Secret { key: string; name: string; code: string; since: string; about: string }
export interface Install { found: boolean; root: string; server: string; version: string; worlds_dir: string }
export interface Goals {
  items?: string[]; sky_items?: string[]; bonus_items?: string[]; sword_shrine?: boolean; pyramid?: boolean;
  min_sky_islands?: number; min_living_trees?: number; max_dungeon_distance?: number; min_dungeon_distance?: number;
  max_evil_distance?: number; min_evil_distance?: number; max_sword_distance?: number; players?: number;
  shared_items?: string[]; min_sword_shrines?: number;
}
