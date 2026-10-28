/**
 * ============================================================================
 * VUE DE GESTION DES CATÉGORIES HIÉRARCHIQUES & ÉTIQUETTES (`TaxonomiesView`)
 * ============================================================================
 * Couvre l'exigence `WP-19` sur le modèle familier de `wp-admin/edit-tags.php` :
 * - Colonne gauche : création ou édition d'un terme (nom, slug, catégorie
 *   parente anti-cycle, description).
 * - Colonne droite : recherche, arbre hiérarchique indenté (`—`), compteur
 *   cliquable vers la liste filtrée des articles (`EntriesView`), fusion
 *   sans perte (`merge`) et suppression avec choix de réaffectation (`reassign_to`).
 */

import React, { useEffect, useMemo } from 'react';
import {
  Tag,
  FolderTree,
  Search,
  Edit3,
  GitMerge,
  Trash2,
  Plus,
  X,
  Check,
} from 'lucide-react';
import { taxonomiesMessages } from '../i18n/messages.fr';
import { useEntriesListStore } from '../stores/useEntriesListStore';
import { useTaxonomiesStore } from '../stores/useTaxonomiesStore';
import type { TermRef } from '../types';

interface TaxonomiesViewProps {
  taxonomySlug: string; // 'category' | 'tag'
  onNavigate: (view: string) => void;
}

interface OrderedTerm {
  term: TermRef;
  depth: number;
}

/** Ordonne les termes en arbre hiérarchique avec profondeur d'indentation. */
function buildOrderedTerms(terms: TermRef[], hierarchical: boolean): OrderedTerm[] {
  if (!hierarchical) {
    return terms.map((term) => ({ term, depth: 0 }));
  }
  const byId = new Set(terms.map((t) => t.id));
  const childrenMap = new Map<string | null, TermRef[]>();

  for (const term of terms) {
    const parentKey = term.parent_id && byId.has(term.parent_id) ? term.parent_id : null;
    const list = childrenMap.get(parentKey) || [];
    list.push(term);
    childrenMap.set(parentKey, list);
  }

  const result: OrderedTerm[] = [];
  const visited = new Set<string>();

  function visit(parentId: string | null, depth: number) {
    const list = childrenMap.get(parentId) || [];
    for (const item of list) {
      if (visited.has(item.id)) continue;
      visited.add(item.id);
      result.push({ term: item, depth });
      visit(item.id, depth + 1);
    }
  }

  visit(null, 0);
  // Ajouter tout terme résiduel éventuel non visité
  for (const term of terms) {
    if (!visited.has(term.id)) {
      result.push({ term, depth: 0 });
    }
  }
  return result;
}

export const TaxonomiesView: React.FC<TaxonomiesViewProps> = ({
  taxonomySlug,
  onNavigate,
}) => {
  const terms = useTaxonomiesStore((s) => s.terms);
  const loading = useTaxonomiesStore((s) => s.loading);
  const saving = useTaxonomiesStore((s) => s.saving);
  const error = useTaxonomiesStore((s) => s.error);
  const search = useTaxonomiesStore((s) => s.search);

  const editingTermId = useTaxonomiesStore((s) => s.editingTermId);
  const name = useTaxonomiesStore((s) => s.name);
  const slug = useTaxonomiesStore((s) => s.slug);
  const parentId = useTaxonomiesStore((s) => s.parentId);
  const description = useTaxonomiesStore((s) => s.description);

  const actionTermId = useTaxonomiesStore((s) => s.actionTermId);
  const actionMode = useTaxonomiesStore((s) => s.actionMode);
  const targetTermId = useTaxonomiesStore((s) => s.targetTermId);

  const setTaxonomy = useTaxonomiesStore((s) => s.setTaxonomy);
  const setSearch = useTaxonomiesStore((s) => s.setSearch);
  const setName = useTaxonomiesStore((s) => s.setName);
  const setSlug = useTaxonomiesStore((s) => s.setSlug);
  const setParentId = useTaxonomiesStore((s) => s.setParentId);
  const setDescription = useTaxonomiesStore((s) => s.setDescription);
  const startCreate = useTaxonomiesStore((s) => s.startCreate);
  const startEdit = useTaxonomiesStore((s) => s.startEdit);
  const saveTerm = useTaxonomiesStore((s) => s.saveTerm);
  const openTermAction = useTaxonomiesStore((s) => s.openTermAction);
  const cancelTermAction = useTaxonomiesStore((s) => s.cancelTermAction);
  const setTargetTermId = useTaxonomiesStore((s) => s.setTargetTermId);
  const confirmTermAction = useTaxonomiesStore((s) => s.confirmTermAction);

  const updateEntriesFilter = useEntriesListStore((s) => s.updateFilter);

  const isCategory = taxonomySlug === 'category';

  useEffect(() => {
    setTaxonomy(taxonomySlug);
  }, [taxonomySlug, setTaxonomy]);

  const orderedTerms = useMemo(() => {
    const q = search.trim().toLowerCase();
    const filtered = q
      ? terms.filter(
          (t) =>
            t.name.toLowerCase().includes(q) ||
            t.slug.toLowerCase().includes(q) ||
            (t.description || '').toLowerCase().includes(q)
        )
      : terms;
    return buildOrderedTerms(filtered, isCategory && !q);
  }, [terms, search, isCategory]);

  const editingTerm = terms.find((t) => t.id === editingTermId);
  const parentCandidates = terms.filter((t) => t.id !== editingTermId);

  const handleOpenEntriesForTerm = (term: TermRef) => {
    updateEntriesFilter('post', {
      termFilter: `${taxonomySlug}:${term.slug}`,
      page: 1,
    });
    onNavigate('entries:post');
  };

  return (
    <div className="space-y-5">
      {/* En-tête de la taxonomie */}
      <div className="flex flex-wrap items-center justify-between gap-3 bg-[#0f1422] border border-white/5 rounded-2xl px-5 py-4">
        <div className="space-y-1">
          <div className="flex items-center gap-2.5">
            {isCategory ? (
              <FolderTree className="w-4 h-4 text-indigo-400" />
            ) : (
              <Tag className="w-4 h-4 text-amber-400" />
            )}
            <h2 className="text-sm font-bold text-white">
              {isCategory
                ? taxonomiesMessages.categoriesTitle
                : taxonomiesMessages.tagsTitle}
            </h2>
            <span className="px-2 py-0.5 rounded-full bg-white/5 text-[11px] text-slate-400">
              {terms.length} terme{terms.length > 1 ? 's' : ''}
            </span>
          </div>
          <p className="text-xs text-slate-400">
            {isCategory
              ? taxonomiesMessages.categoriesSubtitle
              : taxonomiesMessages.tagsSubtitle}
          </p>
        </div>

        <div className="flex items-center gap-2">
          <button
            type="button"
            onClick={() => onNavigate('taxonomy:category')}
            className={`px-3 py-1.5 rounded-xl text-xs font-semibold border transition-colors ${
              isCategory
                ? 'bg-indigo-600/20 border-indigo-500/40 text-indigo-200'
                : 'bg-white/5 border-white/10 text-slate-400 hover:text-white'
            }`}
          >
            Catégories
          </button>
          <button
            type="button"
            onClick={() => onNavigate('taxonomy:tag')}
            className={`px-3 py-1.5 rounded-xl text-xs font-semibold border transition-colors ${
              !isCategory
                ? 'bg-indigo-600/20 border-indigo-500/40 text-indigo-200'
                : 'bg-white/5 border-white/10 text-slate-400 hover:text-white'
            }`}
          >
            Étiquettes
          </button>
        </div>
      </div>

      {error && (
        <p
          role="alert"
          className="text-red-300 text-xs bg-red-950/50 border border-red-500/30 rounded-xl px-4 py-2.5"
        >
          {error}
        </p>
      )}

      {/* Grille 2 colonnes façon WordPress (Formulaire à gauche, Tableau à droite) */}
      <div className="grid grid-cols-1 lg:grid-cols-12 gap-6 items-start">
        {/* COLONNE GAUCHE : FORMULAIRE AJOUT / ÉDITION */}
        <form
          onSubmit={(e) => {
            e.preventDefault();
            void saveTerm();
          }}
          className="lg:col-span-4 rounded-2xl bg-[#0f1422] border border-white/5 p-5 space-y-4"
        >
          <div className="flex items-center justify-between border-b border-white/5 pb-3">
            <h3 className="text-xs font-bold uppercase tracking-wider text-slate-200">
              {editingTerm
                ? taxonomiesMessages.editTermTitle.replace('{name}', editingTerm.name)
                : isCategory
                ? taxonomiesMessages.addCategoryTitle
                : taxonomiesMessages.addTagTitle}
            </h3>
            {editingTerm && (
              <button
                type="button"
                onClick={startCreate}
                className="text-xs text-slate-400 hover:text-white underline"
              >
                {taxonomiesMessages.cancelEdit}
              </button>
            )}
          </div>

          <label className="block space-y-1 text-xs">
            <span className="text-slate-300 font-semibold">{taxonomiesMessages.fieldName}</span>
            <input
              type="text"
              required
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder={isCategory ? 'Ex : Guides Rust' : 'Ex : webassembly'}
              className="w-full rounded-xl bg-black/30 border border-white/10 px-3 py-2 text-xs text-white focus:outline-none focus:border-indigo-500"
            />
          </label>

          <label className="block space-y-1 text-xs">
            <span className="text-slate-300 font-semibold">{taxonomiesMessages.fieldSlug}</span>
            <input
              type="text"
              value={slug}
              onChange={(e) => setSlug(e.target.value)}
              placeholder={taxonomiesMessages.fieldSlugPlaceholder}
              className="w-full rounded-xl bg-black/30 border border-white/10 px-3 py-2 text-xs font-mono text-white focus:outline-none focus:border-indigo-500"
            />
          </label>

          {isCategory && (
            <label className="block space-y-1 text-xs">
              <span className="text-slate-300 font-semibold">
                {taxonomiesMessages.fieldParent}
              </span>
              <select
                value={parentId || ''}
                onChange={(e) => setParentId(e.target.value || null)}
                className="w-full rounded-xl bg-black/30 border border-white/10 px-3 py-2 text-xs text-white focus:outline-none focus:border-indigo-500"
              >
                <option value="">{taxonomiesMessages.parentNone}</option>
                {parentCandidates.map((candidate) => (
                  <option key={candidate.id} value={candidate.id}>
                    {candidate.name} (/{candidate.slug})
                  </option>
                ))}
              </select>
            </label>
          )}

          <label className="block space-y-1 text-xs">
            <span className="text-slate-300 font-semibold">
              {taxonomiesMessages.fieldDescription}
            </span>
            <textarea
              rows={3}
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder="Description facultative affichée sur les pages d’archive…"
              className="w-full rounded-xl bg-black/30 border border-white/10 p-3 text-xs text-white focus:outline-none focus:border-indigo-500"
            />
          </label>

          <button
            type="submit"
            disabled={saving}
            className="w-full py-2.5 px-4 rounded-xl bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 text-white text-xs font-semibold flex items-center justify-center gap-2 shadow-md shadow-indigo-600/20"
          >
            {editingTerm ? <Check className="w-4 h-4" /> : <Plus className="w-4 h-4" />}
            <span>
              {saving
                ? 'Enregistrement…'
                : editingTerm
                ? taxonomiesMessages.updateButton
                : taxonomiesMessages.createButton}
            </span>
          </button>
        </form>

        {/* COLONNE DROITE : RECHERCHE & TABLEAU HIÉRARCHIQUE */}
        <div className="lg:col-span-8 space-y-4">
          <div className="relative">
            <Search className="w-4 h-4 text-slate-400 absolute left-3.5 top-2.5" />
            <input
              type="text"
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              placeholder={taxonomiesMessages.searchPlaceholder}
              className="w-full bg-[#0f1422] border border-white/10 rounded-xl pl-10 pr-4 py-2 text-xs text-white placeholder-slate-500 focus:outline-none focus:border-indigo-500"
            />
          </div>

          <div className="rounded-2xl bg-[#0f1422] border border-white/5 overflow-hidden shadow-sm">
            <table className="w-full text-left text-xs">
              <thead>
                <tr className="border-b border-white/5 bg-white/[0.01] text-slate-400 font-semibold uppercase tracking-wider text-[10px]">
                  <th className="py-3.5 px-4">Nom</th>
                  <th className="py-3.5 px-4">Description</th>
                  <th className="py-3.5 px-4">Slug</th>
                  <th className="py-3.5 px-4 text-center">Contenus</th>
                  <th className="py-3.5 px-4 text-right">Actions</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-white/5">
                {loading ? (
                  <tr>
                    <td colSpan={5} className="py-8 text-center text-slate-500">
                      Chargement des termes…
                    </td>
                  </tr>
                ) : orderedTerms.length === 0 ? (
                  <tr>
                    <td colSpan={5} className="py-10 text-center text-slate-500">
                      {taxonomiesMessages.emptyTerms}
                    </td>
                  </tr>
                ) : (
                  orderedTerms.map(({ term, depth }) => {
                    const isActionOpen = actionTermId === term.id;
                    const targetCandidates = terms.filter((t) => t.id !== term.id);
                    return (
                      <React.Fragment key={term.id}>
                        <tr className="hover:bg-white/[0.02] transition-colors">
                          <td className="py-3.5 px-4 font-semibold text-slate-100">
                            <span className="text-slate-500 mr-1">
                              {'— '.repeat(depth)}
                            </span>
                            {term.name}
                          </td>
                          <td className="py-3.5 px-4 text-slate-400 max-w-xs truncate">
                            {term.description || '—'}
                          </td>
                          <td className="py-3.5 px-4 font-mono text-[11px] text-slate-400">
                            {term.slug}
                          </td>
                          <td className="py-3.5 px-4 text-center">
                            <button
                              type="button"
                              onClick={() => handleOpenEntriesForTerm(term)}
                              title="Voir les articles associés à ce terme"
                              className="px-2.5 py-0.5 rounded-full bg-indigo-500/10 border border-indigo-500/20 text-indigo-300 hover:bg-indigo-500/20 font-semibold text-[11px]"
                            >
                              {term.entry_count ?? 0}
                            </button>
                          </td>
                          <td className="py-3.5 px-4 text-right">
                            <div className="flex items-center justify-end gap-1">
                              <button
                                type="button"
                                onClick={() => startEdit(term)}
                                title="Modifier ce terme"
                                className="p-1.5 rounded-lg text-slate-400 hover:text-indigo-300 hover:bg-indigo-500/10 transition-colors"
                              >
                                <Edit3 className="w-3.5 h-3.5" />
                              </button>
                              {targetCandidates.length > 0 && (
                                <button
                                  type="button"
                                  onClick={() => openTermAction(term.id, 'merge')}
                                  title={taxonomiesMessages.mergeAction}
                                  className="p-1.5 rounded-lg text-slate-400 hover:text-amber-300 hover:bg-amber-500/10 transition-colors"
                                >
                                  <GitMerge className="w-3.5 h-3.5" />
                                </button>
                              )}
                              <button
                                type="button"
                                onClick={() => openTermAction(term.id, 'delete')}
                                title={taxonomiesMessages.deleteAction}
                                className="p-1.5 rounded-lg text-slate-400 hover:text-red-400 hover:bg-red-500/10 transition-colors"
                              >
                                <Trash2 className="w-3.5 h-3.5" />
                              </button>
                            </div>
                          </td>
                        </tr>

                        {/* Ligne d'action Fusion ou Suppression avec réaffectation (WP-19) */}
                        {isActionOpen && (
                          <tr className="bg-slate-900/95 border-y border-indigo-500/30">
                            <td colSpan={5} className="p-4 space-y-3">
                              <div className="flex items-center justify-between">
                                <span className="text-xs font-semibold text-indigo-200">
                                  {actionMode === 'merge'
                                    ? `Fusionner « ${term.name} » vers un autre terme (conserve tous les articles et sous-termes)`
                                    : `Supprimer « ${term.name} » (avec réaffectation optionnelle des articles)`}
                                </span>
                                <button
                                  type="button"
                                  onClick={cancelTermAction}
                                  className="text-slate-400 hover:text-white"
                                >
                                  <X className="w-4 h-4" />
                                </button>
                              </div>

                              <div className="flex flex-wrap items-center gap-3">
                                <select
                                  aria-label="Terme cible de réaffectation"
                                  value={targetTermId}
                                  onChange={(e) => setTargetTermId(e.target.value)}
                                  className="flex-1 min-w-[220px] rounded-xl bg-slate-950 border border-slate-700 px-3 py-1.5 text-xs text-slate-100"
                                >
                                  {actionMode === 'delete' && (
                                    <option value="">
                                      {taxonomiesMessages.reassignNone}
                                    </option>
                                  )}
                                  {targetCandidates.map((candidate) => (
                                    <option key={candidate.id} value={candidate.id}>
                                      Réaffecter vers : {candidate.name} (/{candidate.slug})
                                    </option>
                                  ))}
                                </select>

                                <button
                                  type="button"
                                  onClick={cancelTermAction}
                                  className="px-3 py-1.5 rounded-lg border border-slate-700 text-slate-300 hover:bg-slate-800 text-xs"
                                >
                                  Annuler
                                </button>
                                <button
                                  type="button"
                                  disabled={saving || (actionMode === 'merge' && !targetTermId)}
                                  onClick={() => void confirmTermAction()}
                                  className={`px-3 py-1.5 rounded-lg text-white text-xs font-semibold disabled:opacity-50 ${
                                    actionMode === 'merge'
                                      ? 'bg-amber-600 hover:bg-amber-500'
                                      : 'bg-red-600 hover:bg-red-500'
                                  }`}
                                >
                                  {saving
                                    ? 'Traitement…'
                                    : actionMode === 'merge'
                                    ? taxonomiesMessages.confirmMergeButton
                                    : taxonomiesMessages.confirmDeleteButton}
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
        </div>
      </div>
    </div>
  );
};

