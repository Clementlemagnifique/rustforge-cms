// Recette exclusivement sur la fixture identifiée ; conserve les contenus fictifs.
import assert from 'node:assert/strict';
import {readFileSync, writeFileSync} from 'node:fs';
const state = JSON.parse(readFileSync('scratch/wasm-auth-recipe/runtime.json', 'utf8').replace(/^\uFEFF/, ''));
assert.match(state.database, /^rf_wasm_browser_[a-f0-9]{32}$/);
assert.equal(state.port, 55499); assert.equal(state.closed, false);
const base = 'http://127.0.0.1:55499/api/v1';
async function request(path, {token, body, method = 'GET', status = 200} = {}) {
  const response = await fetch(base + path, {method, headers: {'Content-Type':'application/json', ...(token ? {Authorization: `Bearer ${token}`} : {})}, body:body === undefined ? undefined : JSON.stringify(body)});
  assert.equal(response.status, status, `${method} ${path}`);
  return response.json();
}
assert.equal((await request('/public/settings')).title, 'Recette WASM isolee');
const tokens = {};
for (const role of ['admin', 'author', 'subscriber']) {
  tokens[role] = (await request('/auth/login', {method:'POST', body:{login:`wasm-${role}`, password:'Recipe-only-password-2026!'}})).token;
}
await request('/stats/dashboard', {status:401});
await request('/stats/dashboard', {token:tokens.subscriber, status:403});
await request('/entries?content_type=post', {token:tokens.subscriber, method:'POST', status:403, body:{title:'Interdit', status:'draft'}});
const baseline = await request('/stats/dashboard', {token:tokens.author});
const before = baseline.content_types.find(t => t.slug === 'post');
const prefix = `Recette tableau Rust ${Date.now()}`;
const records = {};
for (const [label, status, published_at] of [
  ['draft', 'draft', null],
  ['published', 'published', new Date(Date.now() - 86400000).toISOString()],
  ['scheduled', 'published', new Date(Date.now() + 86400000).toISOString()],
  ['archived', 'archived', null],
  ['trash', 'trash', null],
]) {
  records[label] = await request('/entries?content_type=post', {token:tokens.author, method:'POST', body:{title:`${prefix} ${label}`, status, published_at,
    blocks:{rf_schema:'rf.document/1', rf_blocks:[{rf_id:'quick-draft-text', rf_type:'paragraph', rf_text:'Texte **Rust** 🌍'}]}}});
}
const other = await request('/entries?content_type=post', {token:tokens.admin, method:'POST', body:{title:`${prefix} autre auteur`, status:'draft'}});
const author = await request('/stats/dashboard', {token:tokens.author});
const count = author.content_types.find(t => t.slug === 'post');
assert.equal(count.published, before.published + 1);
assert.equal(count.drafts, before.drafts + 1);
assert.equal(count.scheduled, before.scheduled + 1);
assert.ok(author.server_version.length > 0);
assert.ok(!author.activity.some(e => e.id === records.trash.id || e.id === other.id));
for (const label of ['draft','published','scheduled','archived']) {
  assert.equal(author.activity.find(e => e.id === records[label].id)?.state, label);
}
assert.ok(author.recent.every(e => e.author_id === records.draft.author_id));
const admin = await request('/stats/dashboard', {token:tokens.admin});
assert.ok(admin.activity.some(e => e.id === other.id));
const reloaded = await request(`/entries/${records.draft.id}`, {token:tokens.author});
assert.equal(reloaded.status, 'draft');
assert.deepEqual(reloaded.blocks, records.draft.blocks);
assert.equal(reloaded.blocks.rf_blocks[0].rf_text, 'Texte **Rust** 🌍');
const result = {results:['Anonyme 401 et abonné 403, y compris création de brouillon',
  'Publiés/brouillons/programmés distincts, archives et corbeille hors compteurs',
  'Activité avec états explicites, corbeille exclue', 'Auteur isolé ; administrateur voit les deux auteurs',
  'Version serveur renseignée ; brouillon et document partagé conservés après relecture'],
  entry_ids:Object.fromEntries(Object.entries(records).map(([k,e]) => [k,e.id]))};
writeFileSync('scratch/wasm-auth-recipe/dashboard-results.json', JSON.stringify(result, null, 2) + '\n');
console.log(result.results.join('\n'));
