/**
 * ============================================================================
 * VUE DE LA MÉDIATHÈQUE, VARIANTES ET TRANSFORMATIONS (`MediaView`)
 * ============================================================================
 * Pilotée par `useMediaStore` (Zustand) :
 * - `WP-37` : Recherche (`q`), filtre par type MIME (`kind`), bascule Grille / Liste,
 *   filtrage et gestion des dossiers virtuels (`media_folders`).
 * - `WP-38` : Téléversement dans le dossier actif et inspecteur latéral d'édition
 *   des métadonnées (`alt`, `caption`, déplacement de dossier, copie d'URL et WebP).
 * - `WP-39` : Édition et transformation d'image (rotation, recadrage) avec
 *   régénération des variantes et restauration réversible de l'original souverain.
 * - `WP-40` : Suppression protégée contre la destruction d'un média utilisé dans
 *   un contenu actif (`409 Conflict` avec confirmation de forçage).
 * - `WP-41` : Chargement optimisé, lazy-loading, copie d'extrait HTML responsive (`srcset` & `sizes`).
 * - `WP-88` : Visualisation des variantes générées (Thumbnail, Medium, Large).
 */

import React, { useEffect, useState } from 'react';
import {
  Upload,
  Trash2,
  File,
  Folder,
  FolderPlus,
  Search,
  Grid,
  List,
  Copy,
  Check,
  X,
  RotateCcw,
  RotateCw,
  Undo2,
  Sparkles,
} from 'lucide-react';
import { mediaMessages } from '../i18n/messages.fr';
import { useMediaStore } from '../stores/useMediaStore';
import type { Media } from '../types';

function formatBytes(bytes: number): string {
  if (!bytes || bytes <= 0) return '0 o';
  if (bytes < 1024) return `${bytes} o`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} Ko`;
  return `${(bytes / (1024 * 1024)).toFixed(2)} Mo`;
}

export const MediaView: React.FC = () => {
  const items = useMediaStore((s) => s.items);
  const total = useMediaStore((s) => s.total);
  const folders = useMediaStore((s) => s.folders);
  const loading = useMediaStore((s) => s.loading);
  const uploading = useMediaStore((s) => s.uploading);
  const savingMeta = useMediaStore((s) => s.savingMeta);
  const rotating = useMediaStore((s) => s.rotating);
  const error = useMediaStore((s) => s.error);
  const message = useMediaStore((s) => s.message);

  const viewMode = useMediaStore((s) => s.viewMode);
  const searchQuery = useMediaStore((s) => s.searchQuery);
  const kindFilter = useMediaStore((s) => s.kindFilter);
  const selectedFolderId = useMediaStore((s) => s.selectedFolderId);
  const newFolderName = useMediaStore((s) => s.newFolderName);

  const selectedMedia = useMediaStore((s) => s.selectedMedia);
  const editAlt = useMediaStore((s) => s.editAlt);
  const editCaption = useMediaStore((s) => s.editCaption);
  const editFolderId = useMediaStore((s) => s.editFolderId);

  const setViewMode = useMediaStore((s) => s.setViewMode);
  const setSearchQuery = useMediaStore((s) => s.setSearchQuery);
  const setKindFilter = useMediaStore((s) => s.setKindFilter);
  const setSelectedFolderId = useMediaStore((s) => s.setSelectedFolderId);
  const setNewFolderName = useMediaStore((s) => s.setNewFolderName);
  const selectMedia = useMediaStore((s) => s.selectMedia);
  const setEditAlt = useMediaStore((s) => s.setEditAlt);
  const setEditCaption = useMediaStore((s) => s.setEditCaption);
  const setEditFolderId = useMediaStore((s) => s.setEditFolderId);

  const fetchAll = useMediaStore((s) => s.fetchAll);
  const uploadFile = useMediaStore((s) => s.uploadFile);
  const createFolder = useMediaStore((s) => s.createFolder);
  const deleteFolder = useMediaStore((s) => s.deleteFolder);
  const saveSelectedMediaMeta = useMediaStore((s) => s.saveSelectedMediaMeta);
  const rotateSelectedMedia = useMediaStore((s) => s.rotateSelectedMedia);
  const restoreSelectedMedia = useMediaStore((s) => s.restoreSelectedMedia);
  const deleteMedia = useMediaStore((s) => s.deleteMedia);

  const [copiedSnippet, setCopiedSnippet] = useState(false);

  useEffect(() => {
    void fetchAll();
  }, [searchQuery, kindFilter, selectedFolderId, fetchAll]);

  const handleFileUpload = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;
    await uploadFile(file);
    e.target.value = '';
  };

  const buildResponsiveHtml = (m: Media): string => {
    const variants = m.metadata?.variants || {};
    const list = Object.values(variants);
    const alt = m.alt || m.original_name;
    if (list.length === 0) {
      return `<img src="${m.url}" alt="${alt}" loading="lazy" decoding="async" />`;
    }
    const srcset = list
      .map((v) => `${v.url} ${v.width}w`)
      .concat(`${m.url} ${m.width || 1200}w`)
      .join(', ');
    return `<img src="${m.url}" srcset="${srcset}" sizes="(max-width: 768px) 100vw, 1200px" loading="lazy" decoding="async" alt="${alt}" />`;
  };

  const handleCopySnippet = async (m: Media) => {
    const html = buildResponsiveHtml(m);
    await navigator.clipboard?.writeText(html);
    setCopiedSnippet(true);
    setTimeout(() => setCopiedSnippet(false), 2000);
  };

  const isRasterImage =
    selectedMedia &&
    selectedMedia.mime_type.startsWith('image/') &&
    selectedMedia.mime_type !== 'image/svg+xml';

  return (
    <div className="space-y-6">
      {/* En-tête de la médiathèque */}
      <div className="flex flex-wrap items-center justify-between gap-4">
        <div>
          <h2 className="text-lg font-bold text-white">{mediaMessages.title}</h2>
          <p className="text-xs text-slate-400 mt-0.5">{mediaMessages.subtitle}</p>
        </div>
        <span className="text-xs font-mono text-slate-400 bg-white/5 px-3 py-1 rounded-full border border-white/10">
          {total} fichier(s)
        </span>
      </div>

      {error && (
        <div
          role="alert"
          className="rounded-xl border border-red-500/30 bg-red-500/10 px-4 py-3 text-xs text-red-200"
        >
          {error}
        </div>
      )}
      {message && (
        <div
          role="status"
          className="rounded-xl border border-emerald-500/30 bg-emerald-500/10 px-4 py-3 text-xs text-emerald-200"
        >
          {message}
        </div>
      )}

      {/* Zone de dépôt / téléversement (WP-38) */}
      <div className="p-6 border-2 border-dashed border-white/10 hover:border-indigo-500/40 rounded-2xl bg-[#0f1422] text-center transition-all">
        <div className="w-10 h-10 rounded-2xl bg-indigo-500/10 border border-indigo-500/20 text-indigo-400 flex items-center justify-center mx-auto mb-2.5">
          <Upload className="w-5 h-5" />
        </div>
        <h4 className="text-sm font-bold text-white mb-1">
          {uploading ? mediaMessages.uploadingPrompt : mediaMessages.uploadPrompt}
        </h4>
        <p className="text-xs text-slate-400 mb-3">{mediaMessages.uploadFormats}</p>
        <label className="inline-flex items-center gap-2 px-4 py-2 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-semibold cursor-pointer shadow-md shadow-indigo-600/20 transition-all">
          <span>{mediaMessages.browseFiles}</span>
          <input
            type="file"
            onChange={(e) => void handleFileUpload(e)}
            disabled={uploading}
            className="hidden"
          />
        </label>
      </div>

      {/* Barre de recherche, filtre MIME et bascule Grille/Liste (WP-37) */}
      <div className="flex flex-wrap items-center justify-between gap-3 p-3.5 rounded-2xl bg-[#0f1422] border border-white/5">
        <div className="flex flex-wrap items-center gap-2 flex-1 min-w-[240px]">
          <div className="relative flex-1 min-w-[200px]">
            <Search className="w-3.5 h-3.5 text-slate-500 absolute left-3 top-1/2 -translate-y-1/2" />
            <input
              type="search"
              aria-label={mediaMessages.searchPlaceholder}
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder={mediaMessages.searchPlaceholder}
              className="w-full rounded-xl bg-slate-950 border border-white/10 pl-8 pr-3 py-1.5 text-xs text-white placeholder-slate-500"
            />
          </div>
          <select
            aria-label={mediaMessages.filterAllKinds}
            value={kindFilter}
            onChange={(e) => setKindFilter(e.target.value)}
            className="rounded-xl bg-slate-950 border border-white/10 px-3 py-1.5 text-xs text-slate-200"
          >
            <option value="">{mediaMessages.filterAllKinds}</option>
            <option value="image/">{mediaMessages.filterImages}</option>
            <option value="video/">{mediaMessages.filterVideo}</option>
            <option value="application/pdf">{mediaMessages.filterDocs}</option>
          </select>
        </div>

        <div className="flex items-center gap-1 rounded-xl bg-black/30 p-1 border border-white/10">
          <button
            type="button"
            onClick={() => setViewMode('grid')}
            className={`flex items-center gap-1.5 px-2.5 py-1 rounded-lg text-xs ${
              viewMode === 'grid'
                ? 'bg-indigo-600 text-white'
                : 'text-slate-400 hover:text-slate-200'
            }`}
          >
            <Grid className="w-3.5 h-3.5" />
            <span>{mediaMessages.viewGrid}</span>
          </button>
          <button
            type="button"
            onClick={() => setViewMode('list')}
            className={`flex items-center gap-1.5 px-2.5 py-1 rounded-lg text-xs ${
              viewMode === 'list'
                ? 'bg-indigo-600 text-white'
                : 'text-slate-400 hover:text-slate-200'
            }`}
          >
            <List className="w-3.5 h-3.5" />
            <span>{mediaMessages.viewList}</span>
          </button>
        </div>
      </div>

      {/* Mise en page 3 colonnes : Dossiers virtuels | Catalogue (Grille/Liste) | Inspecteur */}
      <div className="grid grid-cols-1 lg:grid-cols-12 gap-5 items-start">
        {/* Colonne 1 : Dossiers virtuels (MOD-MED-01) */}
        <aside className="lg:col-span-3 rounded-2xl bg-[#0f1422] border border-white/5 p-4 space-y-3">
          <div className="flex items-center gap-2 text-xs font-semibold text-slate-300">
            <Folder className="w-4 h-4 text-indigo-400" />
            <span>Dossiers virtuels</span>
          </div>

          <div className="space-y-1 text-xs">
            <button
              type="button"
              onClick={() => setSelectedFolderId(null)}
              className={`w-full flex items-center justify-between px-3 py-2 rounded-xl text-left transition-colors ${
                selectedFolderId === null
                  ? 'bg-indigo-600/20 text-indigo-200 border border-indigo-500/30 font-semibold'
                  : 'text-slate-300 hover:bg-white/5'
              }`}
            >
              <span>{mediaMessages.allFolders}</span>
            </button>

            {folders.map((folder) => (
              <div
                key={folder.id}
                className={`flex items-center justify-between gap-1 px-3 py-1.5 rounded-xl transition-colors ${
                  selectedFolderId === folder.id
                    ? 'bg-indigo-600/20 text-indigo-200 border border-indigo-500/30 font-semibold'
                    : 'text-slate-300 hover:bg-white/5'
                }`}
              >
                <button
                  type="button"
                  onClick={() => setSelectedFolderId(folder.id)}
                  className="flex items-center gap-2 truncate flex-1 text-left"
                >
                  <Folder className="w-3.5 h-3.5 text-amber-400 shrink-0" />
                  <span className="truncate">{folder.name}</span>
                </button>
                <button
                  type="button"
                  onClick={() => void deleteFolder(folder.id)}
                  title="Supprimer ce dossier"
                  className="p-1 text-slate-500 hover:text-red-400"
                >
                  <Trash2 className="w-3 h-3" />
                </button>
              </div>
            ))}
          </div>

          <form
            onSubmit={(e) => {
              e.preventDefault();
              void createFolder();
            }}
            className="pt-2 border-t border-white/5 flex items-center gap-1.5"
          >
            <input
              type="text"
              value={newFolderName}
              onChange={(e) => setNewFolderName(e.target.value)}
              placeholder={mediaMessages.newFolderPlaceholder}
              className="flex-1 min-w-0 rounded-lg bg-slate-950 border border-white/10 px-2.5 py-1.5 text-xs text-white"
            />
            <button
              type="submit"
              className="inline-flex items-center gap-1 px-2.5 py-1.5 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-semibold shrink-0"
            >
              <FolderPlus className="w-3.5 h-3.5" />
              <span>{mediaMessages.addFolderButton}</span>
            </button>
          </form>
        </aside>

        {/* Colonne 2 : Catalogue Grille ou Liste (WP-37) */}
        <div className={selectedMedia ? 'lg:col-span-5' : 'lg:col-span-9'}>
          {loading ? (
            <div className="p-12 rounded-2xl bg-[#0f1422] border border-white/5 text-center text-xs text-slate-500">
              {mediaMessages.loadingMedia}
            </div>
          ) : items.length === 0 ? (
            <div className="p-12 rounded-2xl bg-[#0f1422] border border-white/5 text-center text-xs text-slate-500">
              {mediaMessages.emptyMedia}
            </div>
          ) : viewMode === 'grid' ? (
            <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-3.5">
              {items.map((m) => {
                const isSelected = selectedMedia?.id === m.id;
                return (
                  <div
                    key={m.id}
                    onClick={() => selectMedia(m)}
                    className={`group cursor-pointer rounded-xl bg-[#0f1422] border overflow-hidden flex flex-col transition-all ${
                      isSelected
                        ? 'border-indigo-500 ring-2 ring-indigo-500/30'
                        : 'border-white/5 hover:border-indigo-500/30'
                    }`}
                  >
                    <div className="aspect-square bg-black/40 flex items-center justify-center overflow-hidden relative">
                      {m.mime_type.startsWith('image/') ? (
                        <img
                          src={m.webp_url || m.url}
                          alt={m.alt || m.original_name}
                          loading="lazy"
                          className="w-full h-full object-cover group-hover:scale-105 transition-transform"
                        />
                      ) : (
                        <File className="w-8 h-8 text-slate-500" />
                      )}
                      {m.webp_url && (
                        <span className="absolute top-1.5 right-1.5 rounded bg-emerald-500/80 px-1.5 py-0.5 text-[9px] font-bold text-black">
                          WebP
                        </span>
                      )}
                    </div>
                    <div className="p-2.5 flex items-center justify-between gap-1 text-[11px]">
                      <div className="min-w-0">
                        <p className="font-medium text-slate-200 truncate" title={m.original_name}>
                          {m.original_name}
                        </p>
                        <p className="text-[10px] text-slate-500 font-mono">
                          {formatBytes(m.size_bytes)}
                        </p>
                      </div>
                      <button
                        type="button"
                        onClick={(e) => {
                          e.stopPropagation();
                          void deleteMedia(m.id);
                        }}
                        title="Supprimer"
                        className="p-1 text-slate-400 hover:text-red-400"
                      >
                        <Trash2 className="w-3.5 h-3.5" />
                      </button>
                    </div>
                  </div>
                );
              })}
            </div>
          ) : (
            <div className="rounded-2xl bg-[#0f1422] border border-white/5 overflow-hidden">
              <table className="w-full text-left text-xs">
                <thead>
                  <tr className="border-b border-white/5 bg-white/[0.01] text-slate-400 text-[10px] uppercase">
                    <th className="py-3 px-4">Fichier</th>
                    <th className="py-3 px-4">Texte alt</th>
                    <th className="py-3 px-4">Format & Taille</th>
                    <th className="py-3 px-4 text-right">Actions</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-white/5">
                  {items.map((m) => (
                    <tr
                      key={m.id}
                      onClick={() => selectMedia(m)}
                      className={`cursor-pointer hover:bg-white/[0.02] ${
                        selectedMedia?.id === m.id ? 'bg-indigo-950/30' : ''
                      }`}
                    >
                      <td className="py-2.5 px-4 font-medium text-slate-200">
                        {m.original_name}
                      </td>
                      <td className="py-2.5 px-4 text-slate-400">
                        {m.alt || <span className="text-amber-400/70">Non renseigné</span>}
                      </td>
                      <td className="py-2.5 px-4 font-mono text-[11px] text-slate-400">
                        {m.mime_type} · {formatBytes(m.size_bytes)}
                      </td>
                      <td className="py-2.5 px-4 text-right">
                        <button
                          type="button"
                          onClick={(e) => {
                            e.stopPropagation();
                            void deleteMedia(m.id);
                          }}
                          className="p-1 text-slate-400 hover:text-red-400"
                        >
                          <Trash2 className="w-3.5 h-3.5" />
                        </button>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </div>

        {/* Colonne 3 : Inspecteur de métadonnées, Variantes & Retouche (WP-38, WP-39, WP-41, WP-88) */}
        {selectedMedia && (
          <aside className="lg:col-span-4 rounded-2xl bg-[#0f1422] border border-indigo-500/20 p-5 space-y-4">
            <div className="flex items-center justify-between border-b border-white/5 pb-3">
              <h3 className="text-xs font-bold text-white uppercase tracking-wider">
                {mediaMessages.inspectorTitle}
              </h3>
              <button
                type="button"
                onClick={() => selectMedia(null)}
                className="text-slate-400 hover:text-white"
              >
                <X className="w-4 h-4" />
              </button>
            </div>

            <div className="text-xs space-y-1 text-slate-400 font-mono bg-black/30 p-3 rounded-xl border border-white/5">
              <div className="text-slate-200 font-sans font-semibold truncate">
                {selectedMedia.original_name}
              </div>
              <div>
                {selectedMedia.mime_type} · {formatBytes(selectedMedia.size_bytes)}
                {selectedMedia.width && selectedMedia.height
                  ? ` · ${selectedMedia.width}×${selectedMedia.height}px`
                  : ''}
              </div>
              <div className="truncate text-indigo-300">{selectedMedia.url}</div>
            </div>

            {/* Actions de transformation d'image (WP-39) */}
            {isRasterImage && (
              <div className="rounded-xl border border-white/10 bg-black/20 p-3 space-y-2">
                <div className="flex items-center justify-between text-[11px] text-slate-300 font-medium">
                  <span className="flex items-center gap-1.5">
                    <Sparkles className="w-3.5 h-3.5 text-indigo-400" />
                    <span>Retouche & Transformation</span>
                  </span>
                  {selectedMedia.metadata?.original_path && (
                    <span className="text-[10px] text-amber-400/90 font-mono bg-amber-500/10 px-1.5 py-0.5 rounded">
                      Modifié
                    </span>
                  )}
                </div>
                <div className="flex flex-wrap gap-1.5">
                  <button
                    type="button"
                    disabled={rotating}
                    onClick={() => void rotateSelectedMedia(270)}
                    title="Pivoter de 90° vers la gauche"
                    className="inline-flex items-center gap-1 rounded-lg border border-white/10 bg-white/5 hover:bg-white/10 px-2 py-1 text-xs text-slate-200 disabled:opacity-50"
                  >
                    <RotateCcw className="w-3 h-3" />
                    <span>-90°</span>
                  </button>
                  <button
                    type="button"
                    disabled={rotating}
                    onClick={() => void rotateSelectedMedia(90)}
                    title="Pivoter de 90° vers la droite"
                    className="inline-flex items-center gap-1 rounded-lg border border-white/10 bg-white/5 hover:bg-white/10 px-2 py-1 text-xs text-slate-200 disabled:opacity-50"
                  >
                    <RotateCw className="w-3 h-3" />
                    <span>+90°</span>
                  </button>
                  <button
                    type="button"
                    disabled={rotating}
                    onClick={() => void rotateSelectedMedia(180)}
                    title="Pivoter de 180°"
                    className="inline-flex items-center gap-1 rounded-lg border border-white/10 bg-white/5 hover:bg-white/10 px-2 py-1 text-xs text-slate-200 disabled:opacity-50"
                  >
                    <span>180°</span>
                  </button>
                  {selectedMedia.metadata?.original_path && (
                    <button
                      type="button"
                      disabled={rotating}
                      onClick={() => void restoreSelectedMedia()}
                      title="Restaurer l'image originale sans altération"
                      className="inline-flex items-center gap-1 rounded-lg border border-amber-500/30 bg-amber-500/10 hover:bg-amber-500/20 px-2 py-1 text-xs text-amber-200 disabled:opacity-50"
                    >
                      <Undo2 className="w-3 h-3" />
                      <span>Restaurer original</span>
                    </button>
                  )}
                </div>
              </div>
            )}

            {/* Variantes responsives dimensionnées (WP-39, WP-88) */}
            {selectedMedia.metadata?.variants &&
              Object.keys(selectedMedia.metadata.variants).length > 0 && (
                <div className="rounded-xl border border-white/10 bg-black/20 p-3 space-y-2">
                  <div className="text-[11px] font-semibold text-slate-300">
                    Variantes responsives (WP-88)
                  </div>
                  <div className="space-y-1.5">
                    {Object.entries(selectedMedia.metadata.variants).map(([name, varObj]) => (
                      <div
                        key={name}
                        className="flex items-center justify-between text-[11px] p-1.5 rounded-lg bg-white/[0.02] border border-white/5"
                      >
                        <div className="min-w-0">
                          <span className="font-semibold text-indigo-300 capitalize">{name}</span>
                          <span className="text-slate-400 font-mono ml-1.5">
                            {varObj.width}×{varObj.height} px
                          </span>
                        </div>
                        <button
                          type="button"
                          onClick={() => void navigator.clipboard?.writeText(varObj.url)}
                          title="Copier l'URL de cette variante"
                          className="p-1 text-slate-400 hover:text-white"
                        >
                          <Copy className="w-3 h-3" />
                        </button>
                      </div>
                    ))}
                  </div>
                </div>
              )}

            {/* Boutons de copie rapide */}
            <div className="flex flex-wrap gap-2">
              <button
                type="button"
                onClick={() => void navigator.clipboard?.writeText(selectedMedia.url)}
                className="inline-flex items-center gap-1.5 rounded-lg border border-white/10 bg-white/5 px-2.5 py-1.5 text-xs text-slate-200 hover:bg-white/10"
              >
                <Copy className="w-3.5 h-3.5" />
                <span>{mediaMessages.copyUrl}</span>
              </button>
              {selectedMedia.webp_url && (
                <button
                  type="button"
                  onClick={() =>
                    void navigator.clipboard?.writeText(selectedMedia.webp_url || '')
                  }
                  className="inline-flex items-center gap-1.5 rounded-lg border border-emerald-500/30 bg-emerald-500/10 px-2.5 py-1.5 text-xs text-emerald-200 hover:bg-emerald-500/20"
                >
                  <Copy className="w-3.5 h-3.5" />
                  <span>{mediaMessages.copyWebpUrl}</span>
                </button>
              )}
              {isRasterImage && (
                <button
                  type="button"
                  onClick={() => void handleCopySnippet(selectedMedia)}
                  title="Copier le code <img srcset...> responsive"
                  className="inline-flex items-center gap-1.5 rounded-lg border border-indigo-500/30 bg-indigo-500/10 px-2.5 py-1.5 text-xs text-indigo-200 hover:bg-indigo-500/20"
                >
                  {copiedSnippet ? (
                    <>
                      <Check className="w-3.5 h-3.5 text-emerald-400" />
                      <span className="text-emerald-300">Copié !</span>
                    </>
                  ) : (
                    <>
                      <Copy className="w-3.5 h-3.5" />
                      <span>HTML responsive</span>
                    </>
                  )}
                </button>
              )}
            </div>

            <div className="space-y-3 text-xs">
              <label className="block space-y-1">
                <span className="text-slate-300 font-medium">{mediaMessages.fieldAlt}</span>
                <input
                  type="text"
                  value={editAlt}
                  onChange={(e) => setEditAlt(e.target.value)}
                  placeholder="Description pour lecteurs d’écran et SEO"
                  className="w-full rounded-xl bg-slate-950 border border-white/10 px-3 py-2 text-xs text-white"
                />
              </label>

              <label className="block space-y-1">
                <span className="text-slate-300 font-medium">{mediaMessages.fieldCaption}</span>
                <textarea
                  rows={2}
                  value={editCaption}
                  onChange={(e) => setEditCaption(e.target.value)}
                  className="w-full rounded-xl bg-slate-950 border border-white/10 px-3 py-2 text-xs text-white resize-none"
                />
              </label>

              <label className="block space-y-1">
                <span className="text-slate-300 font-medium">{mediaMessages.fieldFolder}</span>
                <select
                  value={editFolderId}
                  onChange={(e) => setEditFolderId(e.target.value)}
                  className="w-full rounded-xl bg-slate-950 border border-white/10 px-3 py-2 text-xs text-white"
                >
                  <option value="">{mediaMessages.folderRoot}</option>
                  {folders.map((f) => (
                    <option key={f.id} value={f.id}>
                      {f.name}
                    </option>
                  ))}
                </select>
              </label>

              <div className="flex items-center justify-between gap-2 pt-2">
                <button
                  type="button"
                  disabled={savingMeta}
                  onClick={() => void saveSelectedMediaMeta()}
                  className="inline-flex items-center gap-1.5 rounded-xl bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 px-3.5 py-2 text-xs font-semibold text-white"
                >
                  <Check className="w-3.5 h-3.5" />
                  <span>
                    {savingMeta ? mediaMessages.savingMeta : mediaMessages.saveMeta}
                  </span>
                </button>
                <button
                  type="button"
                  onClick={() => void deleteMedia(selectedMedia.id)}
                  className="inline-flex items-center gap-1 rounded-xl border border-red-500/30 bg-red-500/10 px-3 py-2 text-xs text-red-200 hover:bg-red-500/20"
                >
                  <Trash2 className="w-3.5 h-3.5" />
                  <span>{mediaMessages.deleteMedia}</span>
                </button>
              </div>
            </div>
          </aside>
        )}
      </div>
    </div>
  );
};
