/**
 * ============================================================================
 * BARRE D'EN-TÊTE SOUVERAINE & FILTRES DE CONTENUS (`EntriesFilterBar`)
 * ============================================================================
 * Reproduit à 100% l'organisation de l'en-tête de `edit.php` (WordPress officiel) :
 * - Titre de page H1 + Bouton d'action inline « Ajouter un article / une page ».
 * - Navigation de statuts horizontale (`.subsubsub`) : Tous (X) | Publié (Y) | Brouillon (Z) | Corbeille (W).
 * - Barre d'outils (.tablenav) :
 *   - Actions groupées (Sélecteur + bouton « Appliquer »).
 *   - Filtre par date (« Toutes les dates »).
 *   - Filtre par catégorie (« Toutes les catégories ») + bouton « Filtrer ».
 *   - Bouton « Modifier les colonnes » (Admin Columns Pro).
 *   - Export CSV de la page.
 *   - Champ et bouton « Rechercher des articles ».
 *   - Commutateur de vue Liste / Kanban.
 */

import React, { useState } from 'react';
import { List, Columns3, SlidersHorizontal, Download, Settings2 } from 'lucide-react';
import { DEFAULT_ENTRY_FILTERS, useEntriesListStore } from '../../stores/useEntriesListStore';
import { useColumnSettingsStore, useColumns } from '../../stores/useColumnSettingsStore';
import type { ContentType } from '../../types';

interface EntriesFilterBarProps {
  contentTypeSlug: string;
  currentType?: ContentType;
  onNavigate: (view: string) => void;
}

export const EntriesFilterBar: React.FC<EntriesFilterBarProps> = ({
  contentTypeSlug,
  currentType,
  onNavigate,
}) => {
  const totalEntries = useEntriesListStore((s) => s.totalEntries);
  const entries = useEntriesListStore((s) => s.entries);
  const viewMode = useEntriesListStore((s) => s.viewMode);
  const setViewMode = useEntriesListStore((s) => s.setViewMode);
  const filtersByType = useEntriesListStore((s) => s.filtersByType);
  const updateFilter = useEntriesListStore((s) => s.updateFilter);
  const availableTerms = useEntriesListStore((s) => s.availableTerms);

  const selectedIds = useEntriesListStore((s) => s.selectedIds);
  const bulkAction = useEntriesListStore((s) => s.bulkAction);
  const bulkBusy = useEntriesListStore((s) => s.bulkBusy);
  const setBulkAction = useEntriesListStore((s) => s.setBulkAction);
  const clearSelection = useEntriesListStore((s) => s.clearSelection);
  const runBulkAction = useEntriesListStore((s) => s.runBulkAction);

  const currentFilters = filtersByType[contentTypeSlug] ?? DEFAULT_ENTRY_FILTERS;
  const { search, statusFilter, termFilter } = currentFilters;

  const [isColumnsOpen, setIsColumnsOpen] = useState(false);
  const [searchInput, setSearchInput] = useState(search);
  const toggleColumn = useColumnSettingsStore((s) => s.toggleColumn);
  const applyPreset = useColumnSettingsStore((s) => s.applyPreset);
  const exportVisibleCsv = useColumnSettingsStore((s) => s.exportVisibleCsv);
  const columns = useColumns(contentTypeSlug);

  const categoryTerms = availableTerms.filter((t) => t.taxonomy === 'category');
  const typeLabel = currentType?.name || (contentTypeSlug === 'page' ? 'Pages' : 'Articles');
  const singularLabel = currentType?.name_singular || (contentTypeSlug === 'page' ? 'page' : 'article');

  // Comptages réels pour la barre subsubsub
  const publishedCount = entries.filter((e) => e.status === 'published').length;
  const draftCount = entries.filter((e) => e.status === 'draft').length;
  const trashCount = entries.filter((e) => e.status === 'trash').length;

  const handleSearchSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    updateFilter(contentTypeSlug, { search: searchInput, page: 1 });
  };

  return (
    <div className="space-y-3 font-sans">
      {/* ==================================================================== */}
      {/* 1. LIGNE DU TITRE & BOUTON D'ACTION INLINE                           */}
      {/* ==================================================================== */}
      <div className="flex items-center gap-3 pt-1">
        <h1 className="text-2xl font-normal text-[#1d2327] dark:text-white tracking-tight">
          {typeLabel}
        </h1>
        <button
          type="button"
          onClick={() => onNavigate(`new:${contentTypeSlug}`)}
          className="px-2.5 py-1 text-xs font-medium rounded border border-[#2271b1] text-[#2271b1] hover:bg-[#2271b1] hover:text-white transition-colors bg-white dark:bg-[#182030] shadow-sm"
        >
          Ajouter un {singularLabel}
        </button>
      </div>

      {/* ==================================================================== */}
      {/* 2. BARRE HORIZONTALE DES STATUTS (.subsubsub)                        */}
      {/* ==================================================================== */}
      <ul className="flex items-center flex-wrap gap-1 text-[13px] text-[#646970] dark:text-slate-400">
        <li>
          <button
            type="button"
            onClick={() => updateFilter(contentTypeSlug, { statusFilter: 'all', page: 1 })}
            className={`transition-colors ${
              statusFilter === 'all'
                ? 'font-semibold text-[#1d2327] dark:text-white'
                : 'text-[#2271b1] dark:text-[#72aee6] hover:underline'
            }`}
          >
            Tous <span className="text-[#646970] dark:text-slate-400 font-normal">({totalEntries})</span>
          </button>
          <span className="mx-1.5 text-slate-300 dark:text-slate-600">|</span>
        </li>
        <li>
          <button
            type="button"
            onClick={() => updateFilter(contentTypeSlug, { statusFilter: 'published', page: 1 })}
            className={`transition-colors ${
              statusFilter === 'published'
                ? 'font-semibold text-[#1d2327] dark:text-white'
                : 'text-[#2271b1] dark:text-[#72aee6] hover:underline'
            }`}
          >
            Publié <span className="text-[#646970] dark:text-slate-400 font-normal">({publishedCount})</span>
          </button>
          <span className="mx-1.5 text-slate-300 dark:text-slate-600">|</span>
        </li>
        <li>
          <button
            type="button"
            onClick={() => updateFilter(contentTypeSlug, { statusFilter: 'draft', page: 1 })}
            className={`transition-colors ${
              statusFilter === 'draft'
                ? 'font-semibold text-[#1d2327] dark:text-white'
                : 'text-[#2271b1] dark:text-[#72aee6] hover:underline'
            }`}
          >
            Brouillon <span className="text-[#646970] dark:text-slate-400 font-normal">({draftCount})</span>
          </button>
          <span className="mx-1.5 text-slate-300 dark:text-slate-600">|</span>
        </li>
        <li>
          <button
            type="button"
            onClick={() => updateFilter(contentTypeSlug, { statusFilter: 'trash', page: 1 })}
            className={`transition-colors ${
              statusFilter === 'trash'
                ? 'font-semibold text-[#1d2327] dark:text-white'
                : 'text-[#2271b1] dark:text-[#72aee6] hover:underline'
            }`}
          >
            Corbeille <span className="text-[#646970] dark:text-slate-400 font-normal">({trashCount})</span>
          </button>
        </li>
      </ul>

      {/* ==================================================================== */}
      {/* 3. BARRE D'OUTILS ET FILTRES (.tablenav.top)                         */}
      {/* ==================================================================== */}
      <div className="flex flex-wrap items-center justify-between gap-3 text-xs pt-1">
        {/* Côté gauche : Actions groupées, Date, Catégories, Filtre, Colonnes */}
        <div className="flex flex-wrap items-center gap-2">
          {/* Actions groupées */}
          <select
            aria-label="Actions groupées"
            value={bulkAction}
            onChange={(e) => setBulkAction(e.target.value)}
            className="border border-[#8c8f94] bg-white dark:bg-[#182030] text-[#2c3338] dark:text-slate-200 rounded px-2 py-1 text-xs focus:ring-1 focus:ring-[#2271b1] focus:border-[#2271b1]"
          >
            <option value="">Actions groupées</option>
            <option value="published">Publier</option>
            <option value="draft">Brouillon</option>
            <option value="archived">Archiver</option>
            {statusFilter !== 'trash' && <option value="trash">Déplacer dans la corbeille</option>}
            <option value="force_delete">Supprimer définitivement</option>
          </select>

          <button
            type="button"
            disabled={!bulkAction || bulkBusy || selectedIds.length === 0}
            onClick={() => void runBulkAction(contentTypeSlug)}
            className="px-2.5 py-1 text-xs font-medium rounded border border-[#2271b1] text-[#2271b1] hover:bg-[#2271b1] hover:text-white disabled:opacity-40 disabled:hover:bg-transparent disabled:hover:text-[#2271b1] transition-colors bg-white dark:bg-[#182030]"
          >
            {bulkBusy ? 'Application...' : 'Appliquer'}
          </button>

          {/* Filtre par date */}
          <select
            aria-label="Filtrer par date"
            className="border border-[#8c8f94] bg-white dark:bg-[#182030] text-[#2c3338] dark:text-slate-200 rounded px-2 py-1 text-xs focus:ring-1 focus:ring-[#2271b1] focus:border-[#2271b1]"
          >
            <option value="">Toutes les dates</option>
            <option value="2026-10">Octobre 2026</option>
            <option value="2026-09">Septembre 2026</option>
            <option value="2026-08">Août 2026</option>
          </select>

          {/* Filtre par catégorie */}
          <select
            aria-label="Filtrer par catégorie"
            value={termFilter}
            onChange={(e) => updateFilter(contentTypeSlug, { termFilter: e.target.value, page: 1 })}
            className="border border-[#8c8f94] bg-white dark:bg-[#182030] text-[#2c3338] dark:text-slate-200 rounded px-2 py-1 text-xs focus:ring-1 focus:ring-[#2271b1] focus:border-[#2271b1]"
          >
            <option value="">Toutes les catégories</option>
            {categoryTerms.map((term) => (
              <option key={term.id} value={`category:${term.slug}`}>
                {term.name}
              </option>
            ))}
          </select>

          <button
            type="button"
            onClick={() => updateFilter(contentTypeSlug, { page: 1 })}
            className="px-2.5 py-1 text-xs font-medium rounded border border-[#2271b1] text-[#2271b1] hover:bg-[#2271b1] hover:text-white transition-colors bg-white dark:bg-[#182030]"
          >
            Filtrer
          </button>

          {/* Modifier les colonnes (Admin Columns Pro) */}
          <div className="relative">
            <button
              type="button"
              onClick={() => setIsColumnsOpen(!isColumnsOpen)}
              className="px-2.5 py-1 text-xs font-medium rounded border border-[#8c8f94] text-[#2c3338] dark:text-slate-200 hover:border-[#2271b1] transition-colors bg-white dark:bg-[#182030] flex items-center gap-1.5"
            >
              <SlidersHorizontal className="w-3.5 h-3.5 text-[#2271b1]" />
              <span>Modifier les colonnes</span>
            </button>

            {isColumnsOpen && (
              <div className="absolute left-0 mt-1 w-64 rounded bg-white dark:bg-[#1d2327] border border-[#c3c4c7] dark:border-[#2c3338] p-3 shadow-xl z-50 text-xs space-y-2.5">
                <div className="flex items-center justify-between pb-1.5 border-b border-slate-200 dark:border-[#2c3338]">
                  <span className="font-semibold text-[#1d2327] dark:text-white">Colonnes affichées</span>
                  <button
                    type="button"
                    onClick={() => {
                      setIsColumnsOpen(false);
                      onNavigate('settings:columns');
                    }}
                    className="text-[11px] text-[#2271b1] dark:text-[#72aee6] hover:underline flex items-center gap-1"
                  >
                    <Settings2 className="w-3 h-3" />
                    <span>Réglages</span>
                  </button>
                </div>

                <div className="grid grid-cols-2 gap-1 pb-1 border-b border-slate-200 dark:border-[#2c3338]">
                  {(['standard', 'editorial', 'seo', 'compact'] as const).map((p) => (
                    <button
                      key={p}
                      type="button"
                      onClick={() => applyPreset(contentTypeSlug, p)}
                      className="px-1.5 py-0.5 rounded bg-slate-100 dark:bg-[#2c3338] hover:bg-slate-200 dark:hover:bg-[#3c434a] text-left text-[11px] text-slate-700 dark:text-slate-300 capitalize"
                    >
                      {p}
                    </button>
                  ))}
                </div>

                <div className="space-y-1 max-h-40 overflow-y-auto">
                  {columns.map((col) => (
                    <label
                      key={col.id}
                      className="flex items-center justify-between px-1.5 py-0.5 rounded hover:bg-slate-50 dark:hover:bg-[#2c3338] cursor-pointer text-[11px]"
                    >
                      <span className={col.visible ? 'text-slate-800 dark:text-slate-200 font-medium' : 'text-slate-400'}>
                        {col.label}
                      </span>
                      <input
                        type="checkbox"
                        checked={col.visible}
                        onChange={() => toggleColumn(contentTypeSlug, col.id)}
                        className="rounded border-[#8c8f94] text-[#2271b1] focus:ring-[#2271b1]"
                      />
                    </label>
                  ))}
                </div>
              </div>
            )}
          </div>

          {/* Export CSV */}
          <button
            type="button"
            onClick={() => exportVisibleCsv(contentTypeSlug, entries)}
            title="Exporter la sélection en CSV"
            className="px-2 py-1 text-xs rounded border border-[#8c8f94] text-[#2c3338] dark:text-slate-200 hover:border-[#2271b1] transition-colors bg-white dark:bg-[#182030] flex items-center gap-1"
          >
            <Download className="w-3 h-3 text-[#2271b1]" />
            <span className="hidden sm:inline">CSV</span>
          </button>
        </div>

        {/* Côté droit : Recherche et bascule Vue Tableau / Kanban */}
        <div className="flex items-center gap-2 ml-auto">
          <form onSubmit={handleSearchSubmit} className="flex items-center gap-1">
            <div className="relative">
              <input
                type="search"
                value={searchInput}
                onChange={(e) => setSearchInput(e.target.value)}
                placeholder="Rechercher des articles"
                className="w-44 sm:w-56 border border-[#8c8f94] bg-white dark:bg-[#182030] text-[#2c3338] dark:text-slate-200 rounded px-2.5 py-1 text-xs focus:ring-1 focus:ring-[#2271b1] focus:border-[#2271b1]"
              />
            </div>
            <button
              type="submit"
              className="px-2.5 py-1 text-xs font-medium rounded border border-[#8c8f94] text-[#2c3338] dark:text-slate-200 hover:border-[#2271b1] transition-colors bg-white dark:bg-[#182030]"
            >
              Rechercher
            </button>
          </form>

          {/* Bascule Tableau / Kanban */}
          <div className="inline-flex rounded border border-[#8c8f94] p-0.5 bg-white dark:bg-[#182030]">
            <button
              type="button"
              onClick={() => setViewMode('table')}
              title="Vue Table classique"
              className={`p-1 rounded text-xs transition-colors ${
                viewMode === 'table'
                  ? 'bg-[#2271b1] text-white'
                  : 'text-slate-600 dark:text-slate-400 hover:text-[#2271b1]'
              }`}
            >
              <List className="w-3.5 h-3.5" />
            </button>
            <button
              type="button"
              onClick={() => setViewMode('kanban')}
              title="Vue Hub Kanban"
              className={`p-1 rounded text-xs transition-colors ${
                viewMode === 'kanban'
                  ? 'bg-[#2271b1] text-white'
                  : 'text-slate-600 dark:text-slate-400 hover:text-[#2271b1]'
              }`}
            >
              <Columns3 className="w-3.5 h-3.5" />
            </button>
          </div>
        </div>
      </div>

      {/* Notification de sélection active */}
      {selectedIds.length > 0 && (
        <div className="flex items-center justify-between px-3 py-1.5 rounded bg-[#f0f6fc] dark:bg-[#1e293b] border-l-4 border-[#2271b1] text-xs text-[#1d2327] dark:text-slate-200 shadow-sm animate-in fade-in duration-100">
          <span>
            <strong>{selectedIds.length}</strong> {selectedIds.length > 1 ? 'éléments sélectionnés' : 'élément sélectionné'}
          </span>
          <button
            type="button"
            onClick={clearSelection}
            className="text-[#2271b1] dark:text-[#72aee6] hover:underline"
          >
            Désélectionner tout
          </button>
        </div>
      )}
    </div>
  );
};
