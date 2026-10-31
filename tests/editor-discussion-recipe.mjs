// Named isolated fixture only. No production data or external services.
import assert from 'node:assert/strict';
import {readFileSync,writeFileSync} from 'node:fs';
const rt=JSON.parse(readFileSync('scratch/wasm-auth-recipe/runtime.json'));
assert.match(rt.database,/^rf_wasm_browser_[a-f0-9]{32}$/);assert.equal(rt.port,55499);assert.equal(rt.closed,false);
const base='http://127.0.0.1:55499';let checks=0;
async function api(path,token,method='GET',body,status=200){const r=await fetch(base+'/api/v1'+path,{method,headers:{'Content-Type':'application/json',...(token?{Authorization:'Bearer '+token}:{})},body:body===undefined?undefined:JSON.stringify(body)});const data=await r.json();assert.equal(r.status,status,`${method} ${path}: ${JSON.stringify(data)}`);checks++;return data;}
assert.equal((await api('/public/settings')).title,'Recette WASM isolee');
const a=(await api('/auth/login',null,'POST',{login:'wasm-admin',password:'Recipe-only-password-2026!'})).token;
const author=(await api('/auth/login',null,'POST',{login:'wasm-author',password:'Recipe-only-password-2026!'})).token;
const subscriber=(await api('/auth/login',null,'POST',{login:'wasm-subscriber',password:'Recipe-only-password-2026!'})).token;
const settings=await api('/settings',a);const keys=['discussion.default_comment_status','discussion.close_comments_for_old_posts','discussion.comment_registration','discussion.comment_moderation'];
let entry,url;
async function html(){const r=await fetch(url);assert.equal(r.status,200);checks++;return r.text();}
async function patch(status){entry=await api('/entries/'+entry.id,a,'PUT',{title:entry.title,fields:{_rf_comment_status:status},expected_updated_at:entry.updated_at});assert.deepEqual(entry.fields._legacy_vault,{keep:'unchanged'});return entry;}
const submission={author_name:'Visiteur recette',author_email:'visitor@example.invalid',content:'Commentaire de recette conservé',website_url_hp:''};
try{
 await api('/settings',a,'PUT',{'discussion.default_comment_status':true,'discussion.close_comments_for_old_posts':false,'discussion.comment_registration':false,'discussion.comment_moderation':false});
 entry=await api('/entries?content_type=page',a,'POST',{title:'Discussion par publication '+Date.now(),status:'published',fields:{_legacy_vault:{keep:'unchanged'}}});url=base+'/fr/page/'+entry.slug;
 assert.ok((await html()).includes('id="comment-form"'));
 const comment=await api('/public/entries/'+entry.id+'/comments',null,'POST',submission);
 await api('/comments/'+comment.id+'/status',a,'PUT',{status:'approved'});
 await patch('closed');let rendered=await html();assert.ok(rendered.includes('Les commentaires sont fermés'));assert.ok(!rendered.includes('id="comment-form"'));assert.ok(rendered.includes(submission.content));
 await api('/public/entries/'+entry.id+'/comments',null,'POST',submission,422);
 const revisions=await api('/entries/'+entry.id+'/revisions',a);const closedRevision=revisions.find(r=>r.snapshot?.fields?._rf_comment_status==='closed');assert.ok(closedRevision,'closed setting is revisioned');
 for(const token of [author,subscriber])await api('/entries/'+entry.id,token,'PUT',{title:entry.title,fields:{_rf_comment_status:'open'},expected_updated_at:entry.updated_at},403);
 await api('/entries/'+entry.id,a,'PUT',{title:entry.title,fields:{_rf_comment_status:true},expected_updated_at:entry.updated_at},422);
 await patch(null);assert.ok((await html()).includes('id="comment-form"'));
 await api('/settings',a,'PUT',{'discussion.default_comment_status':false});assert.ok(!(await html()).includes('id="comment-form"'));
 await patch('open');assert.ok((await html()).includes('id="comment-form"'));await api('/public/entries/'+entry.id+'/comments',null,'POST',submission);
 await api('/settings',a,'PUT',{'discussion.comment_registration':true});await api('/public/entries/'+entry.id+'/comments',null,'POST',submission,401);await api('/public/entries/'+entry.id+'/comments',author,'POST',{content:'Compte connecté'});
 await api('/settings',a,'PUT',{'discussion.comment_registration':false});
 const stale=entry.updated_at;entry=await api('/entries/'+entry.id,a,'PUT',{title:entry.title,excerpt:'Édition sans toucher à la discussion',expected_updated_at:entry.updated_at});assert.equal(entry.fields._rf_comment_status,'open');
 await api('/entries/'+entry.id,a,'PUT',{title:entry.title,fields:{_rf_comment_status:'closed'},expected_updated_at:stale},409);
 entry=await api('/entries/'+entry.id+'/revisions/'+closedRevision.id+'/restore',a,'POST',{expected_updated_at:entry.updated_at});assert.equal(entry.fields._rf_comment_status,'closed');assert.ok(!(await html()).includes('id="comment-form"'));
 const pub=await api('/public/entries/page/'+entry.slug);assert.ok(!('fields' in pub));
}finally{await api('/settings',a,'PUT',Object.fromEntries(keys.map(k=>[k,settings[k]??null])));}
writeFileSync('docs/validations/discussion-document-rust-2026-10-31/fixture.json',JSON.stringify({entry_id:entry.id,title:entry.title,public_url:url,checks},null,2)+'\n');console.log(`PASS ${checks} contrôles : héritage/ouverture/fermeture, SSR/cache, soumission, droits/connexion, version attendue, révision et conservation des champs.`);
