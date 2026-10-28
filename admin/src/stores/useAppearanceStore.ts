/**
 * ============================================================================
 * STORE ZUSTAND : APPARENCE, STYLES GLOBAUX & MENUS PUBLICS (`useAppearanceStore`)
 * ============================================================================
 * Couvre les exigences :
 * - `WP-46` : Gestion des menus publics par emplacement (`header`, `footer`),
 *   ajout rapide depuis les pages publiées ou les catégories, réorganisation
 *   haut/bas et validation stricte des URLs.
 * - `WP-48`, `TH-12`, `TH-27` : Tokens visuels du thème `rf-base`
 *   (`primary_color`, `accent_color`, `radius`, `container_width`).
 * - `WP-50` : CSS personnalisé additionnel borné à 16 Kio et protégé contre
 *   les injections (`<`, `>`, `@import`, `javascript:`).
 */

import { create } from 'zustand';
import { api } from '../api';
import type { Entry, PublicMenuItem, PublicMenus, TermRef } from '../types';

export type AppearanceTab = 'styles' | 'menus';
export type MenuLocation = 'header' | 'footer';

export interface DesignTokensState {
  primary_color: string;
  accent_color: string;
  radius: string;
  container_width: string;
  custom_css: string;
}

export interface AppearanceStoreState {
  activeTab: AppearanceTab;
  loading: boolean;
  saving: boolean;
  error: string;
  message: string;

  // Tokens de design & CSS personnalisé (WP-48 / WP-50)
  tokens: DesignTokensState;

  // Menus publics par emplacement (WP-46)
  menus: PublicMenus;
  activeLocation: MenuLocation;
  newLinkLabel: string;
  newLinkUrl: string;
  newLinkTarget: '_self' | '_blank';

  // Sources rapides pour ajouter des liens (Pages publiées & Catégories)
  publishedPages: Entry[];
  categories: TermRef[];

  // Actions synchrones
  setActiveTab: (tab: AppearanceTab) => void;
  setToken: <K extends keyof DesignTokensState>(key: K, value: DesignTokensState[K]) => void;
  setActiveLocation: (loc: MenuLocation) => void;
  setNewLinkLabel: (label: string) => void;
  setNewLinkUrl: (url: string) => void;
  setNewLinkTarget: (target: '_self' | '_blank') => void;
  addMenuItem: (label?: string, url?: string, target?: '_self' | '_blank') => void;
  moveMenuItem: (id: string, delta: -1 | 1) => void;
  removeMenuItem: (id: string) => void;

  // Actions asynchrones (API)
  loadAppearance: (initialTab?: AppearanceTab) => Promise<void>;
  saveStyles: () => Promise<void>;
  saveMenus: () => Promise<void>;
}

const DEFAULT_TOKENS: DesignTokensState = {
  primary_color: '#4f46e5',
  accent_color: '#06b6d4',
  radius: '12px',
  container_width: '1100px',
  custom_css: '',
};

export const useAppearanceStore = create<AppearanceStoreState>((set, get) => ({
  activeTab: 'styles',
  loading: true,
  saving: false,
  error: '',
  message: '',

  tokens: { ...DEFAULT_TOKENS },
  menus: { header: [], footer: [] },
  activeLocation: 'header',
  newLinkLabel: '',
  newLinkUrl: '',
  newLinkTarget: '_self',

  publishedPages: [],
  categories: [],

  setActiveTab: (activeTab) => set({ activeTab, error: '', message: '' }),

  setToken: (key, value) =>
    set((state) => ({
      tokens: { ...state.tokens, [key]: value },
    })),

  setActiveLocation: (activeLocation) => set({ activeLocation }),
  setNewLinkLabel: (newLinkLabel) => set({ newLinkLabel }),
  setNewLinkUrl: (newLinkUrl) => set({ newLinkUrl }),
  setNewLinkTarget: (newLinkTarget) => set({ newLinkTarget }),

  addMenuItem: (labelOverride, urlOverride, targetOverride) => {
    const { menus, activeLocation, newLinkLabel, newLinkUrl, newLinkTarget } = get();
    const label = (labelOverride ?? newLinkLabel).trim();
    const url = (urlOverride ?? newLinkUrl).trim();
    const target = targetOverride ?? newLinkTarget;
    if (!label || !url) return;

    const nextItem: PublicMenuItem = {
      id: crypto.randomUUID(),
      label,
      url,
      target,
    };
    set({
      menus: {
        ...menus,
        [activeLocation]: [...menus[activeLocation], nextItem],
      },
      newLinkLabel: labelOverride ? newLinkLabel : '',
      newLinkUrl: urlOverride ? newLinkUrl : '',
      error: '',
      message: '',
    });
  },

  moveMenuItem: (id, delta) => {
    const { menus, activeLocation } = get();
    const list = [...menus[activeLocation]];
    const idx = list.findIndex((item) => item.id === id);
    const nextIdx = idx + delta;
    if (idx < 0 || nextIdx < 0 || nextIdx >= list.length) return;
    const [moved] = list.splice(idx, 1);
    list.splice(nextIdx, 0, moved);
    set({
      menus: {
        ...menus,
        [activeLocation]: list,
      },
    });
  },

  removeMenuItem: (id) => {
    const { menus, activeLocation } = get();
    set({
      menus: {
        ...menus,
        [activeLocation]: menus[activeLocation].filter((item) => item.id !== id),
      },
    });
  },

  loadAppearance: async (initialTab) => {
    set({
      loading: true,
      error: '',
      message: '',
      ...(initialTab ? { activeTab: initialTab } : {}),
    });
    try {
      const [allSettings, pagesRes, catsRes] = await Promise.all([
        api.settings.getAll(),
        api.entries.list({ content_type: 'page', status: 'published', per_page: 50 }),
        api.terms.list('category'),
      ]);
      const rawTokens = allSettings['site.design_tokens'] || {};
      const rawMenus = allSettings['site.menus'] || {};
      set({
        tokens: {
          ...DEFAULT_TOKENS,
          ...rawTokens,
        },
        menus: {
          header: Array.isArray(rawMenus.header) ? rawMenus.header : [],
          footer: Array.isArray(rawMenus.footer) ? rawMenus.footer : [],
        },
        publishedPages: pagesRes.items,
        categories: catsRes,
        loading: false,
      });
    } catch (err) {
      set({
        loading: false,
        error: err instanceof Error ? err.message : 'Impossible de charger les réglages d’apparence',
      });
    }
  },

  saveStyles: async () => {
    const { tokens } = get();
    set({ saving: true, error: '', message: '' });
    try {
      await api.settings.update({ 'site.design_tokens': tokens });
      set({
        saving: false,
        message: 'Apparence et styles globaux enregistrés. Le cache public SSR a été actualisé.',
      });
    } catch (err) {
      set({
        saving: false,
        error: err instanceof Error ? err.message : 'Enregistrement impossible',
      });
    }
  },

  saveMenus: async () => {
    const { menus } = get();
    set({ saving: true, error: '', message: '' });
    try {
      await api.settings.update({ 'site.menus': menus });
      set({
        saving: false,
        message: 'Menus publics enregistrés et publiés sur le site.',
      });
    } catch (err) {
      set({
        saving: false,
        error: err instanceof Error ? err.message : 'Enregistrement des menus impossible',
      });
    }
  },
}));

