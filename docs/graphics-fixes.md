# Graphics fixes: Overlay and partial bitmap uploads

## Fixed / physically confirmed

The owner confirmed the gray veil disappeared after the Overlay implementation, then confirmed the scattered squares/stripes disappeared after fixing partial texture updates. The latter were corrupted tutorial hints, not an intentional transition effect.

### Overlay

The game attaches scanline and grain bitmaps using `blendMode="overlay"`. The old glow implementation silently substituted Normal. This covered the scene with gray pixels instead of blending them with the underlying colors.

Bitmap Overlay now snapshots the actual draw framebuffer (resolving MSAA), computes the separable Overlay formula on unpremultiplied RGB, then performs premultiplied source-over. Fixed-function blending is disabled only for that draw. A persistent scratch texture/framebuffer is reused; texture/sampler and framebuffer bindings are restored. Native scanlines, grain, scaling, lighting and Pixel Bender remain enabled.

Four real Mali GPU draw/readback cases cover dark/light backdrop branches, black/white extrema and partially transparent source with opacity. All observed RGBA values exactly match the CPU reference in this run (test permits 2/255 rounding tolerance). See `partial-upload-green.log`.

### Corrupted hints / scene changes

`update_texture` received the complete bitmap plus its dirty rectangle, but called `glTexImage2D` with the dirty rectangle's width/height and the complete bitmap's data. This resized the backing texture while its registry dimensions remained unchanged.

A real GPU regression test created a 4x4 RGBA bitmap and changed a single pixel. Before the fix, readback contained only the first pixel followed by zeroes, and the test exited with code 1 (`partial-upload-red.log`). After using the complete bitmap dimensions for the upload, all 16 pixels match exactly, including the 15 untouched pixels (`partial-upload-green.log`). This is a correctness-first full upload, not an optimized dirty-region implementation.

## Device verification

- ARM64 release build completed; installed artifact read-back SHA-256:
  `954463d0abe485fe0e2a8acad249ec8a41b480eeaee2a8c8c1a8bc85cdb822f2`.
- Spruce principal launch used; no direct competing MainUI/game session.
- Title, scene changes and introductory dialogue captured from actual DRM scanout.
- Owner physically confirmed both symptoms removed.
- Exact test process stopped; gptokeyb exited and the real Spruce menu returned.
- Save files, game logic, commercial assets, audio/input mapping and Tormentum were not changed in this graphics phase.

## Remaining limitations

This is not pixel-perfect desktop parity or full-game validation. Fog/shader numerical parity, other advanced blend modes and non-bitmap Overlay are not established. Snapshot comparison contains time-dependent grain and animation, so it is not a strict rendering golden test.

The corrected renderer costs more: measured title/intro rendering is about 15–18 FPS versus approximately 26–27 FPS in the prior gray-overlay session. The driver swap phase rose from approximately 22 to 44 ms. Those sessions are not a controlled identical-scene performance benchmark; no performance improvement is claimed. Optimization is separate from these correctness fixes.

The packaged binary under `port/lonesurvivor/` is the tested dedicated Lone Survivor runtime. It does not include the commercial game. To update an existing installation, close the game, back up its runtime, replace that binary, mark it executable, sync, then launch the normal menu entry.
