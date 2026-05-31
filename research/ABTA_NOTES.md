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
