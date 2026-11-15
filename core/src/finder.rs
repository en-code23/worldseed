//! Scores a world against what the player hopes to find. Results never contain coordinates,
//! so they can be shown in the spoiler-free mode.
use crate::analyze::Analysis;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Default, Clone)]
#[serde(default)]
pub struct Goals {
    /// Items that must be in some chest.
    pub items: Vec<String>,
    /// Items that must be in a sky island chest.
    pub sky_items: Vec<String>,
    /// Items that are welcome but not required (raise the score).
    pub bonus_items: Vec<String>,
    pub sword_shrine: bool,
    pub pyramid: bool,
    pub min_sky_islands: u32,
    pub min_living_trees: u32,
    /// Distances in tiles from spawn. 0 = ignore.
    pub max_dungeon_distance: i32,
    pub min_dungeon_distance: i32,
    pub max_evil_distance: i32,
    pub min_evil_distance: i32,
    pub max_sword_distance: i32,
    /// Multiplayer: every listed item must appear at least `players` times across all chests.
    pub players: u32,
    pub shared_items: Vec<String>,
    pub min_sword_shrines: u32,
    /// At least this many different rare items anywhere (the "lucky" goal, without naming them).
    pub min_rare_items: u32,
}

#[derive(Serialize, Default)]
pub struct Verdict {
    pub pass: bool,
    pub score: f64,
    pub met: Vec<String>,
    pub missing: Vec<String>,
    /// Spoiler-free hints, e.g. "2 rare items are hidden somewhere".
    pub teaser: Vec<String>,
}

fn has_item(a: &Analysis, name: &str, place: Option<&str>) -> bool {
    a.chests.iter().any(|c| {
        place.map(|p| c.place == p).unwrap_or(true) && c.items.iter().any(|i| i.name.eq_ignore_ascii_case(name))
    })
}

fn count_item(a: &Analysis, name: &str) -> u32 {
    a.chests.iter().flat_map(|c| c.items.iter()).filter(|i| i.name.eq_ignore_ascii_case(name)).count() as u32
}

#[derive(Serialize)]
pub struct Hint {
    pub topic: String,
    /// From vague to exact. The last one may name a place.
    pub levels: Vec<String>,
}

fn third(x: i32, w: i32) -> &'static str {
    if x < w / 3 { "the western third" } else if x < 2 * w / 3 { "the middle third" } else { "the eastern third" }
}

fn depth(a: &Analysis, y: i32) -> &'static str {
    let h = &a.header;
    if (y as f64) < h.surface * 0.4 { "high in the sky" } else if (y as f64) < h.surface { "on the surface" }
    else if (y as f64) < h.rock { "just underground" } else if y < h.height - 200 { "deep in the caverns" } else { "in the Underworld" }
}

/// Optional, graduated hints. Nothing here is shown unless the player asks.
pub fn hints(a: &Analysis) -> Vec<Hint> {
    let h = &a.header;
    let w = h.width;
    let dir = |x: i32| if x < h.spawn_x { "west" } else { "east" };
    let mut out = vec![];
    let mut seen = std::collections::HashSet::new();
    for l in a.loot.iter().filter(|l| l.tier == 2) {
        if !seen.insert(l.item.clone()) { continue; }
        out.push(Hint {
            topic: format!("Something rare ({})", if l.place == "Sky island" { "look up" } else { "look around" }),
            levels: vec![
                format!("A rare treasure waits {} of spawn.", dir(l.x)),
                format!("It is {} in {}, in a {} chest.", depth(a, l.y), third(l.x, w), l.place.to_lowercase()),
                format!("{}: about {} tiles {} of spawn, {}.", l.item, (l.x - h.spawn_x).abs(), dir(l.x), depth(a, l.y)),
            ],
        });
    }
    for p in a.pois.iter().filter(|p| p.kind == "sword_shrine") {
        out.push(Hint {
            topic: "A sword in stone".into(),
            levels: vec![
                format!("Somewhere {} of spawn, a blade glints.", dir(p.x)),
                format!("Search {} in {}.", depth(a, p.y), third(p.x, w)),
                format!("Enchanted Sword shrine about {} tiles {} of spawn.", (p.x - h.spawn_x).abs(), dir(p.x)),
            ],
        });
    }
    if let Some(p) = a.pois.iter().find(|p| p.kind == "pyramid") {
        out.push(Hint {
            topic: "Buried in the sand".into(),
            levels: vec![
                "One desert hides an old pyramid.".into(),
                format!("It stands {} of spawn, in {}.", dir(p.x), third(p.x, w)),
                format!("Pyramid about {} tiles {} of spawn.", (p.x - h.spawn_x).abs(), dir(p.x)),
            ],
        });
    }
    if let Some(p) = a.pois.iter().find(|p| p.kind == "shimmer") {
        out.push(Hint {
            topic: "A shimmering pool".into(),
            levels: vec![
                format!("The Aether lies on the {} side of the world.", a.summary.shimmer_side),
                format!("Dig {} near {}.", depth(a, p.y), third(p.x, w)),
                format!("Shimmer about {} tiles {} of spawn.", (p.x - h.spawn_x).abs(), dir(p.x)),
            ],
        });
    }
    out.push(Hint {
        topic: "Which way is what".into(),
        levels: vec![
            format!("The jungle grows to the {} of the world.", a.summary.jungle_side),
            format!("Snow lies to the {}, the dungeon to the {}.", a.summary.snow_side, a.summary.dungeon_side),
            format!("The dungeon is {} tiles from spawn.", a.summary.dungeon_distance),
        ],
    });
    out
}

pub fn judge(a: &Analysis, g: &Goals) -> Verdict {
    let mut v = Verdict { pass: true, ..Default::default() };
    let s = &a.summary;
    let need = |ok: bool, label: String, v: &mut Verdict, weight: f64| {
        if ok {
            v.met.push(label);
            v.score += weight;
        } else {
            v.missing.push(label);
            v.pass = false;
        }
    };
    for it in &g.items {
        need(has_item(a, it, None), format!("{it} in a chest"), &mut v, 10.0);
    }
    for it in &g.sky_items {
        need(has_item(a, it, Some("Sky island")), format!("{it} on a sky island"), &mut v, 10.0);
    }
    if g.sword_shrine {
        need(s.sword_shrines > 0, "Enchanted Sword shrine".into(), &mut v, 8.0);
    }
    if g.pyramid {
        need(s.pyramids > 0, "Desert pyramid".into(), &mut v, 8.0);
    }
    if g.min_sky_islands > 0 {
        need(s.sky_islands >= g.min_sky_islands, format!("At least {} sky islands", g.min_sky_islands), &mut v, 4.0);
    }
    if g.min_living_trees > 0 {
        need(s.living_trees >= g.min_living_trees, format!("At least {} living trees", g.min_living_trees), &mut v, 4.0);
    }
    if g.max_dungeon_distance > 0 {
        need(s.dungeon_distance <= g.max_dungeon_distance, format!("Dungeon within {} tiles of spawn", g.max_dungeon_distance), &mut v, 4.0);
    }
    if g.min_dungeon_distance > 0 {
        need(s.dungeon_distance >= g.min_dungeon_distance, format!("Dungeon at least {} tiles from spawn", g.min_dungeon_distance), &mut v, 4.0);
    }
    if g.max_evil_distance > 0 {
        need(s.evil_distance >= 0 && s.evil_distance <= g.max_evil_distance, format!("World evil within {} tiles of spawn", g.max_evil_distance), &mut v, 4.0);
    }
    if g.min_evil_distance > 0 {
        need(s.evil_distance < 0 || s.evil_distance >= g.min_evil_distance, format!("World evil at least {} tiles from spawn", g.min_evil_distance), &mut v, 4.0);
    }
    if g.max_sword_distance > 0 {
        need(s.sword_distance >= 0 && s.sword_distance <= g.max_sword_distance, format!("Sword shrine within {} tiles of spawn", g.max_sword_distance), &mut v, 6.0);
    }
    if g.min_sword_shrines > 0 {
        need(s.sword_shrines >= g.min_sword_shrines, format!("At least {} Enchanted Sword shrines", g.min_sword_shrines), &mut v, 6.0);
    }
    if g.players > 1 {
        for it in &g.shared_items {
            let n = count_item(a, it);
            need(n >= g.players, format!("{it} for each of {} players ({n} found)", g.players), &mut v, 5.0);
        }
    }
    if g.min_rare_items > 0 {
        let mut kinds: Vec<&str> = a.loot.iter().filter(|l| l.tier == 2).map(|l| l.item.as_str()).collect();
        kinds.sort();
        kinds.dedup();
        let n = kinds.len() as u32;
        need(n >= g.min_rare_items, format!("At least {} rare item{} hidden somewhere", g.min_rare_items, if g.min_rare_items == 1 { "" } else { "s" }), &mut v, 6.0);
    }
    for it in &g.bonus_items {
        if has_item(a, it, None) {
            v.score += 3.0;
            v.met.push(format!("Bonus: {it}"));
        }
    }
    // general luck: rare finds raise the score a little
    let rare = a.loot.iter().filter(|l| l.tier == 2).count();
    v.score += rare as f64 * 0.5 + s.sword_shrines as f64;
    if rare > 0 {
        v.teaser.push(format!("{rare} rare item{} hidden in chests", if rare == 1 { " is" } else { "s are" }));
    }
    if s.sword_shrines > 0 {
        v.teaser.push(format!("{} Enchanted Sword shrine{} somewhere", s.sword_shrines, if s.sword_shrines == 1 { "" } else { "s" }));
    }
    v.teaser.push(format!("{} sky island{}", s.sky_islands, if s.sky_islands == 1 { "" } else { "s" }));
    if s.pyramids > 0 {
        v.teaser.push("a pyramid in the desert".into());
    }
    v
}
