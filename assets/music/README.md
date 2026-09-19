# EutherBreed score

Original instrumental tracks generated with the local ACE-Step 1.5 turbo model through the existing EutherStudio worker and EutherLink GPU queue.

- **Abyssal Circuit**: exploration; cosmic horror, dark drones and a restrained mechanical pulse. 88 BPM, 65.45-second loop.
- **Quarantine Pulse**: visible nearby threats; darker electronic rhythm. 111 BPM, 69.19-second loop.

`manifest.json` records prompts, generated captions, ACE task IDs, seeds, source/master hashes, source locations, edits and runtime hashes. The 96-second stereo WAV masters and full worker responses remain local and are ignored by Git; the Vorbis loops and manifest are the shipping assets.

Build: `python3 scripts/build-music-loops.py` (NumPy and ffmpeg). Regenerate masters: `python3 scripts/generate-music.py` (the existing local worker must be running and its token file present; credentials are never written to the repo). Completed job IDs are reused, not resubmitted.

Loops use bar-length edits and a one-bar wrap crossfade; static normalization targets -22 dB RMS with a -3 dB peak ceiling. Both files are 48 kHz stereo. This preserves headroom for combat effects. The engine keeps the two music players across loads and transitions, fades between them, and holds threat music for six seconds to prevent rapid toggling. Walls and closed doors occlude music threat detection.

Controls: **F6** toggles music; **F7/F8** decrease/increase music volume. Default level is 32%. Menus, pause and death reduce the music level. Separate music and SFX sliders are available under SETTINGS in the main menu and pause menu. Volume and music mute preferences are saved automatically in `saves/audio-settings.ron`.
