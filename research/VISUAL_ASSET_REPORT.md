# EutherBreed Visual Asset Report

## Purpose

This report translates the useful visual lessons from classic top-down sci-fi shooters into an original EutherBreed asset plan. The reference material can guide density, readability, tile categories, HUD structure, and gore placement, but EutherBreed must not copy, trace, recolor, ship, or derive from commercial assets.

## What The Reference Does Well

- Rooms are built from a small number of strong tile families: floor grids, bright wall frames, dark voids, hazard strips, grates, consoles, pipes, and large machinery.
- The playfield is readable because walls are brighter and heavier than floors, while doors and interactables get strong contrast accents.
- The maps feel bigger than the screen because corridors continue offscreen, rooms have partial reveals, and the HUD frames the viewport like an arcade cabinet.
- Floors are not plain filler. They carry panel seams, lights, arrows, stains, grates, and little debris patches.
- Blood and damage work because they are used as local story details: corpse rooms, lab tables, burst holes, wall smears, and contaminated corners.
- Objects sit inside the tile language. Beds, terminals, crates, machinery, pipes, tanks, vents, and floor holes all reinforce the same facility logic.
- HUD values are mostly compact instruments, not paragraphs: status bars, small counters, segmented rails, and icons.

## Current EutherBreed State

Runtime already has:

- top-down apothecary hero with walk frames
- two-armed contaminant with stride frame
- dark biomech floor variants
- wall panel and wall cap sprites
- quarantine door, exit marker, terminals, pickups, reagent projectile
- split cyber-biomech HUD rails with health and ammo pips
- three connected prototype levels

The biggest visual gap is not the player or HUD anymore. It is level construction variety: the same floor tile covers too much space, walls are not yet a full modular kit, and rooms need more physical set dressing.

## Recommended Next Step

Build a proper original high-resolution tile and object kit before making much bigger maps.

Larger maps will look empty if they are just the current floor repeated over a bigger rectangle. A richer kit first gives every later map pass better visual density, stronger navigation landmarks, and more room identity.

## Priority Asset Kit

### 1. Structural Tiles

Create a modular set that can build rooms, corridors, and chokepoints:

- floor base tiles: clean, ribbed, grated, medical panel, cargo deck
- floor edge/corner tiles for room boundaries
- bright wall panels with north/south/east/west variants
- wall corners, inner corners, caps, pillars, bulkhead sockets
- corridor trims and threshold frames
- dark void/black exterior tiles for off-map areas
- hazard stripe strips and broken stripe variants
- door frames separate from door sprites

Target: `128x128` or `256x256` source tiles, displayed at current scale.

### 2. Room Identity Sets

Make rooms visually different by function:

- quarantine ward: containment beds, sample tanks, med cabinets, warning panels
- lab access: console walls, cable trays, analyzer benches, specimen lockers
- triage vault: sealed storage, freezer pods, reagent racks, reinforced floor plates
- infestation zone: organic mats, ribbed growth, torn panels, wet red-black stains
- service corridor: pipes, vents, grates, electrical boxes, access hatches

Each room set should have floor variants, wall decorations, and 5-10 objects.

### 3. Gore And Damage Decals

Add decals as separate sprites so they can be layered over any floor:

- small blood drops
- directional blood smear
- large burst pool
- drag trail
- corpse stain under object
- acid scorch
- bullet/syringe impact marks
- cracked floor panel
- broken light cone or glow stain

Keep these original and painterly/biomech, not copied splats. They should be occasional landmarks, not wallpaper.

### 4. Objects And Cover

The current rooms need object silhouettes:

- lab table with scattered tools
- med bed with restraints
- corpse pile or sealed body bag
- bio canister rack
- broken terminal
- upright tank
- floor pipe cluster
- wall pipe cluster
- crate/module blocks
- reactor/control plinth
- vent/fan
- floor hole or ruptured grate

These should be collision-aware later: some block movement, some are decorative only, some are interactable.

### 5. Doors And Interactables

Improve gameplay readability:

- locked door state with orange/red lock core
- unlocked door state with teal open route core
- door frame lights
- objective terminal variants
- shop/supply terminal variant
- dead terminal variant
- keycard socket
- exit portal variants by destination

### 6. HUD Icons

The top HUD now works better with pips. Next small pass:

- small health vial/cross icon or apothecary suit icon
- reagent ammo capsule icon
- keycard icon
- bio-sample vial icon
- lives/suit integrity icon

Use icons sparingly; the current labels can remain until icon readability is proven.

## Bigger Map Plan

After the asset kit exists, make one larger vertical slice level rather than many half-dressed rooms.

Recommended shape:

- 3 main rooms
- 2 corridor loops
- 1 locked shortcut
- 1 terminal objective
- 1 optional supply room
- 2 exits, one locked until objective complete
- 2-3 enemy spawn pockets
- 1 memorable gore set piece

This is enough to test navigation, combat pacing, HUD prompts, map overlay, and visual variety without creating a content swamp.

## Implementation Notes

- Keep map polish deferred except for asset-driven readability. The map overlay can wait.
- Add a tile/object metadata layer before the kit grows too large: collision, draw layer, theme tags, and intended scale.
- Prefer reusable modular pieces over one-off full-room images.
- Keep the current dark biomech direction, but introduce brighter structural wall metals for readability.
- Do not use extracted original tiles in runtime assets. Local research can inspect categories and layout patterns only.

## Proposed Production Order

1. Build structural tile kit.
2. Build object/decal kit for quarantine ward and lab access.
3. Add tile/object placement data to level files.
4. Dress one larger vertical slice level.
5. Tune camera scale, collision, spawn pacing, and pickup placement.
6. Only then return to map overlay polish.
