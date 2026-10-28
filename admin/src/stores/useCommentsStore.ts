/**
 * ============================================================================
 * STORE ZUSTAND : GESTION DES COMMENTAIRES & MODÉRATION (`useCommentsStore`)
 * ============================================================================
 * Tâches : `WP-42` à `WP-45` (Cycle de vie, modération, réponses, filtrage,
 * recherche, actions par lot, corbeille et conformité RGPD).
 */

import { create } from 'zustand';
import { api } from '../api';
import type { CommentItem, CommentStatus, CommentCounts } from '../types';

export type CommentFilterTab = 'all' | 'pending' | 'approved' | 'spam' | 'trash';

export interface CommentsStoreState {
  items: CommentItem[];
  counts: CommentCounts;
  activeTab: CommentFilterTab;
  search: string;
  page: number;
  perPage: number;
  total: number;
  totalPages: number;
  selectedIds: string[];
  loading: boolean;
  actionLoading: boolean;
  error: string | null;
  replyingComment: CommentItem | null;
  editingComment: CommentItem | null;

  loadComments: () => Promise<void>;
  loadCounts: () => Promise<void>;
  setActiveTab: (tab: CommentFilterTab) => void;
  setSearch: (search: string) => void;
  setPage: (page: number) => void;
  toggleSelect: (id: string) => void;
  toggleSelectAll: () => void;
  clearSelection: () => void;
  updateStatus: (id: string, status: CommentStatus) => Promise<void>;
  batchUpdateStatus: (status: CommentStatus) => Promise<void>;
  updateCommentContent: (
    id: string,
    data: {
      content: string;
      author_name?: string;
      author_email?: string;
      author_url?: string;
    }
  ) => Promise<void>;
  submitReply: (id: string, content: string) => Promise<void>;
  deleteComment: (id: string) => Promise<void>;
  emptyTrash: () => Promise<void>;
  setReplyingComment: (comment: CommentItem | null) => void;
  setEditingComment: (comment: CommentItem | null) => void;
}

const initialCounts: CommentCounts = {
  all: 0,
  pending: 0,
  approved: 0,
  spam: 0,
  trash: 0,
};

export const useCommentsStore = create<CommentsStoreState>((set, get) => ({
  items: [],
  counts: initialCounts,
  activeTab: 'all',
  search: '',
  page: 1,
  perPage: 20,
  total: 0,
  totalPages: 1,
  selectedIds: [],
  loading: false,
  actionLoading: false,
  error: null,
  replyingComment: null,
  editingComment: null,

  loadCounts: async () => {
    try {
      const counts = await api.comments.counts();
      set({ counts });
    } catch {
      // Échec silencieux pour les compteurs
    }
  },

  loadComments: async () => {
    const { activeTab, search, page, perPage } = get();
    set({ loading: true, error: null });
    try {
      const statusParam = activeTab === 'all' ? undefined : (activeTab as CommentStatus);
      const res = await api.comments.list({
        status: statusParam,
        search: search.trim() || undefined,
        page,
        per_page: perPage,
      });

      set({
        items: res.items,
        total: res.total,
        totalPages: res.total_pages,
        loading: false,
      });

      // Actualiser les compteurs en parallèle
      get().loadCounts();
    } catch (err: any) {
      set({
        error: err.message || 'Impossible de charger les commentaires.',
        loading: false,
      });
    }
  },

  setActiveTab: (tab) => {
    set({ activeTab: tab, page: 1, selectedIds: [], error: null });
    get().loadComments();
  },

  setSearch: (search) => {
    set({ search, page: 1, selectedIds: [] });
    get().loadComments();
  },

  setPage: (page) => {
    set({ page, selectedIds: [] });
    get().loadComments();
  },

  toggleSelect: (id) => {
    const { selectedIds } = get();
    if (selectedIds.includes(id)) {
      set({ selectedIds: selectedIds.filter((item) => item !== id) });
    } else {
      set({ selectedIds: [...selectedIds, id] });
    }
  },

  toggleSelectAll: () => {
    const { items, selectedIds } = get();
    if (selectedIds.length === items.length) {
      set({ selectedIds: [] });
    } else {
      set({ selectedIds: items.map((i) => i.id) });
    }
  },

  clearSelection: () => set({ selectedIds: [] }),

  updateStatus: async (id, status) => {
    set({ actionLoading: true });
    try {
      await api.comments.updateStatus(id, status);
      await get().loadComments();
      set({ actionLoading: false });
    } catch (err: any) {
      set({
        actionLoading: false,
        error: err.message || 'Erreur lors du changement de statut.',
      });
    }
  },

  batchUpdateStatus: async (status) => {
    const { selectedIds } = get();
    if (selectedIds.length === 0) return;
    set({ actionLoading: true });
    try {
      await api.comments.batchUpdate(selectedIds, status);
      set({ selectedIds: [], actionLoading: false });
      await get().loadComments();
    } catch (err: any) {
      set({
        actionLoading: false,
        error: err.message || 'Erreur lors de l’action groupée.',
      });
    }
  },

  updateCommentContent: async (id, data) => {
    set({ actionLoading: true });
    try {
      await api.comments.update(id, data);
      set({ editingComment: null, actionLoading: false });
      await get().loadComments();
    } catch (err: any) {
      set({
        actionLoading: false,
        error: err.message || 'Erreur lors de la modification du commentaire.',
      });
    }
  },

  submitReply: async (id, content) => {
    set({ actionLoading: true });
    try {
      await api.comments.reply(id, content);
      set({ replyingComment: null, actionLoading: false });
      await get().loadComments();
    } catch (err: any) {
      set({
        actionLoading: false,
        error: err.message || 'Erreur lors de la publication de la réponse.',
      });
    }
  },

  deleteComment: async (id) => {
    set({ actionLoading: true });
    try {
      await api.comments.delete(id);
      await get().loadComments();
      set({ actionLoading: false });
    } catch (err: any) {
      set({
        actionLoading: false,
        error: err.message || 'Erreur lors de la suppression définitive.',
      });
    }
  },

  emptyTrash: async () => {
    set({ actionLoading: true });
    try {
      await api.comments.emptyTrash();
      await get().loadComments();
      set({ actionLoading: false });
    } catch (err: any) {
      set({
        actionLoading: false,
        error: err.message || 'Erreur lors du vidage de la corbeille.',
      });
    }
  },

  setReplyingComment: (comment) => set({ replyingComment: comment, editingComment: null }),
  setEditingComment: (comment) => set({ editingComment: comment, replyingComment: null }),
}));
