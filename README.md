# EutherBreed

EutherBreed is a Rust game project for a modern top-down space horror shooter inspired by the structure and tension of classic 1990s sci-fi run-and-gun games.

The player character is the ship's apothecary: a medical and biochemical specialist forced into survival combat during an alien outbreak. The game should use this premise throughout its systems, naming, UI, pickups, upgrades, and objectives.

## Direction

- Original sci-fi setting, story, characters, enemies, assets, audio, and UI.
- Top-down exploration and combat in ships, orbital labs, quarantine wards, and infested facilities.
- Non-linear levels with terminals, locked routes, multiple exits, survival pressure, and limited resources.
- Rust-first implementation with Tauri planned for the desktop shell.

## Repository Layout

```text
crates/
  game_core/     Pure gameplay rules and tests
  euther_game/   First playable Bevy prototype
  abta_tools/    Local-only research tools for original format inspection
assets/            Original EutherBreed assets
assets/campaigns/  RON campaign graph definitions
assets/levels/     RON level definitions
research/          Notes and non-shipping research output
PLAN.md            Project plan and milestone outline
```

## Run The Prototype

```sh
cargo run -p euther_game
```

Headless smoke test, useful when GPU/Vulkan is unavailable:

```sh
cargo run -p euther_game -- --headless-smoke
```

Validate campaign and level content:

```sh
cargo run -p euther_game -- --validate-content
```

Run save/load serialization smoke test:

```sh
cargo run -p euther_game -- --save-smoke
cargo run -p euther_game -- --save-file-smoke /tmp/euther_save_smoke.ron
```

## Scripts

```sh
scripts/headless-smoke.sh
```

Runs the non-GPU startup smoke test.

```sh
scripts/check.sh
```

Runs formatting check, compile check, `game_core` tests, and headless smoke.

```sh
scripts/run.sh
```

Runs the GUI prototype and warns first if `nvidia-smi` reports a local driver problem.

```sh
scripts/build.sh
```

Builds the full workspace in dev mode.

```sh
scripts/release.sh
```

Runs format check, `game_core` tests, and a release build.

ABTA research helper:

```sh
cargo run -p abta_tools -- inspect /home/nichlas/AlienBrT/TA.EPF
cargo run -p abta_tools -- list /home/nichlas/AlienBrT/TA.EPF --ext BLK
```

Controls:

- `WASD` or arrow keys: move
- mouse: aim
- left mouse button: fire reagent round
- `Esc`: quit

Current prototype:

- apothecary movement and mouse aim
- prototype quarantine ward loaded from level data
- reagent projectiles
- contaminant enemies that chase the player
- health, ammo, and bio-sample counters
- basic room walls, pickups, security keycard, locked door, terminal objective, objective-gated exits, level reloads, and collision

## Non-Goals

EutherBreed is not intended to ship original Alien Breed assets, names, story, audio, maps, or extracted commercial content. Original game files and OpenBreed are useful references for research, but this project should remain an original game.
