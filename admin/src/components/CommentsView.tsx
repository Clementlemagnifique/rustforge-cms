/**
 * ============================================================================
 * VUE COMMENTAIRES, DISCUSSIONS & MODÉRATION (`CommentsView`)
 * ============================================================================
 * Tâches : `WP-42` (Cycle de vie des commentaires), `WP-43` (Modération et
 * réponses en cascade), `WP-44` (Conformité RGPD et anonymisation), `WP-45`
 * (Gestion de la corbeille et détection de spam).
 */

import React, { useEffect, useState } from 'react';
import {
  MessageSquare,
  Clock,
  CheckCircle,
  AlertOctagon,
  Trash2,
  Search,
  CornerDownRight,
  Edit2,
  ExternalLink,
  Mail,
  RotateCcw,
  ShieldAlert,
  X,
  Send,
  Save,
} from 'lucide-react';
import { commentsMessages } from '../i18n/messages.fr';
import { useCommentsStore, type CommentFilterTab } from '../stores/useCommentsStore';
import type { CommentItem } from '../types';

export const CommentsView: React.FC = () => {
  const {
    items,
    counts,
    activeTab,
    search,
    page,
    totalPages,
    selectedIds,
    loading,
    actionLoading,
    error,
    replyingComment,
    editingComment,
    loadComments,
    loadCounts,
    setActiveTab,
    setSearch,
    setPage,
    toggleSelect,
    toggleSelectAll,
    updateStatus,
    batchUpdateStatus,
    updateCommentContent,
    submitReply,
    deleteComment,
    emptyTrash,
    setReplyingComment,
    setEditingComment,
  } = useCommentsStore();

  const [bulkAction, setBulkAction] = useState<string>('');
  const [searchInput, setSearchInput] = useState<string>(search);

  // Formulaire d'édition
  const [editName, setEditName] = useState<string>('');
  const [editEmail, setEditEmail] = useState<string>('');
  const [editUrl, setEditUrl] = useState<string>('');
  const [editContent, setEditContent] = useState<string>('');

  // Formulaire de réponse
  const [replyContent, setReplyContent] = useState<string>('');

  useEffect(() => {
    loadComments();
    loadCounts();
  }, [loadComments, loadCounts]);

  const handleOpenEdit = (comment: CommentItem) => {
    setEditName(comment.author_name || '');
    setEditEmail(comment.author_email || '');
    setEditUrl(comment.author_url || '');
    setEditContent(comment.content || '');
    setEditingComment(comment);
  };

  const handleOpenReply = (comment: CommentItem) => {
    setReplyContent('');
    setReplyingComment(comment);
  };

  const handleSearchSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    setSearch(searchInput);
  };

  const handleApplyBulk = () => {
    if (!bulkAction || selectedIds.length === 0) return;
    if (bulkAction === 'approve') batchUpdateStatus('approved');
    else if (bulkAction === 'unapprove') batchUpdateStatus('pending');
    else if (bulkAction === 'spam') batchUpdateStatus('spam');
    else if (bulkAction === 'trash') batchUpdateStatus('trash');
    setBulkAction('');
  };

  const handleSaveEdit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!editingComment) return;
    updateCommentContent(editingComment.id, {
      content: editContent,
      author_name: editName,
      author_email: editEmail,
      author_url: editUrl,
    });
  };

  const handleSendReply = (e: React.FormEvent) => {
    e.preventDefault();
    if (!replyingComment || !replyContent.trim()) return;
    submitReply(replyingComment.id, replyContent.trim());
  };

  const statusTabs: { id: CommentFilterTab; label: string; count: number; icon: React.ReactNode }[] = [
    { id: 'all', label: commentsMessages.statusAll, count: counts.all, icon: <MessageSquare className="w-4 h-4" /> },
    { id: 'pending', label: commentsMessages.statusPending, count: counts.pending, icon: <Clock className="w-4 h-4" /> },
    { id: 'approved', label: commentsMessages.statusApproved, count: counts.approved, icon: <CheckCircle className="w-4 h-4" /> },
    { id: 'spam', label: commentsMessages.statusSpam, count: counts.spam, icon: <AlertOctagon className="w-4 h-4" /> },
    { id: 'trash', label: commentsMessages.statusTrash, count: counts.trash, icon: <Trash2 className="w-4 h-4" /> },
  ];

  return (
    <div className="space-y-6">
      {/* En-tête */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div>
          <h1 className="text-2xl font-bold text-white tracking-tight flex items-center gap-2">
            <MessageSquare className="w-7 h-7 text-indigo-400" />
            Commentaires & Modération
          </h1>
          <p className="text-sm text-slate-400 mt-1">
            Gérez les interactions des visiteurs, validez les messages en attente et protégez votre site contre le spam.
          </p>
        </div>

        {activeTab === 'trash' && counts.trash > 0 && (
          <button
            onClick={() => {
              if (window.confirm(commentsMessages.confirmEmptyTrash)) {
                emptyTrash();
              }
            }}
            disabled={actionLoading}
            className="flex items-center gap-2 px-3 py-2 text-sm font-medium text-rose-400 hover:text-rose-300 bg-rose-500/10 hover:bg-rose-500/20 rounded-lg transition-colors border border-rose-500/20"
          >
            <Trash2 className="w-4 h-4" />
            {commentsMessages.emptyTrash}
          </button>
        )}
      </div>

      {/* Message d'erreur */}
      {error && (
        <div className="p-4 bg-rose-500/10 border border-rose-500/20 rounded-xl text-rose-400 text-sm flex items-center gap-3">
          <ShieldAlert className="w-5 h-5 flex-shrink-0" />
          <span>{error}</span>
        </div>
      )}

      {/* Onglets de filtrage par statut */}
      <div className="flex flex-wrap items-center gap-2 border-b border-slate-800 pb-3">
        {statusTabs.map((tab) => {
          const isActive = activeTab === tab.id;
          return (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={`flex items-center gap-2 px-3.5 py-2 rounded-lg text-sm font-medium transition-all ${
                isActive
                  ? 'bg-indigo-600 text-white shadow-sm'
                  : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800/60'
              }`}
            >
              {tab.icon}
              <span>{tab.label}</span>
              <span
                className={`ml-1 px-1.5 py-0.5 text-xs font-semibold rounded-full ${
                  tab.id === 'pending' && tab.count > 0
                    ? 'bg-amber-500 text-slate-950 font-bold'
                    : isActive
                    ? 'bg-indigo-500/40 text-white'
                    : 'bg-slate-800 text-slate-400'
                }`}
              >
                {tab.count}
              </span>
            </button>
          );
        })}
      </div>

      {/* Barre d'outils (actions groupées + recherche) */}
      <div className="flex flex-col sm:flex-row items-stretch sm:items-center justify-between gap-4">
        <div className="flex items-center gap-2">
          <select
            value={bulkAction}
            onChange={(e) => setBulkAction(e.target.value)}
            disabled={selectedIds.length === 0 || actionLoading}
            aria-label={commentsMessages.bulkSelectAction}
            className="bg-slate-900 border border-slate-700 text-slate-200 text-sm rounded-lg px-3 py-2 focus:ring-2 focus:ring-indigo-500 focus:outline-none disabled:opacity-50"
          >
            <option value="">{commentsMessages.bulkSelectAction}</option>
            {activeTab !== 'approved' && <option value="approve">{commentsMessages.bulkApprove}</option>}
            {activeTab !== 'pending' && <option value="unapprove">{commentsMessages.bulkUnapprove}</option>}
            {activeTab !== 'spam' && <option value="spam">{commentsMessages.bulkSpam}</option>}
            {activeTab !== 'trash' && <option value="trash">{commentsMessages.bulkTrash}</option>}
          </select>
          <button
            onClick={handleApplyBulk}
            disabled={!bulkAction || selectedIds.length === 0 || actionLoading}
            className="px-3 py-2 bg-slate-800 hover:bg-slate-700 text-slate-200 text-sm font-medium rounded-lg transition-colors disabled:opacity-50"
          >
            {commentsMessages.bulkApply}
          </button>
          {selectedIds.length > 0 && (
            <span className="text-xs text-slate-400 ml-2">
              {selectedIds.length} sélectionné(s)
            </span>
          )}
        </div>

        <form onSubmit={handleSearchSubmit} className="relative min-w-[280px]">
          <Search className="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-slate-500" />
          <input
            type="text"
            value={searchInput}
            onChange={(e) => setSearchInput(e.target.value)}
            placeholder={commentsMessages.searchPlaceholder}
            className="w-full pl-9 pr-4 py-2 bg-slate-900 border border-slate-700 rounded-lg text-sm text-slate-200 placeholder-slate-500 focus:ring-2 focus:ring-indigo-500 focus:outline-none"
          />
        </form>
      </div>

      {/* Liste des commentaires */}
      <div className="bg-slate-900/60 border border-slate-800 rounded-xl overflow-hidden shadow-sm">
        {loading ? (
          <div className="p-12 text-center text-slate-400">
            <div className="inline-block animate-spin rounded-full h-8 w-8 border-4 border-slate-700 border-t-indigo-500 mb-3" />
            <p>{commentsMessages.loadingList}</p>
          </div>
        ) : items.length === 0 ? (
          <div className="p-12 text-center text-slate-500">
            <MessageSquare className="w-10 h-10 mx-auto mb-3 opacity-30" />
            <p className="text-base font-medium">{commentsMessages.emptyList}</p>
          </div>
        ) : (
          <div className="divide-y divide-slate-800/80">
            {/* Entête du tableau */}
            <div className="flex items-center px-4 py-3 bg-slate-800/40 text-xs font-semibold text-slate-400 uppercase tracking-wider">
              <div className="w-10 flex items-center justify-center">
                <input
                  type="checkbox"
                  aria-label="Sélectionner tous les commentaires"
                  checked={items.length > 0 && selectedIds.length === items.length}
                  onChange={toggleSelectAll}
                  className="rounded border-slate-700 bg-slate-800 text-indigo-600 focus:ring-indigo-500 w-4 h-4"
                />
              </div>
              <div className="w-1/4 min-w-[180px]">Auteur</div>
              <div className="flex-1 px-4">Commentaire</div>
              <div className="w-40 text-right pr-2">Date & Statut</div>
            </div>

            {/* Lignes de commentaires */}
            {items.map((comment) => {
              const isSelected = selectedIds.includes(comment.id);
              return (
                <div
                  key={comment.id}
                  className={`flex flex-col md:flex-row items-start md:items-center px-4 py-4 transition-colors hover:bg-slate-800/30 ${
                    comment.status === 'pending'
                      ? 'bg-amber-500/5'
                      : comment.status === 'spam'
                      ? 'bg-rose-500/5'
                      : ''
                  }`}
                >
                  <div className="flex items-center gap-3 w-full md:w-auto">
                    <div className="w-10 flex items-center justify-center">
                      <input
                        type="checkbox"
                        aria-label={`Sélectionner le commentaire de ${comment.author_name}`}
                        checked={isSelected}
                        onChange={() => toggleSelect(comment.id)}
                        className="rounded border-slate-700 bg-slate-800 text-indigo-600 focus:ring-indigo-500 w-4 h-4"
                      />
                    </div>
                  </div>

                  {/* Auteur */}
                  <div className="w-full md:w-1/4 min-w-[180px] pr-4 mt-2 md:mt-0">
                    <div className="flex items-center gap-2.5">
                      <div className="w-9 h-9 rounded-full bg-indigo-500/20 text-indigo-300 font-semibold flex items-center justify-center text-sm flex-shrink-0">
                        {comment.author_name.charAt(0).toUpperCase()}
                      </div>
                      <div className="min-w-0">
                        <div className="font-medium text-slate-200 text-sm truncate">
                          {comment.author_name}
                        </div>
                        {comment.author_email && (
                          <a
                            href={`mailto:${comment.author_email}`}
                            className="text-xs text-slate-400 hover:text-indigo-400 flex items-center gap-1 truncate"
                          >
                            <Mail className="w-3 h-3 flex-shrink-0" />
                            <span className="truncate">{comment.author_email}</span>
                          </a>
                        )}
                        {comment.author_url && (
                          <a
                            href={comment.author_url}
                            target="_blank"
                            rel="noopener noreferrer"
                            className="text-xs text-indigo-400/80 hover:text-indigo-300 flex items-center gap-1 truncate"
                          >
                            <ExternalLink className="w-3 h-3 flex-shrink-0" />
                            <span className="truncate">{comment.author_url}</span>
                          </a>
                        )}
                      </div>
                    </div>
                  </div>

                  {/* Corps du commentaire */}
                  <div className="flex-1 px-0 md:px-4 py-2 md:py-0 w-full">
                    <div className="text-sm text-slate-300 line-clamp-3">
                      {comment.content}
                    </div>

                    {/* Actions contextuelles au survol / sous le texte */}
                    <div className="flex flex-wrap items-center gap-3 mt-2 text-xs font-medium">
                      {comment.status === 'pending' && (
                        <button
                          onClick={() => updateStatus(comment.id, 'approved')}
                          disabled={actionLoading}
                          className="text-emerald-400 hover:text-emerald-300 flex items-center gap-1"
                        >
                          <CheckCircle className="w-3.5 h-3.5" />
                          {commentsMessages.approveAction}
                        </button>
                      )}
                      {comment.status === 'approved' && (
                        <button
                          onClick={() => updateStatus(comment.id, 'pending')}
                          disabled={actionLoading}
                          className="text-amber-400 hover:text-amber-300 flex items-center gap-1"
                        >
                          <Clock className="w-3.5 h-3.5" />
                          {commentsMessages.unapproveAction}
                        </button>
                      )}

                      {comment.status !== 'trash' && comment.status !== 'spam' && (
                        <>
                          <button
                            onClick={() => handleOpenReply(comment)}
                            className="text-indigo-400 hover:text-indigo-300 flex items-center gap-1"
                          >
                            <CornerDownRight className="w-3.5 h-3.5" />
                            {commentsMessages.replyAction}
                          </button>
                          <button
                            onClick={() => handleOpenEdit(comment)}
                            className="text-slate-400 hover:text-slate-300 flex items-center gap-1"
                          >
                            <Edit2 className="w-3.5 h-3.5" />
                            {commentsMessages.editAction}
                          </button>
                        </>
                      )}

                      {comment.status !== 'spam' && comment.status !== 'trash' && (
                        <button
                          onClick={() => updateStatus(comment.id, 'spam')}
                          disabled={actionLoading}
                          className="text-rose-400/80 hover:text-rose-300 flex items-center gap-1"
                        >
                          <AlertOctagon className="w-3.5 h-3.5" />
                          {commentsMessages.spamAction}
                        </button>
                      )}

                      {comment.status === 'spam' && (
                        <button
                          onClick={() => updateStatus(comment.id, 'approved')}
                          disabled={actionLoading}
                          className="text-emerald-400 hover:text-emerald-300 flex items-center gap-1"
                        >
                          <RotateCcw className="w-3.5 h-3.5" />
                          {commentsMessages.notSpamAction}
                        </button>
                      )}

                      {comment.status !== 'trash' && (
                        <button
                          onClick={() => updateStatus(comment.id, 'trash')}
                          disabled={actionLoading}
                          className="text-slate-400 hover:text-rose-400 flex items-center gap-1"
                        >
                          <Trash2 className="w-3.5 h-3.5" />
                          {commentsMessages.trashAction}
                        </button>
                      )}

                      {comment.status === 'trash' && (
                        <>
                          <button
                            onClick={() => updateStatus(comment.id, 'approved')}
                            disabled={actionLoading}
                            className="text-emerald-400 hover:text-emerald-300 flex items-center gap-1"
                          >
                            <RotateCcw className="w-3.5 h-3.5" />
                            {commentsMessages.restoreAction}
                          </button>
                          <button
                            onClick={() => {
                              if (window.confirm('Supprimer définitivement ce commentaire ?')) {
                                deleteComment(comment.id);
                              }
                            }}
                            disabled={actionLoading}
                            className="text-rose-400 hover:text-rose-300 flex items-center gap-1"
                          >
                            <Trash2 className="w-3.5 h-3.5" />
                            {commentsMessages.forceDeleteAction}
                          </button>
                        </>
                      )}
                    </div>
                  </div>

                  {/* Date et Statut */}
                  <div className="w-full md:w-40 flex md:flex-col items-center md:items-end justify-between md:justify-center gap-1.5 text-xs text-slate-400 mt-2 md:mt-0 pr-2">
                    <span title={new Date(comment.created_at).toLocaleString('fr-FR')}>
                      {new Date(comment.created_at).toLocaleDateString('fr-FR')}
                    </span>
                    <span
                      className={`px-2 py-0.5 rounded-full text-[11px] font-medium ${
                        comment.status === 'approved'
                          ? 'bg-emerald-500/15 text-emerald-400 border border-emerald-500/30'
                          : comment.status === 'pending'
                          ? 'bg-amber-500/15 text-amber-400 border border-amber-500/30'
                          : comment.status === 'spam'
                          ? 'bg-rose-500/15 text-rose-400 border border-rose-500/30'
                          : 'bg-slate-700/50 text-slate-400 border border-slate-600/30'
                      }`}
                    >
                      {comment.status === 'approved' && commentsMessages.statusApproved}
                      {comment.status === 'pending' && commentsMessages.statusPending}
                      {comment.status === 'spam' && commentsMessages.statusSpam}
                      {comment.status === 'trash' && commentsMessages.statusTrash}
                    </span>
                  </div>
                </div>
              );
            })}
          </div>
        )}

        {/* Pagination */}
        {totalPages > 1 && (
          <div className="flex items-center justify-between px-4 py-3 bg-slate-800/40 border-t border-slate-800 text-sm text-slate-400">
            <span>
              Page {page} sur {totalPages}
            </span>
            <div className="flex items-center gap-2">
              <button
                onClick={() => setPage(page - 1)}
                disabled={page <= 1}
                className="px-3 py-1 bg-slate-800 hover:bg-slate-700 rounded text-slate-200 disabled:opacity-40"
              >
                Précédent
              </button>
              <button
                onClick={() => setPage(page + 1)}
                disabled={page >= totalPages}
                className="px-3 py-1 bg-slate-800 hover:bg-slate-700 rounded text-slate-200 disabled:opacity-40"
              >
                Suivant
              </button>
            </div>
          </div>
        )}
      </div>

      {/* Modal / Tiroir Répondre */}
      {replyingComment && (
        <div className="fixed inset-0 z-50 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4">
          <div className="bg-slate-900 border border-slate-800 rounded-2xl w-full max-w-lg p-6 shadow-2xl relative space-y-4">
            <button
              onClick={() => setReplyingComment(null)}
              className="absolute top-4 right-4 text-slate-400 hover:text-white"
            >
              <X className="w-5 h-5" />
            </button>
            <h2 className="text-lg font-bold text-white flex items-center gap-2">
              <CornerDownRight className="w-5 h-5 text-indigo-400" />
              {commentsMessages.replyTitle}
            </h2>
            <div className="bg-slate-800/60 p-3 rounded-lg border border-slate-700/50 text-xs text-slate-300">
              <span className="font-semibold text-slate-200">{replyingComment.author_name} :</span>{' '}
              {replyingComment.content}
            </div>
            <form onSubmit={handleSendReply} className="space-y-4">
              <div>
                <label className="block text-xs font-semibold text-slate-400 uppercase tracking-wider mb-1.5">
                  {commentsMessages.responseLabel}
                </label>
                <textarea
                  rows={4}
                  value={replyContent}
                  onChange={(e) => setReplyContent(e.target.value)}
                  placeholder="Écrivez votre réponse publique ici..."
                  required
                  className="w-full bg-slate-850 border border-slate-700 rounded-lg p-3 text-sm text-slate-200 focus:ring-2 focus:ring-indigo-500 focus:outline-none"
                />
              </div>
              <div className="flex justify-end gap-3">
                <button
                  type="button"
                  onClick={() => setReplyingComment(null)}
                  className="px-4 py-2 bg-slate-800 hover:bg-slate-700 text-slate-300 text-sm font-medium rounded-lg"
                >
                  {commentsMessages.cancelEdit}
                </button>
                <button
                  type="submit"
                  disabled={actionLoading || !replyContent.trim()}
                  className="flex items-center gap-2 px-4 py-2 bg-indigo-600 hover:bg-indigo-500 text-white text-sm font-medium rounded-lg disabled:opacity-50"
                >
                  <Send className="w-4 h-4" />
                  {commentsMessages.submitReply}
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* Modal / Tiroir Modifier */}
      {editingComment && (
        <div className="fixed inset-0 z-50 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4">
          <div className="bg-slate-900 border border-slate-800 rounded-2xl w-full max-w-xl p-6 shadow-2xl relative space-y-4">
            <button
              onClick={() => setEditingComment(null)}
              className="absolute top-4 right-4 text-slate-400 hover:text-white"
            >
              <X className="w-5 h-5" />
            </button>
            <h2 className="text-lg font-bold text-white flex items-center gap-2">
              <Edit2 className="w-5 h-5 text-indigo-400" />
              {commentsMessages.editTitle}
            </h2>
            <form onSubmit={handleSaveEdit} className="space-y-4">
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <div>
                  <label className="block text-xs font-semibold text-slate-400 uppercase tracking-wider mb-1.5">
                    {commentsMessages.authorLabel}
                  </label>
                  <input
                    type="text"
                    value={editName}
                    onChange={(e) => setEditName(e.target.value)}
                    required
                    className="w-full bg-slate-850 border border-slate-700 rounded-lg p-2.5 text-sm text-slate-200 focus:ring-2 focus:ring-indigo-500 focus:outline-none"
                  />
                </div>
                <div>
                  <label className="block text-xs font-semibold text-slate-400 uppercase tracking-wider mb-1.5">
                    {commentsMessages.emailLabel}
                  </label>
                  <input
                    type="email"
                    value={editEmail}
                    onChange={(e) => setEditEmail(e.target.value)}
                    className="w-full bg-slate-850 border border-slate-700 rounded-lg p-2.5 text-sm text-slate-200 focus:ring-2 focus:ring-indigo-500 focus:outline-none"
                  />
                </div>
              </div>
              <div>
                <label className="block text-xs font-semibold text-slate-400 uppercase tracking-wider mb-1.5">
                  {commentsMessages.urlLabel}
                </label>
                <input
                  type="url"
                  value={editUrl}
                  onChange={(e) => setEditUrl(e.target.value)}
                  className="w-full bg-slate-850 border border-slate-700 rounded-lg p-2.5 text-sm text-slate-200 focus:ring-2 focus:ring-indigo-500 focus:outline-none"
                />
              </div>
              <div>
                <label className="block text-xs font-semibold text-slate-400 uppercase tracking-wider mb-1.5">
                  {commentsMessages.commentLabel}
                </label>
                <textarea
                  rows={5}
                  value={editContent}
                  onChange={(e) => setEditContent(e.target.value)}
                  required
                  className="w-full bg-slate-850 border border-slate-700 rounded-lg p-3 text-sm text-slate-200 focus:ring-2 focus:ring-indigo-500 focus:outline-none"
                />
              </div>
              <div className="flex justify-end gap-3">
                <button
                  type="button"
                  onClick={() => setEditingComment(null)}
                  className="px-4 py-2 bg-slate-800 hover:bg-slate-700 text-slate-300 text-sm font-medium rounded-lg"
                >
                  {commentsMessages.cancelEdit}
                </button>
                <button
                  type="submit"
                  disabled={actionLoading}
                  className="flex items-center gap-2 px-4 py-2 bg-indigo-600 hover:bg-indigo-500 text-white text-sm font-medium rounded-lg disabled:opacity-50"
                >
                  <Save className="w-4 h-4" />
                  {commentsMessages.saveComment}
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
};
