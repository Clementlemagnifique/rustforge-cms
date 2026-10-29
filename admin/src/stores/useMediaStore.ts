/**
 * ============================================================================
 * STORE ZUSTAND : MÉDIATHÈQUE & DOSSIERS VIRTUELS (`useMediaStore`)
 * ============================================================================
 * Couvre les exigences :
 * - `WP-37` : Bibliothèque en mode Grille ou Liste, recherche (`q`), filtre
 *   par type MIME (`kind`), filtrage par dossier virtuel (`folder_id`).
 * - `WP-38` : Téléversement dans le dossier actif et édition des métadonnées
 *   d'accessibilité (`alt`, `caption`, déplacement de dossier).
 * - `WP-40` : Suppression protégée contre la destruction d'un média utilisé
 *   dans un contenu actif (`409 Conflict` avec proposition de forçage explicite).
 * - `MOD-MED-01` / `MOD-MED-02` : Création et suppression de dossiers virtuels.
 */

import { create } from 'zustand';
import { api, ApiError } from '../api';
import type { Media, MediaFolder } from '../types';

export type MediaViewMode = 'grid' | 'list';

export interface MediaStoreState {
  items: Media[];
  total: number;
  folders: MediaFolder[];
  loading: boolean;
  uploading: boolean;
  savingMeta: boolean;
  rotating: boolean;
  error: string;
  message: string;

  // Filtres et mode d'affichage
  viewMode: MediaViewMode;
  searchQuery: string;
  kindFilter: string;
  selectedFolderId: string | null;
  newFolderName: string;

  // Inspecteur de média sélectionné
  selectedMedia: Media | null;
  editAlt: string;
  editCaption: string;
  editFolderId: string;

  // Actions synchrones
  setViewMode: (mode: MediaViewMode) => void;
  setSearchQuery: (q: string) => void;
  setKindFilter: (kind: string) => void;
  setSelectedFolderId: (folderId: string | null) => void;
  setNewFolderName: (name: string) => void;
  selectMedia: (media: Media | null) => void;
  setEditAlt: (alt: string) => void;
  setEditCaption: (caption: string) => void;
  setEditFolderId: (folderId: string) => void;

  // Actions asynchrones (API)
  fetchAll: () => Promise<void>;
  uploadFile: (file: File) => Promise<void>;
  createFolder: () => Promise<void>;
  deleteFolder: (folderId: string) => Promise<void>;
  saveSelectedMediaMeta: () => Promise<void>;
  rotateSelectedMedia: (degrees: number) => Promise<void>;
  restoreSelectedMedia: () => Promise<void>;
  deleteMedia: (id: string) => Promise<void>;
}

export const useMediaStore = create<MediaStoreState>((set, get) => ({
  items: [],
  total: 0,
  folders: [],
  loading: true,
  uploading: false,
  savingMeta: false,
  rotating: false,
  error: '',
  message: '',

  viewMode: 'grid',
  searchQuery: '',
  kindFilter: '',
  selectedFolderId: null,
  newFolderName: '',

  selectedMedia: null,
  editAlt: '',
  editCaption: '',
  editFolderId: '',

  setViewMode: (viewMode) => set({ viewMode }),
  setSearchQuery: (searchQuery) => set({ searchQuery }),
  setKindFilter: (kindFilter) => set({ kindFilter }),
  setSelectedFolderId: (selectedFolderId) => set({ selectedFolderId }),
  setNewFolderName: (newFolderName) => set({ newFolderName }),

  selectMedia: (media) =>
    set({
      selectedMedia: media,
      editAlt: media?.alt || '',
      editCaption: media?.caption || '',
      editFolderId: media?.folder_id || '',
      error: '',
      message: '',
    }),

  setEditAlt: (editAlt) => set({ editAlt }),
  setEditCaption: (editCaption) => set({ editCaption }),
  setEditFolderId: (editFolderId) => set({ editFolderId }),

  fetchAll: async () => {
    const { searchQuery, kindFilter, selectedFolderId, selectedMedia } = get();
    set({ loading: true, error: '' });
    try {
      const [mediaRes, foldersRes] = await Promise.all([
        api.media.list({
          q: searchQuery.trim() || undefined,
          kind: kindFilter || undefined,
          folder_id: selectedFolderId || undefined,
          per_page: 60,
        }),
        api.media.listFolders(),
      ]);
      const refreshedSelected = selectedMedia
        ? mediaRes.items.find((m) => m.id === selectedMedia.id) || null
        : null;
      set({
        items: mediaRes.items,
        total: mediaRes.total,
        folders: foldersRes,
        selectedMedia: refreshedSelected,
        loading: false,
      });
    } catch (err) {
      set({
        loading: false,
        error: err instanceof Error ? err.message : 'Impossible de charger la médiathèque',
      });
    }
  },

  uploadFile: async (file) => {
    const { selectedFolderId, fetchAll } = get();
    set({ uploading: true, error: '', message: '' });
    try {
      const created = await api.media.upload(file, selectedFolderId);
      await fetchAll();
      get().selectMedia(created);
      set({
        uploading: false,
        message: `Fichier « ${created.original_name} » téléversé avec succès.`,
      });
    } catch (err) {
      set({
        uploading: false,
        error: err instanceof Error ? err.message : 'Erreur de téléversement',
      });
    }
  },

  createFolder: async () => {
    const { newFolderName, fetchAll } = get();
    const trimmed = newFolderName.trim();
    if (!trimmed) return;
    set({ error: '', message: '' });
    try {
      const folder = await api.media.createFolder(trimmed, null);
      set({ newFolderName: '', selectedFolderId: folder.id });
      await fetchAll();
    } catch (err) {
      set({
        error: err instanceof Error ? err.message : 'Impossible de créer le dossier',
      });
    }
  },

  deleteFolder: async (folderId) => {
    const { selectedFolderId, fetchAll } = get();
    if (!window.confirm('Supprimer ce dossier virtuel ? Les médias qu’il contient seront replacés à la racine.')) {
      return;
    }
    try {
      await api.media.deleteFolder(folderId);
      if (selectedFolderId === folderId) {
        set({ selectedFolderId: null });
      }
      await fetchAll();
    } catch (err) {
      set({
        error: err instanceof Error ? err.message : 'Suppression du dossier impossible',
      });
    }
  },

  saveSelectedMediaMeta: async () => {
    const { selectedMedia, editAlt, editCaption, editFolderId, fetchAll } = get();
    if (!selectedMedia) return;
    set({ savingMeta: true, error: '', message: '' });
    try {
      const updated = await api.media.update(selectedMedia.id, {
        alt: editAlt,
        caption: editCaption,
        folder_id: editFolderId || null,
        clear_folder: !editFolderId,
      });
      await fetchAll();
      get().selectMedia(updated);
      set({
        savingMeta: false,
        message: 'Métadonnées du média enregistrées.',
      });
    } catch (err) {
      set({
        savingMeta: false,
        error: err instanceof Error ? err.message : 'Enregistrement impossible',
      });
    }
  },

  rotateSelectedMedia: async (degrees) => {
    const { selectedMedia, fetchAll } = get();
    if (!selectedMedia) return;
    set({ rotating: true, error: '', message: '' });
    try {
      const updated = await api.media.rotate(selectedMedia.id, degrees);
      await fetchAll();
      get().selectMedia(updated);
      set({
        rotating: false,
        message: `Image pivotée de ${degrees}° avec succès.`,
      });
    } catch (err) {
      set({
        rotating: false,
        error: err instanceof Error ? err.message : 'Impossible de pivoter l’image',
      });
    }
  },

  restoreSelectedMedia: async () => {
    const { selectedMedia, fetchAll } = get();
    if (!selectedMedia) return;
    if (
      !window.confirm(
        'Restaurer l’image originale ? Les transformations actuelles seront annulées.'
      )
    )
      return;
    set({ rotating: true, error: '', message: '' });
    try {
      const updated = await api.media.restore(selectedMedia.id);
      await fetchAll();
      get().selectMedia(updated);
      set({
        rotating: false,
        message: 'Image originale restaurée avec succès.',
      });
    } catch (err) {
      set({
        rotating: false,
        error: err instanceof Error ? err.message : 'Impossible de restaurer l’original',
      });
    }
  },

  deleteMedia: async (id) => {
    const { fetchAll } = get();
    if (!window.confirm('Supprimer ce média de la bibliothèque ?')) return;
    set({ error: '', message: '' });
    try {
      await api.media.delete(id, false);
      set({ selectedMedia: null });
      await fetchAll();
    } catch (err) {
      if (err instanceof ApiError && err.status === 409) {
        if (
          window.confirm(
            `${err.message}\n\nVoulez-vous forcer la suppression définitive malgré son utilisation ?`
          )
        ) {
          await api.media.delete(id, true);
          set({ selectedMedia: null });
          await fetchAll();
        }
        return;
      }
      set({
        error: err instanceof Error ? err.message : 'Suppression impossible',
      });
    }
  },
}));

