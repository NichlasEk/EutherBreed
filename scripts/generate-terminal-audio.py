#!/usr/bin/env python3
"""Original low-key terminal cues and a seamless ventilation bed (NumPy/ffmpeg)."""
from pathlib import Path
import json, hashlib, subprocess
import numpy as np
OUT=Path(__file__).resolve().parents[1]/'assets/audio'
RATE=48000
rng=np.random.default_rng(190926)
report=[]
for name,duration in [('terminal-used',0.50),('terminal-denied',0.28),('terminal-analysis',1.40),('ventilation',4.0)]:
 t=np.arange(round(duration*RATE))/RATE
 if name=='terminal-used':
  signal=np.zeros_like(t)
  for start,freq in [(0.02,148),(0.24,132)]:
   dt=np.maximum(0,t-start);env=(t>=start)*np.minimum(dt/0.003,1)*np.exp(-dt*27)
   signal+=env*(0.38*np.sin(2*np.pi*freq*dt)+0.12*np.sin(2*np.pi*freq*2.37*dt))
 elif name=='terminal-denied':
  env=np.minimum(t/0.004,1)*np.exp(-t*18)
  signal=env*(0.20*np.sin(2*np.pi*82*t)+0.09*np.sin(2*np.pi*164*t)+0.045*rng.normal(size=len(t)))
 elif name=='terminal-analysis':
  signal=np.zeros_like(t)
  for start,freq in [(0.02,330),(0.30,440),(0.60,660)]:
   dt=np.maximum(0,t-start);env=(t>=start)*np.minimum(dt/0.02,1)*np.exp(-dt*6)
   signal+=env*(0.15*np.sin(2*np.pi*freq*dt)+0.04*np.sin(2*np.pi*freq*2*dt))
  signal+=0.018*np.sin(2*np.pi*(120*t+200*t*t))*np.sin(np.pi*t/duration)**2
 else:
  noise=rng.normal(size=len(t));freq=np.fft.rfftfreq(len(t),1/RATE)
  noise=np.fft.irfft(np.fft.rfft(noise)/(1+(freq/240)**4),n=len(t))
  noise=noise/max(abs(noise))*0.20
  signal=noise+0.055*np.sin(2*np.pi*48*t)+0.025*np.sin(2*np.pi*96*t)
 if name!='ventilation': signal*=np.minimum((duration-t)/0.03,1).clip(0,1)
 dest=OUT/(name+'.ogg')
 subprocess.run(['ffmpeg','-v','error','-y','-f','f32le','-ar',str(RATE),'-ac','1','-i','-','-c:a','libvorbis','-q:a','5',str(dest)],input=signal.astype('<f4').tobytes(),check=True)
 report.append({'file':dest.name,'duration':duration,'peak_db':float(20*np.log10(max(abs(signal)))),'sha256':hashlib.sha256(dest.read_bytes()).hexdigest()})
print(json.dumps(report,indent=2))
