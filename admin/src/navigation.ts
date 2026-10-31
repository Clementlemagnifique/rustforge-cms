import type { AdminMenuItem } from './types';

export function currentMenu(items: AdminMenuItem[], view: string): AdminMenuItem | undefined {
  // Préférer l’entrée la plus précise lorsque parent et enfant partagent une route.
  return [...items].reverse().find(item => item.route === view)
    || (view.startsWith('editor:') ? [...items].reverse().find(item => item.route === `entries:${view.split(':')[2] || 'post'}`) : undefined);
}

export function menuTrail(items: AdminMenuItem[], view: string): AdminMenuItem[] {
  const trail: AdminMenuItem[] = [];
  let node = currentMenu(items, view);
  while (node && trail.length < 3) { trail.unshift(node); node = items.find(item => item.id === node?.parent); }
  return trail;
}

export interface MenuBranch { item: AdminMenuItem; children: MenuBranch[] }
/** Server order and rights are authoritative; no second registry in the Studio. */
export function menuTree(items: AdminMenuItem[], parent: string | null = null): MenuBranch[] {
  return items.filter(item => item.parent === parent).map(item => ({item, children: menuTree(items, item.id)}));
}
