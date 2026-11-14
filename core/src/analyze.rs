//! Finds the landmarks of a world: structures, biomes, shrines, orbs and notable chest loot.
use crate::data::data;
use crate::world::{World, LIQ_SHIMMER};
use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct Poi {
    pub kind: &'static str,
    pub label: String,
    pub x: i32,
    pub y: i32,
    /// bounding box for areas (0 for point markers)
    pub w: i32,
    pub h: i32,
    pub detail: String,
    /// index into `chests` when this marker is a chest
    pub chest: i32,
    /// 0 = normal, 1 = notable loot, 2 = rare loot
    pub tier: u8,
}

#[derive(Serialize, Clone)]
pub struct LootHit {
    pub item: String,
    pub tier: u8,
    pub chest: i32,
    pub place: String,
    pub x: i32,
    pub y: i32,
}

#[derive(Serialize, Clone)]
pub struct Segment {
    pub biome: &'static str,
    pub x0: i32,
    pub x1: i32,
}

#[derive(Serialize, Clone, Default)]
pub struct Summary {
    pub sword_shrines: u32,
    pub sky_islands: u32,
    pub pyramids: u32,
    pub living_trees: u32,
    pub chests: u32,
    pub orbs: u32,
    pub hearts: u32,
    pub altars: u32,
    pub life_crystals: u32,
    pub larvae: u32,
    pub dungeon_side: &'static str,
    pub jungle_side: &'static str,
    pub snow_side: &'static str,
    pub shimmer_side: &'static str,
    pub dungeon_distance: i32,
    pub evil_distance: i32,
    pub sword_distance: i32,
    pub temple_found: bool,
    pub shimmer_found: bool,
}

#[derive(Serialize, Clone)]
pub struct ChestInfo {
    pub x: i32,
    pub y: i32,
    pub kind: String,
    pub place: String,
    pub name: String,
    pub items: Vec<crate::world::ChestItem>,
    pub tier: u8,
}

#[derive(Serialize)]
pub struct Analysis {
    pub header: crate::world::Header,
    pub pois: Vec<Poi>,
    pub chests: Vec<ChestInfo>,
    pub loot: Vec<LootHit>,
    pub surface: Vec<Segment>,
    pub summary: Summary,
}

/// Items worth calling out. Tier 2 = rare finds, tier 1 = notable.
pub const RARE: &[&str] = &[
    "Fledgling Wings", "Enchanted Sword", "Seaweed", "Flower Boots", "Living Loom",
    "Portal Gun", "Sandstorm in a Bottle", "Flying Carpet", "Pharaoh's Mask",
    "Pharaoh's Robe", "Fiberglass Fishing Pole", "Snowball Cannon", "Sky Mill", "Abeemination",
];
pub const NOTABLE: &[&str] = &[
    "Blizzard in a Bottle", "Ice Skates", "Flurry Boots", "Staff of Regrowth", "Feral Claws",
    "Anklet of the Wind", "Lucky Horseshoe", "Shiny Red Balloon", "Starfury", "Celestial Magnet",
    "Piranha Gun", "Scourge of the Corruptor", "Vampire Knives", "Rainbow Gun", "Staff of the Frost Hydra",
    "Desert Tiger Staff", "Muramasa", "Cobalt Shield", "Handgun", "Aqua Scepter", "Magic Missile",
    "Blue Moon", "Valor", "Sunfury", "Flower of Fire", "Hellwing Bow", "Dark Lance", "Flamelash",
    "Treasure Magnet", "Extractinator", "Band of Regeneration", "Water Walking Boots", "Shoe Spikes", "Honey Dispenser", "Lava Charm", "Bast Statue", "Bone Welder",
    "Hermes Boots", "Cloud in a Bottle", "Magic Mirror", "Ice Mirror", "Aglet", "Climbing Claws",
    "Radar", "Enchanted Boomerang", "Flare Gun", "Mace", "Ice Boomerang", "Ice Blade", "Golden Key",
    "Shadow Key", "Lihzahrd Power Cell", "Sandstorm in a Bottle", "Water Candle", "Lifeform Analyzer",
    "Bottled Water", "Life Crystal", "Umbrella", "Fish Finder", "Living Wood Wand", "Leaf Wand",
    "Desert Minecart", "Mysterious Cape", "Red Cape", "Winged Helmet", "Flinx Fur Coat",
];

pub fn loot_tier(name: &str) -> u8 {
    if RARE.iter().any(|n| n.eq_ignore_ascii_case(name)) {
        2
    } else if NOTABLE.iter().any(|n| n.eq_ignore_ascii_case(name)) {
        1
    } else {
        0
    }
}

struct Comp {
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    count: u32,
    sx: i64,
    sy: i64,
}

/// Connected components of grid cells (cell px) that contain at least `min` matching tiles.
fn clusters(wd: &World, cell: i32, min: u32, y_from: i32, y_to: i32, pred: &dyn Fn(usize) -> bool) -> Vec<Comp> {
    let w = wd.header.width;
    let gw = (w + cell - 1) / cell;
    let y_from = y_from.max(0);
    let y_to = y_to.min(wd.header.height);
    if y_to <= y_from {
        return vec![];
    }
    let gh = (y_to - y_from + cell - 1) / cell;
    let mut cnt = vec![0u32; (gw * gh) as usize];
    for y in y_from..y_to {
        let row = (y as usize) * (w as usize);
        let gy = (y - y_from) / cell;
        for x in 0..w {
            if pred(row + x as usize) {
                cnt[(gy * gw + x / cell) as usize] += 1;
            }
        }
    }
    let mut seen = vec![false; cnt.len()];
    let mut out = vec![];
    for start in 0..cnt.len() {
        if seen[start] || cnt[start] < min {
            continue;
        }
        let mut stack = vec![start];
        seen[start] = true;
        let mut c = Comp { x0: i32::MAX, y0: i32::MAX, x1: 0, y1: 0, count: 0, sx: 0, sy: 0 };
        while let Some(i) = stack.pop() {
            let gx = i as i32 % gw;
            let gy = i as i32 / gw;
            let px = gx * cell + cell / 2;
            let py = y_from + gy * cell + cell / 2;
            c.x0 = c.x0.min(gx * cell);
            c.y0 = c.y0.min(y_from + gy * cell);
            c.x1 = c.x1.max((gx + 1) * cell);
            c.y1 = c.y1.max(y_from + (gy + 1) * cell);
            c.count += cnt[i];
            c.sx += px as i64 * cnt[i] as i64;
            c.sy += py as i64 * cnt[i] as i64;
            for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                let nx = gx + dx;
                let ny = gy + dy;
                if nx < 0 || ny < 0 || nx >= gw || ny >= gh {
                    continue;
                }
                let j = (ny * gw + nx) as usize;
                if !seen[j] && cnt[j] >= min {
                    seen[j] = true;
                    stack.push(j);
                }
            }
        }
        out.push(c);
    }
    out
}

/// Merge components whose boxes are within `gap` tiles of each other.
fn merge(mut v: Vec<Comp>, gap: i32) -> Vec<Comp> {
    let mut changed = true;
    while changed {
        changed = false;
        'outer: for i in 0..v.len() {
            for j in (i + 1)..v.len() {
                let (a, b) = (&v[i], &v[j]);
                if a.x0 - gap <= b.x1 && b.x0 - gap <= a.x1 && a.y0 - gap <= b.y1 && b.y0 - gap <= a.y1 {
                    let b = v.remove(j);
                    let a = &mut v[i];
                    a.x0 = a.x0.min(b.x0); a.y0 = a.y0.min(b.y0);
                    a.x1 = a.x1.max(b.x1); a.y1 = a.y1.max(b.y1);
                    a.count += b.count; a.sx += b.sx; a.sy += b.sy;
                    changed = true;
                    break 'outer;
                }
            }
        }
    }
    v
}

fn is_dungeon_wall(w: u16) -> bool {
    matches!(w, 7 | 8 | 9 | 94 | 95 | 96 | 97 | 98 | 99)
}

fn surface_biome(t: u16) -> Option<&'static str> {
    Some(match t {
        147 | 148 | 161 | 162 | 163 | 164 | 200 => "snow",
        59 | 60 | 120 => "jungle",
        53 | 396 | 397 => "desert",
        70 => "mushroom",
        2 | 0 | 1 | 477 | 40 => "forest",
        _ => return None,
    })
}

fn evil_kind(t: u16) -> Option<&'static str> {
    match t {
        23 | 25 | 112 | 398 | 400 | 163 | 32 | 636 => Some("corruption"),
        199 | 203 | 234 | 399 | 401 | 200 | 352 | 662 => Some("crimson"),
        109 | 117 | 116 | 402 | 403 | 164 => Some("hallow"),
        _ => None,
    }
}

pub fn analyze(wd: &World) -> Analysis {
    let d = data();
    let h = &wd.header;
    let w = h.width;
    let ht = h.height;
    let surface = h.surface as i32;
    let rock = h.rock as i32;
    let tiles = &wd.tiles;
    let tile_is = |i: usize, set: &[u16]| -> bool {
        let t = tiles[i];
        t != 0 && set.contains(&(t - 1))
    };
    let mut pois: Vec<Poi> = vec![];
    let mut push = |kind: &'static str, label: &str, x: i32, y: i32, ww: i32, hh: i32, detail: String| Poi {
        kind,
        label: label.to_string(),
        x,
        y,
        w: ww,
        h: hh,
        detail,
        chest: -1,
        tier: 0,
    };

    pois.push(push("spawn", "Spawn", h.spawn_x, h.spawn_y, 0, 0, "Where every new character appears.".into()));
    pois.push(push("dungeon", "Dungeon", h.dungeon_x, h.dungeon_y, 0, 0,
        "Dungeon entrance. The Old Man waits here until Skeletron is defeated.".into()));

    // --- surface biome strip ---
    let mut col_biome: Vec<&'static str> = vec!["sky"; w as usize];
    let top = (surface as f64 * 0.35) as i32;
    for x in 0..w {
        let mut b = "forest";
        for y in top..(rock + 20).min(ht) {
            let i = (y as usize) * (w as usize) + x as usize;
            let t = tiles[i];
            if t == 0 {
                if wd.liquid[i] == 1 && (x < 380 || x > w - 380) {
                    b = "ocean";
                    break;
                }
                continue;
            }
            let t = t - 1;
            if matches!(t, 189 | 196 | 202 | 460) {
                continue; // sky island
            }
            if let Some(e) = evil_kind(t) {
                b = e;
                break;
            }
            if let Some(sb) = surface_biome(t) {
                b = sb;
                break;
            }
            if matches!(t, 5 | 3 | 24 | 61 | 73 | 74 | 110 | 113 | 201 | 72 | 71 | 4 | 10 | 11 | 15 | 18 | 19 | 323 | 27) {
                continue;
            }
            break;
        }
        col_biome[x as usize] = b;
    }
    // smooth into segments (merge runs shorter than 12 columns)
    let mut segs: Vec<Segment> = vec![];
    for x in 0..w {
        let b = col_biome[x as usize];
        match segs.last_mut() {
            Some(s) if s.biome == b => s.x1 = x,
            _ => segs.push(Segment { biome: b, x0: x, x1: x }),
        }
    }
    let mut merged: Vec<Segment> = vec![];
    for s in segs {
        if s.x1 - s.x0 < 12 {
            if let Some(l) = merged.last_mut() {
                l.x1 = s.x1;
                continue;
            }
        }
        match merged.last_mut() {
            Some(l) if l.biome == s.biome => l.x1 = s.x1,
            _ => merged.push(s),
        }
    }

    // --- oceans and underworld ---
    pois.push(push("ocean", "Ocean", 140, surface - 40, 0, 0, "Left ocean: fishing, Angler, Sharks, Duke Fishron later.".into()));
    pois.push(push("ocean", "Ocean", w - 140, surface - 40, 0, 0, "Right ocean.".into()));
    pois.push(push("underworld", "Underworld", w / 2, ht - 110, 0, 0,
        "The Underworld: Hellstone, Wall of Flesh, ruined Obsidian houses.".into()));

    // --- sky islands ---
    let sky_set = [189u16, 196, 202, 460];
    let islands = clusters(wd, 12, 6, 30, (surface as f64 * 0.95) as i32, &|i| tile_is(i, &sky_set));
    let islands: Vec<Comp> = merge(islands, 24).into_iter().filter(|c| c.count >= 40).collect();

    // --- pyramids (sandstone brick structures near the surface) ---
    let pyr = clusters(wd, 10, 12, surface - 200, rock, &|i| tile_is(i, &[151]));
    let pyramids: Vec<Comp> = merge(pyr, 20).into_iter().filter(|c| c.count >= 250).collect();

    // --- jungle temple ---
    let temple = clusters(wd, 16, 30, surface, ht, &|i| tile_is(i, &[226]))
        .into_iter()
        .max_by_key(|c| c.count);

    // --- living trees ---
    let ltrees: Vec<Comp> = merge(clusters(wd, 8, 6, (surface as f64 * 0.3) as i32, rock, &|i| tile_is(i, &[191])), 10)
        .into_iter()
        .filter(|c| c.count >= 60 && c.y1 - c.y0 >= 24)
        .collect();

    // --- shimmer ---
    let shim = clusters(wd, 16, 4, surface, ht, &|i| wd.liquid[i] == LIQ_SHIMMER)
        .into_iter()
        .max_by_key(|c| c.count);

    // --- other caves ---
    let marble: Vec<Comp> = merge(clusters(wd, 16, 40, rock - 100, ht - 200, &|i| tile_is(i, &[367])), 30)
        .into_iter().filter(|c| c.count > 600).collect();
    let granite: Vec<Comp> = merge(clusters(wd, 16, 40, rock - 100, ht - 200, &|i| tile_is(i, &[368])), 30)
        .into_iter().filter(|c| c.count > 600).collect();
    let mushroom: Vec<Comp> = merge(clusters(wd, 16, 20, surface - 50, ht - 200, &|i| tile_is(i, &[70])), 30)
        .into_iter().filter(|c| c.count > 300).collect();
    let hives: Vec<Comp> = merge(clusters(wd, 12, 30, surface, ht - 200, &|i| tile_is(i, &[225])), 20)
        .into_iter().filter(|c| c.count > 300).collect();

    for (n, c) in islands.iter().enumerate() {
        let lake = c.count > 0 && {
            let mut water = 0;
            for y in c.y0..c.y1 {
                for x in c.x0..c.x1.min(w) {
                    if wd.liquid[wd.idx(x, y)] == 1 { water += 1; }
                }
            }
            water > 30
        };
        let mut p = push("sky_island", if lake { "Sky lake" } else { "Sky island" },
            (c.sx / c.count as i64) as i32, (c.sy / c.count as i64) as i32,
            c.x1 - c.x0, c.y1 - c.y0,
            format!("Floating island #{}{}", n + 1, if lake { " with a lake" } else { "" }));
        p.x = c.x0 + (c.x1 - c.x0) / 2;
        p.y = c.y0 + (c.y1 - c.y0) / 2;
        pois.push(p);
    }
    for c in &pyramids {
        pois.push(push("pyramid", "Desert pyramid", c.x0 + (c.x1 - c.x0) / 2, c.y0 + (c.y1 - c.y0) / 2,
            c.x1 - c.x0, c.y1 - c.y0, "Sandstone pyramid. Its chest holds pyramid loot.".into()));
    }
    if let Some(c) = &temple {
        pois.push(push("temple", "Jungle temple", c.x0 + (c.x1 - c.x0) / 2, c.y0 + (c.y1 - c.y0) / 2,
            c.x1 - c.x0, c.y1 - c.y0, "Lihzahrd temple. Locked until Plantera is defeated.".into()));
    }
    for c in &ltrees {
        pois.push(push("living_tree", "Living tree", c.x0 + (c.x1 - c.x0) / 2, c.y0 + (c.y1 - c.y0) / 2,
            c.x1 - c.x0, c.y1 - c.y0, "Living tree with a root room.".into()));
    }
    if let Some(c) = &shim {
        pois.push(push("shimmer", "Shimmer (Aether)", (c.sx / c.count as i64) as i32, (c.sy / c.count as i64) as i32,
            c.x1 - c.x0, c.y1 - c.y0, "The Aether: a pool of Shimmer that transmutes items.".into()));
    }
    for (kind, label, list, det) in [
        ("marble", "Marble cave", &marble, "Marble biome: Hoplites and Medusa."),
        ("granite", "Granite cave", &granite, "Granite biome: Granite Elementals."),
        ("mushroom", "Glowing mushroom biome", &mushroom, "Truffle's home biome."),
        ("hive", "Bee hive", &hives, "Bee hive."),
    ] {
        for c in list.iter() {
            pois.push(push(kind, label, c.x0 + (c.x1 - c.x0) / 2, c.y0 + (c.y1 - c.y0) / 2,
                c.x1 - c.x0, c.y1 - c.y0, det.into()));
        }
    }

    // --- anchors: orbs, altars, crystals, shrines ---
    let mut sm = Summary::default();
    let mut sword_d = i32::MAX;
    for a in &wd.anchors {
        let (u, v) = (a.u as i32, a.v as i32);
        match a.tile {
            31 if u % 36 == 0 && v % 36 == 0 => {
                let heart = (u / 36) % 2 == 1;
                if heart { sm.hearts += 1 } else { sm.orbs += 1 }
                pois.push(push(if heart { "heart" } else { "orb" },
                    if heart { "Crimson heart" } else { "Shadow orb" }, a.x + 1, a.y + 1, 0, 0,
                    if heart { "Smash 3 to summon the Brain of Cthulhu." } else { "Smash 3 to summon the Eater of Worlds." }.into()));
            }
            26 if u % 54 == 0 && v == 0 => {
                sm.altars += 1;
                pois.push(push("altar", if u >= 54 { "Crimson altar" } else { "Demon altar" }, a.x + 1, a.y, 0, 0,
                    "Crafting station for boss summons.".into()));
            }
            12 if u % 36 == 0 && v == 0 => {
                sm.life_crystals += 1;
                pois.push(push("life_crystal", "Life crystal", a.x, a.y, 0, 0, "+20 maximum life.".into()));
            }
            187 if u == 918 && v == 0 => {
                sm.sword_shrines += 1;
                let dd = (a.x - h.spawn_x).abs();
                sword_d = sword_d.min(dd);
                let mut p = push("sword_shrine", "Enchanted Sword shrine", a.x + 1, a.y, 0, 0,
                    "An Enchanted Sword in stone. Free early-game sword.".into());
                p.tier = 2;
                pois.push(p);
            }
            237 if u == 0 && v == 0 => {
                pois.push(push("lihzahrd_altar", "Lihzahrd altar", a.x + 1, a.y, 0, 0, "Summons Golem with a Lihzahrd Power Cell.".into()));
            }
            231 if u % 54 == 0 && v % 54 == 0 => {
                sm.larvae += 1;
                pois.push(push("larva", "Queen Bee larva", a.x + 1, a.y + 1, 0, 0, "Break it to summon Queen Bee.".into()));
            }
            77 if u % 54 == 0 && v == 0 => {
                pois.push(push("hellforge", "Hellforge", a.x + 1, a.y, 0, 0, "Smelts Hellstone.".into()));
            }
            _ => {}
        }
    }

    // --- chests ---
    let in_box = |c: &Comp, x: i32, y: i32| x >= c.x0 - 4 && x <= c.x1 + 4 && y >= c.y0 - 4 && y <= c.y1 + 4;
    let mut chests = vec![];
    let mut loot = vec![];
    for (ci, c) in wd.chests.iter().enumerate() {
        if c.x < 0 || c.y < 0 || c.x >= w || c.y >= ht {
            continue;
        }
        let i = wd.idx(c.x, c.y);
        let kind = match wd.tile(c.x, c.y) {
            Some(t @ (21 | 467 | 88)) => {
                // find the frame anchor recorded at this position
                let a = wd.anchors.iter().find(|a| a.x == c.x && a.y == c.y && a.tile == t);
                a.and_then(|a| d.frame_name(t, a.u as i32, a.v as i32)).unwrap_or_else(|| "Chest".into())
            }
            Some(t) => d.tile_name(t).to_string(),
            None => "Chest".into(),
        };
        if kind.contains("Dresser") || kind == "Dressers" {
            continue;
        }
        let wall = wd.walls[i];
        let place = if islands.iter().any(|s| in_box(s, c.x, c.y)) {
            "Sky island"
        } else if pyramids.iter().any(|s| in_box(s, c.x, c.y)) {
            "Pyramid"
        } else if wall == 87 || temple.as_ref().map(|t| in_box(t, c.x, c.y)).unwrap_or(false) && kind.contains("Lihzahrd") {
            "Jungle temple"
        } else if is_dungeon_wall(wall) || kind.contains("Dungeon") {
            "Dungeon"
        } else if c.y > ht - 200 {
            "Underworld"
        } else if ltrees.iter().any(|s| in_box(s, c.x, c.y)) {
            "Living tree"
        } else if c.y < surface {
            "Surface"
        } else if c.y < rock {
            "Underground"
        } else {
            "Cavern"
        };
        let mut tier = 0u8;
        for it in &c.items {
            let t = loot_tier(&it.name);
            if t > 0 {
                tier = tier.max(t);
                loot.push(LootHit { item: it.name.clone(), tier: t, chest: chests.len() as i32, place: place.into(), x: c.x, y: c.y });
            }
        }
        let ix = chests.len() as i32;
        chests.push(ChestInfo { x: c.x, y: c.y, kind: kind.clone(), place: place.into(), name: c.name.clone(), items: c.items.clone(), tier });
        let mut p = push("chest", &kind, c.x + 1, c.y + 1, 0, 0, place.to_string());
        p.chest = ix;
        p.tier = tier;
        pois.push(p);
        let _ = ci;
    }

    // --- summary ---
    sm.sky_islands = islands.len() as u32;
    sm.pyramids = pyramids.len() as u32;
    sm.living_trees = ltrees.len() as u32;
    sm.chests = chests.len() as u32;
    sm.temple_found = temple.is_some();
    sm.shimmer_found = shim.is_some();
    let side = |x: i32| if x < w / 2 { "left" } else { "right" };
    sm.dungeon_side = side(h.dungeon_x);
    sm.dungeon_distance = (h.dungeon_x - h.spawn_x).abs();
    sm.sword_distance = if sword_d == i32::MAX { -1 } else { sword_d };
    let biome_center = |b: &str| -> Option<i32> {
        let mut best: Option<&Segment> = None;
        for s in &merged {
            if s.biome == b && best.map(|x| x.x1 - x.x0 < s.x1 - s.x0).unwrap_or(true) {
                best = Some(s);
            }
        }
        best.map(|s| (s.x0 + s.x1) / 2)
    };
    sm.jungle_side = biome_center("jungle").map(side).unwrap_or("unknown");
    sm.snow_side = biome_center("snow").map(side).unwrap_or("unknown");
    sm.shimmer_side = shim.as_ref().map(|c| side((c.x0 + c.x1) / 2)).unwrap_or("unknown");
    let evil = if h.crimson { "crimson" } else { "corruption" };
    sm.evil_distance = merged
        .iter()
        .filter(|s| s.biome == evil || (h.drunk && (s.biome == "corruption" || s.biome == "crimson")))
        .map(|s| {
            if h.spawn_x >= s.x0 && h.spawn_x <= s.x1 { 0 } else { (s.x0 - h.spawn_x).abs().min((s.x1 - h.spawn_x).abs()) }
        })
        .min()
        .unwrap_or(-1);

    loot.sort_by(|a, b| b.tier.cmp(&a.tier).then(a.item.cmp(&b.item)));
    Analysis { header: h.clone(), pois, chests, loot, surface: merged, summary: sm }
}
