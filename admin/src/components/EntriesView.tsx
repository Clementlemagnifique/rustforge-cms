/**
 * ============================================================================
 * VUE ORCHESTRÉE DES LISTES D'ARTICLES, PAGES & CONTENUS (`EntriesView`)
 * ============================================================================
 * Orchestre les sous-composants modulaires :
 * - `EntriesFilterBar` : en-tête contextuel (`WP-15`), filtres combinés (`WP-16`),
 *   tri et barre d'actions groupées (`WP-17`).
 * - `EntriesTable` : liste tabulaire et ligne de Modification rapide (`WP-18`).
 * - `EntriesKanban` : hub éditorial Kanban par statut.
 *
 * L'état complet est piloté par `useEntriesListStore` (Zustand) via des
 * sélecteurs atomiques pour éviter tout prop-drilling et re-rendu superflu.
 */

import React, { useEffect } from 'react';
import { DEFAULT_ENTRY_FILTERS, useEntriesListStore } from '../stores/useEntriesListStore';
import type { ContentType } from '../types';
import { EntriesFilterBar } from './entries/EntriesFilterBar';
import { EntriesKanban } from './entries/EntriesKanban';
import { EntriesTable } from './entries/EntriesTable';

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
  const viewMode = useEntriesListStore((s) => s.viewMode);
  const loading = useEntriesListStore((s) => s.loading);
  const error = useEntriesListStore((s) => s.error);
  const totalPages = useEntriesListStore((s) => s.totalPages);
  const filtersByType = useEntriesListStore((s) => s.filtersByType);
  const setActiveContentType = useEntriesListStore((s) => s.setActiveContentType);
  const loadTerms = useEntriesListStore((s) => s.loadTerms);
  const updateFilter = useEntriesListStore((s) => s.updateFilter);

  const currentType = contentTypes.find((ct) => ct.slug === contentTypeSlug);
  const currentFilters = filtersByType[contentTypeSlug] ?? DEFAULT_ENTRY_FILTERS;
  const { page } = currentFilters;

  useEffect(() => {
    void loadTerms();
  }, [loadTerms]);

  useEffect(() => {
    setActiveContentType(contentTypeSlug);
  }, [contentTypeSlug, setActiveContentType]);

  return (
    <div className="space-y-5">
      {error && (
        <p
          role="alert"
          className="text-red-300 text-xs bg-red-950/50 border border-red-500/30 rounded-xl px-4 py-2.5"
        >
          {error}
        </p>
      )}

      <EntriesFilterBar
        contentTypeSlug={contentTypeSlug}
        currentType={currentType}
        onNavigate={onNavigate}
      />

      {viewMode === 'table' ? (
        <EntriesTable contentTypeSlug={contentTypeSlug} onNavigate={onNavigate} />
      ) : (
        <EntriesKanban contentTypeSlug={contentTypeSlug} onNavigate={onNavigate} />
      )}

      <nav aria-label="Pagination des contenus" className="flex gap-4 items-center text-sm">
        <button
          disabled={page <= 1 || loading}
          onClick={() => updateFilter(contentTypeSlug, { page: page - 1 })}
          className="px-3 py-1 rounded-lg bg-[#0f1422] border border-white/10 disabled:opacity-40 text-xs"
        >
          Précédent
        </button>
        <span className="text-xs text-slate-400">
          Page {page} / {totalPages}
        </span>
        <button
          disabled={page >= totalPages || loading}
          onClick={() => updateFilter(contentTypeSlug, { page: page + 1 })}
          className="px-3 py-1 rounded-lg bg-[#0f1422] border border-white/10 disabled:opacity-40 text-xs"
        >
          Suivant
        </button>
      </nav>
    </div>
  );
};
