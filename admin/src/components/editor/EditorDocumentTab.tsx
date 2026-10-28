/**
 * ============================================================================
 * ONGLET DOCUMENT & PUBLICATION (`EditorDocumentTab`)
 * ============================================================================
 * Couvre les exigences :
 * - `WP-27` : Statut éditorial et planification (`published_at`).
 * - `WP-20` : Page parente hiérarchique (`parent_id`) avec garde anti-cycle serveur.
 * - `WP-32` : Extrait (`excerpt`), image mise en avant (`featured_media_id`)
 *   et sélection/création rapide de catégories et étiquettes (`term_ids`).
 */

import React from 'react';
import { Sliders, Image as ImageIcon, Tag } from 'lucide-react';
import { useEditorStore } from '../../stores/useEditorStore';
import type { ContentType, EntryStatus } from '../../types';

interface EditorDocumentTabProps {
  currentType?: ContentType;
}

export const EditorDocumentTab: React.FC<EditorDocumentTabProps> = ({ currentType }) => {
  const status = useEditorStore((s) => s.status);
  const publishedAt = useEditorStore((s) => s.publishedAt);
  const parentId = useEditorStore((s) => s.parentId);
  const excerpt = useEditorStore((s) => s.excerpt);
  const featuredMediaId = useEditorStore((s) => s.featuredMediaId);
  const selectedTermIds = useEditorStore((s) => s.selectedTermIds);
  const availableTerms = useEditorStore((s) => s.availableTerms);
  const newTermName = useEditorStore((s) => s.newTermName);
  const newTermTaxonomy = useEditorStore((s) => s.newTermTaxonomy);
  const mediaLibrary = useEditorStore((s) => s.mediaLibrary);
  const candidateParents = useEditorStore((s) => s.candidateParents);

  const setStatus = useEditorStore((s) => s.setStatus);
  const setPublishedAt = useEditorStore((s) => s.setPublishedAt);
  const setParentId = useEditorStore((s) => s.setParentId);
  const setExcerpt = useEditorStore((s) => s.setExcerpt);
  const setFeaturedMediaId = useEditorStore((s) => s.setFeaturedMediaId);
  const toggleTermId = useEditorStore((s) => s.toggleTermId);
  const setNewTermName = useEditorStore((s) => s.setNewTermName);
  const setNewTermTaxonomy = useEditorStore((s) => s.setNewTermTaxonomy);
  const uploadMedia = useEditorStore((s) => s.uploadMedia);
  const createTerm = useEditorStore((s) => s.createTerm);

  const featuredMedia = mediaLibrary.find((m) => m.id === featuredMediaId);

  return (
    <div className="grid grid-cols-1 lg:grid-cols-2 gap-5">
      {/* Colonne 1 : Extrait, Publication, Hiérarchie */}
      <div className="p-6 rounded-2xl bg-[#0f1422] border border-white/5 space-y-5">
        <h4 className="text-sm font-bold text-white flex items-center gap-2">
          <Sliders className="w-4 h-4 text-emerald-400" />
          <span>Réglages de publication et extrait</span>
        </h4>

        <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
          <label className="block space-y-1 text-xs">
            <span className="text-slate-300 font-semibold">Statut éditorial</span>
            <select
              value={status}
              onChange={(e) => setStatus(e.target.value as EntryStatus)}
              className="w-full rounded-xl border border-white/10 bg-black/30 px-3 py-2 text-xs text-white"
            >
              <option value="draft">Brouillon</option>
              <option value="published">Publié</option>
              <option value="archived">Archivé</option>
            </select>
          </label>

          <label className="block space-y-1 text-xs">
            <span className="text-slate-300 font-semibold">
              Date de publication / planification
            </span>
            <input
              type="datetime-local"
              value={publishedAt}
              onChange={(e) => setPublishedAt(e.target.value)}
              className="w-full rounded-xl border border-white/10 bg-black/30 px-3 py-2 text-xs text-white"
            />
          </label>
        </div>

        {currentType?.hierarchical && (
          <label className="block space-y-1 text-xs">
            <span className="text-slate-300 font-semibold">Page parente (hiérarchie)</span>
            <select
              value={parentId || ''}
              onChange={(e) => setParentId(e.target.value || null)}
              className="w-full rounded-xl border border-white/10 bg-black/30 px-3 py-2 text-xs text-white"
            >
              <option value="">(Aucune page parente — racine)</option>
              {candidateParents.map((parent) => (
                <option key={parent.id} value={parent.id}>
                  {parent.title} (/{parent.slug})
                </option>
              ))}
            </select>
          </label>
        )}

        <label className="block space-y-1 text-xs">
          <span className="text-slate-300 font-semibold">Extrait / Résumé éditorial</span>
          <textarea
            rows={4}
            value={excerpt}
            onChange={(e) => setExcerpt(e.target.value)}
            placeholder="Résumé court affiché dans les listes d’articles, les flux et les aperçus de partage…"
            className="w-full rounded-xl border border-white/10 bg-black/30 p-3 text-xs text-white"
          />
        </label>
      </div>

      {/* Colonne 2 : Image mise en avant & Taxonomies */}
      <div className="p-6 rounded-2xl bg-[#0f1422] border border-white/5 space-y-5">
        <div className="space-y-3">
          <h4 className="text-sm font-bold text-white flex items-center gap-2">
            <ImageIcon className="w-4 h-4 text-indigo-400" />
            <span>Image mise en avant</span>
          </h4>
          <div className="flex flex-wrap items-center gap-3 text-xs">
            <select
              aria-label="Image mise en avant"
              value={featuredMediaId || ''}
              onChange={(e) => setFeaturedMediaId(e.target.value || null)}
              className="flex-1 rounded-xl border border-white/10 bg-black/30 px-3 py-2 text-xs text-white"
            >
              <option value="">(Aucune image mise en avant)</option>
              {mediaLibrary
                .filter((m) => m.mime_type.startsWith('image/'))
                .map((m) => (
                  <option key={m.id} value={m.id}>
                    {m.original_name} ({m.url || `/uploads/${m.path}`})
                  </option>
                ))}
            </select>
            <label className="cursor-pointer rounded-xl border border-white/10 bg-white/5 px-3 py-2 text-xs text-slate-200 hover:bg-white/10">
              Téléverser
              <input
                type="file"
                accept="image/*"
                className="sr-only"
                onChange={async (e) => {
                  const file = e.target.files?.[0];
                  if (!file) return;
                  const uploaded = await uploadMedia(file);
                  if (uploaded) {
                    setFeaturedMediaId(uploaded.id);
                  }
                  e.target.value = '';
                }}
              />
            </label>
          </div>
          {featuredMedia && (
            <p className="text-xs text-slate-400 font-mono">
              Sélectionnée : {featuredMedia.original_name} (
              {featuredMedia.url || `/uploads/${featuredMedia.path}`})
            </p>
          )}
        </div>

        <div className="space-y-3 border-t border-white/5 pt-4">
          <h4 className="text-sm font-bold text-white flex items-center gap-2">
            <Tag className="w-4 h-4 text-amber-400" />
            <span>Catégories et étiquettes</span>
          </h4>
          <div className="flex flex-wrap gap-2 max-h-36 overflow-y-auto">
            {availableTerms.map((term) => {
              const active = selectedTermIds.includes(term.id);
              return (
                <label
                  key={term.id}
                  className={`cursor-pointer rounded-lg border px-2.5 py-1 text-xs flex items-center gap-1.5 ${
                    active
                      ? 'border-indigo-400 bg-indigo-500/20 text-indigo-100'
                      : 'border-white/10 bg-white/5 text-slate-300'
                  }`}
                >
                  <input
                    type="checkbox"
                    checked={active}
                    onChange={(e) => toggleTermId(term.id, e.target.checked)}
                  />
                  <span>{term.name}</span>
                  <span className="text-[10px] text-slate-400 font-mono">({term.taxonomy})</span>
                </label>
              );
            })}
          </div>
          <div className="flex flex-wrap items-center gap-2 pt-2 text-xs">
            <select
              value={newTermTaxonomy}
              onChange={(e) => setNewTermTaxonomy(e.target.value as 'category' | 'tag')}
              className="rounded-xl border border-white/10 bg-black/30 px-2.5 py-1.5 text-xs text-white"
            >
              <option value="category">Catégorie</option>
              <option value="tag">Étiquette</option>
            </select>
            <input
              type="text"
              value={newTermName}
              onChange={(e) => setNewTermName(e.target.value)}
              placeholder="Nouveau terme…"
              className="flex-1 rounded-xl border border-white/10 bg-black/30 px-3 py-1.5 text-xs text-white"
            />
            <button
              type="button"
              onClick={() => void createTerm()}
              className="rounded-xl border border-indigo-400/30 bg-indigo-500/10 px-3 py-1.5 text-xs text-indigo-200 hover:bg-indigo-500/20"
            >
              + Ajouter
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};

