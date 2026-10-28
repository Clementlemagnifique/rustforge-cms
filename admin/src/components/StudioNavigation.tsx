import { useState } from 'react';
import { FileText, LayoutDashboard, LogOut, ChevronDown, PanelLeftClose, PanelLeftOpen } from 'lucide-react';
import type { ContentType, User } from '../types';
import type { AdminMenuItem } from '../types';
import { currentMenu, menuTrail } from '../navigation';

type Preferences = { version: 1; compact: boolean; opened: Record<string, boolean> };
type BranchProps = { node: AdminMenuItem; items: AdminMenuItem[]; active?: string; trail: string[]; compact: boolean; opened: Record<string, boolean>; navigate: (view: string) => void; toggle: (id: string, open: boolean) => void };
function Branch({ node, items, active, trail, compact, opened, navigate, toggle }: BranchProps) {
  const children = items.filter(item => item.parent === node.id);
  const expanded = opened[node.id] ?? trail.includes(node.id);
  const selected = active === node.id;
  const Icon = node.id === 'rf_dashboard' ? LayoutDashboard : FileText;
  return <li>
    <div className={`flex rounded-lg ${selected ? 'bg-indigo-600 text-white' : 'text-slate-400 hover:bg-white/5'}`}>
      <button title={compact ? node.label : undefined} aria-current={selected ? 'page' : undefined}
        onClick={() => node.route ? navigate(node.route) : toggle(node.id, !expanded)}
        className="flex items-center gap-2 min-w-0 flex-1 text-left px-3 py-2 text-sm focus-visible:outline-2 focus-visible:outline-indigo-300">
        {!node.parent && <Icon size={17} aria-hidden="true" className="shrink-0" />}<span className={compact ? 'sr-only' : ''}>{node.label}</span>
      </button>
      {children.length > 0 && !compact && <button type="button" aria-label={`${expanded ? 'Replier' : 'Déplier'} ${node.label}`} aria-expanded={expanded} aria-controls={`children-${node.id}`} onClick={() => toggle(node.id, !expanded)} className="px-2 focus-visible:outline-2 focus-visible:outline-indigo-300"><ChevronDown size={15} className={expanded ? 'rotate-180' : ''} /></button>}
    </div>
    {!compact && children.length > 0 && <ul id={`children-${node.id}`} hidden={!expanded} className="ml-4 border-l border-white/10 pl-2 mt-1 space-y-1">{children.map(child => <Branch key={child.id} {...{ items, active, trail, compact, opened, navigate, toggle }} node={child} />)}</ul>}
  </li>;
}

export function StudioNavigation({ currentView, onNavigate, contentTypes, items, user, onLogout }: { currentView: string; onNavigate: (view: string) => void; contentTypes: ContentType[]; items: AdminMenuItem[]; user: User; onLogout: () => void }) {
  const storageKey = `rf_admin_navigation_v1_${user.id}`;
  const [preferences, setPreferences] = useState<Preferences>(() => {
    try { const p = JSON.parse(localStorage.getItem(storageKey) || 'null'); if (p?.version === 1 && typeof p.compact === 'boolean' && p.opened && typeof p.opened === 'object' && !Array.isArray(p.opened) && Object.values(p.opened).every(v => typeof v === 'boolean')) return p; } catch { /* Préférences locales indisponibles. */ }
    return { version: 1, compact: false, opened: {} };
  });
  function update(next: Preferences) { setPreferences(next); try { localStorage.setItem(storageKey, JSON.stringify(next)); } catch { /* La navigation reste utilisable. */ } }
  const visible = items.filter(item => item.roles.includes(user.role));
  const trail = menuTrail(visible, currentView).map(item => item.id);
  const active = currentMenu(visible, currentView)?.id;
  const toggle = (id: string, open: boolean) => update({ ...preferences, compact: false, opened: { ...preferences.opened, [id]: open } });
  return <aside className={`${preferences.compact ? 'w-20' : 'w-64'} bg-[#0e1422] border-r border-white/10 h-screen flex flex-col`}>
    <div className="p-4 border-b border-white/10 flex items-center justify-between gap-2"><span className="font-bold truncate">{preferences.compact ? 'RF' : 'RustForge CMS'}</span><button aria-label={preferences.compact ? 'Déployer le menu' : 'Réduire le menu'} onClick={() => update({ ...preferences, compact: !preferences.compact })} className="p-1 focus-visible:outline-2 focus-visible:outline-indigo-300">{preferences.compact ? <PanelLeftOpen size={18} /> : <PanelLeftClose size={18} />}</button></div>
    <nav aria-label="Administration" className="p-2 flex-1 overflow-y-auto"><ul className="space-y-1">{visible.filter(item => !item.parent).map(node => <Branch key={node.id} {...{ node, items: visible, active, trail, compact: preferences.compact, opened: preferences.opened, navigate: onNavigate, toggle }} />)}</ul>
      {user.role !== 'subscriber' && !preferences.compact && contentTypes.filter(ct => !['post', 'page'].includes(ct.slug)).length > 0 && <div className="mt-4 border-t border-white/10 pt-3"><p className="px-3 text-xs text-slate-500">Contenus personnalisés</p>{contentTypes.filter(ct => !['post', 'page'].includes(ct.slug)).map(ct => <button key={ct.id} onClick={() => onNavigate(`entries:${ct.slug}`)} className="block text-sm text-slate-400 px-3 py-2">{ct.name}</button>)}</div>}
    </nav>
    <div className="border-t border-white/10 p-3"><p className={`text-sm truncate ${preferences.compact ? 'sr-only' : ''}`}>{user.display_name || user.username}</p><button title="Se déconnecter" onClick={onLogout} className="flex gap-2 items-center text-xs text-slate-400 mt-2"><LogOut size={16} /><span className={preferences.compact ? 'sr-only' : ''}>Se déconnecter</span></button></div>
  </aside>;
}
