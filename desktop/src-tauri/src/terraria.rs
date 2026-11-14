//! Finds the player's Terraria install and drives its dedicated server to generate real worlds.
use serde::Serialize;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Serialize, Clone, Default)]
pub struct Install {
    pub found: bool,
    pub root: String,
    pub server: String,
    pub version: String,
    pub worlds_dir: String,
}

fn home() -> PathBuf {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

/// Where Terraria keeps worlds on this platform.
pub fn worlds_dir() -> PathBuf {
    if cfg!(target_os = "macos") {
        home().join("Library/Application Support/Terraria/Worlds")
    } else if cfg!(target_os = "windows") {
        let docs = std::env::var_os("USERPROFILE").map(PathBuf::from).unwrap_or_else(home).join("Documents");
        docs.join("My Games").join("Terraria").join("Worlds")
    } else {
        std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home().join(".local/share"))
            .join("Terraria/Worlds")
    }
}

fn steam_libraries() -> Vec<PathBuf> {
    let mut roots: Vec<PathBuf> = vec![];
    if cfg!(target_os = "macos") {
        roots.push(home().join("Library/Application Support/Steam"));
    } else if cfg!(target_os = "windows") {
        for v in ["ProgramFiles(x86)", "ProgramFiles"] {
            if let Some(p) = std::env::var_os(v) {
                roots.push(PathBuf::from(p).join("Steam"));
            }
        }
        for d in ["C", "D", "E", "F"] {
            roots.push(PathBuf::from(format!("{d}:\\SteamLibrary")));
            roots.push(PathBuf::from(format!("{d}:\\Steam")));
        }
    } else {
        roots.push(home().join(".steam/steam"));
        roots.push(home().join(".local/share/Steam"));
        roots.push(home().join(".var/app/com.valvesoftware.Steam/.local/share/Steam"));
    }
    let mut libs = roots.clone();
    for r in &roots {
        if let Ok(txt) = std::fs::read_to_string(r.join("steamapps/libraryfolders.vdf")) {
            for line in txt.lines() {
                let l = line.trim();
                if let Some(rest) = l.strip_prefix("\"path\"") {
                    let p = rest.trim().trim_matches('"').replace("\\\\", "\\");
                    libs.push(PathBuf::from(p));
                }
            }
        }
    }
    libs
}

fn server_in(root: &Path) -> Option<PathBuf> {
    let cands = if cfg!(target_os = "macos") {
        vec![root.join("Terraria.app/Contents/MacOS/TerrariaServer"), root.join("Contents/MacOS/TerrariaServer"), root.join("TerrariaServer")]
    } else if cfg!(target_os = "windows") {
        vec![root.join("TerrariaServer.exe")]
    } else {
        vec![root.join("TerrariaServer"), root.join("TerrariaServer.bin.x86_64")]
    };
    cands.into_iter().find(|p| p.exists())
}

fn version_in(root: &Path) -> String {
    for p in [root.join("changelog.txt"), root.join("../changelog.txt"), root.join("../../../changelog.txt")] {
        if let Ok(t) = std::fs::read_to_string(&p) {
            if let Some(l) = t.lines().find(|l| l.trim_start_matches('\u{feff}').starts_with("Version ")) {
                return l.trim_start_matches('\u{feff}').trim_start_matches("Version ").split_whitespace().next().unwrap_or("").to_string();
            }
        }
    }
    String::new()
}

pub fn inspect(root: &Path) -> Install {
    match server_in(root) {
        Some(s) => Install {
            found: true,
            root: root.display().to_string(),
            server: s.display().to_string(),
            version: version_in(root),
            worlds_dir: worlds_dir().display().to_string(),
        },
        None => Install { worlds_dir: worlds_dir().display().to_string(), ..Default::default() },
    }
}

pub fn detect() -> Install {
    for lib in steam_libraries() {
        let root = lib.join("steamapps/common/Terraria");
        let i = inspect(&root);
        if i.found {
            return i;
        }
    }
    if cfg!(target_os = "windows") {
        for p in ["C:\\GOG Games\\Terraria", "C:\\Program Files (x86)\\GOG Galaxy\\Games\\Terraria"] {
            let i = inspect(Path::new(p));
            if i.found {
                return i;
            }
        }
    }
    Install { worlds_dir: worlds_dir().display().to_string(), ..Default::default() }
}

pub struct GenSpec {
    pub size: u8,
    pub difficulty: u8,
    pub evil: u8,
    pub name: String,
    /// Full seed string (1.4.5+) or plain seed text (older versions).
    pub seed: String,
}

fn spawn(server: &Path, savedir: &Path) -> std::io::Result<Child> {
    let dir = server.parent().unwrap_or(Path::new("."));
    let mut cmd = Command::new(server);
    cmd.current_dir(dir)
        .arg("-savedirectory")
        .arg(savedir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    cmd.spawn()
}

/// Runs the real Terraria world generator. Calls `progress(percent, step)` as it goes.
pub fn generate(
    server: &Path,
    savedir: &Path,
    spec: &GenSpec,
    cancel: &Arc<AtomicBool>,
    mut progress: impl FnMut(f32, &str),
) -> Result<PathBuf, String> {
    std::fs::create_dir_all(savedir.join("Worlds")).map_err(|e| e.to_string())?;
    let mut child = spawn(server, savedir).map_err(|e| format!("Could not start TerrariaServer: {e}"))?;
    let mut stdin = child.stdin.take().ok_or("no stdin")?;
    let mut stdout = child.stdout.take().ok_or("no stdout")?;
    let answers = format!(
        "n\n{}\n{}\n{}\n{}\n{}\n\n",
        spec.size, spec.difficulty, spec.evil, spec.name, spec.seed
    );
    let _ = stdin.write_all(answers.as_bytes());
    let _ = stdin.flush();

    let start = Instant::now();
    let mut buf = [0u8; 4096];
    let mut acc = String::new();
    let mut created = false;
    let mut done = false;
    let result = loop {
        if cancel.load(Ordering::Relaxed) {
            break Err("cancelled".to_string());
        }
        if start.elapsed() > Duration::from_secs(900) {
            break Err("World generation timed out".into());
        }
        let n = match stdout.read(&mut buf) {
            Ok(0) => break if done { Ok(()) } else { Err("TerrariaServer exited before the world was saved".into()) },
            Ok(n) => n,
            Err(e) => break Err(e.to_string()),
        };
        acc.push_str(&String::from_utf8_lossy(&buf[..n]));
        // process complete lines (the server uses \n and sometimes \r)
        while let Some(pos) = acc.find(['\n', '\r']) {
            let line: String = acc.drain(..=pos).collect();
            let line = line.trim();
            if line.starts_with("Creating world") {
                created = true;
            }
            if let Some((pct, rest)) = line.split_once('%') {
                if let Ok(p) = pct.trim().parse::<f32>() {
                    let step = rest.trim_start_matches(" - ").split(" - ").next().unwrap_or("");
                    progress(p, step);
                }
            }
        }
        if created && acc.contains("Choose World") {
            done = true;
            break Ok(());
        }
    };
    let _ = child.kill();
    let _ = child.wait();
    result?;
    let worlds = savedir.join("Worlds");
    let mut best: Option<PathBuf> = None;
    for e in std::fs::read_dir(&worlds).map_err(|e| e.to_string())?.flatten() {
        let p = e.path();
        if p.extension().map(|x| x == "wld").unwrap_or(false) {
            best = Some(p);
        }
    }
    best.ok_or_else(|| "The server finished but no world file was written".into())
}

/// Copies a generated world into Terraria's Worlds folder without overwriting anything.
pub fn install(src: &Path, name: &str) -> Result<PathBuf, String> {
    let dir = worlds_dir();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let clean: String = name.chars().map(|c| if c.is_alphanumeric() || c == '-' { c } else { '_' }).collect();
    let clean = if clean.is_empty() { "Worldseed".to_string() } else { clean };
    let mut dest = dir.join(format!("{clean}.wld"));
    let mut n = 2;
    while dest.exists() {
        dest = dir.join(format!("{clean}_{n}.wld"));
        n += 1;
    }
    std::fs::copy(src, &dest).map_err(|e| e.to_string())?;
    Ok(dest)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore]
    fn generates_with_local_install() {
        let i = detect();
        assert!(i.found, "no install");
        let dir = std::env::temp_dir().join("worldseed-test");
        let _ = std::fs::remove_dir_all(&dir);
        let spec = GenSpec { size: 1, difficulty: 3, evil: 3, name: "WsTest".into(), seed: "1.3.3.0.12345".into() };
        let mut last = 0.0;
        let p = generate(Path::new(&i.server), &dir, &spec, &Arc::new(AtomicBool::new(false)), |pct, _| last = pct).unwrap();
        let w = seedcore::World::parse(&std::fs::read(&p).unwrap()).unwrap();
        println!("version {} path {} last {last} mode {} crimson {} seed {}", i.version, p.display(), w.header.game_mode, w.header.crimson, w.header.seed);
        assert_eq!(w.header.game_mode, 2);
        assert!(w.header.crimson);
    }
}
