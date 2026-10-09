import os,signal,subprocess
main=[]
for x in os.listdir('/proc'):
 if x.isdigit():
  try:
   a=open('/proc/'+x+'/cmdline','rb').read().split(b'\0')
   if a and b'ruffle' in os.path.basename(a[0]):raise SystemExit('Game active; refusing launch')
   if a and os.path.basename(a[0])==b'MainUI':main.append(int(x))
  except OSError:pass
assert len(main)==1,main
assert not os.path.lexists('/tmp/gdata_')
assert not os.path.exists('/tmp/cmd_to_run.sh')
s='chmod a+x "/mnt/SDCARD/Emu/PORTS/../../spruce/scripts/emu/standard_launch.sh";"/mnt/SDCARD/Emu/PORTS/../../spruce/scripts/emu/standard_launch.sh" "/mnt/SDCARD/Roms/PORTS/Lone Survivor.sh"\n'
open('/tmp/cmd_to_run.sh','w').write(s);os.chmod('/tmp/cmd_to_run.sh',0o755)
subprocess.run(['sh','-n','/tmp/cmd_to_run.sh'],check=True);os.kill(main[0],signal.SIGTERM)
print('Queued through principal')
