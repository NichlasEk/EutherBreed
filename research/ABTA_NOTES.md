# ABTA Research Notes

This folder is for local research notes and non-shipping analysis output.

Reference install:

```text
/home/nichlas/AlienBrT/
```

Initial files of interest:

- `TA.EPF`
- `TA.EXE`
- `PLAY.EXE`
- `INTRO.ANM`
- `OUTRO.ANM`

OpenBreed reference:

```text
https://github.com/mrpetro/OpenBreed
```

The goal is to understand useful structures such as archive entries, map dimensions, tile references, action codes, and progression patterns. Extracted commercial assets should not be committed or shipped.

## EPF Header Findings

Initial inspection of `/home/nichlas/AlienBrT/TA.EPF`:

```text
magic: EPFS
size_bytes: 2267004
directory_offset: 2263044
directory_size_bytes: 3960
```

The bytes at `directory_offset` contain readable resource names such as `LEVEL1.BLK`, `MAP.01`, `HERO1.SPR`, and `TITLEBAC.LBM`. `abta_tools list` currently uses a heuristic parser for this directory. It is useful for orientation, but the entry metadata is not fully understood yet and some offset/size rows may be wrong until the EPF table format is confirmed.

`TA.EPF` contains at least eight `LEVEL*.BLK` entries and many `MAP.xx` entries. The separate `MAP.xx` resources are a strong hint that Tower Assault keeps map/minimap presentation data distinct from the main level block data. EutherBreed should follow that shape: runtime level data can drive gameplay, while a dedicated map layer presents a simplified readable overview instead of rendering a screenshot of the room.

## Local Runtime Note

On 2026-05-31, GUI smoke testing exposed a local NVIDIA driver mismatch:

```text
nvidia-smi: Failed to initialize NVML: Driver/library version mismatch
loaded kernel module: 595.71.05
nvidia-utils: 610.43.02
```

When this mismatch is present, Vulkan/wgpu may fail with `Unable to find a GPU`, even though the game compiles. Use the headless smoke mode for code checks until the local driver stack is synchronized:

```sh
cargo run -p euther_game -- --headless-smoke
```

Likely system-level fixes are rebooting into the matching NVIDIA kernel module after an upgrade, or reinstalling/downgrading NVIDIA packages so the kernel module and userland libraries match.
