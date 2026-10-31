/**
 * ============================================================================
 * VUE ÉDITEUR GUTENBERG SOUVERAIN (`EntryEditorView`)
 * ============================================================================
 * Reproduit à 100% l'expérience visuelle et ergonomique de WordPress Gutenberg :
 * - Barre supérieure officielle (`GutenbergTopBar`) avec retour, inséreur (+),
 *   undo/redo, aperçu multi-écrans (bureau/tablette/mobile), et bouton Publier.
 * - Tiroir d'insertion latéral gauche (`GutenbergInserterDrawer`) avec Blocs,
 *   Compositions officielles Twenty Twenty-Five et Médiathèque.
 * - Canevas central d'écriture (`GutenbergCanvas`) avec grand titre H1 sans bordure,
 *   séquence de blocs et barre d'outils flottante contextuelle.
 * - Volet latéral droit des réglages (`GutenbergInspectorSidebar`) avec onglets
 *   Article (statut, modèle FSE, catégories, image, extrait) et Bloc.
 * - Mode plein écran immersif et bascule vers l'éditeur de code (Markdown).
 */

import React, { useEffect, useState } from 'react';
import { GitCompare, AlertTriangle, X } from 'lucide-react';
import { useEditorStore } from '../stores/useEditorStore';
import type { ContentType } from '../types';
import { GutenbergTopBar } from './editor/GutenbergTopBar';
import { GutenbergInserterDrawer } from './editor/GutenbergInserterDrawer';
import { GutenbergCanvas } from './editor/GutenbergCanvas';
import { GutenbergInspectorSidebar } from './editor/GutenbergInspectorSidebar';

interface EntryEditorViewProps {
  entryId?: string;
  contentTypeSlug?: string;
  contentTypes: ContentType[];
  onBack: (contentType?: string) => void;
  onSaved: (contentType?: string) => void;
  onDirtyChange: (dirty: boolean) => void;
}

export const EntryEditorView: React.FC<EntryEditorViewProps> = ({
  entryId,
  contentTypeSlug = 'post',
  contentTypes,
  onBack,
  onSaved,
  onDirtyChange,
}) => {
  const [comparedRevisionId, setComparedRevisionId] = useState<string | null>(null);

  const loading = useEditorStore((s) => s.loading);
  const isDirty = useEditorStore((s) => s.isDirty);
  const effectiveType = useEditorStore((s) => s.effectiveType);
  const loadError = useEditorStore((s) => s.loadError);
  const unsupportedNative = useEditorStore((s) => s.unsupportedNative);

  const title = useEditorStore((s) => s.title);
  const slug = useEditorStore((s) => s.slug);
  const excerpt = useEditorStore((s) => s.excerpt);
  const body = useEditorStore((s) => s.body);
  const setBody = useEditorStore((s) => s.setBody);
  const docHistory = useEditorStore((s) => s.docHistory);
  const rawBlocks = useEditorStore((s) => s.rawBlocks);
  const status = useEditorStore((s) => s.status);
  const featuredMediaId = useEditorStore((s) => s.featuredMediaId);
  const parentId = useEditorStore((s) => s.parentId);
  const publishedAt = useEditorStore((s) => s.publishedAt);
  const scheduledUnpublishAt = useEditorStore((s) => s.scheduledUnpublishAt);
  const selectedTermIds = useEditorStore((s) => s.selectedTermIds);

  const isInserterOpen = useEditorStore((s) => s.isInserterOpen);
  const setInserterOpen = useEditorStore((s) => s.setInserterOpen);
  const isInspectorOpen = useEditorStore((s) => s.isInspectorOpen);
  const setInspectorOpen = useEditorStore((s) => s.setInspectorOpen);
  const editorMode = useEditorStore((s) => s.editorMode);
  const fullscreenMode = useEditorStore((s) => s.fullscreenMode);

  const recoverableDraft = useEditorStore((s) => s.recoverableDraft);
  const applyRecoveredDraft = useEditorStore((s) => s.applyRecoveredDraft);
  const discardRecoveredDraft = useEditorStore((s) => s.discardRecoveredDraft);

  const revisions = useEditorStore((s) => s.revisions);
  const closeRevisions = useEditorStore((s) => s.closeRevisions);
  const restoreRevision = useEditorStore((s) => s.restoreRevision);
  const convertToBlocks = useEditorStore((s) => s.convertToBlocks);

  const initEditor = useEditorStore((s) => s.initEditor);
  const loadCandidateParents = useEditorStore((s) => s.loadCandidateParents);
  const persistAutosave = useEditorStore((s) => s.persistAutosave);
  const refreshPreview = useEditorStore((s) => s.refreshPreview);

  const currentType = contentTypes.find((ct) => ct.slug === effectiveType);
  const isHierarchical = !!currentType?.hierarchical;

  // 1. Initialisation de l'article ou de la page
  useEffect(() => {
    void initEditor(entryId, contentTypeSlug);
  }, [entryId, contentTypeSlug, initEditor]);

  // 2. Chargement des pages parentes candidates
  useEffect(() => {
    void loadCandidateParents(effectiveType, entryId, isHierarchical);
  }, [effectiveType, entryId, isHierarchical, loadCandidateParents]);

  // 3. Indicateur de modifications non enregistrées
  useEffect(() => {
    onDirtyChange(isDirty);
  }, [isDirty, onDirtyChange]);

  // 4. Autosauvegarde temporisée locale (1s après saisie)
  useEffect(() => {
    if (!isDirty || loading || unsupportedNative) return;
    const timer = window.setTimeout(() => {
      persistAutosave();
    }, 1000);
    return () => window.clearTimeout(timer);
  }, [
    isDirty,
    loading,
    unsupportedNative,
    title,
    slug,
    excerpt,
    body,
    docHistory,
    rawBlocks,
    status,
    featuredMediaId,
    parentId,
    publishedAt,
    scheduledUnpublishAt,
    selectedTermIds,
    persistAutosave,
  ]);

  // 5. Aperçu serveur temporisé
  useEffect(() => {
    const controller = new AbortController();
    const timer = window.setTimeout(() => {
      void refreshPreview(controller.signal);
    }, 400);
    return () => {
      window.clearTimeout(timer);
      controller.abort();
    };
  }, [body, docHistory, rawBlocks, refreshPreview]);

  if (loadError || unsupportedNative) {
    return (
      <div role="alert" className="p-8 space-y-4 max-w-xl mx-auto text-center font-sans">
        <div className="w-12 h-12 rounded-full bg-red-500/10 text-red-500 flex items-center justify-center mx-auto">
          <AlertTriangle className="w-6 h-6" />
        </div>
        <p className="text-sm text-slate-200">{loadError || 'Format de document non supporté.'}</p>
        <button
          onClick={() => onBack(effectiveType)}
          className="px-4 py-2 rounded bg-[#2271b1] text-white text-xs font-medium"
        >
          Revenir à la liste
        </button>
      </div>
    );
  }

  if (loading) {
    return (
      <div className="h-96 flex items-center justify-center text-xs text-slate-400 font-sans">
        Chargement de l’éditeur Gutenberg...
      </div>
    );
  }

  return (
    <div
      className={`font-sans antialiased flex flex-col ${
        fullscreenMode
          ? 'fixed inset-0 z-50 bg-[#f0f0f1] dark:bg-[#101520]'
          : 'h-[calc(100vh-32px)] -m-4 md:-m-6 bg-[#f0f0f1] dark:bg-[#101520]'
      }`}
    >
      {/* 1. Barre d'outils supérieure de Gutenberg */}
      <GutenbergTopBar currentType={currentType} onBack={onBack} onSaved={onSaved} />

      {/* 2. Bandeau d'autosauvegarde récupérée */}
      {recoverableDraft && (
        <div
          role="status"
          className="bg-amber-50 dark:bg-amber-950/40 border-b border-amber-300 dark:border-amber-700/50 px-4 py-2.5 flex items-center justify-between text-xs text-amber-900 dark:text-amber-200 z-30"
        >
          <span>
            Une sauvegarde automatique plus récente a été trouvée pour ce contenu ({new Date(recoverableDraft.saved_at).toLocaleTimeString('fr-FR')}).
          </span>
          <div className="flex items-center gap-2">
            <button
              type="button"
              onClick={applyRecoveredDraft}
              className="px-2.5 py-1 rounded bg-[#2271b1] text-white text-xs font-semibold shadow-xs"
            >
              Restaurer la sauvegarde
            </button>
            <button
              type="button"
              onClick={discardRecoveredDraft}
              className="px-2 py-1 text-slate-500 hover:text-slate-800 dark:hover:text-white"
            >
              Ignorer
            </button>
          </div>
        </div>
      )}

      {/* 3. Corps principal : Inserter gauche + Canevas central + Inspecteur droit */}
      <div className="flex-1 flex overflow-hidden relative">
        {/* Tiroir d'insertion gauche */}
        <GutenbergInserterDrawer
          isOpen={isInserterOpen}
          onClose={() => setInserterOpen(false)}
        />

        {/* Zone centrale : Canevas Gutenberg ou Éditeur de code */}
        {editorMode === 'visual' ? (
          <GutenbergCanvas />
        ) : (
          <div className="flex-1 overflow-y-auto bg-[#131619] p-8 font-mono text-xs flex justify-center">
            <div className="w-full max-w-4xl space-y-4">
              <div className="flex items-center justify-between text-slate-400 border-b border-white/10 pb-2">
                <span>Éditeur de code (Markdown / CommonMark)</span>
                <button
                  type="button"
                  onClick={() => void convertToBlocks()}
                  className="px-3 py-1 rounded bg-[#2271b1] text-white text-xs font-sans font-medium"
                >
                  Convertir en blocs visuels
                </button>
              </div>
              <textarea
                rows={25}
                value={body}
                onChange={(e) => setBody(e.target.value)}
                placeholder="Rédigez votre code ou Markdown ici..."
                className="w-full bg-transparent text-slate-100 font-mono text-sm leading-relaxed outline-none resize-none"
              />
            </div>
          </div>
        )}

        {/* Inspecteur latéral droit */}
        <GutenbergInspectorSidebar
          isOpen={isInspectorOpen}
          onClose={() => setInspectorOpen(false)}
          currentType={currentType}
        />
      </div>

      {/* 4. Modale de comparaison des révisions */}
      {revisions && (
        <div
          role="dialog"
          aria-label="Historique des révisions"
          className="fixed inset-0 z-50 bg-black/60 backdrop-blur-xs flex items-center justify-center p-4 font-sans"
        >
          <div className="w-full max-w-3xl bg-white dark:bg-[#1d2327] rounded-lg shadow-2xl border border-slate-200 dark:border-[#2c3338] max-h-[85vh] flex flex-col overflow-hidden text-xs">
            <div className="flex items-center justify-between p-4 border-b border-slate-200 dark:border-[#2c3338]">
              <h3 className="font-semibold text-sm text-[#1d2327] dark:text-white">
                Historique des révisions ({revisions.length})
              </h3>
              <button
                type="button"
                onClick={closeRevisions}
                className="p-1 rounded text-slate-400 hover:text-slate-700 dark:hover:text-white"
              >
                <X className="w-4 h-4" />
              </button>
            </div>

            <div className="flex-1 overflow-y-auto p-4 space-y-3">
              {revisions.map((rev) => {
                const isComparing = comparedRevisionId === rev.id;
                return (
                  <div
                    key={rev.id}
                    className="p-3 rounded border border-slate-200 dark:border-[#2c3338] bg-[#f6f7f7] dark:bg-[#131619] space-y-2"
                  >
                    <div className="flex items-center justify-between">
                      <div>
                        <span className="font-semibold text-[#1d2327] dark:text-white">
                          {new Date(rev.created_at).toLocaleString('fr-FR')}
                        </span>
                        <span className="text-slate-400 ml-2">
                          {rev.author_name ? `par ${rev.author_name}` : 'par Admin'}
                        </span>
                      </div>
                      <div className="flex items-center gap-2">
                        <button
                          type="button"
                          onClick={() => setComparedRevisionId(isComparing ? null : rev.id)}
                          className="px-2 py-1 rounded border border-[#8c8f94] hover:bg-white dark:hover:bg-[#2c3338] text-slate-700 dark:text-slate-300 flex items-center gap-1"
                        >
                          <GitCompare className="w-3 h-3" />
                          <span>{isComparing ? 'Masquer diff' : 'Comparer'}</span>
                        </button>
                        <button
                          type="button"
                          onClick={() => void restoreRevision(rev.id, onSaved)}
                          className="px-2.5 py-1 rounded bg-[#2271b1] hover:bg-[#135e96] text-white font-medium"
                        >
                          Restaurer
                        </button>
                      </div>
                    </div>

                    {isComparing && (
                      <div className="grid grid-cols-2 gap-3 pt-2 border-t border-slate-200 dark:border-[#2c3338] text-[11px]">
                        <div className="p-2 rounded bg-amber-50/50 dark:bg-amber-950/20 border border-amber-200 dark:border-amber-900/30">
                          <span className="font-bold text-amber-800 dark:text-amber-300 block mb-1">
                            Révision précédente
                          </span>
                          <p className="font-medium text-[#1d2327] dark:text-white">{rev.title}</p>
                          <pre className="mt-1 font-mono text-[10px] text-slate-600 dark:text-slate-400 whitespace-pre-wrap max-h-32 overflow-y-auto">
                            {rev.body || '(vide)'}
                          </pre>
                        </div>
                        <div className="p-2 rounded bg-blue-50/50 dark:bg-blue-950/20 border border-blue-200 dark:border-blue-900/30">
                          <span className="font-bold text-blue-800 dark:text-blue-300 block mb-1">
                            Version actuelle
                          </span>
                          <p className="font-medium text-[#1d2327] dark:text-white">{title}</p>
                          <pre className="mt-1 font-mono text-[10px] text-slate-600 dark:text-slate-400 whitespace-pre-wrap max-h-32 overflow-y-auto">
                            {body || '(vide)'}
                          </pre>
                        </div>
                      </div>
                    )}
                  </div>
                );
              })}
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
