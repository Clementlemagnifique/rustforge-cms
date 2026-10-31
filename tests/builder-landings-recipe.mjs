// Named isolated recipe. Does not modify the content open in the user's browser.
import assert from 'node:assert/strict';
import {readFileSync,writeFileSync} from 'node:fs';
const runtime=JSON.parse(readFileSync('scratch/wasm-auth-recipe/runtime.json'));assert.equal(runtime.port,55499);assert.equal(runtime.closed,false);assert.match(runtime.database,/^rf_wasm_browser_[a-f0-9]{32}$/);
const base='http://127.0.0.1:55499';let checks=0;
async function api(path,token,method='GET',body,status=200){const r=await fetch(base+'/api/v1'+path,{method,headers:{'Content-Type':'application/json',...(token?{Authorization:'Bearer '+token}:{})},body:body===undefined?undefined:JSON.stringify(body)});const data=await r.json();assert.equal(r.status,status,`${method} ${path}: ${JSON.stringify(data)}`);checks++;return data;}
assert.equal((await api('/public/settings')).title,'Recette WASM isolee');
const admin=(await api('/auth/login',null,'POST',{login:'wasm-admin',password:'Recipe-only-password-2026!'})).token;
const author=(await api('/auth/login',null,'POST',{login:'wasm-author',password:'Recipe-only-password-2026!'})).token;
const saved=[];const stamp=Date.now();
async function html(url){const r=await fetch(url);assert.equal(r.status,200);checks++;return r.text();}
for(const kind of ['post','page']){
 const blocks={rf_schema:'rf.document/1',rf_blocks:[{rf_id:'preserved',rf_type:'paragraph',rf_text:'Contenu conservé — offre découverte'},{rf_id:'features',rf_type:'columns',rf_items:['### Premier bénéfice\nExemple concret','### Deuxième bénéfice\nAutre exemple']},{rf_id:'cta',rf_type:'button',rf_text:'Nous contacter',rf_url:'mailto:contact@example.invalid'}]};
 let entry=await api('/entries?content_type='+kind,admin,'POST',{title:'Landing Rust '+kind+' '+stamp,status:'published',blocks,fields:{_legacy_vault:{keep:'opaque'},_rf_template:'landing',_rf_comment_status:'closed'}});
 const url=base+'/fr/'+kind+'/'+entry.slug;
 let rendered=await html(url);assert.ok(rendered.includes('rf-entry-layout-landing'));assert.ok(!rendered.includes('<header class="glass-nav">'));assert.ok(!rendered.includes('id="comments"'));assert.ok(rendered.includes('Contenu conservé'));assert.ok(rendered.includes('Nous contacter'));assert.ok(rendered.includes('class="rf-columns"'));assert.ok(rendered.includes('class="rf-btn rf-btn-primary"'));assert.ok(rendered.includes('<h1'));
 await api('/entries/'+entry.id,author,'PUT',{title:entry.title,fields:{_rf_template:'wide'},expected_updated_at:entry.updated_at},403);
 await api('/entries/'+entry.id,admin,'PUT',{title:entry.title,fields:{_rf_template:'../../secret'},expected_updated_at:entry.updated_at},422);
 const revisionList=await api('/entries/'+entry.id+'/revisions',admin);const landingRevision=revisionList.find(r=>r.snapshot?.fields?._rf_template==='landing');assert.ok(landingRevision);
 entry=await api('/entries/'+entry.id,admin,'PUT',{title:entry.title,fields:{_rf_template:'wide'},expected_updated_at:entry.updated_at});assert.deepEqual(entry.blocks,blocks);assert.deepEqual(entry.fields._legacy_vault,{keep:'opaque'});assert.equal(entry.fields._rf_comment_status,'closed');
 rendered=await html(url);assert.ok(rendered.includes('rf-entry-layout-wide'));assert.ok(rendered.includes('<header class="glass-nav">'));
 entry=await api('/entries/'+entry.id+'/revisions/'+landingRevision.id+'/restore',admin,'POST',{expected_updated_at:entry.updated_at});assert.equal(entry.fields._rf_template,'landing');assert.ok(!(await html(url)).includes('<header class="glass-nav">'));
 const pub=await api('/public/entries/'+kind+'/'+entry.slug);assert.equal(pub.presentation_template,'landing');assert.ok(!('fields' in pub));
 entry=await api('/entries/'+entry.id,admin,'PUT',{title:entry.title,fields:{_rf_template:null},expected_updated_at:entry.updated_at});assert.ok((await html(url)).includes('rf-entry-layout-default'));assert.deepEqual(entry.fields._legacy_vault,{keep:'opaque'});
 entry=await api('/entries/'+entry.id,admin,'PUT',{title:entry.title,fields:{_rf_template:'landing'},expected_updated_at:entry.updated_at});saved.push({id:entry.id,kind,url,title:entry.title});
}
// A landing can also be the static front page. Restore reading settings in all cases.
const settings=await api('/settings',admin);const keys=['reading.show_on_front','reading.page_on_front'];
try{await api('/settings',admin,'PUT',{'reading.show_on_front':'page','reading.page_on_front':saved.find(e=>e.kind==='page').id});const front=await html(base+'/fr');assert.ok(front.includes('rf-entry-layout-landing'));assert.ok(!front.includes('<header class="glass-nav">'));}finally{await api('/settings',admin,'PUT',Object.fromEntries(keys.map(k=>[k,settings[k]??null])));}
writeFileSync('docs/validations/constructeur-landings-rust-2026-10-31/fixture.json',JSON.stringify({entries:saved,checks},null,2)+'\n');console.log(`PASS ${checks} contrôles : articles/pages mutualisés, landing/largeur/défaut, rendu et accueil, droits, validation, révisions et conservation.`);
