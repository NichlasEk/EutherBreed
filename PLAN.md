# EutherBreed Plan

## Vision

EutherBreed is a modern top-down sci-fi survival shooter inspired by the structure and tension of Alien Breed: Tower Assault, without reusing the original name, story, graphics, audio, or copyrighted content.

The base theme is space horror with alien infestation, but the player character is not a standard marine. The hero is the ship's apothecary: a medical and biochemical specialist forced into combat during a catastrophic outbreak. This should influence naming, systems, UI language, pickups, weapons, and mission goals from the start.

The goal is to keep the parts that still feel strong today:

- dense top-down combat in hostile ships and facilities
- non-linear levels with multiple exits
- terminals, access cards, ammo, credits, and upgrades
- dark corridors, alarms, lockdowns, and limited information
- one-player first, with local co-op as a later target

The game should feel modern in input, readability, resolution, lighting, audio, and pacing while still keeping the pressure and resource management of the classic formula.

## Legal And Creative Direction

EutherBreed should be a spiritual successor, not a clone.

- Use an original title, story, factions, enemies, UI, sprites, sound effects, and music.
- Use the original DOS game only as design and format research.
- Do not ship original assets extracted from `TA.EPF`.
- Use OpenBreed as a reference for understanding file formats and design structure, not as the foundation of the codebase.
- Any imported reference data should stay in local research tooling and not become part of distributable game content.

## Theme Foundation

Working premise:

- The setting is deep space: derelict ships, orbital labs, cargo decks, quarantine wards, hydroponics, reactor rooms, and alien-contaminated medical bays.
- The hero is the ship's apothecary, responsible for medicine, triage, bio-analysis, antidotes, and quarantine protocols.
- The alien threat is biological as much as military: spores, parasites, growths, nests, infected crew, and environmental contamination.
- Combat should exist, but the character fantasy is survival through field medicine, improvised chemistry, diagnosis, containment, and precision.
- Terminals and upgrades can be framed as medical fabricators, pharmacy lockers, lab analyzers, quarantine systems, and shipboard logistics.
- Pickups can include med-gel, stimulants, cultures, reagents, disinfectant charges, oxygen packs, access ampoules, and bio-samples.
- Objectives can include synthesizing cures, sealing vents, rescuing crew, retrieving samples, purging infestations, stabilizing life support, and reaching evacuation routes.

This gives implementation names a useful direction: prefer domain names like `Apothecary`, `MedKit`, `Reagent`, `BioSample`, `QuarantineDoor`, `LabTerminal`, and `Contamination` over generic shooter-only names where it makes sense.

## Reference Material

Local original game path:

```text
/home/nichlas/AlienBrT/
```

Important files observed:

```text
TA.EPF
TA.EXE
PLAY.EXE
INTRO.ANM
OUTRO.ANM
TOWER/HISCORES.DAT
```

OpenBreed:

```text
https://github.com/mrpetro/OpenBreed
```

Useful OpenBreed research areas:

- EPF archive structure
- ABTA map reader
- BLK tileset reader
- SPR sprite reader
- LBM/IFF image handling
- map action codes and level password logic

OpenBreed is C# and not currently playable, so it is best treated as documentation-by-code.

## Technical Direction

Primary language:

- Rust

Desktop shell:

- Tauri

Recommended project shape:

```text
crates/
  game_core/       Pure gameplay simulation and data types
  game_render/     Rendering/input layer or WASM-facing game adapter
  abta_tools/      Local research tools for EPF/map/sprite inspection
src-tauri/         Tauri desktop shell
assets/            Original EutherBreed assets only
research/          Notes and non-shipping analysis output
```

The game logic should be kept separate from rendering and Tauri integration. This makes it easier to test gameplay systems, support tools, and eventually run the game in different frontends.

## Engine Options

Preferred starting path:

- Rust gameplay prototype first.
- Use a proven 2D Rust engine or renderer.
- Add Tauri once the core loop is credible.

Candidates:

- Bevy: strong ECS, good Rust ecosystem, heavier but scalable.
- Macroquad: quick iteration, simple 2D prototype path.
- Custom canvas/WebGL via WASM: closer to Tauri webview, more manual work.

Current recommendation:

Start with Bevy if the goal is a long-lived game with editor-like tooling and many entities. Start with Macroquad if the goal is the fastest playable prototype. Revisit before scaffolding.

## Core Gameplay Pillars

### Exploration

- Top-down ship and facility maps.
- Locked doors, terminals, access cards, elevators, vents, and hidden routes.
- Multiple exits per level, affecting the campaign route.
- Optional rooms with higher risk and better rewards.

### Combat

- Aim and shoot in modern twin-stick/mouse style.
- Allow controlled retreat/backpedal shooting as a nod to Tower Assault.
- Limited ammo pressure.
- Weapon upgrades and situational weapon roles.
- Enemy waves from vents, nests, alarms, and scripted ambushes.
- Medical and chemical tools should double as survival mechanics and weapons where appropriate.

### Survival Pressure

- Low visibility and directional light.
- Audio cues for nearby threats.
- Lockdowns, timed escapes, self-destruct events, and power failures.
- Scarce healing and limited safe rooms.
- Contamination, infection, oxygen, or exposure systems can become later pressure layers if they serve the core loop.

### Progression

- Credits or salvage collected in missions.
- Terminals for supplies and upgrades.
- Campaign graph where exits choose the next location.
- Persistent player state between missions.

## First Vertical Slice

The first playable goal should be small and complete:

- one apothecary character
- one test map
- walls and collision
- camera follow
- mouse/keyboard movement and aiming
- one weapon
- projectiles and hit detection
- one enemy type
- health, ammo, pickup, and death states
- one locked door
- one access card
- one terminal
- one exit
- basic UI for health, ammo, keys, and objective state

This slice should prove the game loop before deeper systems are added.

## Research Tooling

Create a separate command-line tool for local analysis:

```text
euther-abta inspect /home/nichlas/AlienBrT/TA.EPF
euther-abta list /home/nichlas/AlienBrT/TA.EPF
euther-abta extract-map /home/nichlas/AlienBrT/TA.EPF <map-id>
```

Research tool goals:

- list EPF entries
- identify maps, tilesets, sprites, palettes, and sounds
- export reference PNG/JSON files into ignored local output
- document map dimensions and action codes

The research output should inform level design and tooling, not become shipping content.

## Milestones

### Milestone 0: Repository Setup

- Git repository initialized.
- Remote set to `https://github.com/NichlasEk/EutherBreed`.
- Project plan created.
- License decision made.
- Initial README created.

### Milestone 1: Rust Prototype

- Create Rust workspace.
- Pick Bevy or Macroquad.
- Render a moving player in a simple tile room.
- Add camera, collision, aiming, and shooting.
- Add one enemy with basic chase behavior.

### Milestone 2: Level Format

- Define original EutherBreed map format.
- Load tile layers and object layers from file.
- Support doors, terminals, pickups, spawn points, and exits.
- Add a debug level reload workflow.

### Milestone 3: Combat Loop

- Add health, ammo, damage, pickups, and death.
- Add at least two weapons.
- Add enemy spawners and alert states.
- Add basic lighting/fog-of-war.

### Milestone 4: Tower Assault Structure

- Add campaign graph.
- Add multiple exits per level.
- Add persistent inventory and credits.
- Add terminal shop/upgrades.
- Add route-dependent mission selection.

### Milestone 5: Tauri Shell

- Add Tauri app shell.
- Add main menu, settings, save slots, and launch flow.
- Package desktop build.

### Milestone 6: Editor And Tools

- Add level editor or integrate with an existing editor format.
- Add asset validation.
- Add local research import/export helpers.
- Add debug overlays for collision, AI, triggers, and spawn zones.

## Open Questions

- Use Bevy or Macroquad for the first prototype?
- Pixel-art sprites at high resolution, or painted/animated 2D sprites?
- Mouse/keyboard first, gamepad first, or equal priority?
- Local co-op in the first major version or later?
- Hand-authored levels only, or procedural side areas?
- Which license should the code use?

## Immediate Next Steps

1. Add `README.md` with project summary and non-goals.
2. Choose the Rust game framework for the prototype.
3. Scaffold the Rust workspace.
4. Add a tiny moving-player prototype.
5. Add `research/ABTA_NOTES.md` for findings from OpenBreed and the local DOS install.
