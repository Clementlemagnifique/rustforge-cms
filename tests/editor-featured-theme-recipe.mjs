// Isolated named fixture only. Does not alter AI configuration or QA policy.
import assert from 'node:assert/strict';import {readFileSync,writeFileSync} from 'node:fs';
const rt=JSON.parse(readFileSync('scratch/wasm-auth-recipe/runtime.json'));assert.match(rt.database,/^rf_wasm_browser_[a-f0-9]{32}$/);assert.equal(rt.port,55499);assert.equal(rt.closed,false);
const base='http://127.0.0.1:55499';let checks=0;
async function api(path,token,method='GET',body,status=200){const r=await fetch(base+'/api/v1'+path,{method,headers:{'Content-Type':'application/json',...(token?{Authorization:`Bearer ${token}`}:{})},body:body===undefined?undefined:JSON.stringify(body)});const raw=await r.text();let data;try{data=JSON.parse(raw)}catch{data=raw}assert.equal(r.status,status,`${method} ${path}: ${JSON.stringify(data)}`);checks++;return data;}
assert.equal((await api('/public/settings')).title,'Recette WASM isolee');
const tokens={};for(const role of ['admin','author','subscriber'])tokens[role]=(await api('/auth/login',null,'POST',{login:'wasm-'+role,password:'Recipe-only-password-2026!'})).token;
const a=tokens.admin,suffix=Date.now();
async function upload(name,mime,body){const form=new FormData();form.append('file',new Blob([body],{type:mime}),name);const r=await fetch(base+'/api/v1/media',{method:'POST',headers:{Authorization:'Bearer '+a},body:form});assert.equal(r.status,200,await r.clone().text());checks++;return r.json();}
let image=await upload('presentation-rust-'+suffix+'.svg','image/svg+xml','<svg xmlns="http://www.w3.org/2000/svg" width="960" height="360" viewBox="0 0 960 360"><rect width="960" height="360" fill="#135e96"/><text x="60" y="195" fill="white" font-size="50">Présentation Rust</text></svg>');
const asset=await fetch(base+image.url);assert.equal(asset.status,200);assert.match(asset.headers.get('content-type'),/image\/svg\+xml/);const svg=await asset.text();for(const expected of ['xmlns="http://www.w3.org/2000/svg"','width="960"','viewBox="0 0 960 360"','fill="#135e96"'])assert.ok(svg.includes(expected),expected);checks++;
image=await api('/media/'+image.id,a,'PUT',{alt:'Bannière « été » & Rust',caption:'Légende <script>texte</script>'});
const fields={_legacy_vault:{keep:'unchanged'}};
let page=await api('/entries?content_type=page',a,'POST',{title:'Présentation publique Rust '+suffix,status:'published',featured_media_id:image.id,fields,blocks:{rf_schema:'rf.document/1',rf_blocks:[{rf_id:'intro',rf_type:'paragraph',rf_text:'Contenu publié avec image mise en avant.'}]}});
assert.equal(page.featured_media_id,image.id);assert.equal(page.featured_media_alt,image.alt);
await api('/entries?content_type=page',tokens.subscriber,'POST',{title:'Interdit',featured_media_id:image.id},403);
await api('/entries/'+page.id,tokens.author,'PUT',{title:'Interdit',featured_media_id:null,expected_updated_at:page.updated_at},403);
await api('/entries?content_type=page',a,'POST',{title:'Image absente',featured_media_id:'00000000-0000-0000-0000-000000000000'},422);
const text=await upload('document-'+suffix+'.txt','text/plain','Un document texte de recette, pas une image.');
await api('/entries?content_type=page',a,'POST',{title:'Mauvais média',featured_media_id:text.id},422);
const publicUrl=base+'/fr/page/'+page.slug;
async function html(url){const r=await fetch(url);assert.equal(r.status,200);checks++;return (await r.text()).replace(/&#x2f;/g,'/');}
let rendered=await html(publicUrl);assert.ok(rendered.includes('class="rf-featured-image"'));assert.ok(rendered.includes(image.url));assert.ok(rendered.includes('Légende &lt;script&gt;texte&lt;/script&gt;'));assert.ok(!rendered.includes('<script>texte</script>'));
assert.ok((await html(base+'/fr/search?q='+encodeURIComponent(page.title))).includes(image.url));
const publicJson=await api('/public/entries/page/'+page.slug);assert.equal(publicJson.featured_media_alt,image.alt);assert.ok(!('fields' in publicJson));
await api('/media/'+image.id,a,'PUT',{alt:'Texte alternatif actualisé',caption:'Légende actualisée'});
rendered=await html(publicUrl);assert.ok(rendered.includes('Texte alternatif actualisé'));assert.ok(rendered.includes('Légende actualisée'));assert.ok(!rendered.includes('Légende &lt;script&gt;'));
const parent=await api('/terms',a,'POST',{taxonomy:'category',name:'Parent inspecteur '+suffix});
const term=await api('/terms',tokens.author,'POST',{taxonomy:'category',name:'Terme rapide '+suffix,parent_id:parent.id});
page=await api('/entries/'+page.id,a,'PUT',{title:page.title,excerpt:'Extrait conservé',term_ids:[term.id],expected_updated_at:page.updated_at});assert.equal(page.featured_media_id,image.id);assert.deepEqual(page.fields._legacy_vault,fields._legacy_vault);
page=await api('/entries/'+page.id,a,'PUT',{title:page.title,featured_media_id:null,expected_updated_at:page.updated_at});assert.equal(page.featured_media_id,null);assert.equal(page.terms[0].id,term.id);assert.deepEqual(page.fields._legacy_vault,fields._legacy_vault);
assert.ok(!(await html(publicUrl)).includes('class="rf-featured-image"'));
page=await api('/entries/'+page.id,a,'PUT',{title:page.title,featured_media_id:image.id,expected_updated_at:page.updated_at});assert.ok((await html(publicUrl)).includes(image.url));
const original=await api('/settings',a);const keys=['reading.show_on_front','reading.page_on_front'];
try{await api('/settings',a,'PUT',{'reading.show_on_front':'page','reading.page_on_front':page.id});assert.ok((await html(base+'/fr')).includes(image.url));}finally{await api('/settings',a,'PUT',Object.fromEntries(keys.map(k=>[k,original[k]??null])));}
writeFileSync('docs/validations/editeur-theme-images-termes-2026-10-31/fixture.json',JSON.stringify({page_id:page.id,page_title:page.title,public_url:publicUrl,media_id:image.id,media_name:image.original_name,term_id:term.id,parent_id:parent.id,checks},null,2)+'\n');
console.log(`PASS ${checks} contrôles : média réel, validation/droits, alt/légende échappés, recherche et accueil statique, invalidation cache, conservation/effacement, classement et métadonnées historiques.`);
