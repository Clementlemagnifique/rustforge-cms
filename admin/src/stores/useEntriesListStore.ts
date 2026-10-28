/**
 * ============================================================================
 * STORE ZUSTAND : LISTES DE CONTENUS, FILTRES & QUICK EDIT (`useEntriesListStore`)
 * ============================================================================
 * Responsabilités :
 * - Séparation et mémorisation des filtres par type de contenu (`WP-15`).
 * - Filtres combinés : recherche (`q`), statut (`all`, `published`, `draft`,
 *   `archived`, `trash`), catégorie/étiquette (`taxonomy:slug`) et tri (`WP-16`, `WP-17`).
 * - Sélection multiple et exécution d'actions groupées avec rapport d'erreurs (`WP-17`).
 * - Modification rapide en ligne (Quick Edit : titre, slug, statut, termes) par
 *   patch partiel qui préserve intégralement les blocs natifs `rf.document/1`
 *   et le coffre de provenance `_rf_vault` (`WP-18`).
 */

import { create } from 'zustand';
import { api } from '../api';
import { entriesMessages } from '../i18n/messages.fr';
import type { Entry, EntryStatus, TermRef } from '../types';

export interface EntryFilterState {
  search: string;
  statusFilter: string;
  termFilter: string; // "" ou "taxonomy:slug"
  orderFilter: string; // "updated" | "published" | "title" | "created"
  page: number;
}

export const DEFAULT_ENTRY_FILTERS: EntryFilterState = {
  search: '',
  statusFilter: 'all',
  termFilter: '',
  orderFilter: 'updated',
  page: 1,
};

export interface EntriesListStoreState {
  // --- Contexte actif & mode d'affichage ---
  activeContentType: string;
  viewMode: 'table' | 'kanban';
  draggedEntryId: string | null;

  // --- Mémoire de filtres par type de contenu (WP-15) ---
  filtersByType: Record<string, EntryFilterState>;

  // --- Données chargées depuis l'API ---
  entries: Entry[];
  totalEntries: number;
  totalPages: number;
  loading: boolean;
  error: string;
  availableTerms: TermRef[];

  // --- Sélection multiple & actions groupées (WP-17) ---
  selectedIds: string[];
  bulkAction: string;
  bulkBusy: boolean;

  // --- Modification rapide en ligne / Quick Edit (WP-18) ---
  quickEditId: string | null;
  quickTitle: string;
  quickSlug: string;
  quickStatus: EntryStatus;
  quickTermIds: string[];
  quickSaving: boolean;

  // --- Actions de configuration et de filtrage ---
  setActiveContentType: (contentTypeSlug: string) => void;
  setViewMode: (mode: 'table' | 'kanban') => void;
  setDraggedEntryId: (id: string | null) => void;
  updateFilter: (contentTypeSlug: string, patch: Partial<EntryFilterState>) => void;

  // --- Actions de sélection groupée (WP-17) ---
  setBulkAction: (action: string) => void;
  clearSelection: () => void;
  toggleSelectRow: (id: string) => void;
  toggleSelectAllOnPage: () => void;
  runBulkAction: (contentTypeSlug: string) => Promise<void>;

  // --- Actions d'édition rapide en ligne (WP-18) ---
  openQuickEdit: (entry: Entry) => void;
  cancelQuickEdit: () => void;
  setQuickTitle: (title: string) => void;
  setQuickSlug: (slug: string) => void;
  setQuickStatus: (status: EntryStatus) => void;
  toggleQuickTerm: (termId: string) => void;
  saveQuickEdit: (entry: Entry, contentTypeSlug: string) => Promise<void>;

  // --- Actions API ---
  loadTerms: () => Promise<void>;
  fetchEntries: (contentTypeSlug?: string) => Promise<void>;
  deleteEntry: (id: string, force: boolean, contentTypeSlug: string) => Promise<void>;
  updateEntryStatus: (entry: Entry, newStatus: EntryStatus, contentTypeSlug: string) => Promise<void>;
}

export const useEntriesListStore = create<EntriesListStoreState>((set, get) => ({
  activeContentType: 'post',
  viewMode: 'table',
  draggedEntryId: null,

  filtersByType: {},

  entries: [],
  totalEntries: 0,
  totalPages: 1,
  loading: true,
  error: '',
  availableTerms: [],

  selectedIds: [],
  bulkAction: '',
  bulkBusy: false,

  quickEditId: null,
  quickTitle: '',
  quickSlug: '',
  quickStatus: 'draft',
  quickTermIds: [],
  quickSaving: false,

  setActiveContentType: (contentTypeSlug) => {
    const prevType = get().activeContentType;
    if (prevType !== contentTypeSlug) {
      set({
        activeContentType: contentTypeSlug,
        selectedIds: [],
        quickEditId: null,
        bulkAction: '',
      });
    }
    void get().fetchEntries(contentTypeSlug);
  },

  setViewMode: (viewMode) => set({ viewMode }),
  setDraggedEntryId: (draggedEntryId) => set({ draggedEntryId }),

  updateFilter: (contentTypeSlug, patch) => {
    set((state) => {
      const current = state.filtersByType[contentTypeSlug] ?? DEFAULT_ENTRY_FILTERS;
      return {
        filtersByType: {
          ...state.filtersByType,
          [contentTypeSlug]: { ...current, ...patch },
        },
      };
    });
    void get().fetchEntries(contentTypeSlug);
  },

  setBulkAction: (bulkAction) => set({ bulkAction }),
  clearSelection: () => set({ selectedIds: [] }),

  toggleSelectRow: (id) =>
    set((state) => ({
      selectedIds: state.selectedIds.includes(id)
        ? state.selectedIds.filter((item) => item !== id)
        : [...state.selectedIds, id],
    })),

  toggleSelectAllOnPage: () =>
    set((state) => {
      const pageIds = state.entries.map((e) => e.id);
      const allSelected = pageIds.length > 0 && pageIds.every((id) => state.selectedIds.includes(id));
      return {
        selectedIds: allSelected
          ? state.selectedIds.filter((id) => !pageIds.includes(id))
          : Array.from(new Set([...state.selectedIds, ...pageIds])),
      };
    }),

  runBulkAction: async (contentTypeSlug) => {
    const { bulkAction, selectedIds, entries } = get();
    if (!bulkAction || selectedIds.length === 0) return;

    if (bulkAction === 'force_delete') {
      if (
        !window.confirm(
          `Supprimer définitivement ${selectedIds.length} contenu(s) sélectionné(s) ? Cette action est irréversible.`
        )
      ) {
        return;
      }
    } else if (bulkAction === 'trash') {
      if (!window.confirm(`Déplacer ${selectedIds.length} contenu(s) vers la corbeille ?`)) {
        return;
      }
    }

    set({ bulkBusy: true });
    const failedIds: string[] = [];
    const errors: string[] = [];
    const nowIso = new Date().toISOString();

    for (const id of selectedIds) {
      const target = entries.find((item) => item.id === id);
      if (!target) continue;
      try {
        if (bulkAction === 'trash') {
          await api.entries.delete(id, false);
        } else if (bulkAction === 'force_delete') {
          await api.entries.delete(id, true);
        } else if (
          bulkAction === 'published' ||
          bulkAction === 'draft' ||
          bulkAction === 'archived'
        ) {
          await api.entries.update(id, {
            expected_updated_at: target.updated_at,
            status: bulkAction as EntryStatus,
            published_at:
              bulkAction === 'published' && !target.published_at ? nowIso : target.published_at,
          });
        }
      } catch (err) {
        failedIds.push(id);
        errors.push(`${target.title} : ${err instanceof Error ? err.message : 'Erreur'}`);
      }
    }

    set({
      bulkBusy: false,
      selectedIds: failedIds,
      bulkAction: '',
    });
    await get().fetchEntries(contentTypeSlug);
    if (errors.length > 0) {
      set({ error: `Certaines actions groupées ont échoué : ${errors.join(' | ')}` });
    }
  },

  openQuickEdit: (entry) =>
    set({
      quickEditId: entry.id,
      quickTitle: entry.title,
      quickSlug: entry.slug,
      quickStatus: entry.status === 'trash' ? 'draft' : entry.status,
      quickTermIds: (entry.terms || []).map((t) => t.id),
    }),

  cancelQuickEdit: () => set({ quickEditId: null }),
  setQuickTitle: (quickTitle) => set({ quickTitle }),
  setQuickSlug: (quickSlug) => set({ quickSlug }),
  setQuickStatus: (quickStatus) => set({ quickStatus }),

  toggleQuickTerm: (termId) =>
    set((state) => ({
      quickTermIds: state.quickTermIds.includes(termId)
        ? state.quickTermIds.filter((id) => id !== termId)
        : [...state.quickTermIds, termId],
    })),

  saveQuickEdit: async (entry, contentTypeSlug) => {
    const { quickTitle, quickSlug, quickStatus, quickTermIds } = get();
    if (!quickTitle.trim()) {
      window.alert('Le titre ne peut pas être vide.');
      return;
    }
    set({ quickSaving: true });
    try {
      const nowIso = new Date().toISOString();
      await api.entries.update(entry.id, {
        expected_updated_at: entry.updated_at,
        title: quickTitle.trim(),
        slug: quickSlug.trim() || entry.slug,
        status: quickStatus,
        term_ids: quickTermIds,
        published_at:
          quickStatus === 'published' && !entry.published_at ? nowIso : entry.published_at,
      });
      set({ quickEditId: null, quickSaving: false });
      await get().fetchEntries(contentTypeSlug);
    } catch (err) {
      set({ quickSaving: false });
      window.alert(
        `Erreur lors de la modification rapide : ${err instanceof Error ? err.message : 'Erreur'}`
      );
    }
  },

  loadTerms: async () => {
    try {
      const terms = await api.terms.list();
      set({ availableTerms: terms || [] });
    } catch {
      // Les filtres de termes restent optionnels si le chargement échoue
    }
  },

  fetchEntries: async (contentTypeSlug) => {
    const slug = contentTypeSlug || get().activeContentType;
    const filters = get().filtersByType[slug] ?? DEFAULT_ENTRY_FILTERS;
    set({ loading: true });

    try {
      let taxonomyParam: string | undefined;
      let termParam: string | undefined;
      if (filters.termFilter && filters.termFilter.includes(':')) {
        const [tax, termSlug] = filters.termFilter.split(':', 2);
        taxonomyParam = tax || undefined;
        termParam = termSlug || undefined;
      }

      const res = await api.entries.list({
        content_type: slug,
        q: filters.search || undefined,
        status: filters.statusFilter === 'all' ? undefined : filters.statusFilter,
        taxonomy: taxonomyParam,
        term: termParam,
        order: filters.orderFilter || 'updated',
        page: filters.page,
        per_page: 20,
      });

      set({
        entries: res.items,
        totalEntries: res.total,
        totalPages: Math.max(1, res.total_pages),
        error: '',
        loading: false,
      });
    } catch (err) {
      set({
        error: err instanceof Error ? err.message : 'Chargement impossible',
        loading: false,
      });
    }
  },

  deleteEntry: async (id, force, contentTypeSlug) => {
    const message = force ? entriesMessages.confirmForceDelete : entriesMessages.confirmTrash;
    if (!window.confirm(message)) return;
    try {
      await api.entries.delete(id, force);
      set((state) => ({
        selectedIds: state.selectedIds.filter((item) => item !== id),
      }));
      await get().fetchEntries(contentTypeSlug);
    } catch (err) {
      window.alert(err instanceof Error ? err.message : 'Suppression impossible');
    }
  },

  updateEntryStatus: async (entry, newStatus, contentTypeSlug) => {
    try {
      const nowIso = new Date().toISOString();
      await api.entries.update(entry.id, {
        expected_updated_at: entry.updated_at,
        status: newStatus,
        published_at:
          newStatus === 'published' && !entry.published_at ? nowIso : entry.published_at,
      });
      await get().fetchEntries(contentTypeSlug);
    } catch (err) {
      window.alert(
        `Erreur de mise à jour du statut : ${err instanceof Error ? err.message : 'Erreur'}`
      );
    }
  },
}));

