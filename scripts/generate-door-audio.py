#!/usr/bin/env python3
"""Original deterministic synthesized door cues; requires ffmpeg for Vorbis encoding."""
import math
from pathlib import Path
import random
import struct
import subprocess
import tempfile
import wave

OUT = Path(__file__).resolve().parents[1] / 'assets/audio'
OUT.mkdir(exist_ok=True)
RATE = 24000
for name, duration in [('bulkhead-open', 0.95), ('field-release', 0.75)]:
    rng = random.Random(42)
    samples = []
    low_noise = 0.0
    phase = 0.0
    for i in range(int(RATE * duration)):
        t = i / RATE
        u = t / duration
        noise = rng.uniform(-1, 1)
        low_noise = 0.88 * low_noise + 0.12 * noise
        if name == 'bulkhead-open':
            envelope = min(t / 0.045, 1) * min((duration - t) / 0.18, 1)
            phase += math.tau * (70 + 35 * math.sin(u * math.pi)) / RATE
            motor = 0.14 * math.sin(phase) + 0.04 * math.sin(phase * 3)
            latch = math.exp(-t * 55) * noise * 0.28
            stop = math.exp(-abs(t - 0.76) * 70) * low_noise * 0.40
            sample = envelope * (motor + low_noise * 0.23) + latch + stop
        else:
            envelope = min(t / 0.02, 1) * (1 - u) ** 2
            phase += math.tau * (1200 * (1 - u) ** 2 + 90) / RATE
            sample = envelope * (0.13 * math.sin(phase) + 0.075 * noise + 0.07 * math.sin(phase * 1.51))
        samples.append(struct.pack('<h', int(max(-1, min(1, sample)) * 32767)))
    with tempfile.TemporaryDirectory() as temp:
        wav = Path(temp) / 'source.wav'
        with wave.open(str(wav), 'wb') as output:
            output.setparams((1, 2, RATE, 0, 'NONE', 'not compressed'))
            output.writeframes(b''.join(samples))
        subprocess.run(['ffmpeg', '-v', 'error', '-y', '-i', str(wav), '-c:a', 'libvorbis', '-q:a', '4', str(OUT / f'{name}.ogg')], check=True)
