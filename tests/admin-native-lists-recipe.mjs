// Contrats en lecture seule sur la fixture isolée ; aucune entrée métier créée.
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
const runtime=JSON.parse(readFileSync('scratch/wasm-auth-recipe/runtime.json','utf8').replace(/^\uFEFF/,''));
assert.match(runtime.database,/^rf_wasm_browser_[a-f0-9]{32}$/);assert.equal(runtime.port,55499);assert.equal(runtime.closed,false);
const base='http://127.0.0.1:55499';
assert.equal((await(await fetch(base+'/api/v1/public/settings')).json()).title,'Recette WASM isolee');
for (const role of ['admin','author','subscriber']) {
 const login=await fetch(base+'/api/v1/auth/login',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({login:'wasm-'+role,password:'Recipe-only-password-2026!'})});
 assert.equal(login.status,200);const {token}=await login.json();
 const headers={Authorization:`Bearer ${token}`};
 const me=await fetch(base+'/api/v1/auth/me',{headers});assert.equal(me.status,200);const user=await me.json();assert.ok(user.id);
 for(const [path,allowed] of [['entries?content_type=post',role!=='subscriber'],['entries?content_type=page',role!=='subscriber'],['media?',true],['users?',role==='admin']]) {
  const response=await fetch(base+'/api/v1/'+path+'&page=1&per_page=20',{headers});
  assert.equal(response.status,allowed?200:403,role+' '+path);
  if(!allowed)continue;
  const data=await response.json();assert.ok(Array.isArray(data.items));assert.equal(data.page,1);assert.ok(data.items.length<=20);
  if(path.startsWith('entries')) {
   const type=path.includes('=post')?'post':'page';assert.ok(data.items.every(row=>row.content_type===type));
   if(role==='author')assert.ok(data.items.every(row=>row.author_id===user.id));
  }
 }
 if(role==='admin') {
  const first=await(await fetch(base+'/api/v1/users?page=1&per_page=1',{headers})).json();
  const second=await(await fetch(base+'/api/v1/users?page=2&per_page=1',{headers})).json();
  assert.equal(first.page,1);assert.equal(second.page,2);assert.equal(first.total,second.total);
  assert.ok(first.total>=3);assert.notEqual(first.items[0].id,second.items[0].id);
 }
 const empty=await fetch(base+'/api/v1/media?q=rf_nonexistent_1793399999999&page=1&per_page=20',{headers});
 assert.equal(empty.status,200);assert.equal((await empty.json()).items.length,0);
 console.log('PASS '+role+': données paginées, droits serveur et liste vide');
}
