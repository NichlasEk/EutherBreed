# EutherBreed Art Direction

## Target

The game should end up with crisp, high-readable, high-resolution 2D assets rather than low-resolution pixel placeholders. Early art can be simple, but it should already point toward the final look: dark grounded sci-fi horror, medical/quarantine technology, hard industrial surfaces, biological contamination, and strong silhouettes.

The screen should read as dark, dangerous, and atmospheric. Darkness should come from low-key lighting, restrained floor/wall values, narrow pools of color, and strong contrast around interactable objects. Avoid flat black rooms where gameplay readability suffers.

## Asset Standards

- Use original assets only. Do not copy, trace, recolor, or ship art from Alien Breed, OpenBreed, or extracted commercial files.
- Prefer hand-authored or generated-and-curated high-resolution sprites over tiny pixel art.
- Keep assets readable from the top-down camera at the current prototype scale.
- Use consistent perspective: mostly top-down orthographic with a slight readable angle only where it helps object identity.
- Author assets with transparent backgrounds.
- Keep source/work files separate from shipping PNG/WebP atlases when those exist.
- Build visual appeal into the player character from the start; this is the hero the player will stare at for hours.

## Hero Direction

The player character is an adult female ship apothecary: attractive, confident, and deliberately stylized as a hot sci-fi action heroine while still belonging in a serious survival-horror game.

- Give her a clear feminine silhouette and a pronounced bust within a fitted practical suit or armor.
- Keep the design adult, powerful, and readable rather than nude or explicit.
- Lean into medical/biochemical identity: reagent bandolier, injector rig, field medic harness, sealed gloves, visor, compact lab gear, sample canisters.
- Avoid generic marine styling. She should look like someone who fights with chemistry, medicine, and quarantine tools.
- Make her silhouette readable from the top-down camera, including facing direction and weapon/tool direction.
- Use a restrained dark suit palette with medical teal/cyan highlights and small warm reagent accents.

## Initial Production Set

First build a small but cohesive prototype atlas:

- apothecary player sprite with clear facing/readability
- contaminant enemy sprite with hit-flash compatible palette
- floor tile variants for quarantine ward, lab corridor, and triage vault
- wall tile variants: straight, corner, cap, bulkhead detail
- locked/unlocked quarantine door states
- lab analyzer terminal, ship log terminal, supply console
- pickups: reagent rounds, med-gel, bio-sample, security keycard
- exit marker that reads as a route transition, not a generic button

## Current Concept Assets

- `assets/concepts/apothecary_hero_concept.png`: first transparent concept pass for the adult female apothecary hero. Use as direction reference, not final runtime sprite.
- `assets/sprites/source/apothecary_hero_concept_chroma.png`: generated chroma-key source for the concept image.

## Suggested Resolutions

- Characters/enemies: author around `128x128` or `192x192`, displayed smaller in-game as needed.
- Pickups: author around `96x96`, displayed around current pickup footprint.
- Terminals/doors: author around `192x192` to `384x256`, depending on footprint.
- Tiles: author at `128x128` or `256x256`; support atlas reuse and tinting per level.

## Implementation Plan

1. Keep current rectangle rendering until a cohesive first atlas exists.
2. Add a sprite asset loader/resource for atlas handles and named sprite rects.
3. Replace actors and interactables first; keep wall/floor rectangles until tile rules are ready.
4. Add floor and wall tiles once the level format can express tile layout cleanly.
5. Tune gameplay scale, collision radii, HUD contrast, and spawn pacing after visual assets are in place.
