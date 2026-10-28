/**
 * ============================================================================
 * VUE ORCHESTRÉE DE L'ÉDITEUR DE CONTENU (`EntryEditorView`)
 * ============================================================================
 * Orchestre le cycle de vie de l'éditeur autour de `useEditorStore` (Zustand) :
 * - Initialisation de l'article/page et chargement des parents hiérarchiques (`WP-20`).
 * - Synchronisation de l'état `isDirty` vers la garde de navigation du Studio.
 * - Déclenchement temporisé de l'autosauvegarde locale (`WP-29`) et de l'aperçu
 *   serveur (`WP-33`).
 * - Rendu modulaire par onglets :
 *   - `EditorHeaderBar` (actions, autosauvegarde, révisions `WP-28`),
 *   - `EditorTitleSlugBar` (titre, slug, dépublication auto),
 *   - `EditorContentTab` (blocs natifs `rf.document/1`, Markdown, aperçu),
 *   - `EditorDocumentTab` (statut, dates, parent, extrait, image, termes),
 *   - `EditorIntentTab` & `EditorDesignTab` (options IA & tokens de style).
 */

import React, { useEffect } from 'react';
import { Palette, Compass, FileCode, Sliders } from 'lucide-react';
import { editorMessages } from '../i18n/messages.fr';
import { useEditorStore } from '../stores/useEditorStore';
import type { ContentType } from '../types';
import { EditorContentTab } from './editor/EditorContentTab';
import { EditorDocumentTab } from './editor/EditorDocumentTab';
import { EditorHeaderBar } from './editor/EditorHeaderBar';
import { EditorDesignTab, EditorIntentTab } from './editor/EditorMetaTabs';
import { EditorTitleSlugBar } from './editor/EditorTitleSlugBar';

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
  const activeTab = useEditorStore((s) => s.activeTab);
  const loading = useEditorStore((s) => s.loading);
  const isDirty = useEditorStore((s) => s.isDirty);
  const effectiveType = useEditorStore((s) => s.effectiveType);
  const loadError = useEditorStore((s) => s.loadError);
  const unsupportedNative = useEditorStore((s) => s.unsupportedNative);

  // Souscriptions fines pour déclencher l'aperçu et l'autosauvegarde temporisés
  const title = useEditorStore((s) => s.title);
  const slug = useEditorStore((s) => s.slug);
  const excerpt = useEditorStore((s) => s.excerpt);
  const body = useEditorStore((s) => s.body);
  const docHistory = useEditorStore((s) => s.docHistory);
  const rawBlocks = useEditorStore((s) => s.rawBlocks);
  const status = useEditorStore((s) => s.status);
  const featuredMediaId = useEditorStore((s) => s.featuredMediaId);
  const parentId = useEditorStore((s) => s.parentId);
  const publishedAt = useEditorStore((s) => s.publishedAt);
  const scheduledUnpublishAt = useEditorStore((s) => s.scheduledUnpublishAt);
  const selectedTermIds = useEditorStore((s) => s.selectedTermIds);

  const setActiveTab = useEditorStore((s) => s.setActiveTab);
  const markDirty = useEditorStore((s) => s.markDirty);
  const initEditor = useEditorStore((s) => s.initEditor);
  const loadCandidateParents = useEditorStore((s) => s.loadCandidateParents);
  const persistAutosave = useEditorStore((s) => s.persistAutosave);
  const refreshPreview = useEditorStore((s) => s.refreshPreview);

  const currentType = contentTypes.find((ct) => ct.slug === effectiveType);
  const isHierarchical = !!currentType?.hierarchical;

  // 1. Initialisation du store à l'ouverture d'un article ou d'une page
  useEffect(() => {
    void initEditor(entryId, contentTypeSlug);
  }, [entryId, contentTypeSlug, initEditor]);

  // 2. Chargement des pages parentes candidates lorsque le type est hiérarchique
  useEffect(() => {
    void loadCandidateParents(effectiveType, entryId, isHierarchical);
  }, [effectiveType, entryId, isHierarchical, loadCandidateParents]);

  // 3. Synchronisation de l'indicateur `unsaved` avec le shell du Studio
  useEffect(() => {
    onDirtyChange(isDirty);
  }, [isDirty, onDirtyChange]);

  // 4. Autosauvegarde locale temporisée (1s après modification)
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

  // 5. Aperçu serveur temporisé (400ms après modification du corps ou des blocs)
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
      <div role="alert" className="p-8 space-y-4">
        <p>{loadError || editorMessages.unsupported}</p>
        <button onClick={() => onBack(effectiveType)}>Revenir aux contenus</button>
      </div>
    );
  }

  if (loading) {
    return <div className="p-8 text-center text-slate-400">Chargement de l'éditeur...</div>;
  }

  return (
    <div className="space-y-6" onChange={markDirty}>
      <EditorHeaderBar currentType={currentType} onBack={onBack} onSaved={onSaved} />

      <EditorTitleSlugBar />

      {/* Barre d'onglets : Rédaction / Document & Publication / Options IA / Style */}
      <div className="flex flex-wrap items-center gap-2 border-b border-white/5">
        <button
          type="button"
          onClick={() => setActiveTab('content')}
          className={`flex items-center gap-2 px-4 py-2.5 text-xs font-semibold border-b-2 transition-all ${
            activeTab === 'content'
              ? 'border-indigo-500 text-indigo-400 bg-white/[0.02]'
              : 'border-transparent text-slate-400 hover:text-slate-200'
          }`}
        >
          <FileCode className="w-4 h-4" />
          <span>{editorMessages.tabContent}</span>
        </button>

        <button
          type="button"
          onClick={() => setActiveTab('document')}
          className={`flex items-center gap-2 px-4 py-2.5 text-xs font-semibold border-b-2 transition-all ${
            activeTab === 'document'
              ? 'border-emerald-500 text-emerald-400 bg-white/[0.02]'
              : 'border-transparent text-slate-400 hover:text-slate-200'
          }`}
        >
          <Sliders className="w-4 h-4 text-emerald-400" />
          <span>{editorMessages.tabDocument}</span>
        </button>

        <button
          type="button"
          onClick={() => setActiveTab('intent')}
          className={`flex items-center gap-2 px-4 py-2.5 text-xs font-semibold border-b-2 transition-all ${
            activeTab === 'intent'
              ? 'border-purple-500 text-purple-400 bg-white/[0.02]'
              : 'border-transparent text-slate-400 hover:text-slate-200'
          }`}
        >
          <Compass className="w-4 h-4 text-purple-400" />
          <span>{editorMessages.tabIntent}</span>
        </button>

        <button
          type="button"
          onClick={() => setActiveTab('design')}
          className={`flex items-center gap-2 px-4 py-2.5 text-xs font-semibold border-b-2 transition-all ${
            activeTab === 'design'
              ? 'border-cyan-500 text-cyan-400 bg-white/[0.02]'
              : 'border-transparent text-slate-400 hover:text-slate-200'
          }`}
        >
          <Palette className="w-4 h-4 text-cyan-400" />
          <span>{editorMessages.tabDesign}</span>
        </button>
      </div>

      {activeTab === 'content' && <EditorContentTab />}
      {activeTab === 'document' && <EditorDocumentTab currentType={currentType} />}
      {activeTab === 'intent' && <EditorIntentTab />}
      {activeTab === 'design' && <EditorDesignTab />}
    </div>
  );
};
