import os,sys,time,subprocess,json
from pathlib import Path
b=Path('/mnt/SDCARD/Roms/PORTS/lonesurvivor');samples=int(sys.argv[1]);tag=sys.argv[2];assert samples in (1,2,4);assert tag.replace('-','').isalnum()
launch=(b/'logs/device-launch.py').read_text().replace('chmod a+x',f'export LS_MSAA_SAMPLES={samples};chmod a+x')
if '--selftest' in sys.argv:launch=launch.replace('chmod a+x','export LS_OVERLAY_SELFTEST=1;chmod a+x')
exec(compile(launch,'principal-launch','exec'))
time.sleep(8 if '--selftest' in sys.argv else 70)
log=(b/'log.txt').read_text();(b/f'logs/perf-{tag}.log').write_text(log)
if '--selftest' not in sys.argv:
 subprocess.run(['/mnt/SDCARD/spruce/bin64/kmsgrab',str(b/f'logs/perf-{tag}.png')],check=True)
 meta={}
 for p in ['/sys/devices/system/cpu/cpu4/cpufreq/scaling_cur_freq','/sys/class/devfreq/1800000.gpu/cur_freq','/proc/meminfo']:
  try:meta[p]=Path(p).read_text()
  except OSError:pass
 (b/f'logs/perf-{tag}.json').write_text(json.dumps(meta))
subprocess.run(['python3',str(b/'logs/stop-device.py')],check=True)
print(log[-1800:])
if '--selftest' in sys.argv and ('overlay_selftest PASS' not in log or 'partial_texture_selftest PASS' not in log):
 raise SystemExit('GPU correctness gate failed; do not run performance trials')
if 'ERROR ruffle_render_glow' in log or 'panicked at' in log:
 raise SystemExit('Renderer errors invalidate this performance trial')
