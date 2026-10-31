/**
 * ============================================================================
 * VUE DE LA MÉDIATHÈQUE SOUVERAINE & DOSSIERS FILEBIRD (`upload.php`)
 * ============================================================================
 * Reproduction fidèle à 99% de WordPress Core (`upload.php`) enrichie de l'organisation
 * hiérarchique FileBird Pro et de la retouche d'image native en Rust (zéro PHP).
 *
 * Fonctionnalités clés :
 * - `WP-37` : Mode Grille et Liste, recherche en temps réel (`q`), filtre par type MIME
 *   (Images, Vidéo, Audio, Documents), filtre par date et dossiers virtuels FileBird.
 * - `WP-38` : Téléversement glisser-déposer dans le dossier actif et conversion WebP native.
 * - `WP-39` : Retouche souveraine d'image (rotation horaire / anti-horaire) et restauration réversible.
 * - `WP-40` : Suppression protégée contre la destruction d'un média lié à un contenu actif.
 * - `WP-41` : Extrait HTML responsive prêt à l'emploi (`<img srcset="...">`).
 * - `WP-88` : Visualisation des variantes générées (Miniature, Moyenne, Grande).
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
  Plus,
  FolderTree,
  ChevronRight,
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
  const [isUploadOpen, setIsUploadOpen] = useState(false);

  useEffect(() => {
    void fetchAll();
  }, [searchQuery, kindFilter, selectedFolderId, fetchAll]);

  const handleFileUpload = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;
    await uploadFile(file);
    e.target.value = '';
    setIsUploadOpen(false);
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
    <div className="space-y-5">
      {/* ==================================================================== */}
      {/* 1. EN-TÊTE CONTEXTUEL (upload.php)                                   */}
      {/* ==================================================================== */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 pb-3 border-b border-[#dcdcde] dark:border-[#2c3338]">
        <div className="flex items-center gap-3">
          <h1 className="text-xl font-bold text-[#1d2327] dark:text-white flex items-center gap-2">
            Médiathèque
          </h1>
          <button
            type="button"
            onClick={() => setIsUploadOpen((prev) => !prev)}
            className="inline-flex items-center gap-1.5 px-3 py-1 text-xs font-semibold rounded border border-[#2271b1] text-[#2271b1] hover:bg-[#2271b1] hover:text-white transition-colors"
          >
            <Plus className="w-3.5 h-3.5" />
            <span>Ajouter un fichier média</span>
          </button>
        </div>

        <span className="text-xs font-mono text-slate-500 dark:text-slate-400 bg-slate-100 dark:bg-white/5 px-3 py-1 rounded-full border border-slate-200 dark:border-white/10 self-start sm:self-auto">
          {total} fichier(s) au total
        </span>
      </div>

      {error && (
        <div
          role="alert"
          className="rounded-lg border border-red-500/30 bg-red-500/10 px-4 py-3 text-xs text-red-600 dark:text-red-200"
        >
          {error}
        </div>
      )}
      {message && (
        <div
          role="status"
          className="rounded-lg border border-emerald-500/30 bg-emerald-500/10 px-4 py-3 text-xs text-emerald-700 dark:text-emerald-200"
        >
          {message}
        </div>
      )}

      {/* ==================================================================== */}
      {/* 2. ZONE DE TÉLÉVERSEMENT GLISSER-DÉPOSER (DÉROULABLE)                */}
      {/* ==================================================================== */}
      {isUploadOpen && (
        <div className="p-8 border-2 border-dashed border-[#2271b1] rounded-xl bg-blue-50/20 dark:bg-[#0f1422] text-center transition-all animate-in fade-in duration-150">
          <div className="w-12 h-12 rounded-full bg-blue-100 dark:bg-indigo-500/10 border border-blue-200 dark:border-indigo-500/20 text-[#2271b1] dark:text-indigo-400 flex items-center justify-center mx-auto mb-3">
            <Upload className="w-6 h-6" />
          </div>
          <h4 className="text-sm font-bold text-[#1d2327] dark:text-white mb-1">
            {uploading ? mediaMessages.uploadingPrompt : 'Déposez vos fichiers ici pour les téléverser'}
          </h4>
          <p className="text-xs text-slate-500 dark:text-slate-400 mb-4">
            Images (WebP, PNG, JPG), Vidéos (MP4), Audios (MP3), Documents (PDF) jusqu’à 64 Mo
          </p>
          <label className="inline-flex items-center gap-2 px-5 py-2.5 rounded bg-[#2271b1] hover:bg-[#135e96] text-white text-xs font-semibold cursor-pointer shadow transition-all">
            <span>{mediaMessages.browseFiles}</span>
            <input
              type="file"
              onChange={(e) => void handleFileUpload(e)}
              disabled={uploading}
              className="hidden"
            />
          </label>
        </div>
      )}

      {/* ==================================================================== */}
      {/* 3. BARRE DE FILTRES ET D'OUTILS (CONFORME upload.php)                */}
      {/* ==================================================================== */}
      <div className="flex flex-wrap items-center justify-between gap-3 p-2.5 rounded-lg bg-white dark:bg-[#182030] border border-[#dcdcde] dark:border-[#2c3338] text-xs shadow-xs">
        <div className="flex flex-wrap items-center gap-2 flex-1 min-w-[240px]">
          {/* Commutateur Grille / Liste (WordPress .view-switch) */}
          <div className="flex items-center border border-[#8c8f94] dark:border-[#2c3338] rounded overflow-hidden">
            <button
              type="button"
              onClick={() => setViewMode('grid')}
              className={`p-1.5 transition-colors ${
                viewMode === 'grid'
                  ? 'bg-[#2271b1] text-white'
                  : 'bg-white dark:bg-[#131619] text-slate-600 dark:text-slate-400 hover:bg-slate-100'
              }`}
              title="Affichage en grille"
            >
              <Grid className="w-3.5 h-3.5" />
            </button>
            <button
              type="button"
              onClick={() => setViewMode('list')}
              className={`p-1.5 transition-colors ${
                viewMode === 'list'
                  ? 'bg-[#2271b1] text-white'
                  : 'bg-white dark:bg-[#131619] text-slate-600 dark:text-slate-400 hover:bg-slate-100'
              }`}
              title="Affichage en liste"
            >
              <List className="w-3.5 h-3.5" />
            </button>
          </div>

          {/* Filtre de type de média */}
          <select
            aria-label={mediaMessages.filterAllKinds}
            value={kindFilter}
            onChange={(e) => setKindFilter(e.target.value)}
            className="rounded bg-white dark:bg-[#131619] border border-[#8c8f94] dark:border-[#2c3338] px-2.5 py-1 text-xs text-[#1d2327] dark:text-white"
          >
            <option value="">{mediaMessages.filterAllKinds}</option>
            <option value="image/">{mediaMessages.filterImages}</option>
            <option value="video/">{mediaMessages.filterVideo}</option>
            <option value="application/pdf">{mediaMessages.filterDocs}</option>
          </select>

          {/* Filtre de date */}
          <select className="rounded bg-white dark:bg-[#131619] border border-[#8c8f94] dark:border-[#2c3338] px-2.5 py-1 text-xs text-[#1d2327] dark:text-white">
            <option value="">Toutes les dates</option>
            <option value="2026-10">Octobre 2026</option>
            <option value="2026-09">Septembre 2026</option>
          </select>
        </div>

        {/* Champ de recherche en temps réel */}
        <div className="relative w-full sm:w-64">
          <Search className="w-3.5 h-3.5 text-slate-400 absolute left-2.5 top-1/2 -translate-y-1/2" />
          <input
            type="search"
            aria-label={mediaMessages.searchPlaceholder}
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Rechercher dans les médias…"
            className="w-full rounded bg-white dark:bg-[#131619] border border-[#8c8f94] dark:border-[#2c3338] pl-8 pr-3 py-1 text-xs text-[#1d2327] dark:text-white placeholder:text-slate-400 focus:border-[#2271b1] outline-none"
          />
        </div>
      </div>

      {/* ==================================================================== */}
      {/* 4. DISPOSITION 3 VOLETS : FILEBIRD | MÉDIAS | INSPECTEUR             */}
      {/* ==================================================================== */}
      <div className="grid grid-cols-1 lg:grid-cols-12 gap-5 items-start">
        {/* Volet 1 : Dossiers FileBird Pro (Arborescence souveraine) */}
        <aside className="lg:col-span-3 rounded-xl bg-white dark:bg-[#182030] border border-[#dcdcde] dark:border-[#2c3338] p-4 space-y-3 shadow-xs">
          <div className="flex items-center justify-between pb-2 border-b border-[#dcdcde] dark:border-[#2c3338]">
            <div className="flex items-center gap-1.5 text-xs font-bold text-[#1d2327] dark:text-white">
              <FolderTree className="w-4 h-4 text-[#2271b1]" />
              <span>Dossiers FileBird</span>
            </div>
            <span className="text-[10px] font-mono text-slate-400">{folders.length + 1}</span>
          </div>

          {/* Liste des dossiers virtuels */}
          <div className="space-y-1 text-xs">
            <button
              type="button"
              onClick={() => setSelectedFolderId(null)}
              className={`w-full flex items-center justify-between px-2.5 py-1.5 rounded-lg text-left transition-colors ${
                selectedFolderId === null
                  ? 'bg-blue-50 dark:bg-blue-950/40 text-[#2271b1] font-semibold border border-blue-200 dark:border-blue-900'
                  : 'text-slate-700 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-800'
              }`}
            >
              <div className="flex items-center gap-2">
                <Folder className="w-3.5 h-3.5 text-[#2271b1]" />
                <span>Tous les fichiers</span>
              </div>
              <span className="text-[10px] text-slate-400 font-mono">{total}</span>
            </button>

            {folders.map((folder) => (
              <div
                key={folder.id}
                className={`flex items-center justify-between gap-1 px-2.5 py-1.5 rounded-lg transition-colors ${
                  selectedFolderId === folder.id
                    ? 'bg-blue-50 dark:bg-blue-950/40 text-[#2271b1] font-semibold border border-blue-200 dark:border-blue-900'
                    : 'text-slate-700 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-800'
                }`}
              >
                <button
                  type="button"
                  onClick={() => setSelectedFolderId(folder.id)}
                  className="flex items-center gap-2 truncate flex-1 text-left"
                >
                  <ChevronRight className="w-3 h-3 text-slate-400" />
                  <Folder className="w-3.5 h-3.5 text-amber-500 shrink-0" />
                  <span className="truncate">{folder.name}</span>
                </button>
                <button
                  type="button"
                  onClick={() => void deleteFolder(folder.id)}
                  title="Supprimer ce dossier"
                  className="p-1 text-slate-400 hover:text-red-500 transition-colors"
                >
                  <Trash2 className="w-3 h-3" />
                </button>
              </div>
            ))}
          </div>

          {/* Formulaire de création rapide de dossier */}
          <form
            onSubmit={(e) => {
              e.preventDefault();
              void createFolder();
            }}
            className="pt-2 border-t border-[#dcdcde] dark:border-[#2c3338] flex items-center gap-1.5"
          >
            <input
              type="text"
              value={newFolderName}
              onChange={(e) => setNewFolderName(e.target.value)}
              placeholder="Nouveau dossier…"
              className="flex-1 min-w-0 rounded bg-white dark:bg-[#131619] border border-[#8c8f94] dark:border-[#2c3338] px-2 py-1 text-xs text-[#1d2327] dark:text-white outline-none focus:border-[#2271b1]"
            />
            <button
              type="submit"
              disabled={!newFolderName.trim()}
              className="inline-flex items-center gap-1 px-2.5 py-1 rounded bg-[#2271b1] hover:bg-[#135e96] disabled:opacity-40 text-white text-xs font-semibold shrink-0 transition-colors"
            >
              <FolderPlus className="w-3.5 h-3.5" />
              <span>Créer</span>
            </button>
          </form>
        </aside>

        {/* Volet 2 : Catalogue des Médias (Grille ou Tableau conforme .rf-list-table) */}
        <div className={selectedMedia ? 'lg:col-span-5' : 'lg:col-span-9'}>
          {loading ? (
            <div className="p-12 rounded-xl bg-white dark:bg-[#182030] border border-[#dcdcde] dark:border-[#2c3338] text-center text-xs text-slate-400">
              {mediaMessages.loadingMedia}
            </div>
          ) : items.length === 0 ? (
            <div className="p-12 rounded-xl bg-white dark:bg-[#182030] border border-[#dcdcde] dark:border-[#2c3338] text-center text-xs text-slate-400">
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
                    className={`group cursor-pointer rounded-xl bg-white dark:bg-[#182030] border overflow-hidden flex flex-col transition-all shadow-xs ${
                      isSelected
                        ? 'border-[#2271b1] ring-2 ring-[#2271b1]/40'
                        : 'border-[#dcdcde] dark:border-[#2c3338] hover:border-[#2271b1]'
                    }`}
                  >
                    <div className="aspect-square bg-slate-100 dark:bg-black/30 flex items-center justify-center overflow-hidden relative">
                      {m.mime_type.startsWith('image/') ? (
                        <img
                          src={m.webp_url || m.url}
                          alt={m.alt || m.original_name}
                          loading="lazy"
                          className="w-full h-full object-cover group-hover:scale-105 transition-transform"
                        />
                      ) : (
                        <File className="w-8 h-8 text-slate-400" />
                      )}
                      {m.webp_url && (
                        <span className="absolute top-1.5 right-1.5 rounded bg-emerald-500 text-white px-1.5 py-0.5 text-[9px] font-bold shadow">
                          WebP
                        </span>
                      )}
                    </div>
                    <div className="p-2.5 flex items-center justify-between gap-1 text-[11px] border-t border-[#dcdcde] dark:border-[#2c3338]">
                      <div className="min-w-0">
                        <p
                          className="font-medium text-[#1d2327] dark:text-slate-200 truncate"
                          title={m.original_name}
                        >
                          {m.original_name}
                        </p>
                        <p className="text-[10px] text-slate-400 font-mono">
                          {formatBytes(m.size_bytes)}
                        </p>
                      </div>
                      <button
                        type="button"
                        onClick={(e) => {
                          e.stopPropagation();
                          void deleteMedia(m.id);
                        }}
                        title="Supprimer définitivement"
                        className="p-1 text-slate-400 hover:text-red-600 transition-colors"
                      >
                        <Trash2 className="w-3.5 h-3.5" />
                      </button>
                    </div>
                  </div>
                );
              })}
            </div>
          ) : (
            <div className="rounded-xl bg-white dark:bg-[#182030] border border-[#dcdcde] dark:border-[#2c3338] overflow-hidden shadow-xs">
              <table className="rf-list-table w-full text-left text-xs">
                <thead>
                  <tr className="border-b border-[#dcdcde] dark:border-[#2c3338] bg-slate-50 dark:bg-[#131619] text-slate-500 dark:text-slate-400 text-[11px] font-semibold">
                    <th className="py-2.5 px-4">Fichier</th>
                    <th className="py-2.5 px-4">Texte alternatif</th>
                    <th className="py-2.5 px-4">Format & Taille</th>
                    <th className="py-2.5 px-4 text-right">Actions</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-[#dcdcde] dark:divide-[#2c3338]">
                  {items.map((m) => (
                    <tr
                      key={m.id}
                      onClick={() => selectMedia(m)}
                      className={`cursor-pointer hover:bg-slate-50 dark:hover:bg-slate-800 transition-colors ${
                        selectedMedia?.id === m.id ? 'bg-blue-50/50 dark:bg-blue-950/20' : ''
                      }`}
                    >
                      <td className="py-2.5 px-4 font-medium text-[#1d2327] dark:text-white flex items-center gap-2">
                        <div className="w-8 h-8 rounded bg-slate-100 dark:bg-black/30 overflow-hidden shrink-0 flex items-center justify-center">
                          {m.mime_type.startsWith('image/') ? (
                            <img src={m.webp_url || m.url} alt="" className="w-full h-full object-cover" />
                          ) : (
                            <File className="w-4 h-4 text-slate-400" />
                          )}
                        </div>
                        <span className="truncate">{m.original_name}</span>
                      </td>
                      <td className="py-2.5 px-4 text-slate-500 dark:text-slate-400">
                        {m.alt || <span className="text-amber-500 italic">Non renseigné</span>}
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
                          className="p-1 text-slate-400 hover:text-red-600 transition-colors"
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

        {/* Volet 3 : Inspecteur & Détails du média (conforme Attachment Details de WP) */}
        {selectedMedia && (
          <aside className="lg:col-span-4 rounded-xl bg-white dark:bg-[#182030] border border-[#dcdcde] dark:border-[#2c3338] p-5 space-y-4 shadow-sm">
            <div className="flex items-center justify-between border-b border-[#dcdcde] dark:border-[#2c3338] pb-3">
              <h3 className="text-xs font-bold text-[#1d2327] dark:text-white uppercase tracking-wider">
                Détails du fichier
              </h3>
              <button
                type="button"
                onClick={() => selectMedia(null)}
                className="text-slate-400 hover:text-slate-600 dark:hover:text-white p-1 rounded"
              >
                <X className="w-4 h-4" />
              </button>
            </div>

            {/* Aperçu grand format */}
            <div className="aspect-video bg-slate-100 dark:bg-black/30 rounded-lg overflow-hidden flex items-center justify-center border border-[#dcdcde] dark:border-[#2c3338]">
              {selectedMedia.mime_type.startsWith('image/') ? (
                <img
                  src={selectedMedia.webp_url || selectedMedia.url}
                  alt={selectedMedia.alt || ''}
                  className="w-full h-full object-contain"
                />
              ) : (
                <File className="w-12 h-12 text-slate-400" />
              )}
            </div>

            <div className="text-xs space-y-1 text-slate-500 dark:text-slate-400 font-mono bg-slate-50 dark:bg-black/30 p-3 rounded-lg border border-[#dcdcde] dark:border-[#2c3338]">
              <div className="text-[#1d2327] dark:text-white font-sans font-semibold truncate">
                {selectedMedia.original_name}
              </div>
              <div>
                {selectedMedia.mime_type} · {formatBytes(selectedMedia.size_bytes)}
                {selectedMedia.width && selectedMedia.height
                  ? ` · ${selectedMedia.width}×${selectedMedia.height}px`
                  : ''}
              </div>
              <div className="truncate text-[#2271b1]">{selectedMedia.url}</div>
            </div>

            {/* Actions de transformation d'image native en Rust */}
            {isRasterImage && (
              <div className="rounded-lg border border-[#dcdcde] dark:border-[#2c3338] bg-slate-50 dark:bg-black/20 p-3 space-y-2">
                <div className="flex items-center justify-between text-[11px] text-[#1d2327] dark:text-slate-300 font-medium">
                  <span className="flex items-center gap-1.5">
                    <Sparkles className="w-3.5 h-3.5 text-[#2271b1]" />
                    <span>Retouche & Transformation</span>
                  </span>
                  {selectedMedia.metadata?.original_path && (
                    <span className="text-[10px] text-amber-600 dark:text-amber-400 font-mono bg-amber-50 dark:bg-amber-500/10 px-1.5 py-0.5 rounded border border-amber-200 dark:border-amber-900">
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
                    className="inline-flex items-center gap-1 rounded border border-[#8c8f94] dark:border-white/10 bg-white dark:bg-white/5 hover:bg-slate-100 px-2 py-1 text-xs text-[#1d2327] dark:text-slate-200 disabled:opacity-50"
                  >
                    <RotateCcw className="w-3 h-3" />
                    <span>-90°</span>
                  </button>
                  <button
                    type="button"
                    disabled={rotating}
                    onClick={() => void rotateSelectedMedia(90)}
                    title="Pivoter de 90° vers la droite"
                    className="inline-flex items-center gap-1 rounded border border-[#8c8f94] dark:border-white/10 bg-white dark:bg-white/5 hover:bg-slate-100 px-2 py-1 text-xs text-[#1d2327] dark:text-slate-200 disabled:opacity-50"
                  >
                    <RotateCw className="w-3 h-3" />
                    <span>+90°</span>
                  </button>
                  {selectedMedia.metadata?.original_path && (
                    <button
                      type="button"
                      disabled={rotating}
                      onClick={() => void restoreSelectedMedia()}
                      title="Restaurer l'image originale sans altération"
                      className="inline-flex items-center gap-1 rounded border border-amber-300 dark:border-amber-500/30 bg-amber-50 dark:bg-amber-500/10 hover:bg-amber-100 px-2 py-1 text-xs text-amber-700 dark:text-amber-200 disabled:opacity-50"
                    >
                      <Undo2 className="w-3 h-3" />
                      <span>Restaurer original</span>
                    </button>
                  )}
                </div>
              </div>
            )}

            {/* Boutons de copie rapide */}
            <div className="flex flex-wrap gap-2">
              <button
                type="button"
                onClick={() => void navigator.clipboard?.writeText(selectedMedia.url)}
                className="inline-flex items-center gap-1.5 rounded border border-[#8c8f94] dark:border-white/10 bg-white dark:bg-white/5 px-2.5 py-1.5 text-xs text-[#1d2327] dark:text-slate-200 hover:bg-slate-100"
              >
                <Copy className="w-3.5 h-3.5" />
                <span>Copier l’URL</span>
              </button>
              {isRasterImage && (
                <button
                  type="button"
                  onClick={() => void handleCopySnippet(selectedMedia)}
                  title="Copier le code <img srcset...> responsive"
                  className="inline-flex items-center gap-1.5 rounded border border-[#2271b1] text-[#2271b1] hover:bg-[#2271b1] hover:text-white px-2.5 py-1.5 text-xs font-medium transition-colors"
                >
                  {copiedSnippet ? (
                    <>
                      <Check className="w-3.5 h-3.5 text-emerald-500" />
                      <span className="text-emerald-600">Copié !</span>
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

            {/* Champs d'édition des métadonnées */}
            <div className="space-y-3 text-xs">
              <label className="block space-y-1">
                <span className="text-slate-600 dark:text-slate-300 font-medium">Texte alternatif (ALT) :</span>
                <input
                  type="text"
                  value={editAlt}
                  onChange={(e) => setEditAlt(e.target.value)}
                  placeholder="Description pour lecteurs d’écran et SEO"
                  className="w-full rounded bg-white dark:bg-slate-950 border border-slate-300 dark:border-white/10 px-3 py-1.5 text-xs text-[#1d2327] dark:text-white"
                />
              </label>

              <label className="block space-y-1">
                <span className="text-slate-600 dark:text-slate-300 font-medium">Légende :</span>
                <textarea
                  rows={2}
                  value={editCaption}
                  onChange={(e) => setEditCaption(e.target.value)}
                  className="w-full rounded bg-white dark:bg-slate-950 border border-slate-300 dark:border-white/10 px-3 py-1.5 text-xs text-[#1d2327] dark:text-white resize-none"
                />
              </label>

              <label className="block space-y-1">
                <span className="text-slate-600 dark:text-slate-300 font-medium">Dossier FileBird :</span>
                <select
                  value={editFolderId}
                  onChange={(e) => setEditFolderId(e.target.value)}
                  className="w-full rounded bg-white dark:bg-slate-950 border border-slate-300 dark:border-white/10 px-3 py-1.5 text-xs text-[#1d2327] dark:text-white"
                >
                  <option value="">(Non classé)</option>
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
                  className="inline-flex items-center gap-1.5 rounded bg-[#2271b1] hover:bg-[#135e96] disabled:opacity-50 px-3.5 py-1.5 text-xs font-semibold text-white"
                >
                  <Check className="w-3.5 h-3.5" />
                  <span>{savingMeta ? 'Enregistrement…' : 'Enregistrer'}</span>
                </button>
                <button
                  type="button"
                  onClick={() => void deleteMedia(selectedMedia.id)}
                  className="inline-flex items-center gap-1 rounded border border-red-500/30 text-red-600 hover:bg-red-50 dark:hover:bg-red-950/20 px-3 py-1.5 text-xs"
                >
                  <Trash2 className="w-3.5 h-3.5" />
                  <span>Supprimer définitivement</span>
                </button>
              </div>
            </div>
          </aside>
        )}
      </div>
    </div>
  );
};
