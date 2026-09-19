#!/usr/bin/env python3
"""Original synthesized steel impacts and a short pressure-driven bulkhead rupture."""
from pathlib import Path
import hashlib, json, subprocess
import numpy as np
out = Path(__file__).resolve().parents[1] / 'assets/audio'
rate = 48000
rng = np.random.default_rng(190919)
report = []
for name, duration in [('bulkhead-impact', .32), ('bulkhead-breach', 1.15)]:
    t = np.arange(round(rate * duration)) / rate
    noise = rng.normal(size=len(t))
    if name == 'bulkhead-impact':
        signal = .12 * noise * np.exp(-t * 65)
        for freq, gain in [(182,.16),(417,.10),(763,.06),(1207,.035)]:
            signal += gain * np.sin(2*np.pi*freq*t) * np.exp(-t*21)
    else:
        freq = np.fft.rfftfreq(len(t), 1/rate)
        air = np.fft.irfft(np.fft.rfft(noise)/(1+(freq/1100)**2), n=len(t))
        signal = .18 * air * np.exp(-t*5) + .19*np.sin(2*np.pi*(72*t-14*t*t))*np.exp(-t*8)
        for delay, tone in [(.06,421),(.18,730),(.27,292),(.42,910)]:
            dt = np.maximum(0,t-delay)
            signal += (t>=delay) * np.minimum(dt/.001,1) * .10*np.sin(2*np.pi*tone*dt)*np.exp(-dt*20)
    signal *= np.minimum(t/.001,1) * np.minimum((duration-t)/.025,1).clip(0,1)
    dest=out/(name+'.ogg')
    subprocess.run(['ffmpeg','-v','error','-y','-f','f32le','-ar',str(rate),'-ac','1','-i','-','-c:a','libvorbis','-q:a','5',str(dest)], input=signal.astype('<f4').tobytes(),check=True)
    report.append(dict(file=dest.name,duration=duration,peak_db=float(20*np.log10(max(abs(signal)))),sha256=hashlib.sha256(dest.read_bytes()).hexdigest()))
print(json.dumps(report,indent=2))
