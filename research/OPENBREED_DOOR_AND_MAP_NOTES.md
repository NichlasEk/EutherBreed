# OpenBreed Door and Map Notes

These notes summarize verified behavior from the local `~/OpenBreed` source. They are design references only; do not copy original assets or source code into EutherBreed.

## Verified OpenBreed Structure

- ABTA map cells carry two meanings: a graphical tile id and an action id. The README describes action ids as hardcoded executable behavior such as obstacles, doors, exits, spawns, and pickups.
- OpenBreed maps those action ids into entity names through per-level `MapCodesL*.cs` builders.
- Door action types include `DoorStandard`, `DoorRed`, `DoorGreen`, and `DoorBlue`.
- Colored doors map to keycard requirements:
  - `DoorRed` -> `Keycard1`
  - `DoorGreen` -> `Keycard2`
  - `DoorBlue` -> `Keycard3`
  - `DoorStandard` -> no key requirement
- OpenBreed has horizontal and vertical two-cell door templates with a blocking fixture plus `DoorOpenTrigger`.
- Its door script checks whether the actor has the required keycard, plays an opening animation, stamps an opened-door graphic into the map, then removes the blocking body after animation finish.
- Teleporters are paired entry/exit entities by pair id and use a fade-out, position-set, fade-in sequence.

## EutherBreed Direction

- Treat doors as world systems, not decoration.
- Keep three concepts separate:
  - Locked: requirements are missing.
  - Closed: requirements are met, but the physical passage is still blocked.
  - Opened: animation has completed and collision has been removed.
- Door opening should be triggered by player proximity/overlap, matching the original feel more than inventory-state autounlock.
- Larger maps should keep using explicit typed entities for doors, exits, teleports, terminals, shops, pickups, blockers, and hazards instead of baking everything into wall rectangles.
- Map polish can wait; future work should prioritize robust room/door connectivity, then richer tiles and props.
