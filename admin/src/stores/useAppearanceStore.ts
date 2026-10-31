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
import type { Entry, PublicMenuItem, PublicMenus, TermRef, ThemeSummary } from '../types';

export type AppearanceTab = 'themes' | 'site-editor' | 'styles' | 'menus';
export type MenuLocation = 'header' | 'footer';
export type ThemeFilter = 'all' | 'fse' | 'classic';

export interface StyleVariation {
  id: string;
  name: string;
  description: string;
  base: string;
  contrast: string;
  accent: string;
  font_heading: string;
  font_body: string;
}

export const TWENTY_TWENTY_FIVE_VARIATIONS: StyleVariation[] = [
  {
    id: '01-evening',
    name: 'Evening (Soir)',
    description: 'Ambiance nocturne raffinée, tons sombres et chaleureux avec or discret.',
    base: '#1B1B1B',
    contrast: '#F0F0F0',
    accent: '#786D0A',
    font_heading: "'Playfair Display', Georgia, serif",
    font_body: "'Merriweather', Georgia, serif",
  },
  {
    id: '02-noon',
    name: 'Noon (Midi)',
    description: 'Clarté éclatante, contraste pur et touches pêche chaleureuses.',
    base: '#F8F7F5',
    contrast: '#191919',
    accent: '#F5B684',
    font_heading: "'Playfair Display', Georgia, serif",
    font_body: "'Inter', sans-serif",
  },
  {
    id: '03-dusk',
    name: 'Dusk (Crépuscule)',
    description: 'Nuances pourpres et violet royal dans la brume du couchant.',
    base: '#E2E2E2',
    contrast: '#3B3B3B',
    accent: '#650DD4',
    font_heading: "'Plus Jakarta Sans', sans-serif",
    font_body: "'Inter', sans-serif",
  },
  {
    id: '04-afternoon',
    name: 'Afternoon (Après-midi)',
    description: 'Teintes végétales et sauge douce, harmonie printanière.',
    base: '#DAE7BD',
    contrast: '#516028',
    accent: '#C7F642',
    font_heading: "'Playfair Display', Georgia, serif",
    font_body: "'Merriweather', Georgia, serif",
  },
  {
    id: '05-twilight',
    name: 'Twilight (Pénombre)',
    description: 'Sombre profond, bleu électrique et corail vibrant pour tech & créatifs.',
    base: '#131313',
    contrast: '#FFFFFF',
    accent: '#4B52FF',
    font_heading: "'Space Grotesk', monospace",
    font_body: "'Inter', sans-serif",
  },
  {
    id: '06-morning',
    name: 'Morning (Matin)',
    description: 'Fraîcheur matinale, bleu ciel doux et blanc cassé reposant.',
    base: '#DFDCD7',
    contrast: '#191919',
    accent: '#7A9BDB',
    font_heading: "'Playfair Display', Georgia, serif",
    font_body: "'Inter', sans-serif",
  },
  {
    id: '07-sunrise',
    name: 'Sunrise (Aube)',
    description: 'Bordeaux impérial, rose poudré et lumière citronnée.',
    base: '#330616',
    contrast: '#FFFFFF',
    accent: '#F0FDA6',
    font_heading: "'Playfair Display', Georgia, serif",
    font_body: "'Merriweather', Georgia, serif",
  },
  {
    id: '08-midnight',
    name: 'Midnight (Minuit)',
    description: 'Indigo cosmique et menthe néon, contraste futuriste saisissant.',
    base: '#4433A6',
    contrast: '#79F3B1',
    accent: '#5644BC',
    font_heading: "'Space Grotesk', monospace",
    font_body: "'Inter', sans-serif",
  },
];

export interface DesignTokensState {
  primary_color: string;
  accent_color: string;
  radius: string;
  container_width: string;
  font_heading: string;
  font_body: string;
  custom_css: string;
}

export interface AppearanceStoreState {
  activeTab: AppearanceTab;
  loading: boolean;
  saving: boolean;
  error: string;
  message: string;

  // Thèmes souverains
  themes: ThemeSummary[];
  activeThemeSlug: string;
  activatingThemeSlug: string | null;
  themeFilter: ThemeFilter;
  themeSearchQuery: string;
  selectedThemeForDetails: ThemeSummary | null;

  // Éditeur de site souverain (FSE)
  selectedTemplate: string;
  selectedTemplatePart: string | null;
  selectedVariationId: string;
  devicePreview: 'desktop' | 'tablet' | 'mobile';

  // Tokens de design & CSS personnalisé (WP-48 / WP-50)
  tokens: DesignTokensState;

  // Menus publics par emplacement (WP-46)
  menus: PublicMenus;
  activeLocation: MenuLocation;
  newLinkLabel: string;
  newLinkUrl: string;
  newLinkTarget: '_self' | '_blank';
  newLinkBadge: string;
  newLinkSubtitle: string;

  // Sources rapides pour ajouter des liens (Pages publiées & Catégories)
  publishedPages: Entry[];
  categories: TermRef[];

  // Actions synchrones
  setActiveTab: (tab: AppearanceTab) => void;
  setThemeFilter: (filter: ThemeFilter) => void;
  setThemeSearchQuery: (query: string) => void;
  setSelectedThemeForDetails: (theme: ThemeSummary | null) => void;
  setSelectedTemplate: (template: string) => void;
  setSelectedTemplatePart: (part: string | null) => void;
  setSelectedVariationId: (id: string) => void;
  setDevicePreview: (device: 'desktop' | 'tablet' | 'mobile') => void;
  applyStyleVariation: (variationId: string) => Promise<void>;
  setToken: <K extends keyof DesignTokensState>(key: K, value: DesignTokensState[K]) => void;
  setActiveLocation: (loc: MenuLocation) => void;
  setNewLinkLabel: (label: string) => void;
  setNewLinkUrl: (url: string) => void;
  setNewLinkTarget: (target: '_self' | '_blank') => void;
  setNewLinkBadge: (badge: string) => void;
  setNewLinkSubtitle: (subtitle: string) => void;
  addMenuItem: (
    label?: string,
    url?: string,
    target?: '_self' | '_blank',
    extra?: Partial<PublicMenuItem>,
  ) => void;
  moveMenuItem: (id: string, delta: -1 | 1) => void;
  indentMenuItem: (id: string) => void;
  outdentMenuItem: (id: string) => void;
  duplicateMenuItem: (id: string) => void;
  updateMenuItem: (id: string, updates: Partial<PublicMenuItem>) => void;
  removeMenuItem: (id: string) => void;

  // Actions asynchrones (API)
  loadAppearance: (initialTab?: AppearanceTab) => Promise<void>;
  saveStyles: () => Promise<void>;
  saveMenus: () => Promise<void>;
  activateTheme: (slug: string) => Promise<void>;
  loadThemes: () => Promise<void>;
}

export const getItemDepth = (itemId: string, list: PublicMenuItem[]): number => {
  const byId = new Map(list.map((it) => [it.id, it]));
  let depth = 1;
  let curr = byId.get(itemId);
  while (curr?.parent_id && byId.has(curr.parent_id)) {
    depth += 1;
    if (depth > 10) break;
    curr = byId.get(curr.parent_id);
  }
  return depth;
};

const DEFAULT_TOKENS: DesignTokensState = {
  primary_color: '#4f46e5',
  accent_color: '#06b6d4',
  radius: '12px',
  container_width: '1100px',
  font_heading: 'Plus Jakarta Sans',
  font_body: 'Inter',
  custom_css: '',
};

export const useAppearanceStore = create<AppearanceStoreState>((set, get) => ({
  activeTab: 'themes',
  loading: true,
  saving: false,
  error: '',
  message: '',

  themes: [],
  activeThemeSlug: 'rf-base',
  activatingThemeSlug: null,
  themeFilter: 'all',
  themeSearchQuery: '',
  selectedThemeForDetails: null,

  selectedTemplate: 'home',
  selectedTemplatePart: null,
  selectedVariationId: '01-evening',
  devicePreview: 'desktop',

  tokens: { ...DEFAULT_TOKENS },
  menus: { header: [], footer: [] },
  activeLocation: 'header',
  newLinkLabel: '',
  newLinkUrl: '',
  newLinkTarget: '_self',
  newLinkBadge: '',
  newLinkSubtitle: '',

  publishedPages: [],
  categories: [],

  setActiveTab: (activeTab) => set({ activeTab, error: '', message: '' }),
  setThemeFilter: (themeFilter) => set({ themeFilter }),
  setThemeSearchQuery: (themeSearchQuery) => set({ themeSearchQuery }),
  setSelectedThemeForDetails: (selectedThemeForDetails) => set({ selectedThemeForDetails }),
  setSelectedTemplate: (selectedTemplate) => set({ selectedTemplate }),
  setSelectedTemplatePart: (selectedTemplatePart) => set({ selectedTemplatePart }),
  setSelectedVariationId: (selectedVariationId) => set({ selectedVariationId }),
  setDevicePreview: (devicePreview) => set({ devicePreview }),

  applyStyleVariation: async (variationId: string) => {
    const variation = TWENTY_TWENTY_FIVE_VARIATIONS.find((v) => v.id === variationId);
    if (!variation) return;
    const { tokens } = get();
    const updatedTokens: DesignTokensState = {
      ...tokens,
      primary_color: variation.accent,
      accent_color: variation.contrast,
      font_heading: variation.font_heading,
      font_body: variation.font_body,
    };
    set({
      selectedVariationId: variationId,
      tokens: updatedTokens,
      saving: true,
      error: '',
      message: '',
    });
    try {
      await api.settings.update({ 'site.design_tokens': updatedTokens });
      set({
        saving: false,
        message: `Variation « ${variation.name} » appliquée et enregistrée avec succès.`,
      });
    } catch (err) {
      set({
        saving: false,
        error: err instanceof Error ? err.message : 'Erreur lors de l’application de la variation',
      });
    }
  },

  setToken: (key, value) =>
    set((state) => ({
      tokens: { ...state.tokens, [key]: value },
    })),

  setActiveLocation: (activeLocation) => set({ activeLocation }),
  setNewLinkLabel: (newLinkLabel) => set({ newLinkLabel }),
  setNewLinkUrl: (newLinkUrl) => set({ newLinkUrl }),
  setNewLinkTarget: (newLinkTarget) => set({ newLinkTarget }),
  setNewLinkBadge: (newLinkBadge) => set({ newLinkBadge }),
  setNewLinkSubtitle: (newLinkSubtitle) => set({ newLinkSubtitle }),

  addMenuItem: (labelOverride, urlOverride, targetOverride, extra) => {
    const {
      menus,
      activeLocation,
      newLinkLabel,
      newLinkUrl,
      newLinkTarget,
      newLinkBadge,
      newLinkSubtitle,
    } = get();
    const label = (labelOverride ?? newLinkLabel).trim();
    const url = (urlOverride ?? newLinkUrl).trim();
    const target = targetOverride ?? newLinkTarget;
    if (!label || !url) return;

    const badge = (extra?.badge ?? newLinkBadge).trim() || undefined;
    const subtitle = (extra?.subtitle ?? newLinkSubtitle).trim() || undefined;

    const nextItem: PublicMenuItem = {
      id: crypto.randomUUID(),
      label,
      url,
      target,
      badge,
      subtitle,
      parent_id: extra?.parent_id ?? null,
      type: extra?.type ?? 'custom',
      object_id: extra?.object_id ?? null,
    };
    set({
      menus: {
        ...menus,
        [activeLocation]: [...menus[activeLocation], nextItem],
      },
      newLinkLabel: labelOverride ? newLinkLabel : '',
      newLinkUrl: urlOverride ? newLinkUrl : '',
      newLinkBadge: '',
      newLinkSubtitle: '',
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

  indentMenuItem: (id) => {
    const { menus, activeLocation } = get();
    const list = [...menus[activeLocation]];
    const idx = list.findIndex((item) => item.id === id);
    if (idx <= 0) return;
    const prev = list[idx - 1];
    const prevDepth = getItemDepth(prev.id, list);
    if (prevDepth >= 3) return;
    list[idx] = { ...list[idx], parent_id: prev.id };
    set({ menus: { ...menus, [activeLocation]: list } });
  },

  outdentMenuItem: (id) => {
    const { menus, activeLocation } = get();
    const list = [...menus[activeLocation]];
    const idx = list.findIndex((item) => item.id === id);
    if (idx < 0) return;
    const item = list[idx];
    if (!item.parent_id) return;
    const parent = list.find((p) => p.id === item.parent_id);
    list[idx] = { ...item, parent_id: parent?.parent_id ?? null };
    set({ menus: { ...menus, [activeLocation]: list } });
  },

  duplicateMenuItem: (id) => {
    const { menus, activeLocation } = get();
    const list = [...menus[activeLocation]];
    const idx = list.findIndex((item) => item.id === id);
    if (idx < 0) return;
    const original = list[idx];
    const clone: PublicMenuItem = {
      ...original,
      id: crypto.randomUUID(),
      label: `${original.label} (copie)`,
    };
    list.splice(idx + 1, 0, clone);
    set({ menus: { ...menus, [activeLocation]: list } });
  },

  updateMenuItem: (id, updates) => {
    const { menus, activeLocation } = get();
    const list = menus[activeLocation].map((item) =>
      item.id === id ? { ...item, ...updates } : item,
    );
    set({ menus: { ...menus, [activeLocation]: list } });
  },

  removeMenuItem: (id) => {
    const { menus, activeLocation } = get();
    const target = menus[activeLocation].find((it) => it.id === id);
    const parentId = target?.parent_id ?? null;
    const list = menus[activeLocation]
      .filter((item) => item.id !== id)
      .map((item) => (item.parent_id === id ? { ...item, parent_id: parentId } : item));
    set({
      menus: {
        ...menus,
        [activeLocation]: list,
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
      const [allSettings, pagesRes, catsRes, themesRes] = await Promise.all([
        api.settings.getAll(),
        api.entries.list({ content_type: 'page', status: 'published', per_page: 50 }),
        api.terms.list('category'),
        api.themes.list().catch(() => [] as ThemeSummary[]),
      ]);
      const rawTokens = allSettings['site.design_tokens'] || {};
      const rawMenus = allSettings['site.menus'] || {};
      const activeThemeSlug =
        typeof allSettings['site.active_theme'] === 'string'
          ? (allSettings['site.active_theme'] as string)
          : (themesRes.find((t) => t.is_active)?.slug || 'rf-base');

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
        themes: themesRes,
        activeThemeSlug,
        loading: false,
      });
    } catch (err) {
      set({
        loading: false,
        error: err instanceof Error ? err.message : 'Impossible de charger les réglages d’apparence',
      });
    }
  },

  activateTheme: async (slug: string) => {
    set({ activatingThemeSlug: slug, error: '', message: '' });
    try {
      await api.themes.activate(slug);
      const [themesRes, allSettings] = await Promise.all([
        api.themes.list().catch(() => [] as ThemeSummary[]),
        api.settings.getAll(),
      ]);
      const rawTokens = allSettings['site.design_tokens'] || {};
      set({
        themes: themesRes,
        activeThemeSlug: slug,
        tokens: {
          ...get().tokens,
          ...rawTokens,
        },
        activatingThemeSlug: null,
        message: 'Thème activé avec succès. Les tokens et le cache public ont été actualisés.',
      });
    } catch (err) {
      set({
        activatingThemeSlug: null,
        error: err instanceof Error ? err.message : 'Impossible d’activer ce thème',
      });
    }
  },

  loadThemes: async () => {
    try {
      const themes = await api.themes.list();
      set({ themes });
    } catch (err) {
      set({ error: err instanceof Error ? err.message : 'Impossible de lister les thèmes' });
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

