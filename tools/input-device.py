"""TSPS real evdev->SDL->Ruffle smoke test; run only on a stopped game."""
import os, struct, time, subprocess, json
from pathlib import Path
b=Path('/mnt/SDCARD/Roms/PORTS/lonesurvivor')
launch=(b/'logs/device-launch.py').read_text().replace('chmod a+x','export LS_INPUT_TRACE=1;chmod a+x')
exec(compile(launch,'principal-launch','exec'))
time.sleep(10)
fd=os.open('/dev/input/event4',os.O_WRONLY)
def event(t,c,v):
 os.write(fd,struct.pack('llHHi',0,0,t,c,v));os.write(fd,struct.pack('llHHi',0,0,0,0,0))
def press(t,c,on,off=0):
 event(t,c,on);time.sleep(.15);event(t,c,off);time.sleep(.35)
def grab(n):subprocess.run(['/mnt/SDCARD/spruce/bin64/kmsgrab',str(b/f'logs/input-{n}.png')],check=True)
# Test consumable shortcuts at the title screen, not in gameplay.
for name,t,c,on in [('Y',1,308,1),('X',1,307,1),('A',1,304,1),('B',1,305,1),('Back',1,314,1),('LB',1,310,1),('RB',1,311,1),('LT',3,2,255),('RT',3,5,255),('Start',1,315,1)]:
 print('PROBE',name,flush=True);press(t,c,on)
time.sleep(1)
log=(b/'log.txt').read_text();(b/'logs/input-routing.log').write_text(log)
expected={'North':32,'West':49,'South':50,'East':51,'Select':77,'LeftTrigger':70,'RightTrigger':82,'LeftTrigger2':67,'RightTrigger2':88,'Start':80}
results={n:{s: f'input_trace {s} {n} key=Some(KeyCode({code}))' in log for s in ['down','up']} for n,code in expected.items()}
(b/'logs/input-routing.json').write_text(json.dumps(results,indent=2))
print(json.dumps(results));assert all(all(v.values()) for v in results.values()),'Missing routed press/release'
# Undo pause, then progress intro with the assigned interact trigger.
press(1,315,1)
for _ in range(18):press(3,5,255);time.sleep(.65)
time.sleep(2);grab('scene')
press(1,308,1);time.sleep(1);grab('inventory')
press(1,308,1)
press(1,315,1);time.sleep(1);grab('pause')
press(1,315,1)
os.close(fd)
log=(b/'log.txt').read_text();(b/'logs/input-full.log').write_text(log)
assert 'panicked at' not in log and 'ERROR ruffle_render_glow' not in log
subprocess.run(['python3',str(b/'logs/stop-device.py')],check=True)
time.sleep(6);grab('menu-after')
