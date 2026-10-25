"""P6-2: qual comportamento de cue roda com Q ligado/desligado e regra do snap (+0x98).

Uso: python p6_2_quantize_probe.py <pid>   (rekordbox 7.2.8 aberto, faixa com grade no deck, deck PARADO)

Passos: acha o deck por varredura de memória (vtable 0x1454066b0), liga ganchos em
0x142c7d6f0 / 0x142ab0740 / 0x142c7d8cb / 0x142c7d8d0, troca o 0x142c80840 por um
invólucro que registra o double devolvido (é o xmm3 do snap; o contexto de CPU do Frida
não expõe xmm), alterna Q pelo slot 9 e chama unit#71(unit, 1). Depois de capturar o
alvo do snap (rbx) e seus argumentos, varre xmm3 chamando o snap direto, sem gravar cue.
ATENÇÃO: o aperto de pad grava hot cue A na faixa carregada se o pad estiver vazio.
"""
import json
import sys
import time

import frida

JS = r"""
const B = Process.getModuleByName('rekordbox.exe').base;
const R = a => B.add(ptr(a).sub(0x140000000));
const VT = R(0x1454066b0);
let unit = null, snap = null, inDelta = null;

function pattern(p) {
  let v = uint64(p.toString()), s = [];
  for (let i = 0; i < 8; i++) { s.push(('0' + v.and(0xff).toString(16)).slice(-2)); v = v.shr(8); }
  return s.join(' ');
}
const t = () => Date.now();

const getDelta = new NativeFunction(R(0x142c80840), 'double', ['pointer']);
Interceptor.replace(R(0x142c80840), new NativeCallback(self => {
  inDelta = getDelta(self);
  send({ev: 'delta_0x142c80840', t: t(), xmm3: inDelta});
  return inDelta;
}, 'double', ['pointer']));
Interceptor.attach(R(0x142c7d6f0), { onEnter(a) { send({ev: 'qCueBeh_slot11_0x142c7d6f0', t: t(), obj: a[0], idx: a[2].toInt32(), r9: a[3].and(0xff)}); } });
Interceptor.attach(R(0x142ab0740), { onEnter(a) { send({ev: 'cueBeh_slot11_0x142ab0740', t: t(), obj: a[0], idx: a[2].toInt32(), r9: a[3].and(0xff)}); } });
Interceptor.attach(R(0x142c7d8cb), function () {
  const c = this.context;
  snap = {fn: c.rbx, obj: c.rdi, edx: c.rdx.and(0xffffffff).toInt32(), r8d: c.r8.and(0xffffffff).toInt32()};
  send({ev: 'snap_in_0x142c7d8cb', t: t(), fn_rva: c.rbx.sub(B).add(0x140000000), obj: c.rdi, edx: snap.edx, r8d: snap.r8d, xmm3: inDelta});
});
try {
  Interceptor.attach(R(0x142c7d8d0), function () { send({ev: 'snap_out_0x142c7d8d0', t: t(), eax: this.context.rax.and(0xffffffff).toInt32()}); });
} catch (e) { send({ev: 'hook_0x142c7d8d0_failed', err: '' + e}); }

rpc.exports = {
  findUnits() {
    const hits = [];
    for (const r of Process.enumerateRanges('rw-')) {
      try { for (const h of Memory.scanSync(r.base, r.size, pattern(VT))) if (h.address.and(7).isZero()) hits.push(h.address.toString()); } catch (e) {}
    }
    if (hits.length) unit = ptr(hits[0]);
    return hits;
  },
  useUnit(p) { unit = ptr(p); },
  slot(n) { return unit.readPointer().add(8 * n).readPointer().sub(B).add(0x140000000).toString(); },
  setQ(v) { new NativeFunction(unit.readPointer().add(8 * 9).readPointer(), 'void', ['pointer', 'int'])(unit, v); },
  pad(n) { new NativeFunction(unit.readPointer().add(8 * 71).readPointer(), 'void', ['pointer', 'int'])(unit, n); },
  hotCueRec(n) { new NativeFunction(R(0x142ab0e40), 'void', ['pointer', 'int', 'int'])(unit, n, 0); },
  // win64: rcx=obj, edx, r8d, xmm3=delta, [rsp+0x20]=0.0, [rsp+0x28]=0 (bytes iguais aos do chamador)
  snapAt(x) {
    if (!snap) return null;
    return new NativeFunction(snap.fn, 'int', ['pointer', 'int', 'int', 'double', 'double', 'uint8'])(snap.obj, snap.edx, snap.r8d, x, 0, 0);
  },
};
"""


def main():
    log = []
    session = frida.attach(int(sys.argv[1]))
    script = session.create_script(JS)
    script.on('message', lambda m, d: (log.append(m.get('payload', m)), print(json.dumps(m.get('payload', m)))))
    script.load()
    api = script.exports_sync
    units = api.find_units()
    print('units', units, 'slot9', units and api.slot(9), 'slot71', units and api.slot(71))
    if not units:
        return
    for q in (0, 1):
        print('--- Q =', q)
        api.set_q(q)
        time.sleep(1)
        api.pad(1)
        time.sleep(2)
        if not any(isinstance(e, dict) and 'slot11' in e.get('ev', '') for e in log[-6:]):
            api.hot_cue_rec(1)
            time.sleep(2)
    for k in range(4):
        for f in (0.2, 0.5, 0.8):
            x = k + f
            print(json.dumps({'ev': 'snap_direct', 'xmm3': x, 'eax': api.snap_at(x)}))
            time.sleep(0.2)
    script.unload()
    session.detach()


if __name__ == '__main__':
    main()
