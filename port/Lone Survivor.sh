#!/bin/bash

XDG_DATA_HOME=${XDG_DATA_HOME:-$HOME/.local/share}

if [ -d "/opt/system/Tools/PortMaster/" ]; then
  controlfolder="/opt/system/Tools/PortMaster"
elif [ -d "/opt/tools/PortMaster/" ]; then
  controlfolder="/opt/tools/PortMaster"
elif [ -d "$XDG_DATA_HOME/PortMaster/" ]; then
  controlfolder="$XDG_DATA_HOME/PortMaster"
else
  controlfolder="/roms/ports/PortMaster"
fi

source $controlfolder/control.txt
get_controls

[ -f "${controlfolder}/mod_${CFW_NAME}.txt" ] && source "${controlfolder}/mod_${CFW_NAME}.txt"
GAMEDIR="/$directory/ports/lonesurvivor"
BINARY=ruffle-native-multifile.aarch64.lazy
mkdir -p "$GAMEDIR/storage" "$GAMEDIR/logs"
cd "$GAMEDIR" || exit 1
exec > >(tee "$GAMEDIR/log.txt") 2>&1
printf 'LoneSurvivor Ruffle test: %s\n' "$(date)"
[ -f "$GAMEDIR/gamedata/LoneSurvivor.swf" ] || { echo 'Missing gamedata/LoneSurvivor.swf'; exit 1; }
# Upstream's multi-file frontend has a fixed content root. Expose SD data
# using a symlink, not a 300 MB copy into the handheld's RAM-backed /tmp.
if [ -e /tmp/gdata_ ] || [ -L /tmp/gdata_ ]; then
  echo 'Content root /tmp/gdata_ already exists, refusing to replace it.'
  exit 1
fi
ln -s "$GAMEDIR/gamedata" /tmp/gdata_ || exit 1
cleanup() {
  [ "$(readlink /tmp/gdata_ 2>/dev/null)" = "$GAMEDIR/gamedata" ] && rm /tmp/gdata_
  [ -n "${helper_pid:-}" ] && kill "$helper_pid" 2>/dev/null || true
}
trap cleanup EXIT
# The native frontend supplies its own controller-driven cursor and A click.
# gptokeyb2 only supplies the exit helper, avoiding duplicate synthetic clicks.
if [ -z "${sdl_controllerconfig:-}" ]; then
  sdl_controllerconfig="$(grep '^0300a3845e0400008e02000014010000,' "$controlfolder/gamecontrollerdb.txt")"
fi
export SDL_GAMECONTROLLERCONFIG="$sdl_controllerconfig"
export RUST_BACKTRACE=1
export TOR_AUDIO_SAMPLES=1024
export TOR_STREAM_SYNC_ROOT_ONLY=0
# Pixel-art output: validated single-sample buffers, not multisample(samples=1).
# Override LS_MSAA_SAMPLES=4 for the preserved quality/performance A/B baseline.
export LS_MSAA_SAMPLES="${LS_MSAA_SAMPLES:-1}"
export TOR_BITMAP_CACHE_DIR="$GAMEDIR/cache/bitmaps-v1"
if [ -r /proc/asound/cards ] && grep -q '\[audiocodec' /proc/asound/cards; then
  export ALSA_CONFIG_PATH="$GAMEDIR/asound.conf"
fi
$GPTOKEYB2 "ruffle-native-m" -c "$GAMEDIR/lonesurvivor.ini" &
helper_pid=$!
pm_platform_helper "$GAMEDIR/$BINARY"
"$GAMEDIR/$BINARY"
rc=$?
echo "game_exit_code=$rc"
pm_finish
exit "$rc"
