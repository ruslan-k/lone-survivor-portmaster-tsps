# Single-sample framebuffer performance A/B

## Accepted change

The Lone Survivor launcher defaults `LS_MSAA_SAMPLES=1`, with explicit 2/4 overrides for comparison. This uses ordinary GLES renderbuffer_storage for both RGBA8 color and STENCIL_INDEX8 attachments; it does NOT call renderbuffer_storage_multisample(samples=1).

Resolution remains 1280x720; scene stage, nearest bitmap sampling, Overlay, scanlines/grain, Pixel Bender, audio clocks, input, CPU/GPU governors and saves are unchanged. Removing MSAA changes vector-edge antialiasing; this is not a claim of pixel-identical output in all scenes. The pixel-art room screenshots remain valid.

## Rejected first experiment / black-screen explanation

The first samples=1 trial used the multisample allocation API, inherited from the samples=4 renderer. It failed the real GPU Overlay test (zero pixels instead of the expected color), and the gameplay log contained invalid-framebuffer-operation 0x506 errors. The owner reported a black third scene during that experiment. That result is rejected, not counted as acceleration; the previously validated runtime was restored before the allocation path was corrected.

After switching to ordinary single-sample renderbuffer allocation, the four real GPU Overlay cases exactly matched expected RGBA, and the 4x4 partial texture regression test preserved all 15 untouched pixels. Both draw and resolved framebuffer completeness are now checked; framebuffer setup failure stops the renderer instead of continuing with an invalid target. The test runner gates performance trials on passing correctness tests and no renderer errors.

## Real TSPS measurements

Same game/SWF, same new runtime, same principal launcher; change only the sample-count environment override. Each run lasted 70 seconds. Statistics use the last 8 logged 5-second windows, excluding initial startup. Captures show the same dark room, but character positions/animation differ, so this is not a deterministic frame-identical benchmark or an FPS guarantee for the entire game.

- Four-sample: median 16.18 FPS, range 16.00–17.53; median swap 44.17 ms.
- Correct single-sample: median 32.165 FPS, range 29.67–34.28; median swap 16.50 ms.
- Observed median ratio: approximately 1.99x (+98.8%). No renderer errors in these accepted runs.
- Installed default launcher was tested again independently: post-startup windows 29.90, 35.27, 33.77, 32.19, 32.10 FPS; swap about 15.9–16.0 ms.

See `docs/evidence/perf-summary.json`, accepted A/B logs/screenshots and installed-default log. This is measured rendering throughput; matching game-time/audio progression and all later scenes still require gameplay testing.

## Deployment / rollback

Previous runtime and launcher are retained under the device port's logs directory as `runtime-before-perf.aarch64` and `launcher-before-perf.sh`. The accepted runtime keeps the graphics fixes and renderer defaults to the previous quality unless the port launcher supplies the single-sample override. To compare, set `LS_MSAA_SAMPLES=4`; to revert completely, close the game and restore the two preserved files, chmod, sync and verify hashes.

After all trials, only exact game processes were stopped; gptokeyb exited and a real DRM capture verified the Spruce menu returned. Tormentum and system display/audio configuration were not modified.
