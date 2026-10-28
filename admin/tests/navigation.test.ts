import { strict as assert } from 'node:assert';
import { test } from 'node:test';
import { readFileSync } from 'node:fs';
import { currentMenu, menuTrail } from '../src/navigation.ts';
import type { AdminMenuItem } from '../src/types.ts';
const nodes: AdminMenuItem[] = JSON.parse(readFileSync(new URL('../../crates/rustforge-api/src/admin_navigation.json', import.meta.url), 'utf8'));

test('la route de liste sélectionne une seule entrée précise', () => {
  assert.equal(currentMenu(nodes, 'entries:post')?.id, 'rf_post_list');
  assert.deepEqual(menuTrail(nodes, 'entries:post').map(n => n.label), ['Articles', 'Tous les articles']);
});
test('rouvrir une page existante garde son contexte de navigation', () => {
  assert.deepEqual(menuTrail(nodes, 'editor:some-id:page').map(n => n.id), ['rf_pages', 'rf_page_list']);
  assert.equal(currentMenu(nodes, 'section-inconnue'), undefined);
});
test('le fil de navigation conserve les trois niveaux sans confondre les menus du site et du Studio', () => {
  const future = nodes.map(n => n.id === 'rf_nav_settings' ? { ...n, route: 'future-menu-settings' } : n);
  assert.deepEqual(menuTrail(future, 'future-menu-settings').map(n => n.id), ['rf_settings', 'rf_admin_preferences', 'rf_nav_settings']);
});
