import test from 'node:test';
import assert from 'node:assert/strict';
import vm from 'node:vm';
import { readFileSync } from 'node:fs';
const script = readFileSync(new URL('../crates/rustforge-api/templates/comments.js', import.meta.url), 'utf8');
function fixture(required = true, consent = false, saved = null, blocked = false) {
  const element = (value = '') => ({value, disabled:false, hidden:false, textContent:'', style:{}, listeners:{},
    addEventListener(name, fn) { this.listeners[name] = fn; }, focus() { this.focused = true; }});
  const names = Object.fromEntries(['login','password','author_name','author_email','author_url','content','website_url_hp'].map(n => [n, element()]));
  const button = element();
  const ids = Object.fromEntries(['comment-form','comment-login','comment-logout','comment-account-state','comment-fields','comment-identity','comment-status-msg','comment-submit-btn'].map(n => [n, element()]));
  if (consent) { ids['comment-remember'] = element(); ids['comment-remember'].checked = false; }
  const storage = new Map(); const storageEvents = [];
  if (saved !== null) storage.set('rf.comment.identity.v1', JSON.stringify(saved));
  ids['comment-form'].dataset = { entryId:'test-entry' };
  ids['comment-form'].elements = ids['comment-login'].elements = { namedItem:n => names[n] };
  ids['comment-fields'].disabled = required;
  ids['comment-login'].querySelector = () => button;
  ids['comment-identity'].querySelectorAll = () => [names.author_name,names.author_email,names.author_url];
  const calls = [], responses = [];
  const sandbox = { localStorage: {
    getItem(key) { if (blocked) throw new Error('blocked'); return storage.get(key) ?? null; },
    setItem(key,value) { if (blocked) throw new Error('blocked'); storageEvents.push('write'); storage.set(key,value); },
    removeItem(key) { if (blocked) throw new Error('blocked'); storageEvents.push('remove'); storage.delete(key); }
  }, document:{getElementById:n => ids[n]}, FormData:class {
    get(n) { return names[n].disabled ? null : names[n].value; }
  }, fetch:async (url, options) => {
    calls.push({url,...options});
    const response = responses.shift();
    if (response instanceof Error) throw response;
    assert.ok(response, 'Réponse de recette manquante');
    return {ok:response.status===200,status:response.status,json:async()=>response.body};
  }};
  vm.runInNewContext(script,sandbox);
  const fire = async (id,event='submit') => ids[id].listeners[event]({preventDefault(){}});
  const login = async () => {
    names.login.value='lecteur'; names.password.value='secret';
    responses.push({status:200,body:{token:'comment-only',display_name:'Lecteur <test>'}});
    await fire('comment-login');
  };
  return {names,ids,calls,responses,fire,login,button,storage,storageEvents};
}
test('connexion conserve le brouillon, efface le mot de passe et utilise textContent',async()=>{
  const f=fixture(); f.names.content.value='Brouillon'; await f.login();
  assert.equal(f.names.content.value,'Brouillon'); assert.equal(f.names.password.value,'');
  assert.equal(f.ids['comment-fields'].disabled,false);
  assert.equal(f.ids['comment-account-state'].textContent,'Connecté pour commenter : Lecteur <test>');
  assert.equal(f.names.content.focused,true);
});
test('identifiants refusés : erreur lisible, champ protégé, brouillon préservé',async()=>{
  const f=fixture(); f.names.password.value='incorrect';f.names.content.value='Texte';
  f.responses.push({status:401,body:{error:{message:'Identifiants invalides'}}});await f.fire('comment-login');
  assert.equal(f.ids['comment-fields'].disabled,true);assert.equal(f.names.content.value,'Texte');
  assert.equal(f.ids['comment-account-state'].textContent,'Identifiants invalides');assert.equal(f.names.password.value,'');
});
test('envoi authentifié limité au commentaire, sans cookies, puis déconnexion locale',async()=>{
  const f=fixture();await f.login();f.names.content.value='Publié';
  f.responses.push({status:200,body:{id:'created'}});await f.fire('comment-form');
  assert.equal(f.calls[1].url,'/api/v1/public/entries/test-entry/comments');
  assert.equal(f.calls[1].headers.Authorization,'Bearer comment-only');assert.equal(f.calls[1].credentials,'omit');
  assert.equal(JSON.parse(f.calls[1].body).author_name,'');assert.equal(f.names.content.value,'');
  f.names.content.value='Autre brouillon';await f.fire('comment-logout','click');
  assert.equal(f.names.content.value,'Autre brouillon');assert.equal(f.ids['comment-fields'].disabled,true);
});
test('expiration : reconnexion proposée sans perdre le commentaire',async()=>{
  const f=fixture();await f.login();f.names.content.value='À conserver';
  f.responses.push({status:401,body:{error:{message:'non autorisé'}}});await f.fire('comment-form');
  assert.equal(f.names.content.value,'À conserver');assert.equal(f.ids['comment-fields'].disabled,true);
  assert.match(f.ids['comment-status-msg'].textContent,/expiré/);assert.equal(f.names.login.focused,true);
});
test('visiteur : aucun bearer, erreur réseau sans perte',async()=>{
  const f=fixture(false);f.names.content.value='Sans compte';f.names.author_name.value='Invité';
  f.responses.push(new Error('network'));await f.fire('comment-form');
  assert.equal(f.calls[0].headers.Authorization,undefined);assert.equal(f.names.content.value,'Sans compte');
  assert.equal(f.ids['comment-submit-btn'].disabled,false);
});
test('page fermée : aucun formulaire, script inactif',()=>{
  vm.runInNewContext(script,{document:{getElementById:()=>null}});
});

test('identité enregistrée uniquement après choix explicite et envoi réussi',async()=>{
  const f=fixture(false,true);assert.equal(f.storage.size,0);
  f.names.author_name.value='Invité';f.names.author_email.value='invite@example.test';f.names.content.value='Jamais mémorisé';
  f.ids['comment-remember'].checked=true;
  f.responses.push({status:422,body:{error:{message:'Refus'}}});await f.fire('comment-form');
  assert.equal(f.storage.size,0);
  f.responses.push({status:200,body:{id:'ok'}});await f.fire('comment-form');
  const record=JSON.parse(f.storage.get('rf.comment.identity.v1'));
  assert.deepEqual(Object.keys(record).sort(),['email','expires','name','url','version']);
  assert.equal(record.name,'Invité');assert.ok(record.expires>Date.now());
  assert.ok(record.expires<=Date.now()+30*86400000);
  f.ids['comment-remember'].checked=false;await f.fire('comment-remember','change');
  assert.equal(f.storage.size,0);
});
test('identité valide restaurée ; identité expirée ou désactivée oubliée',()=>{
  const record={version:1,name:'Lecteur',email:'lecteur@example.test',url:'',expires:Date.now()+86400000};
  const f=fixture(false,true,record);assert.equal(f.names.author_name.value,'Lecteur');assert.equal(f.ids['comment-remember'].checked,true);
  assert.equal(fixture(false,true,{...record,expires:1}).storage.size,0);
  assert.equal(fixture(false,false,record).storage.size,0);
  assert.equal(fixture(false,true,{...record,token:'forbidden'}).storage.size,0);
});
test('stockage bloqué et absence de consentement ne bloquent pas le commentaire',async()=>{
  for (const [consent,blocked] of [[true,true],[true,false]]) {
    const f=fixture(false,consent,null,blocked);f.names.content.value='Texte';
    f.responses.push({status:200,body:{id:'ok'}});await f.fire('comment-form');
    assert.equal(f.storage.size,0);assert.equal(f.names.content.value,'');
  }
});
test('session authentifiée : aucune identité de compte persistée',async()=>{
  const f=fixture(false,true);await f.login();f.ids['comment-remember'].checked=true;f.names.content.value='Compte';
  f.responses.push({status:200,body:{id:'ok'}});await f.fire('comment-form');assert.equal(f.storage.size,0);
});
