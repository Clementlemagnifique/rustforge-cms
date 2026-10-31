/**
 * ============================================================================
 * TABLEAU DE CONTENUS SOUVERAIN (`EntriesTable` / `.rf-list-table`)
 * ============================================================================
 * Reproduit à 100% le tableau officiel de WordPress `edit.php` (WP_List_Table) :
 * - Checkbox de sélection unitaire et globale.
 * - Colonne Titre avec actions de ligne au survol (Row Actions) :
 *   « Modifier | Modification rapide | Corbeille | Afficher | Dupliquer ».
 * - Modification rapide en ligne (Quick Edit à 2 colonnes fidèle à WordPress).
 * - Colonnes officielles : Titre, Auteur/autrice, Catégories, Étiquettes,
 *   Commentaires (bulle), Date (« Publié YYYY/MM/DD à HH:mm »).
 * - Répétition des en-têtes dans le pied de tableau.
 */

import React from 'react';
import {
  MessageSquare,
  ArrowUpDown,
  ExternalLink,
} from 'lucide-react';
import { useStudioStore } from '../../stores/useStudioStore';
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
  const duplicateEntry = useEntriesListStore((s) => s.duplicateEntry);

  // Quick edit state
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

  const density = useStudioStore((s) => s.studioDensity);
  const paddingY = density === 'compact' ? 'py-1.5' : 'py-2.5';

  const allOnPageSelected =
    entries.length > 0 && entries.every((entry) => selectedIds.includes(entry.id));

  const formatWpDate = (dateStr: string, status: string) => {
    try {
      const d = new Date(dateStr);
      const yyyy = d.getFullYear();
      const mm = String(d.getMonth() + 1).padStart(2, '0');
      const dd = String(d.getDate()).padStart(2, '0');
      const hh = String(d.getHours()).padStart(2, '0');
      const min = String(d.getMinutes()).padStart(2, '0');

      const label = status === 'published' ? 'Publié' : status === 'draft' ? 'Dernière modification' : 'Enregistré';
      return (
        <div className="text-[12px] leading-tight">
          <span className="text-[#646970] dark:text-slate-400 block">{label}</span>
          <span className="font-normal text-[#1d2327] dark:text-slate-200">
            {`${yyyy}/${mm}/${dd} à ${hh}:${min}`}
          </span>
        </div>
      );
    } catch {
      return <span>{dateStr}</span>;
    }
  };

  const renderTableHeader = (_isFooter = false) => (
    <tr className="border-b border-[#c3c4c7] dark:border-[#2c3338] bg-[#f6f7f7] dark:bg-[#131619] text-[#2c3338] dark:text-slate-200 text-xs font-medium">
      <th className={`${paddingY} pl-3 pr-2 w-8 text-center`}>
        <input
          type="checkbox"
          aria-label="Tout sélectionner"
          checked={allOnPageSelected}
          onChange={toggleSelectAllOnPage}
          className="rounded border-[#8c8f94] text-[#2271b1] focus:ring-[#2271b1] w-3.5 h-3.5"
        />
      </th>
      <th className={`${paddingY} px-3 text-left font-semibold`}>
        <div className="inline-flex items-center gap-1 group cursor-pointer hover:text-[#2271b1]">
          <span>Titre</span>
          <ArrowUpDown className="w-3 h-3 opacity-60 group-hover:opacity-100" />
        </div>
      </th>
      <th className={`${paddingY} px-3 text-left font-semibold`}>Auteur/autrice</th>
      <th className={`${paddingY} px-3 text-left font-semibold`}>Catégories</th>
      <th className={`${paddingY} px-3 text-left font-semibold`}>Étiquettes</th>
      <th className={`${paddingY} px-2 text-center w-12 font-semibold`} title="Commentaires">
        <MessageSquare className="w-3.5 h-3.5 mx-auto text-[#646970] dark:text-slate-400" />
      </th>
      <th className={`${paddingY} px-3 text-left font-semibold`}>
        <div className="inline-flex items-center gap-1 group cursor-pointer hover:text-[#2271b1]">
          <span>Date</span>
          <ArrowUpDown className="w-3 h-3 opacity-60 group-hover:opacity-100" />
        </div>
      </th>
    </tr>
  );

  return (
    <div className="border border-[#c3c4c7] dark:border-[#2c3338] rounded bg-white dark:bg-[#182030] shadow-sm overflow-x-auto font-sans">
      <table className="w-full text-left text-xs border-collapse">
        <thead>{renderTableHeader(false)}</thead>
        <tbody className="divide-y divide-[#f0f0f1] dark:divide-[#2c3338]">
          {loading ? (
            <tr>
              <td colSpan={7} className="py-12 text-center text-[#646970] dark:text-slate-400">
                Chargement des éléments...
              </td>
            </tr>
          ) : entries.length === 0 ? (
            <tr>
              <td colSpan={7} className="py-12 text-center text-[#646970] dark:text-slate-400">
                Aucun contenu trouvé.
              </td>
            </tr>
          ) : (
            entries.map((entry) => {
              const isSelected = selectedIds.includes(entry.id);
              const isQuickEditing = quickEditId === entry.id;
              const cats = (entry.terms || []).filter((t) => t.taxonomy === 'category');
              const tags = (entry.terms || []).filter((t) => t.taxonomy === 'tag');

              return (
                <React.Fragment key={entry.id}>
                  <tr
                    className={`group transition-colors ${
                      isSelected
                        ? 'bg-[#f0f6fc] dark:bg-[#1e293b]'
                        : 'hover:bg-[#f6f7f7] dark:hover:bg-[#1a2332]'
                    }`}
                  >
                    {/* Checkbox */}
                    <td className={`${paddingY} pl-3 pr-2 w-8 text-center align-top pt-3`}>
                      <input
                        type="checkbox"
                        checked={isSelected}
                        onChange={() => toggleSelectRow(entry.id)}
                        className="rounded border-[#8c8f94] text-[#2271b1] focus:ring-[#2271b1] w-3.5 h-3.5"
                      />
                    </td>

                    {/* Titre & Row Actions */}
                    <td className={`${paddingY} px-3 align-top`}>
                      <div className="font-semibold text-[13px] leading-snug">
                        <button
                          type="button"
                          onClick={() => onNavigate(`editor:${entry.id}:${entry.content_type}`)}
                          className="text-[#2271b1] dark:text-[#72aee6] hover:underline text-left"
                        >
                          {entry.title || '(Sans titre)'}
                        </button>
                        {entry.status === 'draft' && (
                          <span className="font-normal text-[#646970] dark:text-slate-400 ml-1.5">
                            — Brouillon
                          </span>
                        )}
                        {entry.status === 'trash' && (
                          <span className="font-normal text-[#d63638] ml-1.5">
                            — Corbeille
                          </span>
                        )}
                      </div>

                      {/* Row Actions au survol */}
                      <div className="flex items-center flex-wrap gap-1 text-[11px] pt-1 text-[#2271b1] dark:text-[#72aee6] opacity-0 group-hover:opacity-100 transition-opacity">
                        <button
                          type="button"
                          onClick={() => onNavigate(`editor:${entry.id}:${entry.content_type}`)}
                          className="hover:underline"
                        >
                          Modifier
                        </button>
                        <span className="text-slate-300 dark:text-slate-600">|</span>
                        <button
                          type="button"
                          onClick={() => openQuickEdit(entry)}
                          className="hover:underline"
                        >
                          Modification rapide
                        </button>
                        <span className="text-slate-300 dark:text-slate-600">|</span>
                        {entry.status === 'trash' ? (
                          <>
                            <button
                              type="button"
                              onClick={() => void updateEntryStatus(entry, 'draft', contentTypeSlug)}
                              className="text-emerald-600 hover:underline"
                            >
                              Rétablir
                            </button>
                            <span className="text-slate-300 dark:text-slate-600">|</span>
                            <button
                              type="button"
                              onClick={() => void deleteEntry(entry.id, true, contentTypeSlug)}
                              className="text-[#d63638] hover:underline"
                            >
                              Supprimer définitivement
                            </button>
                          </>
                        ) : (
                          <button
                            type="button"
                            onClick={() => void deleteEntry(entry.id, false, contentTypeSlug)}
                            className="text-[#d63638] hover:underline"
                          >
                            Corbeille
                          </button>
                        )}
                        <span className="text-slate-300 dark:text-slate-600">|</span>
                        <a
                          href={`/fr/${entry.content_type}/${entry.slug}`}
                          target="_blank"
                          rel="noreferrer"
                          className="hover:underline inline-flex items-center gap-0.5"
                        >
                          <span>Afficher</span>
                          <ExternalLink className="w-2.5 h-2.5 opacity-70" />
                        </a>
                        <span className="text-slate-300 dark:text-slate-600">|</span>
                        <button
                          type="button"
                          onClick={() => void duplicateEntry(entry, contentTypeSlug)}
                          className="hover:underline"
                        >
                          Dupliquer
                        </button>
                      </div>
                    </td>

                    {/* Auteur */}
                    <td className={`${paddingY} px-3 text-[#2c3338] dark:text-slate-300 align-top pt-3`}>
                      {entry.author_name || 'Admin'}
                    </td>

                    {/* Catégories */}
                    <td className={`${paddingY} px-3 align-top pt-3`}>
                      {cats.length > 0 ? (
                        <div className="flex flex-wrap gap-1 text-[#2271b1] dark:text-[#72aee6]">
                          {cats.map((c) => (
                            <span key={c.id} className="hover:underline cursor-pointer">
                              {c.name}
                            </span>
                          ))}
                        </div>
                      ) : (
                        <span className="text-slate-400">—</span>
                      )}
                    </td>

                    {/* Étiquettes */}
                    <td className={`${paddingY} px-3 align-top pt-3`}>
                      {tags.length > 0 ? (
                        <div className="flex flex-wrap gap-1 text-[#2271b1] dark:text-[#72aee6]">
                          {tags.map((t) => (
                            <span key={t.id} className="hover:underline cursor-pointer">
                              {t.name}
                            </span>
                          ))}
                        </div>
                      ) : (
                        <span className="text-slate-400">—</span>
                      )}
                    </td>

                    {/* Commentaires */}
                    <td className={`${paddingY} px-2 text-center align-top pt-3`}>
                      <span className="inline-flex items-center justify-center min-w-5 px-1 py-0.5 rounded-full bg-slate-100 dark:bg-slate-800 text-[11px] text-[#646970] dark:text-slate-300 font-medium">
                        0
                      </span>
                    </td>

                    {/* Date */}
                    <td className={`${paddingY} px-3 align-top pt-3`}>
                      {formatWpDate(entry.published_at || entry.updated_at, entry.status)}
                    </td>
                  </tr>

                  {/* ==================================================================== */}
                  {/* MODIFICATION RAPIDE EN LIGNE (Quick Edit WordPress authentique)      */}
                  {/* ==================================================================== */}
                  {isQuickEditing && (
                    <tr className="bg-[#f0f0f1] dark:bg-[#131619] border-y-2 border-[#2271b1]">
                      <td colSpan={7} className="p-4">
                        <div className="max-w-4xl space-y-4">
                          <h4 className="text-xs font-bold text-[#1d2327] dark:text-white uppercase tracking-wider">
                            MODIFICATION RAPIDE
                          </h4>

                          <div className="grid grid-cols-1 md:grid-cols-2 gap-6 text-xs">
                            {/* Colonne Gauche : Titre, Identifiant, Date, Auteur */}
                            <div className="space-y-3">
                              <label className="block">
                                <span className="font-semibold text-[#1d2327] dark:text-slate-200 block mb-1">
                                  Titre
                                </span>
                                <input
                                  type="text"
                                  value={quickTitle}
                                  onChange={(e) => setQuickTitle(e.target.value)}
                                  className="w-full border border-[#8c8f94] bg-white dark:bg-[#182030] text-[#2c3338] dark:text-slate-200 rounded px-2.5 py-1.5 text-xs focus:ring-1 focus:ring-[#2271b1]"
                                />
                              </label>

                              <label className="block">
                                <span className="font-semibold text-[#1d2327] dark:text-slate-200 block mb-1">
                                  Identifiant (Slug)
                                </span>
                                <input
                                  type="text"
                                  value={quickSlug}
                                  onChange={(e) => setQuickSlug(e.target.value)}
                                  className="w-full border border-[#8c8f94] bg-white dark:bg-[#182030] text-[#2c3338] dark:text-slate-200 rounded px-2.5 py-1.5 text-xs font-mono focus:ring-1 focus:ring-[#2271b1]"
                                />
                              </label>

                              <div className="flex items-center gap-4 pt-1">
                                <span className="font-semibold text-[#1d2327] dark:text-slate-200">
                                  Date :
                                </span>
                                <span className="text-[#646970] dark:text-slate-400">
                                  {new Date(entry.updated_at).toLocaleString('fr-FR')}
                                </span>
                              </div>
                            </div>

                            {/* Colonne Droite : Catégories & Statut */}
                            <div className="space-y-3">
                              {availableTerms.length > 0 && (
                                <div>
                                  <span className="font-semibold text-[#1d2327] dark:text-slate-200 block mb-1">
                                    Catégories
                                  </span>
                                  <div className="max-h-28 overflow-y-auto border border-[#8c8f94] bg-white dark:bg-[#182030] rounded p-2 space-y-1">
                                    {availableTerms
                                      .filter((t) => t.taxonomy === 'category')
                                      .map((term) => {
                                        const checked = quickTermIds.includes(term.id);
                                        return (
                                          <label
                                            key={term.id}
                                            className="flex items-center gap-2 cursor-pointer text-[12px] hover:text-[#2271b1]"
                                          >
                                            <input
                                              type="checkbox"
                                              checked={checked}
                                              onChange={() => toggleQuickTerm(term.id)}
                                              className="rounded border-[#8c8f94] text-[#2271b1]"
                                            />
                                            <span>{term.name}</span>
                                          </label>
                                        );
                                      })}
                                  </div>
                                </div>
                              )}

                              <label className="block">
                                <span className="font-semibold text-[#1d2327] dark:text-slate-200 block mb-1">
                                  État (Statut)
                                </span>
                                <select
                                  value={quickStatus}
                                  onChange={(e) => setQuickStatus(e.target.value as EntryStatus)}
                                  className="w-full border border-[#8c8f94] bg-white dark:bg-[#182030] text-[#2c3338] dark:text-slate-200 rounded px-2.5 py-1.5 text-xs focus:ring-1 focus:ring-[#2271b1]"
                                >
                                  <option value="published">Publié</option>
                                  <option value="draft">Brouillon</option>
                                  <option value="archived">Archivé</option>
                                </select>
                              </label>
                            </div>
                          </div>

                          {/* Boutons d'action Quick Edit */}
                          <div className="flex items-center justify-end gap-2 pt-2 border-t border-slate-300 dark:border-[#2c3338]">
                            <button
                              type="button"
                              onClick={cancelQuickEdit}
                              className="px-3 py-1.5 rounded border border-[#8c8f94] bg-white dark:bg-[#182030] hover:bg-slate-50 dark:hover:bg-[#2c3338] text-[#2c3338] dark:text-slate-200 text-xs transition-colors"
                            >
                              Annuler
                            </button>
                            <button
                              type="button"
                              disabled={quickSaving}
                              onClick={() => void saveQuickEdit(entry, contentTypeSlug)}
                              className="px-4 py-1.5 rounded bg-[#2271b1] hover:bg-[#135e96] disabled:opacity-50 text-white text-xs font-semibold shadow-sm transition-colors"
                            >
                              {quickSaving ? 'Enregistrement...' : 'Mettre à jour'}
                            </button>
                          </div>
                        </div>
                      </td>
                    </tr>
                  )}
                </React.Fragment>
              );
            })
          )}
        </tbody>
        <tfoot>{renderTableHeader(true)}</tfoot>
      </table>
    </div>
  );
};
