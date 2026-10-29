/**
 * ============================================================================
 * STORE ZUSTAND : GESTION DES COLONNES & VUES DE DONNÉES (Admin Columns Pro)
 * ============================================================================
 * Couvre les exigences :
 * - `MOD-COL-01` : Contrat et typage des colonnes par type de contenu.
 * - `MOD-COL-02` : Sélection, ordre, largeur et visibilité des colonnes.
 * - `MOD-COL-03` : Rendu riche des cellules (médias, auteurs, taxonomies, champs).
 * - `MOD-COL-04` : Tri dynamique, recherche et filtres.
 * - `MOD-COL-05` : Export CSV des colonnes visibles et préréglages partagés (Presets).
 */

import { create } from 'zustand';
import { api } from '../api';
import type { Entry } from '../types';

export interface ColumnDefinition {
  id: string;
  label: string;
  visible: boolean;
  width?: string;
  sortable?: boolean;
  isCustomField?: boolean;
}

export type ColumnPreset = 'standard' | 'editorial' | 'seo' | 'compact';

export const DEFAULT_COLUMNS_BY_TYPE: Record<string, ColumnDefinition[]> = {
  post: [
    { id: 'image', label: 'Média', visible: true, width: '56px' },
    { id: 'title', label: 'Titre & Slug', visible: true, width: 'auto', sortable: true },
    { id: 'status', label: 'Statut', visible: true, width: '110px', sortable: true },
    { id: 'author', label: 'Auteur', visible: true, width: '130px' },
    { id: 'categories', label: 'Catégories', visible: true, width: '150px' },
    { id: 'date', label: 'Date', visible: true, width: '140px', sortable: true },
    { id: 'words', label: 'Mots & Lecture', visible: false, width: '120px' },
    { id: 'id', label: 'ID', visible: false, width: '90px' },
  ],
  page: [
    { id: 'title', label: 'Titre & Slug', visible: true, width: 'auto', sortable: true },
    { id: 'status', label: 'Statut', visible: true, width: '110px', sortable: true },
    { id: 'author', label: 'Auteur', visible: true, width: '130px' },
    { id: 'date', label: 'Date', visible: true, width: '140px', sortable: true },
    { id: 'words', label: 'Mots & Lecture', visible: false, width: '120px' },
    { id: 'id', label: 'ID', visible: false, width: '90px' },
  ],
};

function getStorageKey(type: string): string {
  return `rf_columns_pref_${type}`;
}

export interface ColumnSettingsState {
  columnsByType: Record<string, ColumnDefinition[]>;
  activePresetByType: Record<string, ColumnPreset>;
  saving: boolean;
  density: 'comfortable' | 'compact';

  setDensity: (density: 'comfortable' | 'compact') => void;
  getColumns: (type: string) => ColumnDefinition[];
  loadColumns: (type: string) => void;
  toggleColumn: (type: string, columnId: string) => void;
  moveColumn: (type: string, fromIndex: number, toIndex: number) => void;
  updateColumnLabel: (type: string, columnId: string, label: string) => void;
  applyPreset: (type: string, preset: ColumnPreset) => void;
  resetToDefault: (type: string) => void;
  saveGlobalPreferences: (type: string) => Promise<boolean>;
  exportVisibleCsv: (type: string, entries: Entry[]) => void;
}

export const useColumnSettingsStore = create<ColumnSettingsState>((set, get) => ({
  columnsByType: {},
  activePresetByType: {},
  saving: false,
  density: 'comfortable',

  setDensity: (density) => set({ density }),

  getColumns: (type: string) => {
    const current = get().columnsByType[type];
    if (current && current.length > 0) return current;

    // Charger depuis le localStorage ou les colonnes par défaut
    try {
      const stored = localStorage.getItem(getStorageKey(type));
      if (stored) {
        const parsed = JSON.parse(stored) as ColumnDefinition[];
        if (Array.isArray(parsed) && parsed.length > 0) {
          set((state) => ({
            columnsByType: { ...state.columnsByType, [type]: parsed },
          }));
          return parsed;
        }
      }
    } catch {
      // Ignorer erreur localStorage
    }

    const fallback = DEFAULT_COLUMNS_BY_TYPE[type] || DEFAULT_COLUMNS_BY_TYPE.post;
    return fallback;
  },

  loadColumns: (type: string) => {
    const cols = get().getColumns(type);
    set((state) => ({
      columnsByType: { ...state.columnsByType, [type]: cols },
    }));
  },

  toggleColumn: (type: string, columnId: string) => {
    const list = [...get().getColumns(type)];
    const updated = list.map((c) => (c.id === columnId ? { ...c, visible: !c.visible } : c));
    try {
      localStorage.setItem(getStorageKey(type), JSON.stringify(updated));
    } catch {
      // Ignorer
    }
    set((state) => ({
      columnsByType: { ...state.columnsByType, [type]: updated },
    }));
  },

  moveColumn: (type: string, fromIndex: number, toIndex: number) => {
    const list = [...get().getColumns(type)];
    if (fromIndex < 0 || fromIndex >= list.length || toIndex < 0 || toIndex >= list.length) return;
    const [moved] = list.splice(fromIndex, 1);
    list.splice(toIndex, 0, moved);
    try {
      localStorage.setItem(getStorageKey(type), JSON.stringify(list));
    } catch {
      // Ignorer
    }
    set((state) => ({
      columnsByType: { ...state.columnsByType, [type]: list },
    }));
  },

  updateColumnLabel: (type: string, columnId: string, label: string) => {
    const list = [...get().getColumns(type)];
    const updated = list.map((c) => (c.id === columnId ? { ...c, label } : c));
    try {
      localStorage.setItem(getStorageKey(type), JSON.stringify(updated));
    } catch {
      // Ignorer
    }
    set((state) => ({
      columnsByType: { ...state.columnsByType, [type]: updated },
    }));
  },

  applyPreset: (type: string, preset: ColumnPreset) => {
    const base = DEFAULT_COLUMNS_BY_TYPE[type] || DEFAULT_COLUMNS_BY_TYPE.post;
    let visibleIds: string[] = [];

    switch (preset) {
      case 'standard':
        visibleIds = ['image', 'title', 'status', 'author', 'categories', 'date'];
        break;
      case 'editorial':
        visibleIds = ['image', 'title', 'status', 'author', 'categories', 'words', 'date'];
        break;
      case 'seo':
        visibleIds = ['image', 'title', 'status', 'date', 'id'];
        break;
      case 'compact':
        visibleIds = ['title', 'status', 'date'];
        break;
    }

    const updated = base.map((col) => ({
      ...col,
      visible: visibleIds.includes(col.id),
    }));

    try {
      localStorage.setItem(getStorageKey(type), JSON.stringify(updated));
    } catch {
      // Ignorer
    }

    set((state) => ({
      columnsByType: { ...state.columnsByType, [type]: updated },
      activePresetByType: { ...state.activePresetByType, [type]: preset },
    }));
  },

  resetToDefault: (type: string) => {
    const def = DEFAULT_COLUMNS_BY_TYPE[type] || DEFAULT_COLUMNS_BY_TYPE.post;
    try {
      localStorage.removeItem(getStorageKey(type));
    } catch {
      // Ignorer
    }
    set((state) => ({
      columnsByType: { ...state.columnsByType, [type]: def },
      activePresetByType: { ...state.activePresetByType, [type]: 'standard' },
    }));
  },

  saveGlobalPreferences: async (type: string) => {
    const cols = get().getColumns(type);
    set({ saving: true });
    try {
      await api.settings.update({
        [`admin.columns.${type}`]: cols,
      });
      set({ saving: false });
      return true;
    } catch (e) {
      console.error('Erreur sauvegarde colonnes globale:', e);
      set({ saving: false });
      return false;
    }
  },

  exportVisibleCsv: (type: string, entries: Entry[]) => {
    const cols = get().getColumns(type).filter((c) => c.visible);
    if (cols.length === 0 || entries.length === 0) return;

    // Ligne d'en-tête
    const header = cols.map((c) => `"${c.label.replace(/"/g, '""')}"`).join(';');

    // Lignes de données
    const rows = entries.map((entry) => {
      return cols
        .map((col) => {
          let val = '';
          switch (col.id) {
            case 'image':
              val = entry.featured_media_url || '';
              break;
            case 'title':
              val = `${entry.title} (/${entry.slug})`;
              break;
            case 'status':
              val = entry.status;
              break;
            case 'author':
              val = entry.author_name || 'Éditeur';
              break;
            case 'categories':
              val = (entry.terms || [])
                .filter((t) => t.taxonomy === 'category')
                .map((t) => t.name)
                .join(', ');
              break;
            case 'tags':
              val = (entry.terms || [])
                .filter((t) => t.taxonomy === 'tag')
                .map((t) => t.name)
                .join(', ');
              break;
            case 'date':
              val = new Date(entry.updated_at).toLocaleDateString('fr-FR');
              break;
            case 'words': {
              const text = entry.body || '';
              const count = text.split(/\s+/).filter(Boolean).length;
              val = `${count} mots`;
              break;
            }
            case 'id':
              val = entry.id;
              break;
            default:
              val = String(entry.fields?.[col.id] ?? '');
              break;
          }
          return `"${String(val).replace(/"/g, '""')}"`;
        })
        .join(';');
    });

    const csvContent = '\uFEFF' + [header, ...rows].join('\r\n');
    const blob = new Blob([csvContent], { type: 'text/csv;charset=utf-8;' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.setAttribute('href', url);
    link.setAttribute(
      'download',
      `export-${type}-${new Date().toISOString().slice(0, 10)}.csv`
    );
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    URL.revokeObjectURL(url);
  },
}));
