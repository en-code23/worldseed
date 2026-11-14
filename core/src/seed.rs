//! Terraria seed strings, special seeds and 1.4.5 secret seeds.
use serde::{Deserialize, Serialize};

#[derive(Serialize, Clone, Copy)]
pub struct Special {
    pub key: &'static str,
    pub name: &'static str,
    pub code: &'static str,
    pub bit: u32,
    pub since: &'static str,
    pub about: &'static str,
}

#[derive(Serialize, Clone, Copy)]
pub struct Secret {
    pub key: &'static str,
    pub name: &'static str,
    pub code: &'static str,
    pub since: &'static str,
    pub about: &'static str,
}

pub const SPECIALS: &[Special] = &[
    Special { key: "drunk", name: "Drunk world", code: "05162020", bit: 1, since: "1.4.0.1", about: "Corruption and Crimson together, wild generation." },
    Special { key: "notthebees", name: "Not the Bees", code: "not the bees", bit: 2, since: "1.4.0.1", about: "Honey, hives and jungle everywhere." },
    Special { key: "ftw", name: "For the Worthy", code: "for the worthy", bit: 4, since: "1.4.0.3", about: "Much harder enemies and bosses." },
    Special { key: "celebration", name: "Celebration Mk 10", code: "celebrationmk10", bit: 8, since: "1.4.2.3", about: "10th anniversary world, colourful and festive." },
    Special { key: "constant", name: "The Constant", code: "the constant", bit: 16, since: "1.4.3", about: "Don't Starve rules: hunger, darkness hurts." },
    Special { key: "remix", name: "Remix (Don't dig up)", code: "dont dig up", bit: 32, since: "1.4.4", about: "Upside-down world. You start deep underground." },
    Special { key: "notraps", name: "No Traps", code: "no traps", bit: 64, since: "1.4.4", about: "Ironically: traps everywhere." },
    Special { key: "zenith", name: "Zenith (Get fixed boi)", code: "get fixed boi", bit: 128, since: "1.4.4", about: "Every special seed at once, plus extras." },
    Special { key: "skyblock", name: "Skyblock", code: "skyblock", bit: 256, since: "1.4.5.0", about: "Start on a small floating island." },
];

pub const SECRETS: &[Secret] = &[
    Secret { key: "coatEverythingEcho", name: "Coat everything Echo", code: "invisible plane", since: "1.4.5.0", about: "The whole world is echo-coated (invisible)." },
    Secret { key: "paintEverythingGray", name: "Paint everything Gray", code: "monochrome", since: "1.4.5.0", about: "Everything painted gray." },
    Secret { key: "paintEverythingNegative", name: "Paint everything Negative", code: "negative infinity", since: "1.4.5.0", about: "Everything painted negative." },
    Secret { key: "coatEverythingIlluminant", name: "Coat everything Illuminant", code: "x-ray vision", since: "1.4.5.0", about: "Everything glows." },
    Secret { key: "digExtraHoles", name: "Dig extra holes", code: "jagged rocks", since: "1.4.5.0", about: "Extra chasms and pits." },
    Secret { key: "noSurface", name: "No surface", code: "mole people", since: "1.4.5.0", about: "There is no surface layer." },
    Secret { key: "roundLandmasses", name: "Round landmasses", code: "planetoids", since: "1.4.5.0", about: "The world is made of planetoids." },
    Secret { key: "surfaceIsInSpace", name: "Surface is in Space", code: "such great heights", since: "1.4.5.0", about: "The surface sits much higher." },
    Secret { key: "extraLiquid", name: "Extra liquid", code: "waterpark", since: "1.4.5.0", about: "The world is flooded." },
    Secret { key: "noSpiderCaves", name: "No spider nests", code: "arachnophobia", since: "1.4.5.0", about: "No spider caves." },
    Secret { key: "hallowOnTheSurface", name: "Hallow on the surface", code: "does that sparkle", since: "1.4.5.0", about: "The Hallow exists from the start." },
    Secret { key: "noInfection", name: "No infection", code: "fish mox", since: "1.4.5.0", about: "No Corruption or Crimson at the start." },
    Secret { key: "worldIsInfected", name: "World is infected", code: "purify this", since: "1.4.5.0", about: "The entire world is infected." },
    Secret { key: "surfaceIsDesert", name: "Surface is Desert", code: "sandy britches", since: "1.4.5.0", about: "The surface is one big desert." },
    Secret { key: "surfaceIsMushrooms", name: "Surface is mushrooms", code: "toadstool", since: "1.4.5.0", about: "Glowing mushrooms on the surface." },
    Secret { key: "worldIsFrozen", name: "World is frozen", code: "winter is coming", since: "1.4.5.0", about: "Snow and ice everywhere." },
    Secret { key: "biggerAbandonedHouses", name: "Bigger abandoned houses", code: "abandoned manors", since: "1.4.5.0", about: "Larger underground cabins." },
    Secret { key: "addTeleporters", name: "Add teleporters", code: "beam me up", since: "1.4.5.0", about: "Teleporters scattered around." },
    Secret { key: "actuallyNoTraps", name: "Actually no traps", code: "more traps please", since: "1.4.5.0", about: "No traps at all." },
    Secret { key: "halloweenGen", name: "Halloween gen", code: "pumpkin season", since: "1.4.5.0", about: "Starts with pumpkins." },
    Secret { key: "rainbowStuff", name: "Rainbow stuff", code: "rainbow road", since: "1.4.5.0", about: "Rainbow blocks and more." },
    Secret { key: "extraLivingTrees", name: "Extra living trees", code: "save the rainforest", since: "1.4.5.0", about: "More and larger living trees." },
    Secret { key: "extraFloatingIslands", name: "Extra floating islands", code: "the care bears movie", since: "1.4.5.0", about: "Many more sky islands." },
    Secret { key: "pooEverywhere", name: "Poo everywhere", code: "truck stop", since: "1.4.5.0", about: "The world is covered in Poo." },
    Secret { key: "portalGunInChests", name: "Portal Gun in chests", code: "we don't even test for that", since: "1.4.5.0", about: "Portal Guns can appear in any chest." },
    Secret { key: "rainsForAYear", name: "Rains for a year", code: "bring a towel", since: "1.4.5.0", about: "It never stops raining." },
    Secret { key: "endlessHalloween", name: "Endless Halloween", code: "hocus pocus", since: "1.4.5.0", about: "Halloween all year." },
    Secret { key: "endlessChristmas", name: "Endless Christmas", code: "jingle all the way", since: "1.4.5.0", about: "Christmas all year." },
    Secret { key: "moreLightning", name: "More lightning", code: "electric boogaloo", since: "1.4.5.7", about: "Lightning strikes more often." },
    Secret { key: "noLightning", name: "No lightning", code: "calm before the storm", since: "1.4.5.7", about: "Lightning never strikes." },
    Secret { key: "randomSpawn", name: "Random spawn", code: "how did i get here", since: "1.4.5.0", about: "You spawn somewhere random." },
    Secret { key: "teamBasedSpawns", name: "Team based spawns", code: "royale with cheese", since: "1.4.5.0", about: "Each team spawns elsewhere." },
    Secret { key: "dualDungeons", name: "Dual dungeons", code: "double daring dangers", since: "1.4.5.0", about: "Two dungeons." },
    Secret { key: "errorWorld", name: "Error world", code: "i am error", since: "1.4.5.0", about: "A glitched world." },
    Secret { key: "graveyardBloodmoonStart", name: "Graveyard Blood Moon start", code: "night of the living dead", since: "1.4.5.0", about: "Graveyards and a Blood Moon on night one." },
    Secret { key: "startInHardmode", name: "Start in Hardmode", code: "too easy", since: "1.4.5.0", about: "The world begins in Hardmode." },
    Secret { key: "vampirism", name: "Vampirism", code: "what a horrible night to have a curse", since: "1.4.5.0", about: "You are a vampire." },
];

#[derive(Deserialize, Default)]
#[serde(default)]
pub struct SeedOptions {
    /// 1 small, 2 medium, 3 large
    pub size: u8,
    /// 1 classic, 2 expert, 3 master, 4 journey
    pub difficulty: u8,
    /// 1 random, 2 corruption, 3 crimson
    pub evil: u8,
    pub specials: Vec<String>,
    pub secrets: Vec<String>,
    pub seed: String,
}

pub fn special_sum(keys: &[String]) -> u32 {
    let mut s = 0;
    for k in keys {
        if let Some(sp) = SPECIALS.iter().find(|x| x.key == k) {
            s |= sp.bit;
        }
    }
    if s & 128 != 0 {
        s |= 255;
    }
    s
}

/// `size.difficulty.evil.specials.secret|secret|seed` — the 1.4.5 full seed format.
pub fn build(o: &SeedOptions) -> String {
    let mut tail: Vec<String> = vec![];
    for k in &o.secrets {
        if let Some(s) = SECRETS.iter().find(|x| x.key == k) {
            tail.push(s.code.to_string());
        }
    }
    tail.push(o.seed.trim().to_string());
    format!(
        "{}.{}.{}.{}.{}",
        o.size.clamp(1, 3),
        o.difficulty.clamp(1, 4),
        o.evil.clamp(1, 3),
        special_sum(&o.specials),
        tail.join("|")
    )
}

pub fn catalog_json() -> String {
    serde_json::json!({ "specials": SPECIALS, "secrets": SECRETS }).to_string()
}
