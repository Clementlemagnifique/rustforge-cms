/**
 * ============================================================================
 * TABLEAU DES CONTENUS & MODIFICATION RAPIDE EN LIGNE (`EntriesTable`)
 * ============================================================================
 * Couvre les exigences :
 * - `WP-17` : Sélection unitaire ou globale sur la page, actions vers corbeille,
 *   restauration et suppression définitive.
 * - `WP-18` : Édition rapide en ligne (Quick Edit : titre, slug, statut,
 *   catégories/étiquettes) par patch partiel qui conserve `rf.document/1` et `_rf_vault`.
 */

import React from 'react';
import {
  Trash2,
  Clock,
  Eye,
  RotateCcw,
  Calendar,
  Edit3,
  Check,
  X,
  Tag,
} from 'lucide-react';
import { entriesMessages } from '../../i18n/messages.fr';
import { useEntriesListStore } from '../../stores/useEntriesListStore';
import type { EntryStatus } from '../../types';

interface EntriesTableProps {
  contentTypeSlug: string;
  onNavigate: (view: string) => void;
}

export const EntriesTable: React.FC<EntriesTableProps> = ({
  contentTypeSlug,
  onNavigate,
}) => {
  const entries = useEntriesListStore((s) => s.entries);
  const loading = useEntriesListStore((s) => s.loading);
  const availableTerms = useEntriesListStore((s) => s.availableTerms);
  const selectedIds = useEntriesListStore((s) => s.selectedIds);
  const toggleSelectRow = useEntriesListStore((s) => s.toggleSelectRow);
  const toggleSelectAllOnPage = useEntriesListStore((s) => s.toggleSelectAllOnPage);
  const deleteEntry = useEntriesListStore((s) => s.deleteEntry);
  const updateEntryStatus = useEntriesListStore((s) => s.updateEntryStatus);

  const quickEditId = useEntriesListStore((s) => s.quickEditId);
  const quickTitle = useEntriesListStore((s) => s.quickTitle);
  const quickSlug = useEntriesListStore((s) => s.quickSlug);
  const quickStatus = useEntriesListStore((s) => s.quickStatus);
  const quickTermIds = useEntriesListStore((s) => s.quickTermIds);
  const quickSaving = useEntriesListStore((s) => s.quickSaving);
  const openQuickEdit = useEntriesListStore((s) => s.openQuickEdit);
  const cancelQuickEdit = useEntriesListStore((s) => s.cancelQuickEdit);
  const setQuickTitle = useEntriesListStore((s) => s.setQuickTitle);
  const setQuickSlug = useEntriesListStore((s) => s.setQuickSlug);
  const setQuickStatus = useEntriesListStore((s) => s.setQuickStatus);
  const toggleQuickTerm = useEntriesListStore((s) => s.toggleQuickTerm);
  const saveQuickEdit = useEntriesListStore((s) => s.saveQuickEdit);

  const allOnPageSelected =
    entries.length > 0 && entries.every((entry) => selectedIds.includes(entry.id));

  return (
    <div className="rounded-2xl bg-[#0f1422] border border-white/5 overflow-hidden shadow-sm">
      <table className="w-full text-left text-xs">
        <thead>
          <tr className="border-b border-white/5 bg-white/[0.01] text-slate-400 font-semibold uppercase tracking-wider text-[10px]">
            <th className="py-3.5 pl-4 pr-2 w-8">
              <input
                type="checkbox"
                aria-label="Tout sélectionner sur la page"
                checked={allOnPageSelected}
                onChange={toggleSelectAllOnPage}
                className="rounded border-white/20 bg-slate-900"
              />
            </th>
            <th className="py-3.5 px-4">Titre, Slug & Taxonomies</th>
            <th className="py-3.5 px-4">Auteur</th>
            <th className="py-3.5 px-4">Statut</th>
            <th className="py-3.5 px-4">Programmation / Date</th>
            <th className="py-3.5 px-4 text-right">Actions</th>
          </tr>
        </thead>
        <tbody className="divide-y divide-white/5">
          {loading ? (
            <tr>
              <td colSpan={6} className="py-8 text-center text-slate-500">
                {entriesMessages.loadingList}
              </td>
            </tr>
          ) : entries.length === 0 ? (
            <tr>
              <td colSpan={6} className="py-12 text-center text-slate-500">
                {entriesMessages.emptyList}
              </td>
            </tr>
          ) : (
            entries.map((entry) => {
              const isSelected = selectedIds.includes(entry.id);
              const isQuickEditing = quickEditId === entry.id;
              return (
                <React.Fragment key={entry.id}>
                  <tr
                    onClick={() => onNavigate(`editor:${entry.id}:${entry.content_type}`)}
                    className={`hover:bg-white/[0.02] cursor-pointer transition-colors group ${
                      isSelected ? 'bg-indigo-950/20' : ''
                    }`}
                  >
                    <td
                      className="py-3.5 pl-4 pr-2"
                      onClick={(e) => {
                        e.stopPropagation();
                        toggleSelectRow(entry.id);
                      }}
                    >
                      <input
                        type="checkbox"
                        aria-label={`Sélectionner ${entry.title}`}
                        checked={isSelected}
                        onChange={() => toggleSelectRow(entry.id)}
                        onClick={(e) => e.stopPropagation()}
                        className="rounded border-white/20 bg-slate-900"
                      />
                    </td>
                    <td className="py-3.5 px-4">
                      <p className="font-semibold text-slate-100 group-hover:text-indigo-400 transition-colors">
                        {entry.title}
                      </p>
                      <div className="flex flex-wrap items-center gap-2 mt-1">
                        <span className="text-[11px] text-slate-500 font-mono">
                          /{entry.slug}
                        </span>
                        {entry.terms &&
                          entry.terms.map((term) => (
                            <span
                              key={term.id}
                              className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded bg-white/5 text-[10px] text-slate-300"
                            >
                              <Tag className="w-2.5 h-2.5 text-indigo-400" />
                              {term.name}
                            </span>
                          ))}
                      </div>
                    </td>
                    <td className="py-3.5 px-4 text-slate-300">
                      {entry.author_name || 'Éditeur'}
                    </td>
                    <td className="py-3.5 px-4">
                      {(() => {
                        const isScheduled =
                          entry.status === 'draft' &&
                          !!entry.published_at &&
                          new Date(entry.published_at).getTime() > Date.now();
                        return (
                          <span
                            className={`text-[10px] px-2 py-0.5 rounded-full font-medium ${
                              entry.status === 'published'
                                ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20'
                                : isScheduled
                                ? 'bg-cyan-500/10 text-cyan-300 border border-cyan-500/20'
                                : entry.status === 'draft'
                                ? 'bg-amber-500/10 text-amber-400 border border-amber-500/20'
                                : entry.status === 'trash'
                                ? 'bg-red-500/10 text-red-400 border border-red-500/20'
                                : 'bg-slate-500/10 text-slate-400'
                            }`}
                          >
                            {entry.status === 'published'
                              ? 'Publié'
                              : isScheduled
                              ? 'Planifié'
                              : entry.status === 'draft'
                              ? 'Brouillon'
                              : entry.status === 'trash'
                              ? 'Corbeille'
                              : 'Archivé'}
                          </span>
                        );
                      })()}
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
                            <span>
                              Expire :{' '}
                              {new Date(entry.scheduled_unpublish_at).toLocaleDateString('fr-FR')}
                            </span>
                          </div>
                        )}
                      </div>
                    </td>
                    <td className="py-3.5 px-4 text-right">
                      <div className="flex items-center justify-end gap-1">
                        <button
                          type="button"
                          onClick={(e) => {
                            e.stopPropagation();
                            openQuickEdit(entry);
                          }}
                          title="Modification rapide (titre, slug, statut, taxonomies)"
                          className="p-1.5 rounded-lg text-slate-400 hover:text-indigo-300 hover:bg-indigo-500/10 transition-colors"
                        >
                          <Edit3 className="w-3.5 h-3.5" />
                        </button>
                        <a
                          href={`/fr/${entry.content_type}/${entry.slug}`}
                          target="_blank"
                          rel="noreferrer"
                          onClick={(e) => e.stopPropagation()}
                          title="Aperçu public"
                          className="p-1.5 rounded-lg text-slate-400 hover:text-white hover:bg-white/5 transition-colors"
                        >
                          <Eye className="w-3.5 h-3.5" />
                        </a>
                        {entry.status === 'trash' ? (
                          <>
                            <button
                              type="button"
                              onClick={(e) => {
                                e.stopPropagation();
                                void updateEntryStatus(entry, 'draft', contentTypeSlug);
                              }}
                              title="Restaurer en brouillon"
                              className="p-1.5 rounded-lg text-emerald-400 hover:bg-emerald-500/10 transition-colors"
                            >
                              <RotateCcw className="w-3.5 h-3.5" />
                            </button>
                            <button
                              type="button"
                              onClick={(e) => {
                                e.stopPropagation();
                                void deleteEntry(entry.id, true, contentTypeSlug);
                              }}
                              title="Supprimer définitivement"
                              className="p-1.5 rounded-lg text-red-400 hover:bg-red-500/10 transition-colors"
                            >
                              <Trash2 className="w-3.5 h-3.5" />
                            </button>
                          </>
                        ) : (
                          <button
                            type="button"
                            onClick={(e) => {
                              e.stopPropagation();
                              void deleteEntry(entry.id, false, contentTypeSlug);
                            }}
                            title="Mettre à la corbeille"
                            className="p-1.5 rounded-lg text-slate-400 hover:text-red-400 hover:bg-red-500/10 transition-colors"
                          >
                            <Trash2 className="w-3.5 h-3.5" />
                          </button>
                        )}
                      </div>
                    </td>
                  </tr>

                  {/* Ligne de Modification rapide en ligne (WP-18) */}
                  {isQuickEditing && (
                    <tr
                      onClick={(e) => e.stopPropagation()}
                      className="bg-slate-900/95 border-y border-indigo-500/30"
                    >
                      <td colSpan={6} className="p-4 space-y-3">
                        <div className="flex items-center justify-between">
                          <span className="text-xs font-semibold text-indigo-300">
                            {entriesMessages.quickEditTitle}
                          </span>
                          <button
                            type="button"
                            onClick={cancelQuickEdit}
                            className="text-slate-400 hover:text-white"
                          >
                            <X className="w-4 h-4" />
                          </button>
                        </div>

                        <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
                          <label className="space-y-1">
                            <span className="text-[11px] text-slate-400">Titre</span>
                            <input
                              type="text"
                              value={quickTitle}
                              onChange={(e) => setQuickTitle(e.target.value)}
                              className="w-full rounded-lg bg-slate-950 border border-slate-700 px-2.5 py-1.5 text-xs text-slate-100"
                            />
                          </label>
                          <label className="space-y-1">
                            <span className="text-[11px] text-slate-400">Slug</span>
                            <input
                              type="text"
                              value={quickSlug}
                              onChange={(e) => setQuickSlug(e.target.value)}
                              className="w-full rounded-lg bg-slate-950 border border-slate-700 px-2.5 py-1.5 text-xs text-slate-100 font-mono"
                            />
                          </label>
                          <label className="space-y-1">
                            <span className="text-[11px] text-slate-400">Statut</span>
                            <select
                              value={quickStatus}
                              onChange={(e) => setQuickStatus(e.target.value as EntryStatus)}
                              className="w-full rounded-lg bg-slate-950 border border-slate-700 px-2.5 py-1.5 text-xs text-slate-100"
                            >
                              <option value="draft">Brouillon</option>
                              <option value="published">Publié</option>
                              <option value="archived">Archivé</option>
                            </select>
                          </label>
                        </div>

                        {availableTerms.length > 0 && (
                          <div className="space-y-1.5">
                            <span className="text-[11px] text-slate-400">
                              Catégories & Étiquettes associées
                            </span>
                            <div className="flex flex-wrap gap-2">
                              {availableTerms.map((term) => {
                                const checked = quickTermIds.includes(term.id);
                                return (
                                  <label
                                    key={term.id}
                                    className={`inline-flex items-center gap-1.5 px-2.5 py-1 rounded-lg border text-[11px] cursor-pointer ${
                                      checked
                                        ? 'bg-indigo-600/20 border-indigo-500/50 text-indigo-200'
                                        : 'bg-slate-950 border-slate-800 text-slate-400'
                                    }`}
                                  >
                                    <input
                                      type="checkbox"
                                      checked={checked}
                                      onChange={() => toggleQuickTerm(term.id)}
                                    />
                                    <span>
                                      {term.taxonomy === 'category' ? 'Cat.' : 'Tag'} : {term.name}
                                    </span>
                                  </label>
                                );
                              })}
                            </div>
                          </div>
                        )}

                        <div className="flex items-center justify-end gap-2 pt-1">
                          <button
                            type="button"
                            onClick={cancelQuickEdit}
                            className="px-3 py-1.5 rounded-lg border border-slate-700 text-slate-300 hover:bg-slate-800 text-xs"
                          >
                            {entriesMessages.quickEditCancel}
                          </button>
                          <button
                            type="button"
                            disabled={quickSaving}
                            onClick={() => void saveQuickEdit(entry, contentTypeSlug)}
                            className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 text-white text-xs font-semibold"
                          >
                            <Check className="w-3.5 h-3.5" />
                            <span>
                              {quickSaving
                                ? entriesMessages.quickEditSaving
                                : entriesMessages.quickEditSave}
                            </span>
                          </button>
                        </div>
                      </td>
                    </tr>
                  )}
                </React.Fragment>
              );
            })
          )}
        </tbody>
      </table>
    </div>
  );
};

