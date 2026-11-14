# Worldseed

A Terraria seed builder, lucky-world finder and world map viewer, for Mac, Windows, Linux, Android and the web.

**Website and web app:** https://en-code23.github.io/worldseed/

## What it does

- **Map.** Opens any Terraria `.wld` file (1.3.5 to 1.4.5) and marks spawn, the dungeon, the jungle temple, desert pyramids, sky islands and lakes, the Shimmer, the Underworld, oceans, Enchanted Sword shrines, shadow orbs, crimson hearts, altars, life crystals, Queen Bee larvae, living trees and every chest. Click a chest to see its loot. Rare items such as Fledgling Wings are highlighted, and you can search for any item.
- **Create.** Size, difficulty, world evil, all 9 special seeds and all 37 secret seeds of 1.4.5. It builds the full seed string (`size.difficulty.evil.specials.secrets|seed`) that Terraria's own seed field accepts. *Lean the dice* draws the world evil with your chosen chance (for example 70% Crimson).
- **Find** (desktop). Generates worlds with **your installed Terraria's own generator** (the dedicated server that ships with the game) and keeps the ones that match your goals. Results are spoiler-free: you see *what* the world holds, never *where*. Presets: Lucky explorer, Speedrun, Challenge, Multiplayer (enough of each item for every player) and Custom.
- **Spoiler shield and hints.** Hide the map and see only a summary, or open hints one step at a time, from vague to exact.
- **Straight into Terraria.** The desktop app copies finished worlds into your Terraria Worlds folder without overwriting anything.

## Why the maps are accurate

Worldseed never re-implements world generation. The desktop app runs `TerrariaServer` from your own install to make the world, then reads the saved file with a Rust parser. A world you get from Worldseed is byte-for-byte what Terraria would make for that seed and version. To make worlds for an older version, switch Terraria to that version with Steam betas (Properties, Betas) and choose the folder again.

The web and mobile apps can't run the game's generator, so they open existing world files and build seeds instead.

## Layout

| Folder | What |
| --- | --- |
| `core/` | Rust: `.wld` reader, landmark detection, map renderer, finder scoring, hints, seed catalog. Compiles to WebAssembly for the web. |
| `web/` | Landing page (`index.html`) and the app (`app/`), Vite + TypeScript. Shared by every platform. |
| `desktop/` | Tauri 2 app: Terraria detection, server driver, parallel finder, installs worlds. |
| `mobile/` | Capacitor projects for Android and iOS wrapping the web app. |
| `tools/` | Data generator for tile, wall and item tables, and the icon generator. |

## Build

```sh
# web
cd web && npm ci && npm run wasm && npm run dev     # http://localhost:5173/app/

# desktop (needs Rust and Node 22)
cd desktop && npm ci && npx tauri dev
npx tauri build

# android / ios
cd mobile && npm ci && npm run sync
npx cap open android   # or: npx cap open ios
```

Tagging `v*` builds the Mac (universal), Windows, Linux and Android downloads in GitHub Actions and attaches them to a release. Pushing to `main` deploys the website to GitHub Pages.

## Notes

- The desktop apps aren't code-signed. On Mac, right-click the app and choose Open the first time. On Windows, choose More info, then Run anyway.
- iOS: there's no App Store build. Use the web app (Safari, Share, Add to Home Screen), or open `mobile/ios` in Xcode and run it on your own device.
- Tile, wall and item tables are derived from [TEdit](https://github.com/TEdit/Terraria-Map-Editor) data (MS-PL). Secret seed codes come from the Terraria Wiki.
- Worldseed is a fan project, not affiliated with Re-Logic. Terraria is a trademark of Re-Logic.
