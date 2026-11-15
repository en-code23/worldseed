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

#[derive(Deserialize)]
struct GenRequest {
    size: u8,
    difficulty: u8,
    evil: u8,
    name: String,
    seed: String,
    job: String,
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
    let res = tauri::async_runtime::spawn_blocking(move || {
        terraria::generate(std::path::Path::new(&install.server), &dir, &spec, &cancel, |pct, step| {
            let _ = app.emit("gen-progress", Progress { job: job.clone(), pct, step: step.to_string() });
        })
    })
    .await
    .map_err(|e| e.to_string())??;
    Ok(res.display().to_string())
}

#[tauri::command]
fn read_world(path: String) -> Result<tauri::ipc::Response, String> {
    let p = PathBuf::from(&path);
    if !p.starts_with(tmp_root()) && !p.starts_with(terraria::worlds_dir()) {
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

#[derive(Serialize, Clone)]
struct FindTick {
    tried: u32,
    hits: u32,
    attempts: u32,
    last_ms: u64,
}

#[derive(Serialize, Clone)]
struct FindHit {
    seed: String,
    path: String,
    score: f64,
    met: Vec<String>,
    teaser: Vec<String>,
}

#[tauri::command]
async fn find(req: FindRequest, app: AppHandle, state: State<'_, AppState>) -> Result<u32, String> {
    let install = state.install.lock().unwrap().clone();
    if !install.found {
        return Err("Terraria was not found. Choose your Terraria folder first.".into());
    }
    let cancel = Arc::new(AtomicBool::new(false));
    *state.cancel.lock().unwrap() = cancel.clone();
    let goals: seedcore::finder::Goals = serde_json::from_value(req.goals.clone()).map_err(|e| e.to_string())?;
    let next = Arc::new(AtomicU32::new(0));
    let tried = Arc::new(AtomicU32::new(0));
    let hits = Arc::new(AtomicU32::new(0));
    let run = tmp_root().join(format!("find-{}", rand_u64() % 1_000_000_000));
    let workers = req.workers.clamp(1, 8);
    let mut handles = vec![];
    for _ in 0..workers {
        let (next, tried, hits, cancel, app, req, goals, run, server) =
            (next.clone(), tried.clone(), hits.clone(), cancel.clone(), app.clone(), req.clone(), goals.clone(), run.clone(), install.server.clone());
        handles.push(std::thread::spawn(move || loop {
            if cancel.load(Ordering::Relaxed) || hits.load(Ordering::Relaxed) >= req.want.max(1) {
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
            let res = terraria::generate(std::path::Path::new(&server), &dir, &spec, &cancel, |_, _| {});
            let ms = t0.elapsed().as_millis() as u64;
            if let Ok(path) = res {
                if let Ok(bytes) = std::fs::read(&path) {
                    if let Ok(w) = seedcore::World::parse(&bytes) {
                        let a = seedcore::analyze(&w);
                        let v = seedcore::finder::judge(&a, &goals);
                        if v.pass && hits.load(Ordering::Relaxed) < req.want.max(1) {
                            hits.fetch_add(1, Ordering::Relaxed);
                            let _ = app.emit("find-hit", FindHit { seed: seed.clone(), path: path.display().to_string(), score: v.score, met: v.met, teaser: v.teaser });
                        } else {
                            let _ = std::fs::remove_dir_all(&dir);
                        }
                    }
                }
            } else {
                let _ = std::fs::remove_dir_all(&dir);
            }
            let t = tried.fetch_add(1, Ordering::Relaxed) + 1;
            let _ = app.emit("find-tick", FindTick { tried: t, hits: hits.load(Ordering::Relaxed), attempts: req.attempts, last_ms: ms });
        }));
    }
    let hits2 = hits.clone();
    tauri::async_runtime::spawn_blocking(move || {
        for h in handles {
            let _ = h.join();
        }
    })
    .await
    .map_err(|e| e.to_string())?;
    Ok(hits2.load(Ordering::Relaxed))
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
        .invoke_handler(tauri::generate_handler![detect, set_install, generate, read_world, install_world, worlds_dir, cancel, find])
        .run(tauri::generate_context!())
        .expect("error while running Worldseed");
}
