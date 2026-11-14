//! Renders a world to an RGBA image, one pixel per tile, using map colors.
use crate::data::data;
use crate::world::World;

const WATER: u32 = 0x093DBF;
const LAVA: u32 = 0xFD2003;
const HONEY: u32 = 0xFEC214;
const SHIMMER: u32 = 0xC9A0F0;
const SKY_TOP: u32 = 0x5C8CE6;
const SKY_LOW: u32 = 0x9CC2F5;
const DIRT_BG: u32 = 0x58402C;
const CAVE_BG: u32 = 0x4A433C;
const HELL_BG: u32 = 0x330F0A;

fn mix(a: u32, b: u32, t: f32) -> u32 {
    let ch = |s: u32| {
        let x = ((a >> s) & 255) as f32;
        let y = ((b >> s) & 255) as f32;
        ((x + (y - x) * t).round() as u32) << s
    };
    ch(16) | ch(8) | ch(0)
}

fn darken(c: u32, f: f32) -> u32 {
    mix(c, 0, f)
}

/// Layer flags: 1 tiles, 2 walls, 4 liquids, 8 background.
pub fn render(w: &World, layers: u32) -> Vec<u8> {
    let d = data();
    let (wd, ht) = (w.header.width as usize, w.header.height as usize);
    let surface = w.header.surface as usize;
    let rock = w.header.rock as usize;
    let hell = ht.saturating_sub(200);
    let mut out = vec![0u8; wd * ht * 4];
    for y in 0..ht {
        let bg = if layers & 8 == 0 {
            0x0B0D12
        } else if y < surface {
            mix(SKY_TOP, SKY_LOW, (y as f32 / surface.max(1) as f32).min(1.0))
        } else if y < rock {
            DIRT_BG
        } else if y < hell {
            CAVE_BG
        } else {
            HELL_BG
        };
        for x in 0..wd {
            let i = y * wd + x;
            let mut c = bg;
            let wall = w.walls[i];
            if layers & 2 != 0 && wall != 0 {
                if let Some(&wc) = d.wall_color.get(wall as usize) {
                    c = wc;
                }
            }
            let lq = w.liquid[i];
            let t = w.tiles[i];
            if layers & 1 != 0 && t != 0 {
                c = d.tile_color.get((t - 1) as usize).copied().unwrap_or(0xFF00FF);
                if w.paint[i] == 29 {
                    c = darken(c, 0.6); // shadow paint
                }
            } else if layers & 4 != 0 && lq != 0 {
                let lc = match lq {
                    1 => WATER,
                    2 => LAVA,
                    3 => HONEY,
                    _ => SHIMMER,
                };
                c = mix(c, lc, 0.85);
            }
            let o = i * 4;
            out[o] = (c >> 16) as u8;
            out[o + 1] = (c >> 8) as u8;
            out[o + 2] = c as u8;
            out[o + 3] = 255;
        }
    }
    out
}

/// Describes what is at a tile, for hover tooltips.
pub fn describe(w: &World, x: i32, y: i32) -> String {
    if x < 0 || y < 0 || x >= w.header.width || y >= w.header.height {
        return String::new();
    }
    let d = data();
    let i = w.idx(x, y);
    let mut parts = vec![];
    if let Some(t) = w.tile(x, y) {
        parts.push(d.tile_name(t).to_string());
    }
    let lq = w.liquid[i];
    if lq != 0 {
        parts.push(["", "Water", "Lava", "Honey", "Shimmer"][lq as usize].to_string());
    }
    let wall = w.walls[i];
    if wall != 0 {
        if let Some(n) = d.wall_name.get(wall as usize) {
            parts.push(n.clone());
        }
    }
    let ys = y as f64;
    let layer = if ys < w.header.surface * 0.35 {
        "Space"
    } else if ys < w.header.surface {
        "Surface"
    } else if ys < w.header.rock {
        "Underground"
    } else if y < w.header.height - 200 {
        "Cavern"
    } else {
        "Underworld"
    };
    if parts.is_empty() {
        parts.push("Air".into());
    }
    format!("{} · {}", parts.join(", "), layer)
}
