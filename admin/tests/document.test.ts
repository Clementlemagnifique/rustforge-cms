import { strict as assert } from 'node:assert';
import { test } from 'node:test';
import {
  newDocument,
  newBlock,
  readDocument,
  moveBlock,
  insertBlockAfter,
  createHistory,
  pushHistory,
  undoHistory,
  redoHistory,
  autosaveStorageKey,
  parseAutosaveDraft,
  DOCUMENT_SCHEMA,
} from '../src/editor/document.ts';

test('le document natif se recharge sans traduire les données ni modifier les identifiants', () => {
  const doc = newDocument();
  doc.rf_blocks[0].rf_text = 'Titre français, English text, 日本語';
  const img = newBlock('image');
  img.rf_url = '/uploads/2026/10/test.webp';
  img.rf_alt = 'Image de test';
  img.rf_caption = 'Légende de test';
  img.rf_media_id = '11111111-2222-3333-4444-555555555555';
  const btn = newBlock('button');
  btn.rf_text = 'Accéder au guide';
  btn.rf_url = '/guide';
  doc.rf_blocks.push(img, btn);
  const loaded = readDocument(JSON.parse(JSON.stringify(doc)));
  assert.deepEqual(loaded, doc);
  assert.equal(loaded?.rf_schema, DOCUMENT_SCHEMA);
});

test('déplacer et insérer des blocs conserve textes et identifiants; les déplacements hors limites ne changent rien', () => {
  const doc = newDocument();
  doc.rf_blocks.push(newBlock('heading'));
  const moved = moveBlock(doc, doc.rf_blocks[1].rf_id, -1);
  assert.equal(moved.rf_blocks[0].rf_id, doc.rf_blocks[1].rf_id);
  assert.equal(moved.rf_blocks[0].rf_type, 'heading');
  assert.equal(moved.rf_blocks[1].rf_type, 'paragraph');
  assert.equal(moveBlock(doc, doc.rf_blocks[0].rf_id, -1), doc);

  const inserted = insertBlockAfter(moved, moved.rf_blocks[0].rf_id, 'image');
  assert.equal(inserted.rf_blocks.length, 3);
  assert.equal(inserted.rf_blocks[1].rf_type, 'image');
});

test('l’historique local undo/redo restaure les états successifs et réinitialise le futur après nouvelle modification', () => {
  const v1 = newDocument();
  v1.rf_blocks[0].rf_text = 'Version 1';
  let hist = createHistory(v1);
  assert.equal(hist.past.length, 0);

  const v2: typeof v1 = JSON.parse(JSON.stringify(v1));
  v2.rf_blocks[0].rf_text = 'Version 2';
  hist = pushHistory(hist, v2);
  assert.equal(hist.past.length, 1);
  assert.equal(hist.present.rf_blocks[0].rf_text, 'Version 2');

  // Pousser le même état ne pollue pas la pile
  assert.equal(pushHistory(hist, v2).past.length, 1);

  const undone = undoHistory(hist);
  assert.equal(undone.present.rf_blocks[0].rf_text, 'Version 1');
  assert.equal(undone.future.length, 1);

  const redone = redoHistory(undone);
  assert.equal(redone.present.rf_blocks[0].rf_text, 'Version 2');
  assert.equal(redone.future.length, 0);

  // Nouvelle modification après undo efface la branche future
  const v3: typeof v1 = JSON.parse(JSON.stringify(v1));
  v3.rf_blocks[0].rf_text = 'Version 3';
  const branched = pushHistory(undone, v3);
  assert.equal(branched.future.length, 0);
  assert.equal(branched.present.rf_blocks[0].rf_text, 'Version 3');
});

test('l’autosauvegarde locale valide le brouillon et ignore les brouillons plus anciens que la version serveur', () => {
  assert.equal(autosaveStorageKey('post', 'abc-123'), 'rf.autosave.v1:post:abc-123');
  assert.equal(autosaveStorageKey('page'), 'rf.autosave.v1:page:new');

  const raw = JSON.stringify({
    rf_autosave_version: 1,
    saved_at: '2026-10-28T12:00:00Z',
    base_updated_at: '2026-10-28T11:00:00Z',
    title: 'Brouillon récupéré',
    slug: 'brouillon-recupere',
    excerpt: 'Extrait',
    body: '',
    blocks: newDocument(),
    status: 'draft',
  });
  const recovered = parseAutosaveDraft(raw, '2026-10-28T11:00:00Z');
  assert.equal(recovered?.title, 'Brouillon récupéré');

  // Si le serveur a été mis à jour après l'autosauvegarde, elle est ignorée
  const stale = parseAutosaveDraft(raw, '2026-10-28T12:30:00Z');
  assert.equal(stale, null);
});

test('le parseur refuse formats futurs et champs inconnus au lieu de les tronquer', () => {
  assert.equal(readDocument([{ type: 'legacy', html: '<div>source</div>' }]), null);
  assert.equal(readDocument({ rf_schema: 'rf.document/2', rf_blocks: [] }), null);
  assert.equal(readDocument({ rf_schema: DOCUMENT_SCHEMA, rf_blocks: [], extra: 'à conserver' }), null);
  const doc = newDocument();
  doc.rf_blocks.push({ ...doc.rf_blocks[0] });
  assert.equal(readDocument(doc), null);
});
