# Controller mapping

- Y → Space: Inventory
- X → 1: Flare
- A → 2: Rotten Meat
- B → 3: Health Tonic
- Back → M: Map
- Start → P: Pause
- LT → C: Draw/Sheath Weapon
- LB → F: Flashlight
- RT → X: Interact/Shoot
- RB → R: Reload

D-pad and left stick retain directional keyboard input. Existing Start+Back exit combination is retained. The game can suppress shortcuts in cutscenes/dialogue or when required items are unavailable.

Ruffle names `left-trigger`/`right-trigger` refer to SDL shoulder buttons LB/RB. Physical LT/RT are `left-trigger-2`/`right-trigger-2`. The old frontend did not forward trigger-axis events. The dedicated runtime now converts SDL TriggerLeft/TriggerRight axis transitions to Ruffle button down/up, with 8000 press / 4000 release hysteresis, per-controller state and release on controller disconnect.

The launcher explicitly disables the old controller-driven pointer and A/B mouse clicks (`LS_NATIVE_MOUSE=0`). This prevents item shortcuts from also clicking the scene. Rendering/performance changes are retained; saves were not edited.

## Verification

- AArch64 release build passed.
- Actual TSPS runtime/config/launcher SHA-256 hashes matched the local artifacts after deployment.
- Synthetic events injected into real TRIMUI Player1 evdev event4 passed through SDL and the running Ruffle frontend. All ten requested keycodes matched exactly on both press and release; see `evidence/input-routing.log` and `input-routing.json`.
- Trigger test used the measured ABS_Z / ABS_RZ range 0–255 (SDL normalizes to 0–32767).
- RT advanced title/intro to a rendered room. No renderer panic or glow errors occurred.
- Inventory/pause screenshots were captured during an active dialogue; they do NOT prove visual inventory/pause operation. Physical owner validation of all gameplay actions is pending. Consumable shortcuts were probed at the title screen, not consumed in gameplay.
- Exact game process was stopped, exit helper gone, MainUI process returned and delayed DRM capture showed the Spruce PORTS menu.

Backups on device: `logs/config-before-input.ron`, `runtime-before-input.aarch64`, `launcher-before-input.sh`. No commercial game or save files are committed.
