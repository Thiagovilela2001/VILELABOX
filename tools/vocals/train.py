#!/usr/bin/env python3
import os
os.environ['OPENBLAS_NUM_THREADS']='1'
os.environ['OMP_NUM_THREADS']='1'
import sys, json, random, subprocess, hashlib
from pathlib import Path
from concurrent.futures import ThreadPoolExecutor
import numpy as np
from scipy.io import wavfile
from scipy.signal import resample_poly
from sklearn.neural_network import MLPClassifier
from sklearn.preprocessing import StandardScaler
from sklearn.metrics import classification_report, roc_auc_score
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'res/vocals'))
from vocal_detector import features, RATE, WINDOW
rng=random.Random(42)
base=Path('/home/thiago/Documentos/drumkit')
pos=[];neg=[]
for p in sorted(base.rglob('*.wav')):
 s=str(p.relative_to(base)).lower()
 if any(t in s for t in ['vocal','acapella','acappella']):pos.append(p)
 elif ('loop' in s or 'one_shot' in s) and any(t in s for t in ['drum','bass','synth','piano','guitar','melod']) and not any(t in s for t in ['vox','voice']):neg.append(p)
rng.shuffle(pos);rng.shuffle(neg)
# Whole packs are kept apart; no randomly adjacent frames in validation.
heldout={'Grime','Dropgun Samples Cyclops Dubstep'}
train_pos=[p for p in pos if p.relative_to(base).parts[0] not in heldout][:650]
train_neg=[p for p in neg if p.relative_to(base).parts[0] not in heldout][:850]
test_pos=[p for p in pos if p.relative_to(base).parts[0] in heldout][:180]
test_neg=[p for p in neg if p.relative_to(base).parts[0] in heldout][:220]
manifest={'train_vocals':[str(p) for p in train_pos], 'train_instrumental':[str(p) for p in train_neg], 'validation_vocals':[str(p) for p in test_pos], 'validation_instrumental':[str(p) for p in test_neg]}
(ROOT/'tools/vocals/training_manifest.json').write_text(json.dumps(manifest,ensure_ascii=False,indent=2))
def load(p):
 try:
  rate,x=wavfile.read(p)
  if x.dtype.kind=='i':x=x.astype(np.float32)/np.iinfo(x.dtype).max
  elif x.dtype.kind=='u':x=(x.astype(np.float32)-128)/128
  else:x=x.astype(np.float32)
  if x.ndim>1:x=x.mean(axis=1)
  if rate!=RATE:
   import math
   g=math.gcd(int(rate),RATE);x=resample_poly(x,RATE//g,int(rate)//g)
  if len(x)<1600:return []
  return [x[int(f*max(0,len(x)-WINDOW)):int(f*max(0,len(x)-WINDOW))+WINDOW] for f in [0.15,0.5,0.85]]
 except Exception:return []
def samples(paths):
 result=[]
 with ThreadPoolExecutor(max_workers=4) as pool:
  for i,clips in enumerate(pool.map(load,paths)):
   for clip in clips:
    if np.sqrt(np.mean(clip*clip))>0.003:result.append(clip)
   if i%200==0:print('decode',i,'/',len(paths),flush=True)
 return result
def make_data(vocals,instruments,seed):
 randomizer=np.random.default_rng(seed)
 X=[];y=[]
 for label,collection in [(0,instruments),(1,vocals)]:
  for audio in collection:
   X.append(features(audio));y.append(label)
   if label:
    neg=instruments[int(randomizer.integers(len(instruments)))];neg=np.pad(neg,(0,max(0,len(audio)-len(neg))))[:len(audio)]
    voice=audio/max(np.sqrt(np.mean(audio*audio)),1e-5)
    background=neg/max(np.sqrt(np.mean(neg*neg)),1e-5)
    mix=voice*10**(randomizer.uniform(-6,6)/20)+background
    X.append(features(mix));y.append(1)
 return np.asarray(X),np.asarray(y)
print('TRAIN files',len(train_pos),len(train_neg),'VALIDATION files',len(test_pos),len(test_neg),flush=True)
v=samples(train_pos);n=samples(train_neg);tv=samples(test_pos);tn=samples(test_neg)
X,y=make_data(v,n,42);Xt,yt=make_data(tv,tn,123)
scaler=StandardScaler().fit(X)
model=MLPClassifier(hidden_layer_sizes=(64,32),max_iter=220,early_stopping=True,validation_fraction=.15,n_iter_no_change=15,random_state=42,alpha=.05,verbose=True)
model.fit(scaler.transform(X),y)
pred=model.predict_proba(scaler.transform(Xt))[:,1]
report={'held_out_packs':sorted(heldout),'train_windows':len(y),'validation_windows':len(yt),'auc':roc_auc_score(yt,pred),'report':classification_report(yt,pred>=.9,output_dict=True)}
print(json.dumps(report,indent=2),flush=True)
(ROOT/'tools/vocals/validation.json').write_text(json.dumps(report,indent=2))
output={'version':'vilelabox-local-vocals-1','threshold':.9,'mean':scaler.mean_.tolist(),'scale':scaler.scale_.tolist(),'weights':[a.tolist() for a in model.coefs_],'biases':[a.tolist() for a in model.intercepts_]}
(ROOT/'res/vocals/model.json').write_text(json.dumps(output,separators=(',',':')))
np.savez_compressed(ROOT/'tools/vocals/validation_features.npz',X=Xt,y=yt,p=pred)
print('Saved model',flush=True)
