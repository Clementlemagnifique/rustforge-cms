// Disposable fake content only; the runtime and site identity are checked before writing.
import assert from 'node:assert/strict';
import {readFileSync,writeFileSync} from 'node:fs';
const state=JSON.parse(readFileSync('scratch/wasm-auth-recipe/runtime.json','utf8'));
assert.match(state.database,/^rf_wasm_browser_[a-f0-9]{32}$/);assert.equal(state.port,55499);assert.equal(state.closed,false);
const base='http://127.0.0.1:55499/api/v1';let checks=0;
async function request(path,token,method='GET',body,status=200){const r=await fetch(base+path,{method,headers:{'Content-Type':'application/json',...(token?{Authorization:`Bearer ${token}`}:{})},body:body===undefined?undefined:JSON.stringify(body)});const value=await r.json();assert.equal(r.status,status,`${method} ${path}: ${JSON.stringify(value)}`);checks++;return value;}
assert.equal((await request('/public/settings')).title,'Recette WASM isolee');
const tokens={};for(const role of ['admin','author','subscriber'])tokens[role]=(await request('/auth/login',null,'POST',{login:'wasm-'+role,password:'Recipe-only-password-2026!'})).token;
const original={title:'Recette révisions Rust — version A '+Date.now(),excerpt:'Extrait A',status:'draft',fields:{_legacy_vault:{opaque:'à conserver'}},blocks:{rf_schema:'rf.document/1',rf_blocks:[{rf_id:'revision-paragraph',rf_type:'paragraph',rf_text:'Paragraphe A **natif**'}]}};
const entry=await request('/entries?content_type=post',tokens.admin,'POST',original);
let revisions=await request(`/entries/${entry.id}/revisions`,tokens.admin);const first=revisions[0];assert.ok(first.snapshot);
const changed=await request('/entries/'+entry.id,tokens.admin,'PUT',{title:'Version B — concurrence',excerpt:'Extrait B',blocks:{rf_schema:'rf.document/1',rf_blocks:[{rf_id:'revision-paragraph',rf_type:'paragraph',rf_text:'Paragraphe B'}]},expected_updated_at:entry.updated_at});
const path=`/entries/${entry.id}/revisions/${first.id}/restore`;
await request(path,tokens.admin,'POST',{expected_updated_at:entry.updated_at},409);
assert.equal((await request('/entries/'+entry.id,tokens.admin)).title,changed.title);
await request(path,tokens.author,'POST',{expected_updated_at:changed.updated_at},403);
await request(path,tokens.subscriber,'POST',{expected_updated_at:changed.updated_at},403);
const restored=await request(path,tokens.admin,'POST',{expected_updated_at:changed.updated_at});assert.equal(restored.title,entry.title);assert.deepEqual(restored.blocks,entry.blocks);assert.equal(restored.excerpt,entry.excerpt);assert.deepEqual(restored.fields._legacy_vault,entry.fields._legacy_vault);assert.notEqual(restored.updated_at,changed.updated_at);
revisions=await request(`/entries/${entry.id}/revisions`,tokens.admin);assert.equal(revisions.length,3);assert.ok(revisions.some(row=>row.title===changed.title));
writeFileSync('docs/validations/revisions-rust-2026-10-31/fixture.json',JSON.stringify({entry_id:entry.id,revision_id:first.id,original_title:entry.title},null,2)+'\n');
console.log(`PASS ${checks} requêtes : restauration du document complet, historique conservé, conflit 409 sans écrasement et refus 403 inter-auteurs/abonné.`);
