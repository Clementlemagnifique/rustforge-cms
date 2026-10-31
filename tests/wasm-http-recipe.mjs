// Recette destructive uniquement pour la base fictive identifiée par runtime.json.
import {readFileSync,writeFileSync} from 'node:fs';
import assert from 'node:assert/strict';
import init,{inspect_document} from '../admin/public/wasm/rustforge_studio.js';
const runtime=JSON.parse(readFileSync('scratch/wasm-auth-recipe/runtime.json','utf8').replace(/^\uFEFF/,''));
assert.match(runtime.database,/^rf_wasm_browser_[a-f0-9]{32}$/);
assert.equal(runtime.port,55499);assert.equal(runtime.closed,false);
const base='http://127.0.0.1:55499/api/v1';
const results=[];
async function request(path,{method='GET',token,body,status=200}={}) {
  const response=await fetch(base+path,{method,headers:{'Content-Type':'application/json',...(token?{Authorization:`Bearer ${token}`}:{})},body:body===undefined?undefined:JSON.stringify(body)});
  assert.equal(response.status,status,`${method} ${path}`);
  const raw=await response.text();return raw?JSON.parse(raw):null;
}
const settings=await request('/public/settings');
assert.equal(settings.title,'Recette WASM isolee','Refus de modifier une autre installation');
const tokens={};
for(const role of ['admin','author','subscriber'])tokens[role]=(await request('/auth/login',{method:'POST',body:{login:`wasm-${role}`,password:'Recipe-only-password-2026!'}})).token;
await request('/admin/navigation',{status:401});
await request('/settings',{token:tokens.author,status:403});
await request('/settings',{token:tokens.subscriber,status:403});
await request('/admin/modules',{method:'PUT',token:tokens.author,body:{lms:true},status:403});
results.push('Droits : anonyme 401, réglages auteur/abonné et activation auteur 403');
const initial=await request('/admin/navigation',{token:tokens.admin});
try {
  const off=await request('/admin/modules',{method:'PUT',token:tokens.admin,body:{lms:false}});
  assert.ok(off.items.every(item=>item.module!=='lms'));
  await request('/lms/courses',{token:tokens.admin,status:404});
  const on=await request('/admin/modules',{method:'PUT',token:tokens.admin,body:{lms:true}});
  assert.ok(on.items.some(item=>item.module==='lms'));
  await request('/lms/courses',{token:tokens.admin});
  results.push('Module LMS : masqué et 404 quand Off, menu et endpoint disponibles quand On');
} finally {await request('/admin/modules',{method:'PUT',token:tokens.admin,body:{lms:initial.modules.lms===true}});}
await init({module_or_path:readFileSync('admin/public/wasm/rustforge_studio_bg.wasm')});
const blocks={rf_schema:'rf.document/1',rf_blocks:[{rf_id:'first',rf_type:'paragraph',rf_text:'Recette **Rust** 🌍'},{rf_id:'second',rf_type:'heading',rf_level:2,rf_text:'Voisin conservé'}]};
const native=JSON.parse(inspect_document('',JSON.stringify(blocks)));assert.equal(native.ok,true);
const body={title:'Recette WASM HTTP '+Date.now(),body:'',blocks,status:'draft'};
await request('/entries?content_type=post',{method:'POST',token:tokens.subscriber,body,status:403});
const entry=await request('/entries?content_type=post',{method:'POST',token:tokens.author,body});
assert.deepEqual(entry.blocks,blocks);assert.equal(entry.body_html,native.data.html);assert.equal(entry.body,native.data.body);
await request(`/entries/${entry.id}`,{method:'PUT',token:tokens.subscriber,body:{title:'Interdit'},status:403});
const update=await request(`/entries/${entry.id}`,{method:'PUT',token:tokens.author,body:{title:body.title+' modifié',expected_updated_at:entry.updated_at}});
assert.deepEqual(update.blocks,blocks);
await request(`/entries/${entry.id}`,{method:'PUT',token:tokens.author,body:{title:'Version périmée',expected_updated_at:entry.updated_at},status:409});
const invalid={rf_schema:'rf.document/1',rf_blocks:[{rf_id:'bad',rf_type:'button',rf_text:'Interdit',rf_url:'javascript:alert(1)'}]};
assert.equal(JSON.parse(inspect_document('',JSON.stringify(invalid))).ok,false);
await request(`/entries/${entry.id}`,{method:'PUT',token:tokens.author,body:{blocks:invalid,expected_updated_at:update.updated_at},status:422});
const final=await request(`/entries/${entry.id}`,{token:tokens.author});assert.deepEqual(final.blocks,blocks);assert.equal(final.title,update.title);
const revisions=await request(`/entries/${entry.id}/revisions`,{token:tokens.author});assert.ok(revisions.length>=2);
results.push('Sauvegarde auteur : rendu WASM/serveur identique, blocs voisins conservés');
results.push('Conflit 409 et document invalide 422 sans écrasement ; révisions présentes');
const adminEntry=await request('/entries?content_type=post',{method:'POST',token:tokens.admin,body:{...body,title:body.title+' admin'}});
await request(`/entries/${adminEntry.id}`,{method:'PUT',token:tokens.author,body:{title:'Accès interdit'},status:403});
results.push('Auteur refusé sur une entrée appartenant à un autre compte');
writeFileSync('scratch/wasm-auth-recipe/http-results.json',JSON.stringify({results,entry_ids:[entry.id,adminEntry.id],browser_tested:false},null,2)+'\n');
console.log(results.join('\n'));
