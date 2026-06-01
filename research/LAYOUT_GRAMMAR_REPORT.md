# EutherBreed Layout Grammar Report

## Goal

Use classic top-down sci-fi shooter structure as design grammar without copying exact maps, room shapes, coordinates, encounter placement, names, or art.

The useful thing to borrow is how the levels think:

- large connected maps
- small rooms attached to long corridors
- locked shortcuts
- terminals that change route access
- supply stations before danger spikes
- lifts, teleporters, and exits as route transitions
- area scans as map rewards

## Current Local Reference Observation

Local archive inspection shows many dedicated map resources:

```text
MAP.01 through MAP.55
```

That supports the design assumption that the reference game treats map knowledge as a first-class system. EutherBreed should do the same with original map data and original map presentation.

## EutherBreed Rules

- Layout ideas are allowed.
- Exact maps are not.
- Do not trace room outlines from screenshots.
- Do not copy object placement or tile art.
- Rebuild every level around EutherBreed fiction: quarantine wards, apothecary stations, lab spines, bio-containment, triage vaults.

## Core Layout Pieces

### Spine

A long route that gives the player orientation. It can be a corridor, tram spine, service duct, or lab artery.

Use for:

- main traversal
- enemy pressure
- multiple side-room choices
- visible locked exits

### Loop

A route that returns to a known space after the player earns access.

Use for:

- shortcuts
- surprise enemy pressure
- relief after objective completion
- making maps feel less linear

### Pocket Room

A small optional room connected to a spine or loop.

Use for:

- ammo
- med-gel
- area scan
- lore terminal
- single ambush

### Lock

A door, lift, powered bridge, quarantine field, or sealed exit that blocks route progress.

Use for:

- keycard pickups
- terminal objectives
- multi-step route clarity

### Station

A shop/supply point or terminal alcove.

Use for:

- resupply before hard rooms
- objective interaction
- player planning pause

### Teleporter/Lift/Exit

Route transition devices. In the current prototype these are represented by `LevelExit`, but the fiction can label them as lifts, transport pads, breach doors, or teleporters.

Use for:

- level transitions
- alternate exits
- return routes
- locked progression

### Area Scan

A map access pickup. The current prototype now supports `AreaScan` pickups: Shift-map requires acquiring the scan in the current level.

Use for:

- rewarding exploration
- making large maps less overwhelming
- encouraging pocket-room searches

## Implemented Prototype Pass

This pass adds:

- `research_spine`: a larger original level with spine/loop structure
- new campaign link from `lab_access_corridor` into `research_spine`
- route from `research_spine` into `triage_vault`
- area scan pickup type
- Shift-map locked until the local area scan is acquired
- supply console that gives ammo and med-gel once
- terminal prompts that distinguish analyzer, log, and supply station
- level-bounds movement and a following camera so larger maps are not constrained to the old prototype room size

## Next Level Design Target

Build the next large level with this shape:

```text
entry
  -> spine corridor
    -> supply pocket
    -> area scan pocket
    -> locked lift
    -> objective terminal loop
    -> gore set-piece room
    -> alternate exit
```

Required assets for this to look good:

- lift/teleporter exit variants
- supply station sprite variant
- area scan pickup sprite
- door frame and lock-state sprites
- gore decals and corpse-room props
- brighter structural wall set for map readability

## Design Constraint

Do not make much bigger maps until the object/tile kit catches up. The current `research_spine` is intentionally a gameplay/layout prototype. It proves route grammar, but it will still need dressing before it should represent final visual quality.
