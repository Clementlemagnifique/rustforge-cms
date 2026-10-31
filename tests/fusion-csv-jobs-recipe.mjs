// Guarded isolated fixture only. Fake records are retained for visible UI verification.
import assert from 'node:assert/strict';
import {readFileSync,writeFileSync} from 'node:fs';
import {execFileSync} from 'node:child_process';
import {randomUUID} from 'node:crypto';
const rt=JSON.parse(readFileSync('scratch/wasm-auth-recipe/runtime.json'));
assert.match(rt.database,/^rf_wasm_browser_[a-f0-9]{32}$/);assert.equal(rt.port,55499);assert.equal(rt.closed,false);
const base='http://127.0.0.1:55499';let checks=0;
async function api(path,token,method='GET',body,status=200){const r=await fetch(base+'/api/v1'+path,{method,headers:{'Content-Type':'application/json',...(token?{Authorization:`Bearer ${token}`}:{})},body:body===undefined?undefined:JSON.stringify(body)});const data=await r.json();assert.equal(r.status,status,`${method} ${path}: ${JSON.stringify(data)}`);checks++;return data;}
assert.equal((await api('/public/settings')).title,'Recette WASM isolee');
const a=(await api('/auth/login',null,'POST',{login:'wasm-admin',password:'Recipe-only-password-2026!'})).token;
const s=(await api('/auth/login',null,'POST',{login:'wasm-subscriber',password:'Recipe-only-password-2026!'})).token;
const suffix=Date.now();const term=async(name,taxonomy='category',parent_id=null)=>api('/terms',a,'POST',{name:`${name} ${suffix}`,taxonomy,parent_id});
const source=await term('Fusion recette source'),target=await term('Fusion recette cible'),child=await term('Fusion recette enfant','category',source.id),tag=await term('Fusion recette tag','tag');
const entry=await api('/entries?content_type=post',a,'POST',{title:'Recette fusion '+suffix,term_ids:[source.id,target.id]});
await api('/terms/'+source.id+'/merge',a,'POST',{target_id:source.id},422);
await api('/terms/'+source.id+'/merge',a,'POST',{target_id:tag.id},422);
await api('/terms/'+source.id+'/merge',a,'POST',{target_id:child.id},422);
await api('/terms/'+source.id+'/merge',s,'POST',{target_id:target.id},403);
await api('/terms/'+source.id+'/merge',a,'POST',{target_id:target.id});
const terms=await api('/terms',a);assert.ok(!terms.some(t=>t.id===source.id));assert.equal(terms.find(t=>t.id===child.id).parent_id,target.id);
const merged=await api('/entries/'+entry.id,a);assert.deepEqual(merged.terms.map(t=>t.id),[target.id]);
const uiSource=await term('Fusion visuelle source'),uiTarget=await term('Fusion visuelle cible');
await api('/admin/modules',a,'PUT',{forms:true});
const form=await api('/forms',a,'POST',{name:'CSV recette 101 soumissions '+suffix,is_active:true,fields:{type:'object',properties:{nom:{type:'string'}},additionalProperties:false}});
const job=randomUUID(),uiJob=randomUUID();
// All interpolated database identifiers/UUIDs are validated; values below are fixed fake data.
for(const id of [form.id,job,uiJob])assert.match(id,/^[a-f0-9-]{36}$/);
execFileSync('psql',[`postgres://rustforge_test@127.0.0.1:55496/${rt.database}`,'-v','ON_ERROR_STOP=1','-c',`
INSERT INTO form_submissions(id,form_id,data,created_at,ip_hash) SELECT gen_random_uuid(),'${form.id}',jsonb_build_object('nom','Élodie; "Bonjour"','formule','=1+1','index',n,'multiligne',E'Un\\nDeux'),now(),'recipe-fake-network-hash' FROM generate_series(1,101) n;
INSERT INTO rf_jobs(id,kind,payload_version,payload,dedup_key,state) VALUES ('${job}','recipe.restore',1,'{}','${job}','paused_restore'),('${uiJob}','recipe.restore',1,'{}','${uiJob}','paused_restore');`],{stdio:'pipe'});
const p1=await api('/forms/'+form.id+'/submissions?page=1&per_page=100',a),p2=await api('/forms/'+form.id+'/submissions?page=2&per_page=100',a);assert.equal(p1.items.length,100);assert.equal(p2.items.length,1);assert.equal(p1.total,101);assert.equal(new Set([...p1.items,...p2.items].map(r=>r.id)).size,101);
await api('/forms/'+form.id+'/submissions',s,'GET',undefined,403);
let j=(await api('/jobs?state=paused_restore',a)).items.find(j=>j.id===job);assert.ok(j);
await api('/jobs/'+job+'/resume-restored',s,'POST',{expected_updated_at:j.updated_at},403);
await api('/jobs/'+job+'/resume-restored',a,'POST',{expected_updated_at:'2000-01-01T00:00:00Z'},409);
await api('/jobs/'+job+'/resume-restored',a,'POST',{expected_updated_at:j.updated_at});
j=(await api('/jobs?state=queued',a)).items.find(j=>j.id===job);assert.ok(j);assert.equal(j.attempts,0);
await api('/jobs/'+job+'/resume-restored',a,'POST',{expected_updated_at:j.updated_at},409);
writeFileSync('docs/validations/fusion-csv-reprise-2026-10-31/fixture.json',JSON.stringify({ui_source:uiSource,ui_target:uiTarget,form_id:form.id,ui_job:uiJob,checks},null,2)+'\n');
console.log(`PASS ${checks} contrôles HTTP : fusion, enfants/dédoublonnage, refus cible/droits, 101 soumissions paginées, reprise versionnée et double action refusée.`);
