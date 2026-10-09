"""Exercise streamed inference with held-out vocals over instrumental audio."""
import json, math, sys, tempfile, time
from pathlib import Path
import numpy as np
from scipy.io import wavfile
from scipy.signal import resample_poly
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'res/vocals'))
from vocal_detector import analyze
manifest=json.loads((ROOT/'tools/vocals/training_manifest.json').read_text())
def load(path):
    rate, x=wavfile.read(path)
    if x.dtype.kind=='i': x=x.astype(np.float32)/np.iinfo(x.dtype).max
    elif x.dtype.kind=='u': x=(x.astype(np.float32)-128)/128
    else: x=x.astype(np.float32)
    if x.ndim>1:x=x.mean(axis=1)
    g=math.gcd(rate,44100); x=resample_poly(x,44100//g,rate//g)
    x=np.tile(x, math.ceil(44100*6/len(x)))[:44100*6]
    return x/max(np.sqrt(np.mean(x*x)),1e-5)*.1
results=[]
for i in range(3):
    voice=load(manifest['validation_vocals'][i]); background=load(manifest['validation_instrumental'][i])
    audio=np.concatenate([np.zeros(44100*2),background,voice+background,background,np.zeros(44100*2)]).astype('<f4')
    with tempfile.NamedTemporaryFile() as raw:
        audio.tofile(raw.name); start=time.monotonic(); result=analyze(raw.name,44100,ROOT/'res/vocals/model.json')
    def coverage(a,b):return sum(max(0,min(b,e)-max(a,s)) for s,e in result['segments'])/(b-a)
    results.append({'vocal_source':manifest['validation_vocals'][i], 'instrumental_source':manifest['validation_instrumental'][i], 'duration':len(audio)/44100, 'seconds_to_analyze':time.monotonic()-start, 'vocal_coverage':coverage(8,14),'instrumental_false_positive_coverage':(coverage(2,8)+coverage(14,20))/2,'segments':result['segments']})
print(json.dumps(results,ensure_ascii=False,indent=2))
(ROOT/'tools/vocals/sequence_validation.json').write_text(json.dumps(results,ensure_ascii=False,indent=2))
