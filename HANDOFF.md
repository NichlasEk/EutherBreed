# EutherBreed Handoff

Date: 2026-06-03

## Update 2026-09-19: doors, ward and HUD

This update supersedes the older visual/layout state below.

- `door_visuals.rs` builds fixed tracks/jambs, independently cropped sliding leaves and animated blue electric fields. Gameplay door sprites retain a fixed invisible footprint; collision is removed only when the 0.85-second opening completes.
- Door visual children belong to the door entity and are recursively cleaned up. Saved open doors reconstruct without a blocking field.
- Original synthesized Vorbis cues live in `assets/audio/`; their deterministic source is `scripts/generate-door-audio.py`.
- Rebuilt quarantine ward as six rooms with a physical keycard -> lab sample -> analyzer -> transit route, an analysis-gated return shortcut and an optional supply room. Existing content IDs were retained where possible. Use a fresh game to review the new layout; old saves can contain old coordinates.
- Room-specific floor selection, labels, light strips and wall shadows/bevels are implemented in `setup.rs`.
- User observed HUD blinking during the first graphical check: two cameras were rendering simultaneously. There is now one persistent app camera, and HUD roots are reused across new runs and hidden in the main menu. The bottom HUD has two bounded rows and a current-room label.
- `--visual-smoke <directory>` captures closed/opening/open doors, then performs a real campaign transition and captures the destination. It asserts one camera and two HUD roots, uses its own save path and exits automatically.
- Tests cover locked/opening/open collision, both orientations/kinds, restored visuals, child cleanup, animated arc endpoints and physical player-radius reachability through the revised ward.

Validation: 78 unit tests passed (72 core + 6 game); content, headless, entry, save and notice smoke checks passed. Graphical smoke passed through a real level transition with one camera and two HUD roots. Render captures are under `/tmp/eutherbreed-visual/`; Vulkan emits the previously observed SPIR-V validation warnings on this host. Full manual combat balancing remains to be done.

## Current State

Repository:

- Path: `/home/nichlas/EutherBreed`
- Branch: `main`
- Worktree at handoff: clean
- Latest commit: `f5094b6 Add game over and terminal prompts`

EutherBreed is currently a Rust/Bevy top-down sci-fi survival shooter prototype. It is an original project with classic Alien Breed / Tower Assault style structure as design inspiration only. Do not reuse original names, story, maps, extracted graphics, audio, or exact UI art.

The playable identity is now clear enough to keep building around:

- Hero: ship apothecary / biochemical specialist.
- Core loop: explore hostile sections, gather reagents/bio-samples/keycards, use terminals, unlock routes, survive contamination pressure, choose exits.
- Tone: dark biomechanical sci-fi, serious survival horror, not parody.

## Latest Commits

Recent useful commits:

- `f5094b6 Add game over and terminal prompts`
- `e950823 Fix same-level route reloads`
- `e4430aa Add room graph to quarantine ward`
- `6255d12 Add prototype lives HUD state`
- `a6432e6 Add room graphs to prototype levels`
- `a06a8ca Expand editor and pause summaries`
- `c7ad220 Add pause inventory map shell`
- `1da7f16 Add prototype main menu`
- `edf4778 Add editor node linking tools`
- `4fc2a4d Add prototype level editor mode`

## Implemented

Core game:

- Bevy app with main menu, gameplay, pause menu, and game over screen.
- Movement, mouse aim, shooting, contaminants, pickups, doors, terminals, exits, transitions.
- HUD with split top/bottom cyber-industrial rails.
- Dynamic `LIVES` state in HUD.
- Health 0 now either:
  - allows section restart with `R` if lives remain
  - routes to Game Over when lives are exhausted
- Game Over screen has Continue, Main Menu, Quit.
- Continue resets run state cleanly.
- Fullscreen toggle: `F11`.

Campaign/content:

- Four campaign levels:
  - `prototype_quarantine_ward`
  - `lab_access_corridor`
  - `triage_vault`
  - `research_spine`
- All four levels now have semantic `sections`.
- Lab and triage have room graphs, doors, objectives, terminals, and decor passes.
- Quarantine ward has room graph, objective router action, and decor pass.
- Research spine remains the most advanced large level.
- Same-level route reload bug fixed. A route to the same level with another entry now reloads correctly instead of silently returning.

Editor:

- Prototype editor mode exists.
- Main menu has an Editor button.
- Editor supports level picker with `[` and `]`.
- Editor inspector shows selected entity details.
- Editor can place/move/delete/rotate entities.
- Editor can link terminal-to-door and auto-connect door sections.
- Editor has quick terminal rule shortcuts:
  - `1` add ammo action
  - `2` heal action
  - `3` grant clearance action
  - `4` objective action / door objective requirement
- Editor palette includes decor, pickups, contaminants, doors, terminals.

Pause/inventory/map:

- `Esc`: pause/status
- `I`: inventory
- `M`: strategic map summary
- `Shift`: quick tactical overlay if area scan is acquired
- Pause inventory now lists ammo, bio-samples, area scan, access tokens/keycards, pickup ids.
- Pause map summarizes doors, exits, objectives, sections.

Terminal prompts:

- Nearby terminals now show what they do before pressing `E`, such as:
  - ammo/heal
  - objective
  - unlock door
  - key/access token
  - area scan
  - bio-sample requirement

## Controls

Gameplay:

- `WASD` / arrows: move
- Mouse: aim
- Mouse button / `Z` / `X` / `C`: shoot, depending current code path
- `E`: interact with terminals/transitions
- `R`: restart section after death if lives remain
- `Shift`: tactical map overlay when area scan exists
- `Esc`: pause
- `I`: inventory tab
- `M`: map tab
- `F5`: quicksave
- `F9`: quickload
- `F11`: toggle fullscreen

Editor:

- `WASD` / arrows: pan
- `Shift`: faster pan
- `+` / `-`: zoom
- `[` / `]`: previous/next campaign level
- Mouse left: select/place
- `Space`: place palette item
- `M`: move selected to cursor
- `R`: rotate selected decor
- `B`: toggle door kind
- `L`: toggle door lock
- `G`: link selected terminal to nearest door, or auto-connect selected door sections
- `1`: terminal AddAmmo action
- `2`: terminal Heal action
- `3`: terminal GrantClearance action
- `4`: terminal CompleteObjective or door objective requirement
- `Delete` / `Backspace`: remove selected
- `Tab` / `Q` / `E`: palette navigation
- `Ctrl+S`: save level

## Verification At Handoff

Latest checked commands passed:

```sh
cargo check -p euther_game
cargo test -p euther_game
cargo run -p euther_game -- --menu-smoke
cargo run -p euther_game -- --headless-smoke
cargo run -p euther_game -- --validate-content
git diff --check
```

Previous same-session checks also passed:

```sh
cargo test -p game_core
cargo run -p euther_game -- --editor-smoke prototype_quarantine_ward
cargo run -p euther_game -- --editor-smoke lab_access_corridor
cargo run -p euther_game -- --editor-smoke triage_vault
cargo run -p euther_game -- --editor-smoke research_spine
```

## Known Issues / Watch List

- User reported a freeze near an exit/route area. A likely same-level route reload bug was fixed in `e950823`, but the exact GUI situation should be retested.
- If freezing still happens, next suspects:
  - exit overlap lock edge case
  - spawn/entry position inside trigger zone
  - runtime Bevy panic not visible in screenshot
  - door/wall collision edge after transition
- Vulkan validation warnings about SPIR-V atomics have appeared on this machine. They seem driver/wgpu/validation-layer related and not gameplay logic, but keep an eye on runtime output.
- Map polish is intentionally deferred. Do not spend the next pass making map UI fancy unless explicitly requested.
- Some player walk sprite sheet experiments are intentionally left in place but user may replace art manually.
- Some collision/object placement still needs manual gameplay testing in GUI, especially newly sectioned older levels.

## Important Commands

Run game:

```sh
scripts/run.sh
```

Direct run:

```sh
cargo run -p euther_game
```

Editor:

```sh
cargo run -p euther_game -- --editor research_spine
```

Content validation:

```sh
cargo run -p euther_game -- --validate-content
```

Core checks:

```sh
cargo check -p euther_game
cargo test -p game_core
cargo test -p euther_game
```

Smoke checks:

```sh
cargo run -p euther_game -- --headless-smoke
cargo run -p euther_game -- --menu-smoke
cargo run -p euther_game -- --entry-smoke
cargo run -p euther_game -- --notice-smoke
```

## Next Plan

Recommended next development order:

1. Retest the freeze spot from the screenshot in GUI.
   - Confirm same-level route reload fix.
   - If it still freezes, capture terminal output and patch the remaining route/trigger issue.

2. Finish game rules foundation.
   - Save/load should include lives or consciously reset lives per run.
   - Define game over continue semantics more strictly.
   - Add mission complete / campaign complete state.
   - Add better objective completion feedback.

3. Improve terminal UI beyond prompt text.
   - Small terminal panel or modal with title, requirement, effects.
   - Separate supply station, analyzer, ship log, area scan terminal presentations.

4. Improve inventory into actual item model.
   - Store named keycards/access tokens.
   - Display area scan, bio-samples, reagents, keycards, mission items.
   - Later: upgrades/credits/shop if desired.

5. Editor evolution.
   - Visual section editing.
   - Draw/move section bounds.
   - Door-to-section linking by click.
   - Better object palette categories.
   - Node/link mode for terminals, doors, objectives, exits.

6. Gameplay pressure.
   - Enemy archetypes.
   - Spawn pacing per objective state.
   - Better enemy movement and attack telegraph.
   - Gore/death feedback.
   - Balance ammo, med-gel, bio-samples, spawn intervals.

7. Content pass.
   - Play each level start to exit.
   - Fix unreachable/annoying rooms.
   - Add more variation in rooms and props.
   - Keep map polish deferred until core flow is stronger.

## Creative Direction Reminder

Keep pushing toward:

- dark, high-resolution tiles and props
- biomechanical / Giger-adjacent original art direction
- readable top-down silhouettes
- sexy but functional apothecary hero design, viewed clearly from top-down
- bloody, unsettling medical/ship environments
- original level layouts inspired by classic design principles, not copied maps
