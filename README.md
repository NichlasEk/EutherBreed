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

## License

EutherBreed is currently private/prototype software. See [LICENSE](LICENSE) for the all-rights-reserved project terms.

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
cargo run -p euther_game -- --entry-smoke
cargo run -p euther_game -- --notice-smoke
```

Run save/load serialization smoke test:

```sh
cargo run -p euther_game -- --save-smoke
cargo run -p euther_game -- --save-file-smoke /tmp/euther_save_smoke.ron
cargo run -p euther_game -- --load-file-smoke /tmp/euther_save_smoke.ron
cargo run -p euther_game -- --runtime-save-smoke /tmp/euther_runtime_save_smoke.ron
cargo run -p euther_game -- --autosave-smoke /tmp/euther_autosave_smoke.ron
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
scripts/run-gl.sh
scripts/run.sh --gl
```

Runs the GUI prototype with the OpenGL backend, useful if Vulkan validation noise or driver issues get in the way.

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
- left mouse button or `Z`/`X`/`C`: fire reagent round
- hold `Shift`: show map overlay after collecting an area scan
- `R`: restart current section after health reaches 0
- `F5`: quick save to `saves/slot1.ron`
- `F9`: quick load from `saves/slot1.ron`
- `F11`: toggle borderless fullscreen
- `Esc`: quit

Current prototype:

- apothecary movement and mouse aim
- prototype quarantine ward loaded from level data
- four connected prototype sections: quarantine ward, lab access corridor, research spine, and triage vault
- reagent projectiles
- contaminant enemies that chase the player
- muzzle/impact/death flashes for combat feedback
- split cybernetic top/bottom HUD rails for status, objective, prompts, and notices
- health, ammo, and bio-sample counters
- area scan pickups unlock the hold-to-view map overlay on `Shift` per section
- death state with `R` restart for the current section
- quarantine ward has a first designed loop with keycard pressure, terminal pressure, and exit routing
- basic room walls, pickups, security keycard, locked door, terminal objective, objective-gated exits, level reloads, and collision
- save files preserve run position and level-local progress per campaign level, including collected pickups, unlocked doors, activated terminals, and killed contaminants
- autosave writes the current slot after successful level transitions
- exits can route the apothecary to named entry points on the destination level
- HUD notice feedback reports quicksave, quickload, missing saves, and autosave status

## Non-Goals

EutherBreed is not intended to ship original Alien Breed assets, names, story, audio, maps, or extracted commercial content. Original game files and OpenBreed are useful references for research, but this project should remain an original game.

## Door and quarantine pass (2026-09-19)

- Bulkheads have separate sliding, cropped leaves, fixed jambs/rails, status lights and a synthesized motor/pressure-release cue.
- Energy barriers have animated blue electric strands, layered blue light and a discharge cue. Their field fades during release; collision remains until the animation finishes.
- The quarantine ward now contains six rooms: reception, isolation, sample analysis, culture storage, medical stores and decontamination transit. Find the isolation keycard, collect the lab sample and use the analyzer with `E` to release the exit field and return shortcut.
- Floors differ by room type; walls have shadows/bevels, and rooms have labels and light strips.
- The bottom HUD uses two clipped, single-line rows and displays the current room name. The camera and HUD persist across level changes; the HUD is hidden on the main menu.

Start a fresh play session to try the revised ward. Older quicksaves retain coordinates from the previous layout.

Reproducible graphical smoke check (opens a window, saves four screenshots, checks camera/HUD counts through a level transition, then exits):

```sh
cargo run -p euther_game -- --visual-smoke /tmp/eutherbreed-visual
```

The check uses `/tmp/eutherbreed-visual/smoke-save.ron`, leaving the normal save slot alone. Original door sounds can be regenerated with `python3 scripts/generate-door-audio.py` (requires ffmpeg).

## Graphics and music pass (2026-09-19)

- Decor and terminals now use complete regions of the original atlas instead of cropped standalone exports, with preserved proportions and smaller room-scale footprints.
- Wall tile ends terminate at their collision boundaries. All four maps have a decor placement pass; rotated bounds are checked against walls, doors and map edges. Room signs avoid props where space permits.
- Smaller enemy sprites have navigation clearance matching their rotated visual footprint.
- Two original ACE-Step horror/electronic loops adapt to visible nearby threats. **F6** toggles music; **F7/F8** adjust its volume independently of effects. Details and provenance: `assets/music/README.md`.
- To inspect a particular map without overwriting the normal save slot: `cargo run -p euther_game -- --visual-smoke /tmp/eutherbreed-art-review --visual-level research_spine`. This captures the full map before checking a transition.

## Terminal feedback and audio settings (2026-09-19)

- **SETTINGS** in the main menu or Esc pause menu offers separate music and SFX sliders. Drag with the mouse, or use Tab and left/right arrows. Preferences save automatically in `saves/audio-settings.ron`.
- Successful analysis changes the terminal display and sample indicator. Completing the sector's required objectives changes its lighting and brings up a faint ventilation hum; objective-controlled energy barriers release remotely. These changes return when loading a save.
- Pressing **E** at an already-used terminal gives a hollow double knock and a small display reaction. Missing requirements give a short dull buzz. Neither adds an explanatory text box.
- Original synthesized cues can be regenerated with `python3 scripts/generate-terminal-audio.py` (NumPy and ffmpeg).
- Automated graphical interaction review: `cargo run -p euther_game -- --visual-smoke /tmp/eutherbreed-terminal-review-new --terminal-review`. Use a fresh output directory for default-volume assertions. It captures terminal responses, verifies save/load state and adjusts both audio channels using isolated saves/settings.
