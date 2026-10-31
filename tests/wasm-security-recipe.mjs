// Read-only regression recipe: verify the deployed Studio bootstrap and CSP.
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
const runtime=JSON.parse(readFileSync('scratch/wasm-auth-recipe/runtime.json','utf8').replace(/^\uFEFF/,''));
assert.match(runtime.database,/^rf_wasm_browser_[a-f0-9]{32}$/);
assert.equal(runtime.port,55499); assert.equal(runtime.closed,false);
const base='http://127.0.0.1:55499';
const settings=await (await fetch(base+'/api/v1/public/settings')).json();
assert.equal(settings.title,'Recette WASM isolee');
for (const path of ['/','/api/v1/public/settings','/admin/','/admin/rust/']) {
  const response=await fetch(base+path); assert.equal(response.status,200,path);
  const csp=response.headers.get('content-security-policy');
  const script=csp.split(';').find(d=>d.trim().startsWith('script-src '));
  assert.ok(script.includes("'self'"));
  assert.ok(!script.includes("'unsafe-inline'"));
  assert.ok(!script.includes("'unsafe-eval'"));
  assert.equal(script.includes("'wasm-unsafe-eval'"),path.startsWith('/admin/'),path);
  if(path==='/admin/rust/') {
    const html=await response.text();
    const scripts=[...html.matchAll(/<script\b([^>]*)>([\s\S]*?)<\/script>/g)];
    assert.equal(scripts.length,1); assert.equal(scripts[0][2].trim(),'');
    const src=scripts[0][1].match(/src="([^" ]+)"/)[1];
    assert.match(src,/^\/admin\/rust\/bootstrap-[a-f0-9]{16}\.js$/);
    const bootstrapResponse=await fetch(base+src); assert.equal(bootstrapResponse.status,200);
    assert.match(bootstrapResponse.headers.get('content-type'),/javascript/);
    const bootstrap=await bootstrapResponse.text();
    const wasmPath=bootstrap.match(/module_or_path: '([^']+\.wasm)'/)[1];
    assert.match(wasmPath,/^\/admin\/rust\/[^/]+\.wasm$/);
    const wasm=await fetch(base+wasmPath); assert.equal(wasm.status,200);
    assert.match(wasm.headers.get('content-type'),/application\/wasm/);
    assert.deepEqual([...new Uint8Array(await wasm.arrayBuffer()).slice(0,4)],[0,97,115,109]);
  }
}
console.log('PASS: deployed bootstrap external, WASM MIME/magic valid; WASM allowed only in admin, inline JS/eval blocked.');
