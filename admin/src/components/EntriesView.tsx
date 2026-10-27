import React, { useState, useEffect } from 'react';
import { 
  Plus, 
  Search, 
  Trash2, 
  Clock, 
  Eye,
  List,
  Columns3,
  Send,
  Archive,
  RotateCcw,
  Calendar
} from 'lucide-react';
import { api } from '../api';
import type { Entry, ContentType, EntryStatus } from '../types';

interface EntriesViewProps {
  contentTypeSlug: string;
  contentTypes: ContentType[];
  onNavigate: (view: string) => void;
}

export const EntriesView: React.FC<EntriesViewProps> = ({
  contentTypeSlug,
  contentTypes,
  onNavigate,
}) => {
  const [entries, setEntries] = useState<Entry[]>([]);
  const [loading, setLoading] = useState(true);
  const [search, setSearch] = useState('');
  const [statusFilter, setStatusFilter] = useState<string>('all');
  const [viewMode, setViewMode] = useState<'table' | 'kanban'>('table');
  const [draggedEntryId, setDraggedEntryId] = useState<string | null>(null);

  const currentType = contentTypes.find((ct) => ct.slug === contentTypeSlug);

  const fetchEntries = async () => {
    setLoading(true);
    try {
      const res = await api.entries.list({
        content_type: contentTypeSlug,
        q: search || undefined,
      });
      setEntries(res.items);
    } catch (err) {
      console.error(err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchEntries();
  }, [contentTypeSlug, search]);

  const handleDelete = async (id: string, e: React.MouseEvent) => {
    e.stopPropagation();
    if (!confirm('Voulez-vous déplacer ce contenu vers la corbeille ?')) return;
    try {
      await api.entries.delete(id);
      fetchEntries();
    } catch (err: any) {
      alert(err.message);
    }
  };

  const handleUpdateStatus = async (entry: Entry, newStatus: EntryStatus, e?: React.MouseEvent) => {
    if (e) e.stopPropagation();
    try {
      await api.entries.update(entry.id, {
        title: entry.title,
        slug: entry.slug,
        excerpt: entry.excerpt,
        body: entry.body,
        status: newStatus,
        intent: entry.intent,
        design_tokens: entry.design_tokens,
        fields: entry.fields,
        published_at: newStatus === 'published' && !entry.published_at ? new Date().toISOString() : entry.published_at,
        scheduled_unpublish_at: entry.scheduled_unpublish_at,
      });
      fetchEntries();
    } catch (err: any) {
      alert(`Erreur de mise à jour du statut : ${err.message}`);
    }
  };

  const handleDragStart = (id: string) => {
    setDraggedEntryId(id);
  };

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
  };

  const handleDropOnColumn = (targetStatus: EntryStatus) => {
    if (!draggedEntryId) return;
    const entry = entries.find((e) => e.id === draggedEntryId);
    if (entry && entry.status !== targetStatus) {
      handleUpdateStatus(entry, targetStatus);
    }
    setDraggedEntryId(null);
  };

  const filtered = entries.filter((entry) => {
    if (statusFilter === 'all') return true;
    return entry.status === statusFilter;
  });

  const drafts = filtered.filter((e) => e.status === 'draft');
  const published = filtered.filter((e) => e.status === 'published');
  const archived = filtered.filter((e) => e.status === 'archived');

  return (
    <div className="space-y-5">
      {/* Barre d'action & Contrôles */}
      <div className="flex flex-col sm:flex-row items-stretch sm:items-center justify-between gap-3">
        <div className="flex items-center gap-3 flex-1 max-w-xl">
          <div className="relative flex-1">
            <Search className="w-4 h-4 text-slate-400 absolute left-3 top-3" />
            <input
              type="text"
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              placeholder={`Rechercher dans ${currentType?.name || 'les contenus'}...`}
              className="w-full bg-[#0f1422] border border-white/10 rounded-xl pl-9 pr-4 py-2 text-xs text-white placeholder-slate-500 focus:outline-none focus:border-indigo-500"
            />
          </div>

          {viewMode === 'table' && (
            <select
              value={statusFilter}
              onChange={(e) => setStatusFilter(e.target.value)}
              className="bg-[#0f1422] border border-white/10 rounded-xl px-3 py-2 text-xs text-slate-300 focus:outline-none focus:border-indigo-500"
            >
              <option value="all">Tous les statuts</option>
              <option value="published">Publiés</option>
              <option value="draft">Brouillons</option>
              <option value="archived">Archivés</option>
            </select>
          )}

          {/* Toggle Vue Tableau / Hub Kanban */}
          <div className="flex items-center p-1 bg-[#0f1422] rounded-xl border border-white/10">
            <button
              onClick={() => setViewMode('table')}
              title="Vue Tableau Liste"
              className={`p-1.5 rounded-lg text-xs transition-colors ${
                viewMode === 'table'
                  ? 'bg-indigo-600 text-white shadow-sm'
                  : 'text-slate-400 hover:text-white'
              }`}
            >
              <List className="w-4 h-4" />
            </button>
            <button
              onClick={() => setViewMode('kanban')}
              title="Hub Éditorial Kanban"
              className={`p-1.5 rounded-lg text-xs transition-colors ${
                viewMode === 'kanban'
                  ? 'bg-indigo-600 text-white shadow-sm'
                  : 'text-slate-400 hover:text-white'
              }`}
            >
              <Columns3 className="w-4 h-4" />
            </button>
          </div>
        </div>

        <button
          onClick={() => onNavigate(`new:${contentTypeSlug}`)}
          className="px-4 py-2 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-semibold flex items-center justify-center gap-2 shadow-md shadow-indigo-600/20 transition-all hover:scale-105 active:scale-95"
        >
          <Plus className="w-4 h-4" />
          <span>Nouveau {currentType?.name_singular || 'Contenu'}</span>
        </button>
      </div>

      {/* ============================================================== */}
      {/* VUE 1 : TABLEAU LISTE CLASSIQUE                                */}
      {/* ============================================================== */}
      {viewMode === 'table' ? (
        <div className="rounded-2xl bg-[#0f1422] border border-white/5 overflow-hidden shadow-sm">
          <table className="w-full text-left text-xs">
            <thead>
              <tr className="border-b border-white/5 bg-white/[0.01] text-slate-400 font-semibold uppercase tracking-wider text-[10px]">
                <th className="py-3.5 px-4">Titre & Slug</th>
                <th className="py-3.5 px-4">Auteur</th>
                <th className="py-3.5 px-4">Statut</th>
                <th className="py-3.5 px-4">Programmation / Date</th>
                <th className="py-3.5 px-4 text-right">Actions</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-white/5">
              {loading ? (
                <tr>
                  <td colSpan={5} className="py-8 text-center text-slate-500">
                    Chargement des contenus...
                  </td>
                </tr>
              ) : filtered.length === 0 ? (
                <tr>
                  <td colSpan={5} className="py-12 text-center text-slate-500">
                    Aucun contenu trouvé. Créez-en un ou utilisez l'Assistant IA !
                  </td>
                </tr>
              ) : (
                filtered.map((entry) => (
                  <tr
                    key={entry.id}
                    onClick={() => onNavigate(`editor:${entry.id}`)}
                    className="hover:bg-white/[0.02] cursor-pointer transition-colors group"
                  >
                    <td className="py-3.5 px-4">
                      <p className="font-semibold text-slate-100 group-hover:text-indigo-400 transition-colors">
                        {entry.title}
                      </p>
                      <p className="text-[11px] text-slate-500 font-mono mt-0.5">/{entry.slug}</p>
                    </td>
                    <td className="py-3.5 px-4 text-slate-300">
                      {entry.author_name || 'Éditeur'}
                    </td>
                    <td className="py-3.5 px-4">
                      <span
                        className={`text-[10px] px-2 py-0.5 rounded-full font-medium ${
                          entry.status === 'published'
                            ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20'
                            : entry.status === 'draft'
                            ? 'bg-amber-500/10 text-amber-400 border border-amber-500/20'
                            : 'bg-slate-500/10 text-slate-400'
                        }`}
                      >
                        {entry.status === 'published' ? 'Publié' : entry.status === 'draft' ? 'Brouillon' : 'Archivé'}
                      </span>
                    </td>
                    <td className="py-3.5 px-4 text-slate-400">
                      <div className="space-y-1">
                        <div className="flex items-center gap-1.5">
                          <Clock className="w-3.5 h-3.5 text-slate-500" />
                          <span>{new Date(entry.updated_at).toLocaleDateString('fr-FR')}</span>
                        </div>
                        {entry.scheduled_unpublish_at && (
                          <div className="flex items-center gap-1 text-[10px] text-amber-400">
                            <Calendar className="w-3 h-3" />
                            <span>Expire : {new Date(entry.scheduled_unpublish_at).toLocaleDateString('fr-FR')}</span>
                          </div>
                        )}
                      </div>
                    </td>
                    <td className="py-3.5 px-4 text-right">
                      <div className="flex items-center justify-end gap-1">
                        <a
                          href={`/api/v1/public/entries/${entry.content_type}/${entry.slug}`}
                          target="_blank"
                          rel="noreferrer"
                          onClick={(e) => e.stopPropagation()}
                          title="Aperçu public"
                          className="p-1.5 rounded-lg text-slate-400 hover:text-white hover:bg-white/5 transition-colors"
                        >
                          <Eye className="w-3.5 h-3.5" />
                        </a>
                        <button
                          onClick={(e) => handleDelete(entry.id, e)}
                          title="Mettre à la corbeille"
                          className="p-1.5 rounded-lg text-slate-400 hover:text-red-400 hover:bg-red-500/10 transition-colors"
                        >
                          <Trash2 className="w-3.5 h-3.5" />
                        </button>
                      </div>
                    </td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      ) : (
        /* ============================================================== */
        /* VUE 2 : HUB ÉDITORIAL KANBAN (DRAG & DROP)                    */
        /* ============================================================== */
        <div className="grid grid-cols-1 md:grid-cols-3 gap-5">
          {/* COLONNE 1 : BROUILLONS */}
          <div
            onDragOver={handleDragOver}
            onDrop={() => handleDropOnColumn('draft')}
            className="flex flex-col bg-[#0f1422] rounded-2xl border border-white/5 p-4 min-h-[450px]"
          >
            <div className="flex items-center justify-between pb-3 mb-3 border-b border-white/5">
              <div className="flex items-center gap-2">
                <span className="w-2.5 h-2.5 rounded-full bg-amber-400" />
                <h3 className="text-xs font-bold uppercase tracking-wider text-slate-200">
                  Brouillons / En Préparation
                </h3>
              </div>
              <span className="px-2 py-0.5 rounded-full text-[10px] font-bold bg-white/5 text-slate-400">
                {drafts.length}
              </span>
            </div>

            <div className="space-y-3 flex-1 overflow-y-auto">
              {drafts.map((entry) => (
                <div
                  key={entry.id}
                  draggable
                  onDragStart={() => handleDragStart(entry.id)}
                  onClick={() => onNavigate(`editor:${entry.id}`)}
                  className="p-3.5 rounded-xl bg-white/[0.02] hover:bg-white/[0.05] border border-white/5 cursor-grab active:cursor-grabbing transition-all hover:scale-[1.01] group space-y-2"
                >
                  <div className="flex items-start justify-between gap-2">
                    <h4 className="text-xs font-bold text-slate-100 group-hover:text-indigo-400 transition-colors line-clamp-2">
                      {entry.title}
                    </h4>
                  </div>
                  <p className="text-[11px] text-slate-400 font-mono">/{entry.slug}</p>
                  
                  <div className="flex items-center justify-between pt-2 border-t border-white/5 text-[10px] text-slate-500">
                    <span>{entry.author_name || 'Éditeur'}</span>
                    <button
                      onClick={(e) => handleUpdateStatus(entry, 'published', e)}
                      title="Publier immédiatement"
                      className="flex items-center gap-1 px-2 py-1 rounded bg-emerald-500/10 text-emerald-400 hover:bg-emerald-500/20 font-semibold transition-colors"
                    >
                      <Send className="w-3 h-3" />
                      <span>Publier</span>
                    </button>
                  </div>
                </div>
              ))}
              {drafts.length === 0 && (
                <div className="h-28 border border-dashed border-white/5 rounded-xl flex items-center justify-center text-[11px] text-slate-600">
                  Déposez un contenu ici
                </div>
              )}
            </div>
          </div>

          {/* COLONNE 2 : PUBLIÉS */}
          <div
            onDragOver={handleDragOver}
            onDrop={() => handleDropOnColumn('published')}
            className="flex flex-col bg-[#0f1422] rounded-2xl border border-white/5 p-4 min-h-[450px]"
          >
            <div className="flex items-center justify-between pb-3 mb-3 border-b border-white/5">
              <div className="flex items-center gap-2">
                <span className="w-2.5 h-2.5 rounded-full bg-emerald-400" />
                <h3 className="text-xs font-bold uppercase tracking-wider text-slate-200">
                  Publiés / En Ligne
                </h3>
              </div>
              <span className="px-2 py-0.5 rounded-full text-[10px] font-bold bg-white/5 text-slate-400">
                {published.length}
              </span>
            </div>

            <div className="space-y-3 flex-1 overflow-y-auto">
              {published.map((entry) => (
                <div
                  key={entry.id}
                  draggable
                  onDragStart={() => handleDragStart(entry.id)}
                  onClick={() => onNavigate(`editor:${entry.id}`)}
                  className="p-3.5 rounded-xl bg-white/[0.02] hover:bg-white/[0.05] border border-white/5 cursor-grab active:cursor-grabbing transition-all hover:scale-[1.01] group space-y-2"
                >
                  <div className="flex items-start justify-between gap-2">
                    <h4 className="text-xs font-bold text-slate-100 group-hover:text-indigo-400 transition-colors line-clamp-2">
                      {entry.title}
                    </h4>
                  </div>
                  <p className="text-[11px] text-slate-400 font-mono">/{entry.slug}</p>

                  {entry.scheduled_unpublish_at && (
                    <div className="flex items-center gap-1 text-[10px] text-amber-400 bg-amber-500/10 px-2 py-0.5 rounded border border-amber-500/20">
                      <Clock className="w-3 h-3 shrink-0" />
                      <span>Expire : {new Date(entry.scheduled_unpublish_at).toLocaleDateString('fr-FR')}</span>
                    </div>
                  )}

                  <div className="flex items-center justify-between pt-2 border-t border-white/5 text-[10px] text-slate-500">
                    <a
                      href={`/api/v1/public/entries/${entry.content_type}/${entry.slug}`}
                      target="_blank"
                      rel="noreferrer"
                      onClick={(e) => e.stopPropagation()}
                      className="flex items-center gap-1 hover:text-white transition-colors"
                    >
                      <Eye className="w-3 h-3" />
                      <span>Voir en direct</span>
                    </a>
                    <button
                      onClick={(e) => handleUpdateStatus(entry, 'archived', e)}
                      title="Dépublier / Archiver"
                      className="flex items-center gap-1 px-2 py-1 rounded bg-slate-500/10 text-slate-300 hover:bg-slate-500/20 font-semibold transition-colors"
                    >
                      <Archive className="w-3 h-3" />
                      <span>Archiver</span>
                    </button>
                  </div>
                </div>
              ))}
              {published.length === 0 && (
                <div className="h-28 border border-dashed border-white/5 rounded-xl flex items-center justify-center text-[11px] text-slate-600">
                  Déposez un contenu ici
                </div>
              )}
            </div>
          </div>

          {/* COLONNE 3 : ARCHIVÉS */}
          <div
            onDragOver={handleDragOver}
            onDrop={() => handleDropOnColumn('archived')}
            className="flex flex-col bg-[#0f1422] rounded-2xl border border-white/5 p-4 min-h-[450px]"
          >
            <div className="flex items-center justify-between pb-3 mb-3 border-b border-white/5">
              <div className="flex items-center gap-2">
                <span className="w-2.5 h-2.5 rounded-full bg-slate-500" />
                <h3 className="text-xs font-bold uppercase tracking-wider text-slate-200">
                  Archivés / Dépubliés
                </h3>
              </div>
              <span className="px-2 py-0.5 rounded-full text-[10px] font-bold bg-white/5 text-slate-400">
                {archived.length}
              </span>
            </div>

            <div className="space-y-3 flex-1 overflow-y-auto">
              {archived.map((entry) => (
                <div
                  key={entry.id}
                  draggable
                  onDragStart={() => handleDragStart(entry.id)}
                  onClick={() => onNavigate(`editor:${entry.id}`)}
                  className="p-3.5 rounded-xl bg-white/[0.02] hover:bg-white/[0.05] border border-white/5 cursor-grab active:cursor-grabbing transition-all hover:scale-[1.01] group space-y-2 opacity-80 hover:opacity-100"
                >
                  <div className="flex items-start justify-between gap-2">
                    <h4 className="text-xs font-bold text-slate-300 group-hover:text-indigo-400 transition-colors line-clamp-2">
                      {entry.title}
                    </h4>
                  </div>
                  <p className="text-[11px] text-slate-500 font-mono">/{entry.slug}</p>

                  <div className="flex items-center justify-between pt-2 border-t border-white/5 text-[10px] text-slate-500">
                    <span>Archivé</span>
                    <button
                      onClick={(e) => handleUpdateStatus(entry, 'draft', e)}
                      title="Rétablir en brouillon"
                      className="flex items-center gap-1 px-2 py-1 rounded bg-indigo-500/10 text-indigo-300 hover:bg-indigo-500/20 font-semibold transition-colors"
                    >
                      <RotateCcw className="w-3 h-3" />
                      <span>Rétablir</span>
                    </button>
                  </div>
                </div>
              ))}
              {archived.length === 0 && (
                <div className="h-28 border border-dashed border-white/5 rounded-xl flex items-center justify-center text-[11px] text-slate-600">
                  Déposez un contenu ici
                </div>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
