/**
 * ============================================================================
 * STORE ZUSTAND : CATÉGORIES HIÉRARCHIQUES & ÉTIQUETTES (`useTaxonomiesStore`)
 * ============================================================================
 * Couvre l'exigence `WP-19` :
 * - Création, renommage, édition du slug, description et catégorie parente
 *   anti-cycle.
 * - Fusion d'un terme vers un autre (`POST /api/v1/terms/{id}/merge`) sans
 *   perdre les articles ni les sous-catégories associés.
 * - Suppression avec choix explicite de réaffectation (`DELETE /api/v1/terms/{id}?reassign_to=...`).
 */

import { create } from 'zustand';
import { api } from '../api';
import type { TermRef } from '../types';

export type TermActionMode = 'merge' | 'delete' | null;

export interface TaxonomiesStoreState {
  // --- Contexte et liste ---
  taxonomySlug: string;
  terms: TermRef[];
  loading: boolean;
  saving: boolean;
  error: string;
  search: string;

  // --- Formulaire de création / édition ---
  editingTermId: string | null;
  name: string;
  slug: string;
  parentId: string | null;
  description: string;

  // --- Panneau en ligne de fusion ou suppression avec réaffectation ---
  actionTermId: string | null;
  actionMode: TermActionMode;
  targetTermId: string;

  // --- Mutateurs synchrones ---
  setTaxonomy: (taxonomySlug: string) => void;
  setSearch: (search: string) => void;
  setName: (name: string) => void;
  setSlug: (slug: string) => void;
  setParentId: (parentId: string | null) => void;
  setDescription: (description: string) => void;
  startCreate: () => void;
  startEdit: (term: TermRef) => void;
  openTermAction: (termId: string, mode: 'merge' | 'delete') => void;
  cancelTermAction: () => void;
  setTargetTermId: (targetId: string) => void;

  // --- Actions API ---
  fetchTerms: (taxonomySlug?: string) => Promise<void>;
  saveTerm: () => Promise<void>;
  confirmTermAction: () => Promise<void>;
}

export const useTaxonomiesStore = create<TaxonomiesStoreState>((set, get) => ({
  taxonomySlug: 'category',
  terms: [],
  loading: true,
  saving: false,
  error: '',
  search: '',

  editingTermId: null,
  name: '',
  slug: '',
  parentId: null,
  description: '',

  actionTermId: null,
  actionMode: null,
  targetTermId: '',

  setTaxonomy: (taxonomySlug) => {
    set({
      taxonomySlug,
      search: '',
      editingTermId: null,
      name: '',
      slug: '',
      parentId: null,
      description: '',
      actionTermId: null,
      actionMode: null,
      targetTermId: '',
      error: '',
    });
    void get().fetchTerms(taxonomySlug);
  },

  setSearch: (search) => set({ search }),
  setName: (name) => set({ name }),
  setSlug: (slug) => set({ slug }),
  setParentId: (parentId) => set({ parentId }),
  setDescription: (description) => set({ description }),

  startCreate: () =>
    set({
      editingTermId: null,
      name: '',
      slug: '',
      parentId: null,
      description: '',
      error: '',
    }),

  startEdit: (term) =>
    set({
      editingTermId: term.id,
      name: term.name,
      slug: term.slug,
      parentId: term.parent_id || null,
      description: term.description || '',
      actionTermId: null,
      actionMode: null,
      error: '',
    }),

  openTermAction: (termId, mode) => {
    const candidates = get().terms.filter((t) => t.id !== termId);
    set({
      actionTermId: termId,
      actionMode: mode,
      targetTermId: mode === 'merge' && candidates.length > 0 ? candidates[0].id : '',
      error: '',
    });
  },

  cancelTermAction: () =>
    set({
      actionTermId: null,
      actionMode: null,
      targetTermId: '',
    }),

  setTargetTermId: (targetTermId) => set({ targetTermId }),

  fetchTerms: async (taxonomySlug) => {
    const tax = taxonomySlug || get().taxonomySlug;
    set({ loading: true });
    try {
      const list = await api.terms.list(tax);
      set({ terms: list || [], loading: false, error: '' });
    } catch (err) {
      set({
        loading: false,
        error: err instanceof Error ? err.message : 'Chargement des termes impossible',
      });
    }
  },

  saveTerm: async () => {
    const { taxonomySlug, editingTermId, name, slug, parentId, description } = get();
    if (!name.trim()) {
      set({ error: 'Le nom du terme est obligatoire.' });
      return;
    }

    set({ saving: true, error: '' });
    try {
      if (editingTermId) {
        await api.terms.update(editingTermId, {
          name: name.trim(),
          slug: slug.trim() || undefined,
          description: description.trim(),
          parent_id: taxonomySlug === 'category' ? parentId : null,
        });
      } else {
        await api.terms.create({
          taxonomy: taxonomySlug,
          name: name.trim(),
          slug: slug.trim() || undefined,
          description: description.trim(),
          parent_id: taxonomySlug === 'category' ? parentId : null,
        });
      }
      set({
        saving: false,
        editingTermId: null,
        name: '',
        slug: '',
        parentId: null,
        description: '',
      });
      await get().fetchTerms(taxonomySlug);
    } catch (err) {
      set({
        saving: false,
        error: err instanceof Error ? err.message : 'Enregistrement impossible',
      });
    }
  },

  confirmTermAction: async () => {
    const { taxonomySlug, actionTermId, actionMode, targetTermId, editingTermId } = get();
    if (!actionTermId || !actionMode) return;
    if (actionMode === 'merge' && !targetTermId) {
      set({ error: 'Veuillez choisir le terme cible pour la fusion.' });
      return;
    }

    set({ saving: true, error: '' });
    try {
      if (actionMode === 'merge') {
        await api.terms.merge(actionTermId, targetTermId);
      } else {
        await api.terms.delete(actionTermId, targetTermId || undefined);
      }
      set({
        saving: false,
        actionTermId: null,
        actionMode: null,
        targetTermId: '',
        editingTermId: editingTermId === actionTermId ? null : editingTermId,
      });
      await get().fetchTerms(taxonomySlug);
    } catch (err) {
      set({
        saving: false,
        error: err instanceof Error ? err.message : 'Opération impossible',
      });
    }
  },
}));

