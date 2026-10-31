/**
 * ============================================================================
 * VUE UTILISATEURS, RÔLES, PROFIL & SESSIONS (`UsersView`)
 * ============================================================================
 * Tâches : `WP-52` (Liste, recherche, filtre par rôle, création), `WP-53`
 * (Profil personnel et mot de passe), `WP-54` (Suppression avec réattribution
 * des contenus), `WP-55` (Sessions actives et révocation).
 */

import React, { useEffect } from 'react';
import {
  Users,
  UserPlus,
  UserCheck,
  Shield,
  Trash2,
  Edit3,
  KeyRound,
  Search,
  CheckCircle2,
  AlertTriangle,
  LogOut,
} from 'lucide-react';
import { usersMessages } from '../i18n/messages.fr';
import { useUsersStore, type UsersTab } from '../stores/useUsersStore';
import { useStudioStore } from '../stores/useStudioStore';
import type { Role, User } from '../types';

interface UsersViewProps {
  initialTab?: UsersTab;
}

const ROLE_LABELS: Record<Role, string> = {
  admin: usersMessages.roleAdmin,
  editor: usersMessages.roleEditor,
  author: usersMessages.roleAuthor,
  subscriber: usersMessages.roleSubscriber,
};

const ROLE_BADGE_CLASSES: Record<Role, string> = {
  admin: 'bg-purple-500/15 text-purple-300 border-purple-500/30',
  editor: 'bg-indigo-500/15 text-indigo-300 border-indigo-500/30',
  author: 'bg-emerald-500/15 text-emerald-300 border-emerald-500/30',
  subscriber: 'bg-slate-500/15 text-slate-300 border-slate-500/30',
};

export const UsersView: React.FC<UsersViewProps> = ({ initialTab = 'list' }) => {
  const currentUser = useStudioStore((s) => s.user);
  const setUser = useStudioStore((s) => s.setUser);

  const activeTab = useUsersStore((s) => s.activeTab);
  const items = useUsersStore((s) => s.items);
  const loading = useUsersStore((s) => s.loading);
  const error = useUsersStore((s) => s.error);
  const notice = useUsersStore((s) => s.notice);

  const search = useUsersStore((s) => s.search);
  const roleFilter = useUsersStore((s) => s.roleFilter);

  const newUsername = useUsersStore((s) => s.newUsername);
  const newEmail = useUsersStore((s) => s.newEmail);
  const newDisplayName = useUsersStore((s) => s.newDisplayName);
  const newRole = useUsersStore((s) => s.newRole);
  const newPassword = useUsersStore((s) => s.newPassword);
  const creating = useUsersStore((s) => s.creating);

  const editingUserId = useUsersStore((s) => s.editingUserId);
  const editEmail = useUsersStore((s) => s.editEmail);
  const editDisplayName = useUsersStore((s) => s.editDisplayName);
  const editBio = useUsersStore((s) => s.editBio);
  const editRole = useUsersStore((s) => s.editRole);
  const editPassword = useUsersStore((s) => s.editPassword);
  const savingProfile = useUsersStore((s) => s.savingProfile);

  const sessions = useUsersStore((s) => s.sessions);
  const loadingSessions = useUsersStore((s) => s.loadingSessions);
  const revokingSessions = useUsersStore((s) => s.revokingSessions);

  const deletingUser = useUsersStore((s) => s.deletingUser);
  const reassignTargetId = useUsersStore((s) => s.reassignTargetId);
  const deleting = useUsersStore((s) => s.deleting);

  const setActiveTab = useUsersStore((s) => s.setActiveTab);
  const setSearch = useUsersStore((s) => s.setSearch);
  const setRoleFilter = useUsersStore((s) => s.setRoleFilter);
  const loadUsers = useUsersStore((s) => s.loadUsers);
  const setNewField = useUsersStore((s) => s.setNewField);
  const createUser = useUsersStore((s) => s.createUser);
  const selectUserForEdit = useUsersStore((s) => s.selectUserForEdit);
  const setEditField = useUsersStore((s) => s.setEditField);
  const saveEditedUser = useUsersStore((s) => s.saveEditedUser);
  const revokeTargetSessions = useUsersStore((s) => s.revokeTargetSessions);
  const openDeleteModal = useUsersStore((s) => s.openDeleteModal);
  const closeDeleteModal = useUsersStore((s) => s.closeDeleteModal);
  const setReassignTargetId = useUsersStore((s) => s.setReassignTargetId);
  const confirmDeleteUser = useUsersStore((s) => s.confirmDeleteUser);

  const isAdmin = currentUser?.role === 'admin';

  useEffect(() => {
    if (!currentUser) return;
    if (!isAdmin || initialTab === 'profile') {
      void selectUserForEdit(currentUser);
    } else {
      setActiveTab(initialTab);
    }
    void loadUsers(currentUser);
  }, [currentUser, initialTab, isAdmin, loadUsers, selectUserForEdit, setActiveTab]);

  useEffect(() => {
    if (!currentUser || !isAdmin) return;
    void loadUsers(currentUser);
  }, [currentUser, isAdmin, search, roleFilter, loadUsers]);

  if (!currentUser) return null;

  const targetProfileUser: User =
    items.find((u) => u.id === editingUserId) || currentUser;

  const reassignCandidates = items.filter((u) => u.id !== deletingUser?.id);

  return (
    <div className="rf-users-view space-y-6 max-w-6xl">
      {/* En-tête & Navigation par onglets */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
        <div>
          <h1>{usersMessages.title}</h1>
          <p className="text-xs text-slate-400">{usersMessages.subtitle}</p>
        </div>

        <div className="rf-view-tabs flex items-center gap-2 bg-[#0f1422] p-1 rounded-xl border border-white/5">
          {isAdmin && (
            <>
              <button
                type="button"
                onClick={() => setActiveTab('list')}
                className={`px-3 py-1.5 rounded-lg text-xs font-medium flex items-center gap-1.5 transition-colors ${
                  activeTab === 'list'
                    ? 'bg-indigo-600 text-white'
                    : 'text-slate-400 hover:text-white'
                }`}
              >
                <Users className="w-3.5 h-3.5" />
                <span>{usersMessages.tabList}</span>
              </button>
              <button
                type="button"
                onClick={() => setActiveTab('new')}
                className={`px-3 py-1.5 rounded-lg text-xs font-medium flex items-center gap-1.5 transition-colors ${
                  activeTab === 'new'
                    ? 'bg-indigo-600 text-white'
                    : 'text-slate-400 hover:text-white'
                }`}
              >
                <UserPlus className="w-3.5 h-3.5" />
                <span>{usersMessages.tabNew}</span>
              </button>
            </>
          )}
          <button
            type="button"
            onClick={() => void selectUserForEdit(currentUser)}
            className={`px-3 py-1.5 rounded-lg text-xs font-medium flex items-center gap-1.5 transition-colors ${
              activeTab === 'profile'
                ? 'bg-indigo-600 text-white'
                : 'text-slate-400 hover:text-white'
            }`}
          >
            <UserCheck className="w-3.5 h-3.5" />
            <span>{usersMessages.tabProfile}</span>
          </button>
        </div>
      </div>

      {/* Alertes & notifications */}
      {error && (
        <div
          role="alert"
          className="p-3.5 rounded-xl bg-rose-500/10 border border-rose-500/30 text-rose-300 text-xs flex items-center gap-2"
        >
          <AlertTriangle className="w-4 h-4 shrink-0" />
          <span>{error}</span>
        </div>
      )}
      {notice && (
        <div
          role="status"
          className="p-3.5 rounded-xl bg-emerald-500/10 border border-emerald-500/30 text-emerald-300 text-xs flex items-center gap-2"
        >
          <CheckCircle2 className="w-4 h-4 shrink-0" />
          <span>{notice}</span>
        </div>
      )}

      {/* ONGLET 1 : LISTE DES UTILISATEURS (`WP-52`) */}
      {activeTab === 'list' && isAdmin && (
        <div className="space-y-4">
          <div className="flex flex-col sm:flex-row items-stretch sm:items-center gap-3 bg-[#0f1422] p-4 rounded-2xl border border-white/5">
            <div className="relative flex-1">
              <Search className="w-4 h-4 text-slate-500 absolute left-3 top-1/2 -translate-y-1/2" />
              <input
                type="search"
                value={search}
                onChange={(e) => setSearch(e.target.value)}
                placeholder={usersMessages.searchPlaceholder}
                className="w-full bg-black/30 border border-white/10 rounded-xl pl-9 pr-3 py-2 text-xs text-white focus:outline-none focus:border-indigo-500"
              />
            </div>

            <select
              aria-label={usersMessages.filterAllRoles}
              value={roleFilter}
              onChange={(e) => setRoleFilter(e.target.value as Role | '')}
              className="bg-black/30 border border-white/10 rounded-xl px-3 py-2 text-xs text-white focus:outline-none focus:border-indigo-500"
            >
              <option value="">{usersMessages.filterAllRoles}</option>
              <option value="admin">{usersMessages.roleAdmin}</option>
              <option value="editor">{usersMessages.roleEditor}</option>
              <option value="author">{usersMessages.roleAuthor}</option>
              <option value="subscriber">{usersMessages.roleSubscriber}</option>
            </select>
          </div>

          <div className="rounded-2xl bg-[#0f1422] border border-white/5 overflow-hidden">
            {loading ? (
              <div className="p-8 text-center text-slate-500 text-xs">
                Chargement des comptes…
              </div>
            ) : (
              <table className="w-full text-left border-collapse text-xs">
                <thead>
                  <tr className="border-b border-white/5 text-slate-400 uppercase text-[10px] tracking-wider">
                    <th className="py-3 px-4">Utilisateur</th>
                    <th className="py-3 px-4">E-mail</th>
                    <th className="py-3 px-4">Rôle</th>
                    <th className="py-3 px-4">Publications</th>
                    <th className="py-3 px-4 text-right">Actions</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-white/5">
                  {items.map((u) => (
                    <tr key={u.id} className="hover:bg-white/[0.02]">
                      <td className="py-3 px-4">
                        <div className="font-semibold text-white">{u.display_name}</div>
                        <div className="text-[11px] text-slate-500 font-mono">@{u.username}</div>
                      </td>
                      <td className="py-3 px-4 text-slate-300">{u.email}</td>
                      <td className="py-3 px-4">
                        <span
                          className={`rf-user-role inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full text-[11px] font-medium border ${ROLE_BADGE_CLASSES[u.role]}`}
                        >
                          <Shield className="w-3 h-3" />
                          {ROLE_LABELS[u.role]}
                        </span>
                      </td>
                      <td className="py-3 px-4 text-slate-300">{u.entry_count ?? 0}</td>
                      <td className="py-3 px-4 text-right space-x-2">
                        <button
                          type="button"
                          onClick={() => void selectUserForEdit(u)}
                          className="inline-flex items-center gap-1 px-2.5 py-1.5 rounded-lg bg-white/5 hover:bg-white/10 text-slate-200 transition-colors"
                        >
                          <Edit3 className="w-3.5 h-3.5" />
                          <span>Modifier</span>
                        </button>
                        {u.id !== currentUser.id && (
                          <button
                            type="button"
                            onClick={() => openDeleteModal(u)}
                            className="inline-flex items-center gap-1 px-2.5 py-1.5 rounded-lg bg-rose-500/10 hover:bg-rose-500/20 text-rose-300 transition-colors"
                          >
                            <Trash2 className="w-3.5 h-3.5" />
                            <span>Supprimer</span>
                          </button>
                        )}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            )}
          </div>
        </div>
      )}

      {/* ONGLET 2 : CRÉER UN COMPTE (`WP-52`) */}
      {activeTab === 'new' && isAdmin && (
        <form
          onSubmit={(e) => {
            e.preventDefault();
            void createUser(currentUser);
          }}
          className="p-6 rounded-2xl bg-[#0f1422] border border-white/5 space-y-4 max-w-2xl"
        >
          <h4 className="text-sm font-bold text-white">{usersMessages.createTitle}</h4>

          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div>
              <label className="text-[11px] text-slate-400 block mb-1">
                {usersMessages.fieldUsername}
              </label>
              <input
                type="text"
                required
                value={newUsername}
                onChange={(e) => setNewField('username', e.target.value)}
                placeholder="ex: marie.curie"
                className="w-full bg-black/30 border border-white/10 rounded-xl px-3 py-2 text-xs text-white focus:outline-none focus:border-indigo-500"
              />
            </div>

            <div>
              <label className="text-[11px] text-slate-400 block mb-1">
                {usersMessages.fieldEmail}
              </label>
              <input
                type="email"
                required
                value={newEmail}
                onChange={(e) => setNewField('email', e.target.value)}
                placeholder="marie@exemple.fr"
                className="w-full bg-black/30 border border-white/10 rounded-xl px-3 py-2 text-xs text-white focus:outline-none focus:border-indigo-500"
              />
            </div>

            <div>
              <label className="text-[11px] text-slate-400 block mb-1">
                {usersMessages.fieldDisplayName}
              </label>
              <input
                type="text"
                value={newDisplayName}
                onChange={(e) => setNewField('displayName', e.target.value)}
                placeholder="Marie Curie"
                className="w-full bg-black/30 border border-white/10 rounded-xl px-3 py-2 text-xs text-white focus:outline-none focus:border-indigo-500"
              />
            </div>

            <div>
              <label className="text-[11px] text-slate-400 block mb-1">
                {usersMessages.fieldRole}
              </label>
              <select
                value={newRole}
                onChange={(e) => setNewField('role', e.target.value)}
                className="w-full bg-black/30 border border-white/10 rounded-xl px-3 py-2 text-xs text-white focus:outline-none focus:border-indigo-500"
              >
                <option value="subscriber">{usersMessages.roleSubscriber}</option>
                <option value="author">{usersMessages.roleAuthor}</option>
                <option value="editor">{usersMessages.roleEditor}</option>
                <option value="admin">{usersMessages.roleAdmin}</option>
              </select>
            </div>
          </div>

          <div>
            <label className="text-[11px] text-slate-400 block mb-1">
              {usersMessages.fieldPassword}
            </label>
            <input
              type="password"
              required
              minLength={10}
              value={newPassword}
              onChange={(e) => setNewField('password', e.target.value)}
              placeholder="Minimum 10 caractères"
              className="w-full bg-black/30 border border-white/10 rounded-xl px-3 py-2 text-xs text-white focus:outline-none focus:border-indigo-500"
            />
          </div>

          <div className="pt-2">
            <button
              type="submit"
              disabled={creating}
              className="px-4 py-2 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-semibold transition-colors"
            >
              {creating ? usersMessages.creatingButton : usersMessages.createButton}
            </button>
          </div>
        </form>
      )}

      {/* ONGLET 3 : PROFIL & SESSIONS ACTIVES (`WP-53`, `WP-55`) */}
      {activeTab === 'profile' && (
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          <form
            onSubmit={(e) => {
              e.preventDefault();
              void saveEditedUser(currentUser, setUser);
            }}
            className="lg:col-span-2 p-6 rounded-2xl bg-[#0f1422] border border-white/5 space-y-4"
          >
            <div className="flex items-center justify-between">
              <div>
                <h4 className="text-sm font-bold text-white">
                  Profil de {targetProfileUser.display_name} (@{targetProfileUser.username})
                </h4>
                <p className="text-[11px] text-slate-400">
                  Identifiant permanent : <span className="font-mono">{targetProfileUser.username}</span>
                </p>
              </div>
              <span
                className={`px-2.5 py-0.5 rounded-full text-[11px] font-medium border ${ROLE_BADGE_CLASSES[targetProfileUser.role]}`}
              >
                {ROLE_LABELS[targetProfileUser.role]}
              </span>
            </div>

            <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
              <div>
                <label className="text-[11px] text-slate-400 block mb-1">
                  {usersMessages.fieldDisplayName}
                </label>
                <input
                  type="text"
                  required
                  value={editDisplayName}
                  onChange={(e) => setEditField('displayName', e.target.value)}
                  className="w-full bg-black/30 border border-white/10 rounded-xl px-3 py-2 text-xs text-white focus:outline-none focus:border-indigo-500"
                />
              </div>

              <div>
                <label className="text-[11px] text-slate-400 block mb-1">
                  {usersMessages.fieldEmail}
                </label>
                <input
                  type="email"
                  required
                  value={editEmail}
                  onChange={(e) => setEditField('email', e.target.value)}
                  className="w-full bg-black/30 border border-white/10 rounded-xl px-3 py-2 text-xs text-white focus:outline-none focus:border-indigo-500"
                />
              </div>
            </div>

            {isAdmin && (
              <div>
                <label className="text-[11px] text-slate-400 block mb-1">
                  {usersMessages.fieldRole}
                </label>
                <select
                  value={editRole}
                  onChange={(e) => setEditField('role', e.target.value)}
                  className="w-full bg-black/30 border border-white/10 rounded-xl px-3 py-2 text-xs text-white focus:outline-none focus:border-indigo-500"
                >
                  <option value="subscriber">{usersMessages.roleSubscriber}</option>
                  <option value="author">{usersMessages.roleAuthor}</option>
                  <option value="editor">{usersMessages.roleEditor}</option>
                  <option value="admin">{usersMessages.roleAdmin}</option>
                </select>
              </div>
            )}

            <div>
              <label className="text-[11px] text-slate-400 block mb-1">
                {usersMessages.fieldBio}
              </label>
              <textarea
                rows={3}
                value={editBio}
                onChange={(e) => setEditField('bio', e.target.value)}
                placeholder="Courte présentation affichée sur les articles de cet auteur…"
                className="w-full bg-black/30 border border-white/10 rounded-xl px-3 py-2 text-xs text-white focus:outline-none focus:border-indigo-500"
              />
            </div>

            <div>
              <label className="text-[11px] text-slate-400 block mb-1">
                Nouveau mot de passe (laisser vide pour conserver l’actuel)
              </label>
              <input
                type="password"
                minLength={10}
                value={editPassword}
                onChange={(e) => setEditField('password', e.target.value)}
                placeholder="Nouveau mot de passe (10 caractères min.)"
                className="w-full bg-black/30 border border-white/10 rounded-xl px-3 py-2 text-xs text-white focus:outline-none focus:border-indigo-500"
              />
            </div>

            <div className="pt-2">
              <button
                type="submit"
                disabled={savingProfile}
                className="px-4 py-2 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-semibold transition-colors"
              >
                {savingProfile
                  ? usersMessages.savingProfileButton
                  : usersMessages.saveProfileButton}
              </button>
            </div>
          </form>

          {/* Panneau Sessions actives (`WP-55`) */}
          <div className="p-6 rounded-2xl bg-[#0f1422] border border-white/5 space-y-4">
            <div className="flex items-center gap-2 text-indigo-400">
              <KeyRound className="w-4 h-4" />
              <h4 className="text-xs font-bold text-white uppercase tracking-wider">
                {usersMessages.sessionsTitle}
              </h4>
            </div>

            {loadingSessions ? (
              <p className="text-xs text-slate-500">Chargement des sessions…</p>
            ) : sessions.length === 0 ? (
              <p className="text-xs text-slate-400">{usersMessages.noActiveSessions}</p>
            ) : (
              <ul className="space-y-2 text-xs">
                {sessions.map((sess) => (
                  <li
                    key={sess.id}
                    className="p-3 rounded-xl bg-black/30 border border-white/5 space-y-1"
                  >
                    <div className="font-mono text-[10px] text-slate-400">
                      Session #{sess.id.slice(0, 8)}
                    </div>
                    <div className="text-[11px] text-slate-300">
                      Ouverte : {new Date(sess.created_at).toLocaleString('fr-FR')}
                    </div>
                    <div className="text-[11px] text-slate-500">
                      Expire : {new Date(sess.expires_at).toLocaleString('fr-FR')}
                    </div>
                  </li>
                ))}
              </ul>
            )}

            <button
              type="button"
              disabled={revokingSessions || sessions.length === 0}
              onClick={() => void revokeTargetSessions(targetProfileUser.id)}
              className="w-full px-3 py-2 rounded-xl bg-rose-500/15 hover:bg-rose-500/25 disabled:opacity-40 text-rose-300 text-xs font-medium flex items-center justify-center gap-2 transition-colors"
            >
              <LogOut className="w-3.5 h-3.5" />
              <span>{usersMessages.revokeSessionsButton}</span>
            </button>
          </div>
        </div>
      )}

      {/* MODALE DE SUPPRESSION AVEC RÉATTRIBUTION (`WP-54`) */}
      {deletingUser && (
        <div className="fixed inset-0 bg-black/70 z-50 flex items-center justify-center p-4">
          <div className="bg-[#0f1422] border border-white/10 rounded-2xl max-w-md w-full p-6 space-y-4">
            <h4 className="text-sm font-bold text-white">{usersMessages.deleteModalTitle}</h4>
            <p className="text-xs text-slate-300">
              Vous êtes sur le point de supprimer le compte{' '}
              <strong>{deletingUser.display_name}</strong> (@{deletingUser.username},{' '}
              {deletingUser.entry_count ?? 0} publication(s)).
            </p>

            <div>
              <label className="text-[11px] text-slate-400 block mb-1">
                {usersMessages.deleteReassignLabel}
              </label>
              <select
                value={reassignTargetId}
                onChange={(e) => setReassignTargetId(e.target.value)}
                className="w-full bg-black/40 border border-white/10 rounded-xl px-3 py-2 text-xs text-white focus:outline-none focus:border-indigo-500"
              >
                <option value="">{usersMessages.deleteNoReassign}</option>
                {reassignCandidates.map((u) => (
                  <option key={u.id} value={u.id}>
                    Réattribuer à {u.display_name} (@{u.username})
                  </option>
                ))}
              </select>
            </div>

            <div className="flex items-center justify-end gap-2 pt-2">
              <button
                type="button"
                onClick={closeDeleteModal}
                className="px-3 py-1.5 rounded-xl bg-white/5 hover:bg-white/10 text-slate-300 text-xs"
              >
                Annuler
              </button>
              <button
                type="button"
                disabled={deleting}
                onClick={() => void confirmDeleteUser(currentUser)}
                className="px-4 py-1.5 rounded-xl bg-rose-600 hover:bg-rose-500 text-white text-xs font-semibold"
              >
                {deleting ? 'Suppression…' : usersMessages.deleteConfirmButton}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

