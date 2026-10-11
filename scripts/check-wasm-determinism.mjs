// So hash từng tick của fixture chạy trong WASM với file hash native; ghi report.json.
// Dùng qua scripts/check-wasm-determinism.sh.
import { createRequire } from 'node:module';
import { readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

const [out, ticksArg, ...seeds] = process.argv.slice(2);
const ticks = Number(ticksArg);
const require = createRequire(import.meta.url);
const wasm = require(join(out, 'pkg', 'wasm_fingerprint.js'));
const report = { date: new Date().toISOString(), node: process.version, ticks, seeds: [] };
let failed = false;
for (const seed of seeds) {
  const native = readFileSync(join(out, `native-${seed}.txt`), 'utf8').trim().split('\n');
  const web = Array.from(wasm.fingerprint(BigInt(seed), ticks), (h) => h.toString(16).padStart(16, '0'));
  const machNative = readFileSync(join(out, `mach-${seed}.txt`), 'utf8').trim().split('\n');
  const machWeb = Array.from(wasm.mach_fingerprint(BigInt(seed), ticks), (h) => h.toString(16).padStart(16, '0'));
  const machFirst = machNative.findIndex((hash, tick) => hash !== machWeb[tick]);
  const machSame = machNative.length === ticks && machWeb.length === ticks && machFirst === -1;
  const first = native.findIndex((hash, tick) => hash !== web[tick]);
  const same = native.length === ticks && web.length === ticks && first === -1;
  report.seeds.push({ seed: Number(seed), ticks: web.length, identical: same, firstMismatch: same ? null : first, final: web.at(-1), mach: { identical: machSame, firstMismatch: machSame ? null : machFirst, final: machWeb.at(-1) } });
  console.log(`seed ${seed}: ${same ? 'khớp' : `LỆCH ở tick ${first}`} ${web.length} hash; cuối ${web.at(-1)}`);
  console.log(`Mạch seed ${seed}: ${machSame ? 'khớp' : `LỆCH ở tick ${machFirst}`} ${machWeb.length} hash; cuối ${machWeb.at(-1)}`);
  failed ||= !same || !machSame;
}
writeFileSync(join(out, 'report.json'), `${JSON.stringify(report, null, 2)}\n`);
process.exit(failed ? 1 : 0);
