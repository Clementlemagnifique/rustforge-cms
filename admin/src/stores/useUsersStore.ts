/**
 * ============================================================================
 * STORE ZUSTAND : UTILISATEURS, RÔLES, PROFIL & SESSIONS (`useUsersStore`)
 * ============================================================================
 * Tâches : `WP-52` (Liste, ajout et filtre par rôle), `WP-53` (Profil et mot de
 * passe), `WP-54` (Suppression avec réattribution des contenus), `WP-55`
 * (Sessions actives et révocation).
 */

import { create } from 'zustand';
import { api } from '../api';
import type { Role, User, UserSession } from '../types';

export type UsersTab = 'list' | 'new' | 'profile';

interface UsersState {
  activeTab: UsersTab;
  items: User[];
  total: number;
  loading: boolean;
  error: string | null;
  notice: string | null;

  // Filtres de liste (`WP-52`)
  search: string;
  roleFilter: Role | '';

  // Création d'un compte (`WP-52`)
  newUsername: string;
  newEmail: string;
  newDisplayName: string;
  newRole: Role;
  newPassword: string;
  creating: boolean;

  // Édition de compte / Mon profil (`WP-53`)
  editingUserId: string | null;
  editEmail: string;
  editDisplayName: string;
  editBio: string;
  editRole: Role;
  editPassword: string;
  savingProfile: boolean;

  // Sessions actives (`WP-55`)
  sessions: UserSession[];
  loadingSessions: boolean;
  revokingSessions: boolean;

  // Modale de suppression avec réattribution (`WP-54`)
  deletingUser: User | null;
  reassignTargetId: string;
  deleting: boolean;

  // Actions
  setActiveTab: (tab: UsersTab) => void;
  setSearch: (q: string) => void;
  setRoleFilter: (role: Role | '') => void;
  loadUsers: (currentUser: User) => Promise<void>;

  setNewField: (field: 'username' | 'email' | 'displayName' | 'role' | 'password', value: string) => void;
  createUser: (currentUser: User) => Promise<void>;

  selectUserForEdit: (target: User) => Promise<void>;
  setEditField: (field: 'email' | 'displayName' | 'bio' | 'role' | 'password', value: string) => void;
  saveEditedUser: (currentUser: User, onCurrentUserUpdated?: (u: User) => void) => Promise<void>;
  revokeTargetSessions: (userId: string) => Promise<void>;

  openDeleteModal: (user: User) => void;
  closeDeleteModal: () => void;
  setReassignTargetId: (id: string) => void;
  confirmDeleteUser: (currentUser: User) => Promise<void>;
  clearFeedback: () => void;
}

export const useUsersStore = create<UsersState>((set, get) => ({
  activeTab: 'list',
  items: [],
  total: 0,
  loading: false,
  error: null,
  notice: null,

  search: '',
  roleFilter: '',

  newUsername: '',
  newEmail: '',
  newDisplayName: '',
  newRole: 'author',
  newPassword: '',
  creating: false,

  editingUserId: null,
  editEmail: '',
  editDisplayName: '',
  editBio: '',
  editRole: 'author',
  editPassword: '',
  savingProfile: false,

  sessions: [],
  loadingSessions: false,
  revokingSessions: false,

  deletingUser: null,
  reassignTargetId: '',
  deleting: false,

  setActiveTab: (tab) => set({ activeTab: tab, error: null, notice: null }),

  setSearch: (q) => set({ search: q }),

  setRoleFilter: (role) => set({ roleFilter: role }),

  loadUsers: async (currentUser) => {
    set({ loading: true, error: null });
    try {
      if (currentUser.role === 'admin') {
        const { search, roleFilter } = get();
        const res = await api.users.list({
          search: search.trim() || undefined,
          role: roleFilter || undefined,
          per_page: 100,
        });
        set({ items: res.items, total: res.total, loading: false });
      } else {
        set({ items: [currentUser], total: 1, loading: false });
      }
    } catch (err: any) {
      set({ error: err.message || 'Impossible de charger les utilisateurs.', loading: false });
    }
  },

  setNewField: (field, value) => {
    if (field === 'username') set({ newUsername: value });
    else if (field === 'email') set({ newEmail: value });
    else if (field === 'displayName') set({ newDisplayName: value });
    else if (field === 'role') set({ newRole: value as Role });
    else if (field === 'password') set({ newPassword: value });
  },

  createUser: async (currentUser) => {
    const { newUsername, newEmail, newDisplayName, newRole, newPassword } = get();
    set({ creating: true, error: null, notice: null });
    try {
      const created = await api.users.create({
        username: newUsername.trim(),
        email: newEmail.trim(),
        display_name: newDisplayName.trim() || undefined,
        role: newRole,
        password: newPassword,
      });
      set({
        creating: false,
        newUsername: '',
        newEmail: '',
        newDisplayName: '',
        newRole: 'author',
        newPassword: '',
        activeTab: 'list',
        notice: `Compte « ${created.display_name} » (@${created.username}) créé avec succès.`,
      });
      await get().loadUsers(currentUser);
    } catch (err: any) {
      set({ creating: false, error: err.message || 'Échec de la création du compte.' });
    }
  },

  selectUserForEdit: async (target) => {
    set({
      editingUserId: target.id,
      editEmail: target.email,
      editDisplayName: target.display_name,
      editBio: target.bio || '',
      editRole: target.role,
      editPassword: '',
      activeTab: 'profile',
      loadingSessions: true,
      error: null,
      notice: null,
    });
    try {
      const sessions = await api.users.listSessions(target.id);
      set({ sessions, loadingSessions: false });
    } catch {
      set({ sessions: [], loadingSessions: false });
    }
  },

  setEditField: (field, value) => {
    if (field === 'email') set({ editEmail: value });
    else if (field === 'displayName') set({ editDisplayName: value });
    else if (field === 'bio') set({ editBio: value });
    else if (field === 'role') set({ editRole: value as Role });
    else if (field === 'password') set({ editPassword: value });
  },

  saveEditedUser: async (currentUser, onCurrentUserUpdated) => {
    const { editingUserId, editEmail, editDisplayName, editBio, editRole, editPassword } = get();
    const targetId = editingUserId || currentUser.id;
    set({ savingProfile: true, error: null, notice: null });
    try {
      const updated = await api.users.update(targetId, {
        email: editEmail.trim(),
        display_name: editDisplayName.trim(),
        bio: editBio,
        role: currentUser.role === 'admin' ? editRole : undefined,
        password: editPassword.trim() ? editPassword : undefined,
      });
      set({
        savingProfile: false,
        editPassword: '',
        notice: `Profil de « ${updated.display_name} » enregistré.`,
      });
      if (updated.id === currentUser.id && onCurrentUserUpdated) {
        onCurrentUserUpdated(updated);
      }
      await get().loadUsers(updated.id === currentUser.id ? updated : currentUser);
    } catch (err: any) {
      set({
        savingProfile: false,
        error: err.message || 'Échec de la mise à jour du profil.',
      });
    }
  },

  revokeTargetSessions: async (userId) => {
    set({ revokingSessions: true, error: null, notice: null });
    try {
      const res = await api.users.revokeSessions(userId);
      set({
        revokingSessions: false,
        sessions: [],
        notice: `${res.revoked} session(s) active(s) révoquée(s).`,
      });
    } catch (err: any) {
      set({
        revokingSessions: false,
        error: err.message || 'Impossible de révoquer les sessions.',
      });
    }
  },

  openDeleteModal: (user) =>
    set({
      deletingUser: user,
      reassignTargetId: '',
      error: null,
      notice: null,
    }),

  closeDeleteModal: () => set({ deletingUser: null, reassignTargetId: '' }),

  setReassignTargetId: (id) => set({ reassignTargetId: id }),

  confirmDeleteUser: async (currentUser) => {
    const { deletingUser, reassignTargetId } = get();
    if (!deletingUser) return;
    set({ deleting: true, error: null, notice: null });
    try {
      await api.users.delete(deletingUser.id, reassignTargetId || undefined);
      set({
        deleting: false,
        deletingUser: null,
        reassignTargetId: '',
        notice: `Compte « ${deletingUser.display_name} » supprimé.`,
      });
      await get().loadUsers(currentUser);
    } catch (err: any) {
      set({
        deleting: false,
        error: err.message || 'Impossible de supprimer cet utilisateur.',
      });
    }
  },

  clearFeedback: () => set({ error: null, notice: null }),
}));

