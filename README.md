# Lone Survivor — Director's Cut on TrimUI Smart Pro S

Experimental PortMaster-style port using a dedicated Ruffle-Handheld runtime.

**Status: work in progress.** Pixel Bender shaders now compile on the real TSPS GLES driver and the game reaches the introduction. Bitmap ShaderFilter execution is implemented without removing the game's lighting. Full-game, save/reload and shader numerical parity validation are still in progress. This is not a claim of complete Flash/Pixel Bender compatibility.

## Included / excluded

- Included: runtime sources, upstream Naga Pixel Bender translator, GLES backend changes, notices, launcher/config, owner-side patch recipe and diagnostic reports.
- Excluded: commercial game SWF, decompiled game source, installer, save files, extracted shader bytecode and backups. Supply your own legally obtained game.
- Existing Tormentum installation/runtime is not modified.

## Runtime design

Pixel Bender bytecode → upstream Naga IR → GLSL ES 3.00 → TSPS Mali GPU. Normal bitmap outputs and `BitmapData.applyFilter(ShaderFilter)` use a separate scratch framebuffer, then upload/read back the result. Shader uniform values and image inputs are preserved; unsupported paths report errors instead of pretending to be implemented.

The renderer currently supports the game-relevant bitmap path. Float ByteArray ShaderJob outputs and external RawTexture inputs are not implemented yet. Other pre-existing glow filter limitations remain; no universal Flash compatibility claim.

## Game adaptation

AIR FileStream persistence is replaced by ByteArray serialization plus flushed SharedObject storage. Game shaders and lighting logic remain intact. The true Flixel game frame is 160×90, scaled to a 640×360 stage and then to 1280×720; AIR-only fullScreenSourceRect and NO_SCALE are replaced.

## Controls

[Requested keyboard/controller mapping and device test evidence](docs/controls.md): Y=Space, X=1, A=2, B=3, Back=M, Start=P, LT=C, LB=F, RT=X, RB=R. D-pad/left stick retain movement; A/B mouse clicks are disabled. Trigger axes now forward key press/release events.

## Verification

Real device launch uses Spruce's principal flow. Shader creation logs show NewFilter and NormalMapper compiling rather than panicking. Physical owner confirmed picture and sound. Remote synthetic controller events advance the introductory dialogue; this is not a substitute for physical button validation. Fullscreen correction is verified by a real DRM kmsgrab capture.

## Graphics correctness update

The gray Overlay veil and corrupted tutorial/transition bitmaps are fixed on TSPS and physically confirmed by the owner. Four GPU Overlay cases and a partial-upload regression test pass. The tested dedicated runtime is included under `port/lonesurvivor/`; no game SWF or saves are included. See [graphics fixes, evidence and limitations](docs/graphics-fixes.md). A subsequent [single-sample framebuffer A/B](docs/performance-single-sample.md) increased measured room rendering from a median 16.18 to 32.165 FPS; the launcher now selects the validated single-sample path. Full-game and pixel-perfect desktop parity remain unverified.

See docs/ for current evidence and limitations. Installation/build instructions will be completed with the tested package; do not treat the source snapshot as a stable release.
