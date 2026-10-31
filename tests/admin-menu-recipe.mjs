// Read-only checks of the real navigation projection on the identified test installation.
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
const runtime=JSON.parse(readFileSync('scratch/wasm-auth-recipe/runtime.json','utf8').replace(/^\uFEFF/,''));
assert.match(runtime.database,/^rf_wasm_browser_[a-f0-9]{32}$/);assert.equal(runtime.port,55499);assert.equal(runtime.closed,false);
const base='http://127.0.0.1:55499/api/v1';
assert.equal((await(await fetch(base+'/public/settings')).json()).title,'Recette WASM isolee');
assert.equal((await fetch(base+'/admin/navigation')).status,401);
for(const role of ['admin','author','subscriber']) {
 const login=await fetch(base+'/auth/login',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({login:`wasm-${role}`,password:'Recipe-only-password-2026!'})});
 assert.equal(login.status,200);const {token}=await login.json();
 const response=await fetch(base+'/admin/navigation',{headers:{Authorization:`Bearer ${token}`}});assert.equal(response.status,200);
 const nav=await response.json();
 if(role==='admin') {
  assert.equal(nav.menu_items.length,JSON.parse(readFileSync('crates/rustforge-api/src/admin_navigation.json','utf8')).length);
  assert.deepEqual(nav.menu_items.filter(n=>!n.parent).slice(0,8).map(n=>n.id),['rf_dashboard','rf_posts','rf_pages','rf_forms','rf_events','rf_media','rf_galleries','rf_comments']);
  for(const id of ['rf_media_drive','rf_media_onedrive','rf_media_dropbox','rf_gallery_models','rf_comment_notifications','rf_ai_knowledge','rf_lms_cohorts','rf_commerce_downloads']) {
   const item=nav.menu_items.find(n=>n.id===id);assert.ok(item.planning);assert.equal(item.route,`plan:${id}`);
  }
  assert.ok(nav.items.some(n=>n.id==='rf_translations'));
  for(const item of nav.menu_items) {
   if(item.module&&nav.modules[item.module]!==true) assert.ok(item.planning);
   if(item.planning) assert.match(item.route,/^plan:rf_[a-z0-9_]+$/);
   if(item.parent) assert.ok(nav.menu_items.some(n=>n.id===item.parent));
  }
 } else {
  assert.deepEqual(nav.menu_items,nav.items);assert.equal(nav.catalog.length,0);
  assert.ok(nav.menu_items.every(n=>!n.planning&&!n.route?.startsWith('plan:')));
 }
 console.log(`PASS ${role}: authorized navigation and planning separation`);
}
