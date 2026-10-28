/**
 * ============================================================================
 * STORE ZUSTAND : SHELL DU STUDIO & SESSION (`useStudioStore`)
 * ============================================================================
 * Responsabilités :
 * - État d'installation et d'authentification (`user`, `isInstalled`, `loading`).
 * - Navigation par hash (`currentView`, `mobileNavigation`, garde `unsaved`).
 * - Projection du menu d'administration (`navigation`, `navigationError`) filtré
 *   par rôle et modules actifs (`WP-08`, `WP-10`).
 * - Métadonnées transverses (`contentTypes`, `stats`) et ouverture de la modale IA.
 *
 * Règle d'usage : consommer via des sélecteurs atomiques `useStudioStore(s => s.xxx)`
 * pour éviter les re-rendus inutiles dans l'arbre React.
 */

import { create } from 'zustand';
import { api, setToken } from '../api';
import { studioMessages } from '../i18n/messages.fr';
import type { AdminNavigation, ContentType, DashboardStats, User } from '../types';

/** Extrait la route logique depuis `window.location.hash` en ignorant les jetons d'installation. */
export function routeFromHash(): string {
  try {
    const view = decodeURIComponent(window.location.hash.slice(1));
    return view.startsWith('setup=') ? 'dashboard' : view || 'dashboard';
  } catch {
    return 'dashboard';
  }
}

export interface StudioStoreState {
  // --- État de session et d'installation ---
  user: User | null;
  isInstalled: boolean | null;
  loading: boolean;
  connectionError: string;

  // --- Navigation et garde d'édition ---
  currentView: string;
  mobileNavigation: boolean;
  unsaved: boolean;
  navState: { userId: string; data: AdminNavigation } | null;
  navigationError: string;

  // --- Données transverses du Studio ---
  contentTypes: ContentType[];
  stats: DashboardStats | null;
  isAiModalOpen: boolean;

  // --- Actions synchrones ---
  setUser: (user: User | null) => void;
  setInstalledSession: (token: string, user: User) => void;
  setUnsaved: (unsaved: boolean) => void;
  setMobileNavigation: (open: boolean | ((prev: boolean) => boolean)) => void;
  setAiModalOpen: (open: boolean | ((prev: boolean) => boolean)) => void;
  updateNavigationData: (data: AdminNavigation) => void;
  syncFromHash: () => void;
  navigate: (view: string, afterSave?: boolean) => void;
  logout: () => void;

  // --- Actions asynchrones (API) ---
  checkAuth: () => Promise<void>;
  refreshData: () => Promise<void>;
}

export const useStudioStore = create<StudioStoreState>((set, get) => ({
  user: null,
  isInstalled: null,
  loading: true,
  connectionError: '',

  currentView: routeFromHash(),
  mobileNavigation: false,
  unsaved: false,
  navState: null,
  navigationError: '',

  contentTypes: [],
  stats: null,
  isAiModalOpen: false,

  setUser: (user) => set({ user }),

  setInstalledSession: (token, user) => {
    setToken(token);
    set({ user, isInstalled: true });
    void get().refreshData();
  },

  setUnsaved: (unsaved) => set({ unsaved }),

  setMobileNavigation: (open) =>
    set((state) => ({
      mobileNavigation: typeof open === 'function' ? open(state.mobileNavigation) : open,
    })),

  setAiModalOpen: (open) =>
    set((state) => ({
      isAiModalOpen: typeof open === 'function' ? open(state.isAiModalOpen) : open,
    })),

  updateNavigationData: (data) => {
    const currentUser = get().user;
    if (currentUser) {
      set({ navState: { userId: currentUser.id, data } });
    }
  },

  syncFromHash: () => {
    set({ currentView: routeFromHash() });
  },

  navigate: (view, afterSave = false) => {
    const { unsaved } = get();
    if (unsaved && !afterSave && !window.confirm(studioMessages.unsavedConfirm)) {
      return;
    }
    set({
      unsaved: false,
      currentView: view,
      mobileNavigation: false,
    });
    window.location.hash = view;
  },

  logout: () => {
    setToken(null);
    set({
      user: null,
      navState: null,
      unsaved: false,
    });
  },

  checkAuth: async () => {
    set({ loading: true, connectionError: '' });
    try {
      const statusRes = await api.setup.status();
      if (!statusRes.is_installed) {
        set({ isInstalled: false, user: null, loading: false });
        return;
      }

      try {
        const me = await api.auth.me();
        set({ isInstalled: true, user: me, loading: false });
        await get().refreshData();
      } catch (error) {
        if (error instanceof Error && 'status' in error && (error as { status?: number }).status === 401) {
          set({ isInstalled: true, user: null, loading: false });
        } else {
          throw error;
        }
      }
    } catch (error) {
      set({
        connectionError: error instanceof Error ? error.message : 'Connexion au site impossible',
        loading: false,
      });
    }
  },

  refreshData: async () => {
    const userId = get().user?.id;
    if (!userId) return;
    try {
      const [cts, st, nav] = await Promise.all([
        api.contentTypes.list(),
        api.stats.dashboard(),
        api.admin.navigation(),
      ]);
      set({
        contentTypes: cts,
        stats: st,
        navState: { userId, data: nav },
        navigationError: '',
      });
    } catch (err) {
      set({
        navState: null,
        navigationError: err instanceof Error ? err.message : 'Menu indisponible',
      });
    }
  },
}));

