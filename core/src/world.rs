//! Reader for Terraria PC world files (.wld), save versions 194+ (1.3.x – 1.4.5).
use serde::Serialize;

pub struct Reader<'a> {
    d: &'a [u8],
    pub pos: usize,
}

type R<T> = Result<T, String>;

impl<'a> Reader<'a> {
    pub fn new(d: &'a [u8]) -> Self {
        Reader { d, pos: 0 }
    }
    fn need(&self, n: usize) -> R<()> {
        if self.pos + n > self.d.len() {
            Err(format!("unexpected end of file at byte {}", self.pos))
        } else {
            Ok(())
        }
    }
    pub fn u8(&mut self) -> R<u8> {
        self.need(1)?;
        self.pos += 1;
        Ok(self.d[self.pos - 1])
    }
    pub fn bool(&mut self) -> R<bool> {
        Ok(self.u8()? != 0)
    }
    pub fn i16(&mut self) -> R<i16> {
        self.need(2)?;
        let v = i16::from_le_bytes([self.d[self.pos], self.d[self.pos + 1]]);
        self.pos += 2;
        Ok(v)
    }
    pub fn i32(&mut self) -> R<i32> {
        self.need(4)?;
        let v = i32::from_le_bytes(self.d[self.pos..self.pos + 4].try_into().unwrap());
        self.pos += 4;
        Ok(v)
    }
    pub fn u32(&mut self) -> R<u32> {
        Ok(self.i32()? as u32)
    }
    pub fn i64(&mut self) -> R<i64> {
        self.need(8)?;
        let v = i64::from_le_bytes(self.d[self.pos..self.pos + 8].try_into().unwrap());
        self.pos += 8;
        Ok(v)
    }
    pub fn f64(&mut self) -> R<f64> {
        Ok(f64::from_bits(self.i64()? as u64))
    }
    pub fn skip(&mut self, n: usize) -> R<()> {
        self.need(n)?;
        self.pos += n;
        Ok(())
    }
    pub fn string(&mut self) -> R<String> {
        let mut n: usize = 0;
        let mut shift = 0;
        loop {
            let b = self.u8()?;
            n |= ((b & 0x7f) as usize) << shift;
            shift += 7;
            if b < 128 || shift > 28 {
                break;
            }
        }
        self.need(n)?;
        let s = String::from_utf8_lossy(&self.d[self.pos..self.pos + n]).into_owned();
        self.pos += n;
        Ok(s)
    }
}

#[derive(Serialize, Clone, Default)]
pub struct Header {
    pub version: u32,
    pub name: String,
    pub seed: String,
    pub width: i32,
    pub height: i32,
    pub game_mode: i32,
    pub drunk: bool,
    pub for_the_worthy: bool,
    pub celebration: bool,
    pub the_constant: bool,
    pub not_the_bees: bool,
    pub remix: bool,
    pub no_traps: bool,
    pub zenith: bool,
    pub skyblock: bool,
    pub spawn_x: i32,
    pub spawn_y: i32,
    pub surface: f64,
    pub rock: f64,
    pub dungeon_x: i32,
    pub dungeon_y: i32,
    pub crimson: bool,
    pub hardmode: bool,
    pub game_version: String,
}

pub const LIQ_NONE: u8 = 0;
pub const LIQ_WATER: u8 = 1;
pub const LIQ_LAVA: u8 = 2;
pub const LIQ_HONEY: u8 = 3;
pub const LIQ_SHIMMER: u8 = 4;

/// A framed tile anchor captured during parsing (top-left of multi-tile objects we care about).
#[derive(Clone, Copy)]
pub struct Anchor {
    pub tile: u16,
    pub x: i32,
    pub y: i32,
    pub u: i16,
    pub v: i16,
}

#[derive(Serialize, Clone)]
pub struct ChestItem {
    pub id: i32,
    pub name: String,
    pub stack: i16,
    pub prefix: u8,
}

#[derive(Serialize, Clone)]
pub struct Chest {
    pub x: i32,
    pub y: i32,
    pub name: String,
    pub items: Vec<ChestItem>,
}

pub struct World {
    pub header: Header,
    /// tile type + 1, 0 = empty. Column-major is how the file stores it; we store row-major.
    pub tiles: Vec<u16>,
    pub walls: Vec<u16>,
    pub liquid: Vec<u8>,
    pub paint: Vec<u8>,
    pub anchors: Vec<Anchor>,
    pub chests: Vec<Chest>,
}

/// Tiles whose frame anchors we record.
fn anchor_wanted(t: u16) -> bool {
    matches!(t, 21 | 467 | 26 | 31 | 12 | 187 | 237 | 231 | 77 | 88 | 236 | 238)
}

impl World {
    #[inline]
    pub fn idx(&self, x: i32, y: i32) -> usize {
        (y as usize) * (self.header.width as usize) + x as usize
    }
    #[inline]
    pub fn tile(&self, x: i32, y: i32) -> Option<u16> {
        let t = self.tiles[self.idx(x, y)];
        if t == 0 { None } else { Some(t - 1) }
    }

    pub fn parse(d: &[u8]) -> R<World> {
        let mut r = Reader::new(d);
        let version = r.u32()?;
        if version < 194 {
            return Err(format!("World save version {version} is too old (needs Terraria 1.3.5 or newer)."));
        }
        let magic = &d[4..11];
        if magic != b"relogic" && magic != b"xindong" {
            return Err("This is not a Terraria world file.".into());
        }
        r.pos = 11;
        let ftype = r.u8()?;
        if ftype != 2 {
            return Err("This file is not a world (maybe a player or map file).".into());
        }
        r.skip(4 + 8)?; // revision, flags
        let nsec = r.i16()? as usize;
        let mut sec = Vec::with_capacity(nsec);
        for _ in 0..nsec {
            sec.push(r.i32()? as usize);
        }
        let nbits = r.i16()? as usize;
        let mut frame_imp = vec![false; nbits];
        let mut byte = 0u8;
        let mut mask = 128u8;
        for b in frame_imp.iter_mut() {
            if mask != 128 {
                mask <<= 1;
            } else {
                byte = r.u8()?;
                mask = 1;
            }
            *b = byte & mask == mask;
        }
        if nsec < 3 {
            return Err("World file is missing sections.".into());
        }

        // --- header section ---
        r.pos = sec[0];
        let mut h = Header { version, ..Default::default() };
        h.name = r.string()?;
        h.seed = r.string()?;
        r.skip(8)?; // worldgen version
        r.skip(16)?; // guid
        r.skip(4)?; // world id
        r.skip(16)?; // bounds
        h.height = r.i32()?;
        h.width = r.i32()?;
        if !(100..=20000).contains(&h.width) || !(100..=10000).contains(&h.height) {
            return Err(format!("Unexpected world size {}x{}", h.width, h.height));
        }
        if version >= 209 {
            h.game_mode = r.i32()?;
            if version >= 222 { h.drunk = r.bool()?; }
            if version >= 227 { h.for_the_worthy = r.bool()?; }
            if version >= 238 { h.celebration = r.bool()?; }
            if version >= 239 { h.the_constant = r.bool()?; }
            if version >= 241 { h.not_the_bees = r.bool()?; }
            if version >= 249 { h.remix = r.bool()?; }
            if version >= 266 { h.no_traps = r.bool()?; }
            h.zenith = if version < 267 { h.remix && h.drunk } else { r.bool()? };
            if version >= 302 { h.skyblock = r.bool()?; }
        } else {
            h.game_mode = if r.bool()? { 1 } else { 0 };
        }
        r.skip(8)?; // creation time
        if version >= 284 { r.skip(8)?; } // last played
        r.skip(1)?; // moon type
        r.skip(12 + 16 + 12 + 16 + 12)?; // tree x/style, cave back x/style, ice/jungle/hell bg
        h.spawn_x = r.i32()?;
        h.spawn_y = r.i32()?;
        h.surface = r.f64()?;
        h.rock = r.f64()?;
        r.skip(8 + 1 + 4 + 1 + 1)?; // time, daytime, moon phase, blood moon, eclipse
        h.dungeon_x = r.i32()?;
        h.dungeon_y = r.i32()?;
        h.crimson = r.bool()?;
        r.skip(10)?; // downed boss flags
        r.skip(1)?; // slime king (>=118)
        r.skip(3 + 4)?; // saved npcs, downed invasions
        r.skip(1 + 1 + 1 + 4)?; // orb smashed, meteor, orb count, altar count
        h.hardmode = r.bool()?;
        // game version string lives in the generation manifest at the end of the header
        if let Some(i) = find(&d[sec[0]..sec[1].min(d.len())], b"\"Version\":\"") {
            let s = sec[0] + i + 11;
            let e = d[s..].iter().position(|&c| c == b'"').map(|p| s + p).unwrap_or(s);
            h.game_version = String::from_utf8_lossy(&d[s..e]).into_owned();
        }

        // --- tiles ---
        r.pos = sec[1];
        let w = h.width as usize;
        let hh = h.height as usize;
        let n = w * hh;
        let mut tiles = vec![0u16; n];
        let mut walls = vec![0u16; n];
        let mut liquid = vec![0u8; n];
        let mut paint = vec![0u8; n];
        let mut anchors = Vec::new();
        for x in 0..w {
            let mut y = 0usize;
            while y < hh {
                let h1 = r.u8()?;
                                let mut h3 = 0u8;
                let mut h4 = 0u8;
                if h1 & 1 != 0 {
                    let h2 = r.u8()?;
                    if h2 & 1 != 0 {
                        h3 = r.u8()?;
                        if version >= 269 && h3 & 1 != 0 {
                            h4 = r.u8()?;
                        }
                    }
                }
                let _ = h4;
                let mut ttype: i32 = -1;
                let mut u: i16 = 0;
                let mut v: i16 = 0;
                let mut tcolor = 0u8;
                if h1 & 2 != 0 {
                    ttype = if h1 & 0x20 == 0 {
                        r.u8()? as i32
                    } else {
                        let lo = r.u8()? as i32;
                        let hi = r.u8()? as i32;
                        hi << 8 | lo
                    };
                    let framed = (ttype as usize) >= frame_imp.len() || frame_imp[ttype as usize];
                    if framed {
                        u = r.i16()?;
                        v = r.i16()?;
                    }
                    if h3 & 8 != 0 {
                        tcolor = r.u8()?;
                    }
                }
                let mut wall: u16 = 0;
                if h1 & 4 != 0 {
                    wall = r.u8()? as u16;
                    if h3 & 16 != 0 {
                        r.skip(1)?;
                    }
                }
                let lt = (h1 & 0x18) >> 3;
                let mut liq = LIQ_NONE;
                if lt != 0 {
                    let _amount = r.u8()?;
                    liq = if version >= 269 && h3 & 0x80 != 0 { LIQ_SHIMMER } else { lt };
                }
                if h3 > 1 && version >= 222 && h3 & 0x40 != 0 {
                    wall |= (r.u8()? as u16) << 8;
                }
                let rle = match (h1 & 0xC0) >> 6 {
                    0 => 0usize,
                    1 => r.u8()? as usize,
                    _ => r.i16()? as u16 as usize,
                };
                let tv = if ttype >= 0 { (ttype as u16).wrapping_add(1) } else { 0 };
                if ttype >= 0 && anchor_wanted(ttype as u16) {
                    anchors.push(Anchor { tile: ttype as u16, x: x as i32, y: y as i32, u, v });
                }
                let end = (y + 1 + rle).min(hh);
                for yy in y..end {
                    let i = yy * w + x;
                    tiles[i] = tv;
                    walls[i] = wall;
                    liquid[i] = liq;
                    paint[i] = tcolor;
                }
                y = end;
            }
        }

        // --- chests ---
        r.pos = sec[2];
        let total = r.i16()? as usize;
        let mut max_items = 40usize;
        if version < 294 {
            max_items = r.i16()? as usize;
        }
        let dat = crate::data::data();
        let mut chests = Vec::with_capacity(total);
        for _ in 0..total {
            let cx = r.i32()?;
            let cy = r.i32()?;
            let name = r.string()?;
            let slots = if version >= 294 { r.i32()? as usize } else { max_items };
            let mut items = Vec::new();
            for _ in 0..slots {
                let stack = r.i16()?;
                if stack > 0 {
                    let id = r.i32()?;
                    let prefix = r.u8()?;
                    items.push(ChestItem { id, name: dat.item_name(id), stack, prefix });
                }
            }
            chests.push(Chest { x: cx, y: cy, name, items });
        }

        Ok(World { header: h, tiles, walls, liquid, paint, anchors, chests })
    }
}

fn find(hay: &[u8], needle: &[u8]) -> Option<usize> {
    hay.windows(needle.len()).rposition(|w| w == needle)
}
