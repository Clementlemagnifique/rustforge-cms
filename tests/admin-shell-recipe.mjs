// Lecture seule de l'instance fictive : contrat de menu et styles réellement servis.
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
const runtime=JSON.parse(readFileSync('scratch/wasm-auth-recipe/runtime.json','utf8').replace(/^\uFEFF/,''));
assert.match(runtime.database,/^rf_wasm_browser_[a-f0-9]{32}$/);
assert.equal(runtime.port,55499); assert.equal(runtime.closed,false);
const base='http://127.0.0.1:55499';
assert.equal((await(await fetch(base+'/api/v1/public/settings')).json()).title,'Recette WASM isolee');
const login=await fetch(base+'/api/v1/auth/login',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({login:'wasm-admin',password:'Recipe-only-password-2026!'})});
assert.equal(login.status,200); const {token}=await login.json();
const nav=await(await fetch(base+'/api/v1/admin/navigation',{headers:{Authorization:`Bearer ${token}`}})).json();
assert.equal(nav.menu_items.length,JSON.parse(readFileSync('crates/rustforge-api/src/admin_navigation.json','utf8')).length);
assert.deepEqual(nav.menu_items.filter(n=>n.separator_before).map(n=>n.id),['rf_posts','rf_ai','rf_appearance','rf_users','rf_editorial']);
assert.ok(nav.menu_items.filter(n=>n.separator_before).every(n=>!n.parent));
assert.ok(nav.menu_items.some(n=>n.planning&&n.features.length>0));
for(const path of ['/admin/','/admin/rust/']) {
 const response=await fetch(base+path); assert.equal(response.status,200);
 const html=await response.text();
 const links=[...html.matchAll(/href="([^" ]+\.css)"/g)].map(m=>m[1]);
 assert.ok(links.length>0);
 const styles=await Promise.all(links.map(async href=>{
  const css=await fetch(new URL(href,base+path));assert.equal(css.status,200);return css.text();
 }));
 assert.ok(styles.some(css=>/--rf-menu-width\s*:\s*160px/.test(css)),path+' géométrie partagée');
 assert.ok(styles.some(css=>css.includes('separator-before')),path+' séparateurs');
 assert.ok(styles.some(css=>css.includes('--rf-panel')),path+' palette partagée');
}
console.log('PASS: menu complet, cinq séparateurs, fiches prévues et socle CSS partagé servis sur les deux administrations.');
