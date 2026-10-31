import { test } from 'node:test';
import assert from 'node:assert/strict';
import { csvCell, validColumns } from '../admin/src/stores/columnPreferences.ts';

test('CSV neutralise les formules et conserve les délimiteurs', () => {
  for (const text of ['=1+1','+SUM(A1)','-1+1','@SUM(A1)',' \t=1','\u0000=1','\ttext']) assert.ok(csvCell(text).startsWith('"\''));
  assert.equal(csvCell('a;"b"'), '"a;""b"""');
  assert.equal(csvCell('Ordinaire'), '"Ordinaire"');
});
test('les préférences persistées sont validées avant affichage', () => {
  const col={id:'title',label:'Titre',visible:true,width:'140px'};
  assert.equal(validColumns([col])[0].id,'title');
  for (const bad of [null,[],[col,col],[{...col,width:'expression(x)'}],[{...col,visible:'yes'}],[{...col,label:'x'.repeat(121)}]]) assert.equal(validColumns(bad),null);
});
