import test from 'node:test';
import assert from 'node:assert/strict';
import vm from 'node:vm';
import { readFileSync } from 'node:fs';
const script = readFileSync(new URL('../crates/rustforge-api/templates/comments.js', import.meta.url), 'utf8');
function fixture(required = true) {
  const element = (value = '') => ({value, disabled:false, hidden:false, textContent:'', style:{}, listeners:{},
    addEventListener(name, fn) { this.listeners[name] = fn; }, focus() { this.focused = true; }});
  const names = Object.fromEntries(['login','password','author_name','author_email','author_url','content','website_url_hp'].map(n => [n, element()]));
  const button = element();
  const ids = Object.fromEntries(['comment-form','comment-login','comment-logout','comment-account-state','comment-fields','comment-identity','comment-status-msg','comment-submit-btn'].map(n => [n, element()]));
  ids['comment-form'].dataset = { entryId:'test-entry' };
  ids['comment-form'].elements = ids['comment-login'].elements = { namedItem:n => names[n] };
  ids['comment-fields'].disabled = required;
  ids['comment-login'].querySelector = () => button;
  ids['comment-identity'].querySelectorAll = () => [names.author_name,names.author_email,names.author_url];
  const calls = [], responses = [];
  const sandbox = { document:{getElementById:n => ids[n]}, FormData:class {
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
  return {names,ids,calls,responses,fire,login,button};
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
