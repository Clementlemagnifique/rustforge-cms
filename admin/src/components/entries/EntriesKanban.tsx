/**
 * ============================================================================
 * HUB ÉDITORIAL KANBAN DRAG & DROP (`EntriesKanban`)
 * ============================================================================
 * Permet de déplacer les contenus entre Brouillon (`draft`), Publié (`published`)
 * et Archivé (`archived`) par glisser-déposer ou action directe sans écraser
 * les blocs natifs `rf.document/1` ni le coffre `_rf_vault`.
 */

import React from 'react';
import { Clock, Eye, Send, Archive, RotateCcw } from 'lucide-react';
import { useEntriesListStore } from '../../stores/useEntriesListStore';
import type { EntryStatus } from '../../types';

interface EntriesKanbanProps {
  contentTypeSlug: string;
  onNavigate: (view: string) => void;
}

export const EntriesKanban: React.FC<EntriesKanbanProps> = ({
  contentTypeSlug,
  onNavigate,
}) => {
  const entries = useEntriesListStore((s) => s.entries);
  const draggedEntryId = useEntriesListStore((s) => s.draggedEntryId);
  const setDraggedEntryId = useEntriesListStore((s) => s.setDraggedEntryId);
  const updateEntryStatus = useEntriesListStore((s) => s.updateEntryStatus);

  const drafts = entries.filter((e) => e.status === 'draft');
  const published = entries.filter((e) => e.status === 'published');
  const archived = entries.filter((e) => e.status === 'archived');

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
  };

  const handleDropOnColumn = (targetStatus: EntryStatus) => {
    if (!draggedEntryId) return;
    const entry = entries.find((e) => e.id === draggedEntryId);
    if (entry && entry.status !== targetStatus) {
      void updateEntryStatus(entry, targetStatus, contentTypeSlug);
    }
    setDraggedEntryId(null);
  };

  return (
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
              onDragStart={() => setDraggedEntryId(entry.id)}
              onClick={() => onNavigate(`editor:${entry.id}:${entry.content_type}`)}
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
                  type="button"
                  onClick={(e) => {
                    e.stopPropagation();
                    void updateEntryStatus(entry, 'published', contentTypeSlug);
                  }}
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
              onDragStart={() => setDraggedEntryId(entry.id)}
              onClick={() => onNavigate(`editor:${entry.id}:${entry.content_type}`)}
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
                  <span>
                    Expire : {new Date(entry.scheduled_unpublish_at).toLocaleDateString('fr-FR')}
                  </span>
                </div>
              )}

              <div className="flex items-center justify-between pt-2 border-t border-white/5 text-[10px] text-slate-500">
                <a
                  href={`/fr/${entry.content_type}/${entry.slug}`}
                  target="_blank"
                  rel="noreferrer"
                  onClick={(e) => e.stopPropagation()}
                  className="flex items-center gap-1 hover:text-white transition-colors"
                >
                  <Eye className="w-3 h-3" />
                  <span>Voir en direct</span>
                </a>
                <button
                  type="button"
                  onClick={(e) => {
                    e.stopPropagation();
                    void updateEntryStatus(entry, 'archived', contentTypeSlug);
                  }}
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
              onDragStart={() => setDraggedEntryId(entry.id)}
              onClick={() => onNavigate(`editor:${entry.id}:${entry.content_type}`)}
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
                  type="button"
                  onClick={(e) => {
                    e.stopPropagation();
                    void updateEntryStatus(entry, 'draft', contentTypeSlug);
                  }}
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
  );
};

