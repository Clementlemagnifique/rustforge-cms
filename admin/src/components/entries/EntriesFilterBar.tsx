/**
 * ============================================================================
 * BARRE D'EN-TÊTE, FILTRES COMBINÉS, TRI & ACTIONS GROUPÉES (`EntriesFilterBar`)
 * ============================================================================
 * Couvre les exigences :
 * - `WP-15` : En-tête contextuel distinct par type de contenu (Articles vs Pages).
 * - `WP-16` : Filtres combinés recherche (`q`), statut (dont `trash`) et termes.
 * - `WP-17` : Tri (`updated`, `published`, `title`, `created`) et barre d'actions
 *   groupées sur la sélection active.
 */

import React from 'react';
import { Plus, Search, List, Columns3, ArrowUpDown } from 'lucide-react';
import { entriesMessages } from '../../i18n/messages.fr';
import { DEFAULT_ENTRY_FILTERS, useEntriesListStore } from '../../stores/useEntriesListStore';
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
  const { search, statusFilter, termFilter, orderFilter } = currentFilters;

  const categoryTerms = availableTerms.filter((t) => t.taxonomy === 'category');
  const tagTerms = availableTerms.filter((t) => t.taxonomy === 'tag');
  const typeLabel = currentType?.name || (contentTypeSlug === 'page' ? 'Pages' : 'Articles');

  return (
    <div className="space-y-4">
      {/* En-tête contextuel par type de contenu (WP-15) */}
      <div className="flex flex-wrap items-center justify-between gap-3 bg-[#0f1422] border border-white/5 rounded-2xl px-4 py-3">
        <div className="flex items-center gap-3">
          <span className="px-2.5 py-1 rounded-lg bg-indigo-500/10 border border-indigo-500/20 text-indigo-300 text-xs font-semibold">
            {typeLabel}
          </span>
          <span className="text-xs text-slate-400">
            {totalEntries} élément{totalEntries > 1 ? 's' : ''}
            {contentTypeSlug === 'page'
              ? ' • Contenus hiérarchiques (parent / modèle)'
              : ' • Flux éditorial chronologique & taxonomies'}
          </span>
        </div>

        <button
          onClick={() => onNavigate(`new:${contentTypeSlug}`)}
          className="px-4 py-2 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-semibold flex items-center justify-center gap-2 shadow-md shadow-indigo-600/20 transition-all"
        >
          <Plus className="w-4 h-4" />
          <span>
            {contentTypeSlug === 'page'
              ? 'Nouvelle page'
              : contentTypeSlug === 'post'
              ? 'Nouvel article'
              : `Nouveau ${currentType?.name_singular || 'contenu'}`}
          </span>
        </button>
      </div>

      {/* Contrôles de recherche, statut, taxonomies, tri et bascule Table / Kanban */}
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div className="flex flex-wrap items-center gap-2.5 flex-1">
          <div className="relative min-w-[220px] flex-1 max-w-md">
            <Search className="w-4 h-4 text-slate-400 absolute left-3 top-2.5" />
            <input
              type="text"
              value={search}
              onChange={(e) =>
                updateFilter(contentTypeSlug, { search: e.target.value, page: 1 })
              }
              placeholder={entriesMessages.searchPlaceholder.replace(
                '{name}',
                currentType?.name || 'les contenus'
              )}
              className="w-full bg-[#0f1422] border border-white/10 rounded-xl pl-9 pr-4 py-2 text-xs text-white placeholder-slate-500 focus:outline-none focus:border-indigo-500"
            />
          </div>

          {viewMode === 'table' && (
            <>
              <select
                aria-label="Filtrer par statut"
                value={statusFilter}
                onChange={(e) =>
                  updateFilter(contentTypeSlug, { statusFilter: e.target.value, page: 1 })
                }
                className="bg-[#0f1422] border border-white/10 rounded-xl px-3 py-2 text-xs text-slate-200 focus:outline-none focus:border-indigo-500"
              >
                <option value="all">{entriesMessages.statusAll}</option>
                <option value="published">{entriesMessages.statusPublished}</option>
                <option value="draft">{entriesMessages.statusDraft}</option>
                <option value="archived">{entriesMessages.statusArchived}</option>
                <option value="trash">{entriesMessages.statusTrash}</option>
              </select>

              <select
                aria-label="Filtrer par catégorie ou étiquette"
                value={termFilter}
                onChange={(e) =>
                  updateFilter(contentTypeSlug, { termFilter: e.target.value, page: 1 })
                }
                className="bg-[#0f1422] border border-white/10 rounded-xl px-3 py-2 text-xs text-slate-200 focus:outline-none focus:border-indigo-500"
              >
                <option value="">{entriesMessages.termsAll}</option>
                {categoryTerms.length > 0 && (
                  <optgroup label={entriesMessages.categoriesGroup}>
                    {categoryTerms.map((term) => (
                      <option key={term.id} value={`category:${term.slug}`}>
                        Catégorie : {term.name}
                      </option>
                    ))}
                  </optgroup>
                )}
                {tagTerms.length > 0 && (
                  <optgroup label={entriesMessages.tagsGroup}>
                    {tagTerms.map((term) => (
                      <option key={term.id} value={`tag:${term.slug}`}>
                        Étiquette : {term.name}
                      </option>
                    ))}
                  </optgroup>
                )}
              </select>

              <div className="flex items-center gap-1.5 bg-[#0f1422] border border-white/10 rounded-xl px-2.5 py-1.5">
                <ArrowUpDown className="w-3.5 h-3.5 text-slate-400" />
                <select
                  aria-label="Trier les contenus"
                  value={orderFilter}
                  onChange={(e) =>
                    updateFilter(contentTypeSlug, { orderFilter: e.target.value, page: 1 })
                  }
                  className="bg-transparent text-xs text-slate-200 focus:outline-none"
                >
                  <option value="updated">{entriesMessages.orderUpdated}</option>
                  <option value="published">{entriesMessages.orderPublished}</option>
                  <option value="title">{entriesMessages.orderTitle}</option>
                  <option value="created">{entriesMessages.orderCreated}</option>
                </select>
              </div>
            </>
          )}
        </div>

        {/* Sélecteur de vue Tableau / Kanban */}
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

      {/* Barre d'actions groupées (WP-17) */}
      {viewMode === 'table' && selectedIds.length > 0 && (
        <div className="flex flex-wrap items-center justify-between gap-3 bg-indigo-950/40 border border-indigo-500/30 rounded-xl px-4 py-2.5 text-xs">
          <div className="flex items-center gap-3">
            <span className="font-semibold text-indigo-200">
              {selectedIds.length} contenu{selectedIds.length > 1 ? 's' : ''} sélectionné
              {selectedIds.length > 1 ? 's' : ''}
            </span>
            <button
              type="button"
              onClick={clearSelection}
              className="text-slate-400 hover:text-white underline"
            >
              {entriesMessages.clearSelection}
            </button>
          </div>
          <div className="flex items-center gap-2">
            <select
              aria-label="Action groupée"
              value={bulkAction}
              onChange={(e) => setBulkAction(e.target.value)}
              className="bg-[#0f1422] border border-white/15 rounded-lg px-3 py-1.5 text-xs text-slate-100"
            >
              <option value="">{entriesMessages.bulkSelectAction}</option>
              <option value="published">{entriesMessages.bulkPublish}</option>
              <option value="draft">{entriesMessages.bulkDraft}</option>
              <option value="archived">{entriesMessages.bulkArchive}</option>
              {statusFilter !== 'trash' && (
                <option value="trash">{entriesMessages.bulkTrash}</option>
              )}
              <option value="force_delete">{entriesMessages.bulkForceDelete}</option>
            </select>
            <button
              type="button"
              disabled={!bulkAction || bulkBusy}
              onClick={() => void runBulkAction(contentTypeSlug)}
              className="px-3 py-1.5 rounded-lg bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 text-white font-semibold"
            >
              {bulkBusy ? entriesMessages.bulkApplying : entriesMessages.bulkApply}
            </button>
          </div>
        </div>
      )}
    </div>
  );
};

