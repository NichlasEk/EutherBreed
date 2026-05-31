# EutherBreed Handoff

Date: 2026-05-31

## Current State

Repository:

- Path: `/home/nichlas/EutherBreed`
- Remote: `https://github.com/NichlasEk/EutherBreed`
- Branch: `main`
- Latest gameplay commit before this handoff: `e718af4 Add map overlay and section restart`
- Local branch is ahead of `origin/main`.

The project is a Rust/Bevy prototype for an original top-down sci-fi survival shooter. The game is inspired by the structure and tension of Alien Breed: Tower Assault, but it must not reuse the original name, story, graphics, audio, maps, or extracted assets.

The player character is the ship's apothecary: a medical and biochemical specialist forced into survival combat during an alien outbreak. Keep the setting serious sci-fi. The apothecary theme should influence systems and language, but core progression objects should still feel grounded: keycards, security terminals, quarantine systems, lab analyzers, ship logistics, and bio-samples.

## Implemented Prototype Features

- Rust workspace with:
  - `crates/game_core`: pure gameplay/data rules and tests
  - `crates/euther_game`: Bevy prototype
  - `crates/abta_tools`: local-only research tooling
- Three connected prototype sections:
  - `prototype_quarantine_ward`
  - `lab_access_corridor`
  - `triage_vault`
- Movement, mouse aiming, projectiles, contaminants, pickups, walls, terminals, locked doors, exits, and level transitions.
- Campaign graph with named entry points for exits.
- Persistent run state:
  - current level
  - player position
  - vitals
  - per-level pickups, doors, terminals, objectives, and killed handplaced contaminants
- Quicksave/quickload:
  - `F5`: save to `saves/slot1.ron`
  - `F9`: load from `saves/slot1.ron`
- Autosave after successful level transitions.
- HUD notices for saves, loads, autosaves, interactions, damage, and contaminant kills.
- Dynamic level spawn points:
  - configured in RON level files
  - capped temporary contaminants
  - dynamic contaminants do not pollute persistent killed-enemy state
- Basic level theme tinting and section/exit HUD text.
- Hold-to-view map overlay:
  - `Shift`: shows a simple generated map for the current level
  - built from EutherBreed level data, not copied from original game resources
- Death/restart:
  - health 0 shows a restart prompt
  - `R`: restarts the current section with fresh local state
- First visible character/enemy/tile sprite pass is in place.
- Current apothecary walk sprite frames are intentionally left for manual replacement; code supports frame cycling.

## Explicitly Deferred

Do **not** spend the next pass polishing the map UI. The map is intentionally functional only for now.

Deferred map work:

- fog of war / known-vs-unknown rooms
- map labels and icon polish
- nicer frame, legend, blinking player marker
- extracted/reference-informed map renderer work

Next effort should drive the playable slice: level flow, pressure, combat feel, enemy pacing, objectives, and balancing.

## Important Commands

Run the full local verification suite:

```sh
./scripts/check.sh
```

Run the GUI prototype:

```sh
scripts/run.sh
```

Run directly:

```sh
cargo run -p euther_game
```

Run headless when graphics are broken:

```sh
cargo run -p euther_game -- --headless-smoke
```

Validate content:

```sh
cargo run -p euther_game -- --validate-content
cargo run -p euther_game -- --entry-smoke
cargo run -p euther_game -- --notice-smoke
```

## Last Verification

The most recent focused verification passed after the map/restart pass:

- `cargo fmt --check`
- `cargo check -p euther_game`
- `cargo run -p euther_game -- --validate-content`
- `cargo run -p euther_game -- --headless-smoke`
- `cargo run -p euther_game -- --save-smoke`
- `cargo run -p euther_game -- --entry-smoke`
- `cargo test -p euther_game`

Older full-suite verification via `./scripts/check.sh` passed after commit `e9e8715`.

That suite included:

- `cargo fmt --check`
- `cargo check`
- 49 `game_core` tests
- ABTA tooling help smoke
- campaign/level content validation
- entry smoke
- notice smoke
- save/load/runtime/autosave smoke tests
- headless startup smoke

## Graphics Reboot Context

The next practical step is to reboot and fix/check local graphics. Earlier work favored headless mode because the GUI/GPU path was unreliable.

After reboot:

1. Check system graphics health:

```sh
nvidia-smi
```

2. Run the guarded GUI script:

```sh
scripts/run.sh
```

3. If Bevy still fails through the default backend, try the already-used GL fallback pattern:

```sh
env WGPU_BACKEND=gl cargo run -p euther_game
```

4. If the GUI launches, inspect:

- level tinting and contrast
- contaminant hit flash
- HUD text positioning
- section/exit HUD row
- spawn pacing in each level
- whether sprite scale/readability feels acceptable at current resolution

## Recommended Next Development Pass

Keep driving the first playable slice:

- Improve `prototype_quarantine_ward` layout into a real loop:
  - start pressure
  - risky keycard side room
  - locked route / shortcut
  - terminal objective deeper in the section
  - exit pressure after objective completion
- Tune contamination pressure:
  - slower before keycard
  - faster after keycard
  - strongest after terminal objective
- Add combat feel:
  - muzzle flash
  - impact flash
  - enemy death pop / sample pickup feedback
- Balance:
  - ammo and med-gel amounts
  - spawn interval
  - starting enemy positions
  - exit distance and objective friction

## Useful Design Notes

- Keep keycards/security access as grounded sci-fi objects.
- Apothecary framing should be substantial, not jokey:
  - reagent rounds
  - bio-samples
  - quarantine terminals
  - lab analysis objectives
  - med/logistics supplies
- The current goal is still a playable vertical slice, not a full remake.
- Tauri remains planned, but the Bevy prototype should become credible first.
