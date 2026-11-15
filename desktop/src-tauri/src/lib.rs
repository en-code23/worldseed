mod terraria;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager, State};

#[derive(Default)]
struct AppState {
    install: Mutex<terraria::Install>,
    cancel: Mutex<Arc<AtomicBool>>,
}

/// Per-user working folder for generated worlds (never the shared /tmp).
fn tmp_root() -> PathBuf {
    let base = if cfg!(target_os = "windows") {
        std::env::var_os("LOCALAPPDATA").map(PathBuf::from)
    } else if cfg!(target_os = "macos") {
        std::env::var_os("HOME").map(|h| PathBuf::from(h).join("Library/Caches"))
    } else {
        std::env::var_os("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache")))
    };
    base.unwrap_or_else(std::env::temp_dir).join("app.worldseed").join("worlds")
}

fn rand_u64() -> u64 {
    // xorshift seeded from time + a counter; good enough for picking seeds
    static CTR: AtomicU32 = AtomicU32::new(0);
    let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos() as u64;
    let mut x = t ^ ((CTR.fetch_add(1, Ordering::Relaxed) as u64) << 32) ^ 0x9E37_79B9_7F4A_7C15;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    x
}

#[tauri::command]
fn detect(state: State<AppState>) -> terraria::Install {
    let i = terraria::detect();
    *state.install.lock().unwrap() = i.clone();
    i
}

#[tauri::command]
fn set_install(path: String, state: State<AppState>) -> terraria::Install {
    let i = terraria::inspect(std::path::Path::new(&path));
    if i.found {
        *state.install.lock().unwrap() = i.clone();
    }
    i
}

/// Per-user folder where Worldseed keeps worlds it made that are not in Terraria yet.
fn library_dir() -> PathBuf {
    let base = if cfg!(target_os = "windows") {
        std::env::var_os("APPDATA").map(PathBuf::from)
    } else if cfg!(target_os = "macos") {
        std::env::var_os("HOME").map(|h| PathBuf::from(h).join("Library/Application Support"))
    } else {
        std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))
    };
    base.unwrap_or_else(std::env::temp_dir).join("app.worldseed").join("library")
}

/// Moves a freshly generated world into the library under a name that is free.
fn keep_in_library(src: &std::path::Path, name: &str) -> Result<PathBuf, String> {
    let dir = library_dir();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let clean: String = name.chars().map(|c| if c.is_alphanumeric() || c == '-' { c } else { '_' }).collect();
    let clean = if clean.is_empty() { "World".to_string() } else { clean };
    let mut dest = dir.join(format!("{clean}.wld"));
    let mut n = 2;
    while dest.exists() {
        dest = dir.join(format!("{clean}_{n}.wld"));
        n += 1;
    }
    if std::fs::rename(src, &dest).is_err() {
        std::fs::copy(src, &dest).map_err(|e| e.to_string())?;
    }
    Ok(dest)
}

fn inside_known_folders(p: &std::path::Path) -> bool {
    p.starts_with(tmp_root()) || p.starts_with(terraria::worlds_dir()) || p.starts_with(library_dir())
}

#[derive(Serialize)]
struct LibEntry {
    path: String,
    /// "terraria" (in the Worlds folder) or "worldseed" (kept by Worldseed)
    source: &'static str,
    modified: u64,
    bytes: u64,
    header: Option<seedcore::world::Header>,
    error: Option<String>,
}

fn scan(dir: &std::path::Path, source: &'static str, out: &mut Vec<LibEntry>) {
    use std::io::Read;
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let p = e.path();
        if p.extension().map(|x| x != "wld").unwrap_or(true) {
            continue;
        }
        let meta = e.metadata().ok();
        let modified = meta.as_ref().and_then(|m| m.modified().ok()).and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_secs()).unwrap_or(0);
        // the header section sits at the start of the file; 512 KB covers it many times over
        let mut buf = Vec::new();
        let read = std::fs::File::open(&p).and_then(|f| f.take(512 * 1024).read_to_end(&mut buf));
        let (header, error) = match read {
            // a damaged file becomes an error row, never a crash of the whole list
            Ok(_) => match std::panic::catch_unwind(|| seedcore::World::parse_header(&buf)) {
                Ok(Ok(h)) => (Some(h), None),
                Ok(Err(e)) => (None, Some(e)),
                Err(_) => (None, Some("World file is damaged.".into())),
            },
            Err(e) => (None, Some(e.to_string())),
        };
        out.push(LibEntry { path: p.display().to_string(), source, modified, bytes: meta.map(|m| m.len()).unwrap_or(0), header, error });
    }
}

#[tauri::command]
async fn list_worlds() -> Vec<LibEntry> {
    tauri::async_runtime::spawn_blocking(|| {
        let mut out = vec![];
        scan(&terraria::worlds_dir(), "terraria", &mut out);
        scan(&library_dir(), "worldseed", &mut out);
        out.sort_by(|a, b| b.modified.cmp(&a.modified));
        out
    })
    .await
    .unwrap_or_default()
}

/// Removes a world Worldseed kept. Worlds in Terraria's own folder are never deleted from here.
#[tauri::command]
fn remove_world(path: String) -> Result<(), String> {
    let p = PathBuf::from(&path);
    if !p.starts_with(library_dir()) || p.extension().map(|x| x != "wld").unwrap_or(true) {
        return Err("Worldseed only removes worlds it keeps itself. Delete Terraria worlds in Terraria.".into());
    }
    std::fs::remove_file(&p).map_err(|e| e.to_string())
}

#[tauri::command]
fn library_path() -> String {
    library_dir().display().to_string()
}

#[derive(Deserialize)]
struct GenRequest {
    size: u8,
    difficulty: u8,
    evil: u8,
    name: String,
    seed: String,
    job: String,
    /// keep a copy in the library (true for "generate and open map")
    #[serde(default)]
    keep: bool,
}

#[derive(Serialize, Clone)]
struct Progress {
    job: String,
    pct: f32,
    step: String,
}

#[tauri::command]
async fn generate(req: GenRequest, app: AppHandle, state: State<'_, AppState>) -> Result<String, String> {
    let install = state.install.lock().unwrap().clone();
    if !install.found {
        return Err("Terraria was not found. Choose your Terraria folder first.".into());
    }
    let cancel = Arc::new(AtomicBool::new(false));
    *state.cancel.lock().unwrap() = cancel.clone();
    let dir = tmp_root().join(format!("gen-{}", rand_u64() % 1_000_000_000));
    let spec = terraria::GenSpec { size: req.size, difficulty: req.difficulty, evil: req.evil, name: req.name, seed: req.seed };
    let job = req.job.clone();
    let keep = req.keep;
    let keep_name = spec.name.clone();
    let res = tauri::async_runtime::spawn_blocking(move || {
        terraria::generate(std::path::Path::new(&install.server), &dir, &spec, &cancel, |pct, step| {
            let _ = app.emit("gen-progress", Progress { job: job.clone(), pct, step: step.to_string() });
        })
    })
    .await
    .map_err(|e| e.to_string())??;
    if keep {
        return keep_in_library(&res, &keep_name).map(|p| p.display().to_string());
    }
    Ok(res.display().to_string())
}

#[tauri::command]
fn read_world(path: String) -> Result<tauri::ipc::Response, String> {
    let p = PathBuf::from(&path);
    if !inside_known_folders(&p) {
        return Err("Refusing to read a file outside Worldseed's folders".into());
    }
    std::fs::read(&p).map(tauri::ipc::Response::new).map_err(|e| e.to_string())
}

#[tauri::command]
fn install_world(path: String, name: String) -> Result<String, String> {
    terraria::install(std::path::Path::new(&path), &name).map(|p| p.display().to_string())
}

#[tauri::command]
fn worlds_dir() -> String {
    terraria::worlds_dir().display().to_string()
}

#[tauri::command]
fn cancel(state: State<AppState>) {
    state.cancel.lock().unwrap().store(true, Ordering::Relaxed);
}

#[derive(Deserialize, Clone)]
struct FindRequest {
    size: u8,
    difficulty: u8,
    /// 1 random, 2 corruption, 3 crimson
    evil: u8,
    /// When set (0..1), the evil is drawn with this chance of Crimson instead of the game's 50/50.
    crimson_chance: Option<f64>,
    specials: Vec<String>,
    secrets: Vec<String>,
    goals: serde_json::Value,
    attempts: u32,
    workers: u32,
    want: u32,
    name: String,
    /// Older versions (before 1.4.5) take a plain seed and ignore specials/secrets here.
    full_format: bool,
}

/// Everything the search reports while it runs.
#[derive(Serialize, Clone)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum FindEvent {
    /// one worker's live generation progress
    Progress { worker: u32, attempt: u32, pct: f32, step: String },
    /// a world was generated and checked
    Tick { tried: u32, hits: u32, attempts: u32, last_ms: u64 },
    Hit { seed: String, path: String, score: f64, met: Vec<String>, teaser: Vec<String> },
    /// a world could not be generated or read
    Failed { attempt: u32, message: String },
}

/// The search loop, free of Tauri so it can be tested against the real server.
fn run_find(req: FindRequest, goals: seedcore::finder::Goals, server: String, cancel: Arc<AtomicBool>, emit: Arc<dyn Fn(FindEvent) + Send + Sync>) -> Result<u32, String> {
    let next = Arc::new(AtomicU32::new(0));
    let tried = Arc::new(AtomicU32::new(0));
    let hits = Arc::new(AtomicU32::new(0));
    let failed_in_a_row = Arc::new(AtomicU32::new(0));
    let last_error = Arc::new(Mutex::new(String::new()));
    let run = tmp_root().join(format!("find-{}", rand_u64() % 1_000_000_000));
    let workers = req.workers.clamp(1, 8);
    emit(FindEvent::Tick { tried: 0, hits: 0, attempts: req.attempts, last_ms: 0 });
    let mut handles = vec![];
    for worker in 0..workers {
        let (next, tried, hits, failed_in_a_row, last_error, cancel, emit, req, goals, run, server) = (
            next.clone(), tried.clone(), hits.clone(), failed_in_a_row.clone(), last_error.clone(),
            cancel.clone(), emit.clone(), req.clone(), goals.clone(), run.clone(), server.clone(),
        );
        handles.push(std::thread::spawn(move || loop {
            if cancel.load(Ordering::Relaxed) || hits.load(Ordering::Relaxed) >= req.want.max(1) {
                break;
            }
            // the same failure on every world (wrong version, broken install) stops the search
            if failed_in_a_row.load(Ordering::Relaxed) >= workers.max(2) {
                cancel.store(true, Ordering::Relaxed);
                break;
            }
            let i = next.fetch_add(1, Ordering::Relaxed);
            if i >= req.attempts {
                break;
            }
            let number = (rand_u64() % 2_147_483_646 + 1).to_string();
            let evil = match req.crimson_chance {
                Some(p) => {
                    if (rand_u64() % 1_000_000) as f64 / 1_000_000.0 < p { 3 } else { 2 }
                }
                None => req.evil,
            };
            let seed = if req.full_format {
                seedcore::seed::build(&seedcore::seed::SeedOptions {
                    size: req.size,
                    difficulty: req.difficulty,
                    evil,
                    specials: req.specials.clone(),
                    secrets: req.secrets.clone(),
                    seed: number,
                })
            } else {
                number
            };
            let dir = run.join(format!("try-{i}"));
            let spec = terraria::GenSpec { size: req.size, difficulty: req.difficulty, evil, name: req.name.clone(), seed: seed.clone() };
            let t0 = std::time::Instant::now();
            let mut last_pct = -1i32;
            let res = terraria::generate(std::path::Path::new(&server), &dir, &spec, &cancel, |pct, step| {
                // throttle to whole percents so the UI isn't flooded
                if pct as i32 != last_pct {
                    last_pct = pct as i32;
                    emit(FindEvent::Progress { worker, attempt: i + 1, pct, step: step.to_string() });
                }
            });
            let ms = t0.elapsed().as_millis() as u64;
            let outcome: Result<(), String> = (|| {
                let path = res?;
                let bytes = std::fs::read(&path).map_err(|e| format!("Could not read the new world: {e}"))?;
                let w = seedcore::World::parse(&bytes)?;
                let a = seedcore::analyze(&w);
                let v = seedcore::finder::judge(&a, &goals);
                if v.pass && hits.fetch_add(1, Ordering::Relaxed) < req.want.max(1) {
                    // found worlds go to the library so they survive a restart
                    let kept = keep_in_library(&path, &req.name)?;
                    emit(FindEvent::Hit { seed: seed.clone(), path: kept.display().to_string(), score: v.score, met: v.met, teaser: v.teaser });
                } else if v.pass {
                    hits.fetch_sub(1, Ordering::Relaxed);
                }
                Ok(())
            })();
            let _ = std::fs::remove_dir_all(&dir);
            match outcome {
                Ok(()) => failed_in_a_row.store(0, Ordering::Relaxed),
                Err(e) if e == "cancelled" => break,
                Err(e) => {
                    failed_in_a_row.fetch_add(1, Ordering::Relaxed);
                    *last_error.lock().unwrap() = e.clone();
                    emit(FindEvent::Failed { attempt: i + 1, message: e });
                }
            }
            let t = tried.fetch_add(1, Ordering::Relaxed) + 1;
            emit(FindEvent::Tick { tried: t, hits: hits.load(Ordering::Relaxed), attempts: req.attempts, last_ms: ms });
        }));
    }
    for h in handles {
        let _ = h.join();
    }
    let _ = std::fs::remove_dir_all(&run);
    let found = hits.load(Ordering::Relaxed).min(req.want.max(1));
    if found == 0 && failed_in_a_row.load(Ordering::Relaxed) >= workers.max(2) {
        return Err(format!("Terraria could not make worlds, so the search stopped. Last error: {}", last_error.lock().unwrap()));
    }
    Ok(found)
}

#[tauri::command]
async fn find(req: FindRequest, app: AppHandle, state: State<'_, AppState>) -> Result<u32, String> {
    let install = state.install.lock().unwrap().clone();
    if !install.found {
        return Err("Terraria was not found. Choose your Terraria folder first.".into());
    }
    let cancel = Arc::new(AtomicBool::new(false));
    *state.cancel.lock().unwrap() = cancel.clone();
    let goals: seedcore::finder::Goals = serde_json::from_value(req.goals.clone()).map_err(|e| format!("The goals could not be read: {e}"))?;
    let emit: Arc<dyn Fn(FindEvent) + Send + Sync> = Arc::new(move |ev| {
        let _ = app.emit("find", ev);
    });
    tauri::async_runtime::spawn_blocking(move || run_find(req, goals, install.server, cancel, emit))
        .await
        .map_err(|e| e.to_string())?
}

#[cfg(test)]
mod find_tests {
    use super::*;
    /// Runs a real search against the local Terraria install. `cargo test -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn finds_worlds_end_to_end() {
        let install = terraria::detect();
        assert!(install.found);
        let req = FindRequest {
            size: 1, difficulty: 1, evil: 1, crimson_chance: Some(0.7), specials: vec![], secrets: vec![],
            goals: serde_json::json!({ "sword_shrine": true, "min_rare_items": 1 }),
            attempts: 4, workers: 2, want: 1, name: "Find Test".into(), full_format: true,
        };
        let goals: seedcore::finder::Goals = serde_json::from_value(req.goals.clone()).unwrap();
        let progress = Arc::new(AtomicU32::new(0));
        let p2 = progress.clone();
        let emit: Arc<dyn Fn(FindEvent) + Send + Sync> = Arc::new(move |ev| match ev {
            FindEvent::Progress { .. } => { p2.fetch_add(1, Ordering::Relaxed); }
            other => println!("{}", serde_json::to_string(&other).unwrap()),
        });
        let t = std::time::Instant::now();
        let r = run_find(req, goals, install.server, Arc::new(AtomicBool::new(false)), emit);
        println!("result {r:?} after {:?}, {} progress events", t.elapsed(), progress.load(Ordering::Relaxed));
        if let Ok(n) = r {
            if n > 0 {
                for e in std::fs::read_dir(library_dir()).unwrap().flatten() {
                    if e.file_name().to_string_lossy().starts_with("Find_Test") {
                        let _ = std::fs::remove_file(e.path());
                    }
                }
            }
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _ = std::fs::remove_dir_all(tmp_root());
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(AppState::default())
        .setup(|app| {
            let _ = app.get_webview_window("main");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![detect, set_install, generate, read_world, install_world, worlds_dir, cancel, find, list_worlds, remove_world, library_path])
        .run(tauri::generate_context!())
        .expect("error while running Worldseed");
}

