#!/usr/bin/env python3
"""Generate EutherBreed originals through the existing authenticated ACE-Step worker."""
import hashlib, json, time, urllib.request
from pathlib import Path
ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'assets/music'
TOKEN = Path('/home/nichlas/ai/eutherstudio/worker/config/shared_token').read_text().strip()
BASE = 'http://127.0.0.1:8795'
TRACKS = [
 ('abyssal-circuit', 'Instrumental cosmic horror survival game soundtrack, 88 BPM, D minor, oppressive alien medical spaceship ambience, immense sub bass drones, dissonant distant wordless choir textures, detuned metallic bells, restrained dark cyber techno pulse, sparse industrial clicks, slowly evolving unsettling harmony, ancient incomprehensible presence inside machinery. Subtle steady groove underneath long atmospheric phrases. No vocals or lyrics, no pop melody, no big risers or drops, no abrupt ending. Seamless looping game underscore, spacious mix, ominous and beautiful.'),
 ('quarantine-pulse', 'Instrumental cosmic horror meets dark cyber techno combat game soundtrack, 112 BPM, D minor, ominous pulsing analog bass, tight muted industrial kick, syncopated metallic percussion, distorted machine rhythms, eerie dissonant string and wordless choir pads, haunted alien laboratory, terrifying ancient intelligence waking inside computers. Controlled relentless tension, layered rhythm with breathing space for gunfire, no EDM festival drop, no cheerful melody, no singing, no lyrics, sustained ending for looping.')]
def request(path, payload=None):
    data = None if payload is None else json.dumps(payload).encode()
    req = urllib.request.Request(BASE+path, data=data, headers={'Authorization':'Bearer '+TOKEN, 'Content-Type':'application/json'})
    with urllib.request.urlopen(req, timeout=45) as r: return json.load(r)
for name, prompt in TRACKS:
    manifest = OUT / (name + '.json')
    if manifest.exists() and (OUT / (name + '-master.wav')).exists():
        print(name, 'already downloaded', flush=True); continue
    job_id = 'eutherbreed-20260919-' + name
    job = request('/jobs', {'job_id':job_id,'user':'nichlas','prompt':prompt,'instrumental':True,'duration_seconds':96,'format':'wav','pause_gpu_services':False})
    deadline = time.time()+2400
    last = ''
    while time.time()<deadline:
        job=request('/jobs/'+job_id)
        status = str((job.get('status'), job.get('phase'), job.get('status_text')))
        if status!=last: print(name,status,flush=True); last=status
        if job['status']=='done': break
        if job['status'] in ('error','failed'): raise RuntimeError(job)
        time.sleep(5)
    else: raise TimeoutError(job_id)
    if job.get('mode') == 'dryrun': raise RuntimeError('Refusing placeholder audio')
    req=urllib.request.Request(BASE+'/jobs/'+job_id+'/result',headers={'Authorization':'Bearer '+TOKEN})
    raw=urllib.request.urlopen(req,timeout=120).read()
    if raw[:4]!=b'RIFF': raise RuntimeError('Expected WAV master')
    (OUT/(name+'-master.wav')).write_bytes(raw)
    job['sha256']=hashlib.sha256(raw).hexdigest()
    manifest.write_text(json.dumps(job,indent=2)+'\n')
    print(name,'downloaded',len(raw),flush=True)
