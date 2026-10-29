/**
 * ============================================================================
 * STORE ZUSTAND : RÉGLAGES GÉNÉRAUX, LECTURE & ÉCRITURE (`useSettingsStore`)
 * ============================================================================
 * Tâches : `WP-69` à `WP-81` (Identité, e-mail d'administration, fuseau,
 * inscription/rôle par défaut, accueil statique ou derniers articles,
 * pagination publique, visibilité moteurs `noindex` et défauts d'écriture).
 */

import { create } from 'zustand';
import { api } from '../api';
import type { Entry, TermRef } from '../types';

export type SettingsTab =
  | 'general'
  | 'reading'
  | 'writing'
  | 'permalinks'
  | 'discussion'
  | 'media';

interface SettingsState {
  activeTab: SettingsTab;
  settings: Record<string, any>;
  pages: Entry[];
  categories: TermRef[];
  loading: boolean;
  saving: boolean;
  saved: boolean;
  uploadingIcon: boolean;
  error: string | null;

  setActiveTab: (tab: SettingsTab) => void;
  loadAll: () => Promise<void>;
  setSetting: (key: string, value: any) => void;
  saveSettings: () => Promise<void>;
  uploadSiteIcon: (file: File) => Promise<void>;
  deleteSiteIcon: () => Promise<void>;
}

export const useSettingsStore = create<SettingsState>((set, get) => ({
  activeTab: 'general',
  settings: {},
  pages: [],
  categories: [],
  loading: false,
  saving: false,
  saved: false,
  uploadingIcon: false,
  error: null,

  setActiveTab: (tab) => set({ activeTab: tab, error: null }),

  loadAll: async () => {
    set({ loading: true, error: null });
    try {
      const [settings, pagesRes, categories] = await Promise.all([
        api.settings.getAll(),
        api.entries.list({ content_type: 'page', status: 'published', per_page: 100 }),
        api.terms.list('category'),
      ]);
      const allPages = [...pagesRes.items];
      for (let page = 2; page <= pagesRes.total_pages; page += 1) {
        const next = await api.entries.list({ content_type: 'page', status: 'published', per_page: 100, page });
        allPages.push(...next.items);
      }
      const now = Date.now();
      set({
        settings,
        pages: allPages.filter((p) => p.published_at && new Date(p.published_at).getTime() <= now
          && (!p.scheduled_unpublish_at || new Date(p.scheduled_unpublish_at).getTime() > now)),
        categories,
        loading: false,
      });
    } catch (err: any) {
      set({
        error: err.message || 'Impossible de charger les réglages.',
        loading: false,
      });
    }
  },

  setSetting: (key, value) => {
    set((s) => ({
      settings: {
        ...s.settings,
        [key]: value,
      },
      saved: false,
      error: null,
    }));
  },

  saveSettings: async () => {
    const { settings } = get();
    set({ saving: true, saved: false, error: null });
    try {
      const updated = await api.settings.update(settings);
      set({
        settings: updated,
        saving: false,
        saved: true,
      });
    } catch (err: any) {
      set({
        saving: false,
        error: err.message || 'Erreur lors de la sauvegarde des réglages.',
      });
    }
  },

  uploadSiteIcon: async (file: File) => {
    set({ uploadingIcon: true, error: null });
    try {
      const res = await api.settings.uploadSiteIcon(file);
      set((s) => ({
        settings: {
          ...s.settings,
          'site.icon': res.icon,
        },
        uploadingIcon: false,
        saved: true,
      }));
    } catch (err: any) {
      set({
        uploadingIcon: false,
        error: err.message || "Erreur lors du téléversement de l'icône.",
      });
    }
  },

  deleteSiteIcon: async () => {
    set({ uploadingIcon: true, error: null });
    try {
      await api.settings.deleteSiteIcon();
      set((s) => ({
        settings: {
          ...s.settings,
          'site.icon': null,
        },
        uploadingIcon: false,
        saved: true,
      }));
    } catch (err: any) {
      set({
        uploadingIcon: false,
        error: err.message || "Erreur lors de la suppression de l'icône.",
      });
    }
  },
}));

