import os,signal,time
for x in os.listdir('/proc'):
 if x.isdigit():
  try:
   a=open('/proc/'+x+'/cmdline','rb').read().split(b'\0')
   if a and b'ruffle-native' in os.path.basename(a[0]) and 'lonesurvivor' in os.readlink('/proc/'+x+'/cwd').lower():
    print('STOP',x);os.kill(int(x),signal.SIGKILL)
  except OSError:pass
for _ in range(30):
 time.sleep(.5)
 main=[];game=[]
 for x in os.listdir('/proc'):
  if x.isdigit():
   try:
    a=open('/proc/'+x+'/cmdline','rb').read().split(b'\0')
    if a and os.path.basename(a[0])==b'MainUI':main.append(x)
    if a and b'ruffle-native' in os.path.basename(a[0]):game.append(x)
   except OSError:pass
 if len(main)==1 and not game:print('CLEAN game=[] MainUI='+str(main));break
else:raise SystemExit('Cleanup verification failed')
