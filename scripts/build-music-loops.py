#!/usr/bin/env python3
"""Trim ACE-Step masters to bar-length loops, wrap-crossfade and normalize."""
import hashlib,json,subprocess,tempfile
from pathlib import Path
import numpy as np
ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/'assets/music'
report=[]
for name,bars in [('abyssal-circuit',24),('quarantine-pulse',32)]:
    source=OUT/(name+'-master.wav'); job=json.loads((OUT/(name+'.json')).read_text())
    meta=json.loads(job['detail']['generation']['ace_result']['data'][0]['result'])[0]
    bpm=float(meta['metas']['bpm']);rate=48000
    raw=subprocess.check_output(['ffmpeg','-v','error','-i',str(source),'-f','f32le','-ac','2','-ar',str(rate),'-'])
    pcm=np.frombuffer(raw,dtype='<f4').reshape(-1,2).copy()
    start=round(8*60/bpm*rate);length=round(bars*4*60/bpm*rate);fade=round(4*60/bpm*rate)
    if start+length+fade>len(pcm):raise ValueError('master too short')
    loop=pcm[start:start+length].copy();phase=np.linspace(0,np.pi/2,fade,endpoint=False)[:,None]
    loop[:fade]=pcm[start+length:start+length+fade]*np.cos(phase)+loop[:fade]*np.sin(phase)
    rms=float(np.sqrt(np.mean(loop**2)));peak=float(np.max(np.abs(loop)))
    loop*=min(10**(-22/20)/rms,10**(-3/20)/peak)
    destination=OUT/(name+'.ogg')
    subprocess.run(['ffmpeg','-v','error','-y','-f','f32le','-ar',str(rate),'-ac','2','-i','-','-c:a','libvorbis','-q:a','6',str(destination)],input=loop.astype('<f4').tobytes(),check=True)
    provenance={'name':name,'generator':'ACE-Step 1.5','model':meta['dit_model'],'task_id':job['detail']['generation']['ace_task_id'],'seed':meta['seed_value'],'prompt':job['prompt'],'generated_caption':meta['prompt'],'bpm':bpm,'bars':bars,'sample_rate':rate,'channels':2,'loop_seconds':length/rate,'crossfade_seconds':fade/rate,'rms_db':float(20*np.log10(np.sqrt(np.mean(loop**2)))),'peak_db':float(20*np.log10(np.abs(loop).max())),'master_path':job['output_path'],'master_sha256':job['sha256'],'runtime_sha256':hashlib.sha256(destination.read_bytes()).hexdigest()}
    report.append(provenance)
(OUT/'manifest.json').write_text(json.dumps(report,indent=2)+'\n')
for r in report:print(r['name'],r['loop_seconds'],'seconds',r['rms_db'],'dB RMS',r['peak_db'],'dB peak')
