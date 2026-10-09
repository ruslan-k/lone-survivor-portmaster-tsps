# Lone Survivor Director's Cut — feasibility result

Status: BLOCKED on TSPS Ruffle-Handheld GLES/glow. Not a playable port.

## Input and artifacts
- User installer: `/var/home/ruslan/Downloads/lone_survivor_directors_cut/setup_lone_survivor_-_the_directors_cut_1.0_(21178).exe`.
- Extracted with innoextract 1.9 in Debian bookworm container.
- Main content: LoneSurvivor.swf; captive Adobe AIR 3.1 descriptor; modified Flixel framework; SWF version 14, 640x480, 24 FPS.
- Original remains in extracted/. Reproducible exploratory patch: patch_game.py and scripts/.
- Patched SWF SHA256: 818bed5e51a69cb4a684e41598042a7a7f29bf08d43713ade20cea5100521655.

## Work completed
- Desktop original AIR-mode Ruffle reached title screen (desktop-air.png).
- Experimental AIR-free patch replaced TeleSave FileStream with ByteArray serialization stored in flushed SharedObject. Map save/load parameters changed accordingly. Original serialization order retained.
- NativeApplication exit calls replaced with pause; stage changed to showAll. This is exploratory: proper in-game quit integration and final scaling remain unfinished.
- Patched desktop Flash-player Ruffle responded to X presses and reached introductory dialogue (desktop-flash-player.png). No full save/load or full-game regression claim.
- Test launcher, config and patched SWF deployed to a separate lonesurvivor directory. Existing Tormentum untouched.
- Config button spelling must be `dpad-up`, not `d-pad-up` in this runtime.

## TSPS hard blocker
Principal-flow launch succeeded, SDL audio obtained 44100 Hz / 2 channels / 1024 samples. Preload ended in 0.090 seconds. Game then panicked with:

```
Failed to compile PixelBender shader: Unimplemented("compile_pixelbender_shader")
game_exit_code=101
```

Ruffle-Handheld glow backend explicitly returns Unimplemented from both compile_pixelbender_shader and run_pixelbender_shader (repository/source/runtime/ruffle_render_glow/src/lib.rs). This is a renderer capability gap, not installer extraction, missing files, input mapping, or memory evidence.

PlayState constructs SuperFog, SuperDream, Superflat and SuperflatOriginal shaders; optional SuperGrain shader. Lighting is gameplay-relevant: PlayState reads the post-filter buffer into player.lightness, and Map checks RGB <24 for darkness. Removing filters is therefore not a faithful working port.

## Remaining engineering
Implement functional Pixel Bender execution and ShaderFilter applyFilter support in the GLES renderer, or validate a different renderer with Pixel Bender support on TSPS. Then validate gameplay, pad, correct scaling, light/readback behavior, audio and save/load. No evidence yet that this alternative is feasible or fast enough on TSPS.

## Device cleanup
Game exited automatically; Spruce MainUI returned. Broken launcher moved out of PORTS menu into lonesurvivor/logs/Lone Survivor.sh.disabled; diagnostic directory retained. Game not left running. Read-back and menu screenshot performed.
