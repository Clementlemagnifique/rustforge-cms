import { inspectDocument, type DocumentMetrics } from '../../editor/wasmKernel';
/**
 * ============================================================================
 * BARRE SUPÉRIEURE DE L'ÉDITEUR GUTENBERG (`GutenbergTopBar`)
 * ============================================================================
 * Reproduit à 100% la barre supérieure officielle de Gutenberg (`.edit-post-header`) :
 * - Gauche : Bouton retour vers la liste des contenus, Inséreur de blocs (+),
 *   Outils Annuler / Rétablir, Vue arborescente du document, Métriques.
 * - Droite : Statut d'enregistrement, Bouton Enregistrer le brouillon,
 *   Sélecteur d'aperçu multi-écrans (Bureau / Tablette / Mobile), Bouton primaire
 *   « Publier / Mettre à jour », Bouton d'inspecteur (roue dentée), Menu Kebab ⋮.
 */

import React, { useState, useRef, useEffect } from 'react';
import {
  Plus,
  Undo2,
  Redo2,
  ListTree,
  ChevronLeft,
  Settings,
  MoreVertical,
  Monitor,
  Tablet,
  Smartphone,
  ExternalLink,
  Code,
  Maximize2,
  History,
  Check,
  Flame,
} from 'lucide-react';
import { useEditorStore } from '../../stores/useEditorStore';
import type { ContentType } from '../../types';

interface GutenbergTopBarProps {
  currentType?: ContentType;
  onBack: (contentType?: string) => void;
  onSaved: (contentType?: string) => void;
}

export const GutenbergTopBar: React.FC<GutenbergTopBarProps> = ({
  currentType,
  onBack,
  onSaved,
}) => {
  const [previewMenuOpen, setPreviewMenuOpen] = useState(false);
  const [optionsMenuOpen, setOptionsMenuOpen] = useState(false);
  const [metricsOpen, setMetricsOpen] = useState(false);
  const [showOutline, setShowOutline] = useState(false);

  const previewRef = useRef<HTMLDivElement>(null);
  const optionsRef = useRef<HTMLDivElement>(null);

  const title = useEditorStore((s) => s.title);
  const body = useEditorStore((s) => s.body);
  const slug = useEditorStore((s) => s.slug);
  const status = useEditorStore((s) => s.status);
  const saving = useEditorStore((s) => s.saving);
  const isDirty = useEditorStore((s) => s.isDirty);
  const effectiveType = useEditorStore((s) => s.effectiveType);
  const docHistory = useEditorStore((s) => s.docHistory);

  const isInserterOpen = useEditorStore((s) => s.isInserterOpen);
  const toggleInserter = useEditorStore((s) => s.toggleInserter);
  const isInspectorOpen = useEditorStore((s) => s.isInspectorOpen);
  const toggleInspector = useEditorStore((s) => s.toggleInspector);
  const editorMode = useEditorStore((s) => s.editorMode);
  const setEditorMode = useEditorStore((s) => s.setEditorMode);
  const fullscreenMode = useEditorStore((s) => s.fullscreenMode);
  const toggleFullscreenMode = useEditorStore((s) => s.toggleFullscreenMode);
  const previewDevice = useEditorStore((s) => s.previewDevice);
  const setPreviewDevice = useEditorStore((s) => s.setPreviewDevice);

  const undo = useEditorStore((s) => s.undo);
  const redo = useEditorStore((s) => s.redo);
  const saveEntry = useEditorStore((s) => s.saveEntry);
  const setStatus = useEditorStore((s) => s.setStatus);
  const fetchRevisions = useEditorStore((s) => s.fetchRevisions);

  const canUndo = (docHistory?.past?.length ?? 0) > 0;
  const canRedo = (docHistory?.future?.length ?? 0) > 0;
  const isHierarchical = !!currentType?.hierarchical;
  const rfDocument = docHistory?.present ?? null;

  const [kernelState, setKernelState] = useState<{key: string; metrics?: DocumentMetrics; error?: string} | null>(null);
  const kernelKey = JSON.stringify([body, rfDocument]);
  useEffect(() => {
    let active = true;
    const timer = window.setTimeout(() => {
      inspectDocument(body || '', rfDocument).then(result => {
        if (active) setKernelState({key: kernelKey, metrics: result.metrics});
      }).catch(() => { if (active) setKernelState({key: kernelKey, error: 'Métriques Rust indisponibles ; validation serveur conservée'}); });
    }, 150);
    return () => { active = false; window.clearTimeout(timer); };
  }, [body, rfDocument, kernelKey]);
  const metrics = kernelState?.key === kernelKey ? kernelState.metrics : undefined;
  const wordCount = metrics?.words ?? '—';
  const charCount = metrics?.characters ?? '—';
  const readingTime = metrics?.reading_minutes ?? '—';
  const blockCount = rfDocument?.rf_blocks?.length || 0;

  // Fermer les menus déroulants lors d'un clic extérieur
  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      const target = e.target as Node;
      if (previewRef.current && !previewRef.current.contains(target)) setPreviewMenuOpen(false);
      if (optionsRef.current && !optionsRef.current.contains(target)) setOptionsMenuOpen(false);
    };
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  const handlePublish = async () => {
    if (status !== 'published') {
      setStatus('published');
    }
    await saveEntry(isHierarchical, onSaved);
  };

  const handleSaveDraft = async () => {
    if (status !== 'draft') {
      setStatus('draft');
    }
    await saveEntry(isHierarchical, onSaved);
  };

  return (
    <header className="h-14 bg-white dark:bg-[#1d2327] border-b border-[#dcdcde] dark:border-[#2c3338] px-3 flex items-center justify-between text-xs font-sans select-none z-30 shrink-0">
      {/* ==================================================================== */}
      {/* ZONE GAUCHE : Retour, + Inséreur, Undo/Redo, Structure, Métriques    */}
      {/* ==================================================================== */}
      <div className="flex items-center gap-1.5 sm:gap-2">
        {/* 1. Bouton Retour avec logo RustForge */}
        <button
          type="button"
          onClick={() => onBack(effectiveType)}
          title={`Retour à la liste des ${effectiveType === 'page' ? 'pages' : 'articles'}`}
          className="p-1.5 rounded-md hover:bg-[#f0f0f1] dark:hover:bg-[#2c3338] text-[#1d2327] dark:text-[#c3c4c7] transition-colors flex items-center gap-1"
        >
          <ChevronLeft className="w-4 h-4" />
          <div className="w-5 h-5 rounded-full bg-gradient-to-tr from-amber-500 to-indigo-500 flex items-center justify-center text-white shadow-xs">
            <Flame className="w-3 h-3 fill-white" />
          </div>
        </button>

        <div className="h-4 w-px bg-[#dcdcde] dark:bg-[#2c3338] mx-0.5" />

        {/* 2. Bouton Inséreur de Blocs (+) */}
        <button
          type="button"
          onClick={toggleInserter}
          title="Ajouter un bloc ou une composition (Ctrl+Alt+T)"
          className={`p-2 rounded transition-all ${
            isInserterOpen
              ? 'bg-[#1d2327] dark:bg-[#2271b1] text-white shadow-sm'
              : 'hover:bg-[#f0f0f1] dark:hover:bg-[#2c3338] text-[#1d2327] dark:text-white'
          }`}
        >
          <Plus className={`w-4 h-4 transition-transform ${isInserterOpen ? 'rotate-45' : ''}`} />
        </button>

        {/* 3. Boutons Annuler / Rétablir */}
        <button
          type="button"
          disabled={!canUndo}
          onClick={undo}
          title="Annuler (Ctrl+Z)"
          className="p-2 rounded text-[#2c3338] dark:text-[#c3c4c7] hover:bg-[#f0f0f1] dark:hover:bg-[#2c3338] disabled:opacity-30 disabled:hover:bg-transparent transition-colors"
        >
          <Undo2 className="w-4 h-4" />
        </button>

        <button
          type="button"
          disabled={!canRedo}
          onClick={redo}
          title="Rétablir (Ctrl+Y)"
          className="p-2 rounded text-[#2c3338] dark:text-[#c3c4c7] hover:bg-[#f0f0f1] dark:hover:bg-[#2c3338] disabled:opacity-30 disabled:hover:bg-transparent transition-colors"
        >
          <Redo2 className="w-4 h-4" />
        </button>

        {/* 4. Vue structure / Arborescence du document */}
        <button
          type="button"
          onClick={() => setShowOutline(!showOutline)}
          title="Vue détaillée de la structure"
          className={`p-2 rounded text-[#2c3338] dark:text-[#c3c4c7] hover:bg-[#f0f0f1] dark:hover:bg-[#2c3338] transition-colors ${
            showOutline ? 'bg-[#f0f0f1] dark:bg-[#2c3338] text-[#2271b1]' : ''
          }`}
        >
          <ListTree className="w-4 h-4" />
        </button>

        {/* 5. Métriques de document */}
        <div className="relative hidden md:block">
          <button
            type="button"
            onClick={() => setMetricsOpen(!metricsOpen)}
            className="px-2 py-1 rounded text-[11px] text-[#646970] dark:text-slate-400 hover:bg-[#f0f0f1] dark:hover:bg-[#2c3338] transition-colors"
          >
            {wordCount} mots · ~{readingTime} min
          </button>

          {metricsOpen && (
            <div className="absolute left-0 mt-2 w-56 rounded bg-white dark:bg-[#1d2327] border border-[#dcdcde] dark:border-[#2c3338] p-3 shadow-xl z-50 text-xs space-y-2">
              {kernelState?.key === kernelKey && kernelState.error && <p role="status">{kernelState.error}</p>}
                <h5 className="font-semibold text-[#1d2327] dark:text-white border-b border-slate-100 dark:border-[#2c3338] pb-1.5">
                Statistiques du document
              </h5>
              <div className="space-y-1 text-slate-600 dark:text-slate-300">
                <div className="flex justify-between">
                  <span>Mots :</span>
                  <strong className="text-[#1d2327] dark:text-white">{wordCount}</strong>
                </div>
                <div className="flex justify-between">
                  <span>Caractères :</span>
                  <strong className="text-[#1d2327] dark:text-white">{charCount}</strong>
                </div>
                <div className="flex justify-between">
                  <span>Blocs :</span>
                  <strong className="text-[#1d2327] dark:text-white">{blockCount}</strong>
                </div>
                <div className="flex justify-between">
                  <span>Temps de lecture :</span>
                  <strong className="text-[#1d2327] dark:text-white">~{readingTime} min</strong>
                </div>
              </div>
            </div>
          )}
        </div>
      </div>

      {/* ==================================================================== */}
      {/* ZONE CENTRALE : Titre indicatif (si renseigné)                       */}
      {/* ==================================================================== */}
      <div className="hidden lg:block max-w-xs truncate text-center text-slate-500 font-medium text-xs">
        {title || '(Sans titre)'}
      </div>

      {/* ==================================================================== */}
      {/* ZONE DROITE : Statut, Enregistrer, Prévisualiser, Publier, Réglages  */}
      {/* ==================================================================== */}
      <div className="flex items-center gap-1.5 sm:gap-2">
        {/* Statut d'enregistrement */}
        <span className="text-[11px] text-[#646970] dark:text-slate-400 hidden sm:inline">
          {saving ? 'Enregistrement...' : isDirty ? 'Non enregistré' : 'Enregistré'}
        </span>

        {/* Bouton Enregistrer le brouillon */}
        {status === 'draft' && (
          <button
            type="button"
            disabled={saving || !isDirty}
            onClick={handleSaveDraft}
            className="px-2.5 py-1.5 rounded hover:bg-[#f0f0f1] dark:hover:bg-[#2c3338] text-[#2271b1] dark:text-[#72aee6] font-medium disabled:opacity-40 transition-colors"
          >
            Enregistrer le brouillon
          </button>
        )}

        {/* Sélecteur d'aperçu multi-écrans */}
        <div ref={previewRef} className="relative">
          <button
            type="button"
            onClick={() => setPreviewMenuOpen(!previewMenuOpen)}
            className="px-2.5 py-1.5 rounded hover:bg-[#f0f0f1] dark:hover:bg-[#2c3338] text-[#2c3338] dark:text-slate-200 font-medium flex items-center gap-1 transition-colors"
          >
            {previewDevice === 'desktop' && <Monitor className="w-3.5 h-3.5" />}
            {previewDevice === 'tablet' && <Tablet className="w-3.5 h-3.5" />}
            {previewDevice === 'mobile' && <Smartphone className="w-3.5 h-3.5" />}
            <span>Prévisualiser</span>
          </button>

          {previewMenuOpen && (
            <div className="absolute right-0 mt-2 w-56 rounded bg-white dark:bg-[#1d2327] border border-[#dcdcde] dark:border-[#2c3338] py-1 shadow-xl z-50 text-xs">
              <div className="px-3 py-1.5 text-[10px] text-slate-400 uppercase tracking-wider font-semibold">
                Format d’affichage
              </div>
              <button
                type="button"
                onClick={() => {
                  setPreviewDevice('desktop');
                  setPreviewMenuOpen(false);
                }}
                className={`w-full text-left px-3 py-1.5 flex items-center gap-2 hover:bg-[#f0f0f1] dark:hover:bg-[#2c3338] ${
                  previewDevice === 'desktop' ? 'font-semibold text-[#2271b1]' : 'text-slate-700 dark:text-slate-300'
                }`}
              >
                <Monitor className="w-3.5 h-3.5" />
                <span>Bureau (Plein écran)</span>
              </button>
              <button
                type="button"
                onClick={() => {
                  setPreviewDevice('tablet');
                  setPreviewMenuOpen(false);
                }}
                className={`w-full text-left px-3 py-1.5 flex items-center gap-2 hover:bg-[#f0f0f1] dark:hover:bg-[#2c3338] ${
                  previewDevice === 'tablet' ? 'font-semibold text-[#2271b1]' : 'text-slate-700 dark:text-slate-300'
                }`}
              >
                <Tablet className="w-3.5 h-3.5" />
                <span>Tablette (768 px)</span>
              </button>
              <button
                type="button"
                onClick={() => {
                  setPreviewDevice('mobile');
                  setPreviewMenuOpen(false);
                }}
                className={`w-full text-left px-3 py-1.5 flex items-center gap-2 hover:bg-[#f0f0f1] dark:hover:bg-[#2c3338] ${
                  previewDevice === 'mobile' ? 'font-semibold text-[#2271b1]' : 'text-slate-700 dark:text-slate-300'
                }`}
              >
                <Smartphone className="w-3.5 h-3.5" />
                <span>Mobile (375 px)</span>
              </button>
              <div className="border-t border-slate-100 dark:border-[#2c3338] my-1" />
              <a
                href={`/fr/${effectiveType}/${slug || 'apercu'}`}
                target="_blank"
                rel="noreferrer"
                className="w-full text-left px-3 py-1.5 flex items-center gap-2 text-[#2271b1] hover:bg-[#f0f0f1] dark:hover:bg-[#2c3338]"
              >
                <span>Aperçu dans un nouvel onglet</span>
                <ExternalLink className="w-3 h-3 ml-auto" />
              </a>
            </div>
          )}
        </div>

        {/* Bouton Primaire : Publier / Mettre à jour */}
        <button
          type="button"
          disabled={saving}
          onClick={handlePublish}
          className="px-3.5 py-1.5 rounded bg-[#2271b1] hover:bg-[#135e96] text-white font-medium text-xs shadow-xs disabled:opacity-50 transition-colors"
        >
          {saving ? 'Enregistrement...' : status === 'published' ? 'Mettre à jour' : 'Publier...'}
        </button>

        {/* Bouton Roue dentée : Affiche/Masque l'inspecteur latéral */}
        <button
          type="button"
          onClick={toggleInspector}
          title="Réglages (Ctrl+Shift+,)"
          className={`p-2 rounded transition-colors ${
            isInspectorOpen
              ? 'bg-[#1d2327] dark:bg-[#2271b1] text-white shadow-xs'
              : 'hover:bg-[#f0f0f1] dark:hover:bg-[#2c3338] text-[#2c3338] dark:text-slate-200'
          }`}
        >
          <Settings className="w-4 h-4" />
        </button>

        {/* Menu Options Kebab ⋮ */}
        <div ref={optionsRef} className="relative">
          <button
            type="button"
            onClick={() => setOptionsMenuOpen(!optionsMenuOpen)}
            title="Options supplémentaires"
            className="p-2 rounded hover:bg-[#f0f0f1] dark:hover:bg-[#2c3338] text-[#2c3338] dark:text-slate-200 transition-colors"
          >
            <MoreVertical className="w-4 h-4" />
          </button>

          {optionsMenuOpen && (
            <div className="absolute right-0 mt-2 w-64 rounded bg-white dark:bg-[#1d2327] border border-[#dcdcde] dark:border-[#2c3338] py-1 shadow-2xl z-50 text-xs">
              <div className="px-3 py-1.5 text-[10px] text-slate-400 uppercase tracking-wider font-semibold">
                Affichage
              </div>
              <button
                type="button"
                onClick={() => {
                  toggleFullscreenMode();
                  setOptionsMenuOpen(false);
                }}
                className="w-full text-left px-3 py-1.5 flex items-center justify-between hover:bg-[#f0f0f1] dark:hover:bg-[#2c3338] text-slate-700 dark:text-slate-200"
              >
                <div className="flex items-center gap-2">
                  <Maximize2 className="w-3.5 h-3.5" />
                  <span>Mode plein écran</span>
                </div>
                {fullscreenMode && <Check className="w-3.5 h-3.5 text-[#2271b1]" />}
              </button>

              <div className="border-t border-slate-100 dark:border-[#2c3338] my-1" />
              <div className="px-3 py-1.5 text-[10px] text-slate-400 uppercase tracking-wider font-semibold">
                Éditeur
              </div>
              <button
                type="button"
                onClick={() => {
                  setEditorMode(editorMode === 'visual' ? 'code' : 'visual');
                  setOptionsMenuOpen(false);
                }}
                className="w-full text-left px-3 py-1.5 flex items-center justify-between hover:bg-[#f0f0f1] dark:hover:bg-[#2c3338] text-slate-700 dark:text-slate-200"
              >
                <div className="flex items-center gap-2">
                  <Code className="w-3.5 h-3.5" />
                  <span>{editorMode === 'visual' ? 'Éditeur de code (Markdown)' : 'Éditeur visuel (Blocs)'}</span>
                </div>
              </button>

              <div className="border-t border-slate-100 dark:border-[#2c3338] my-1" />
              <button
                type="button"
                onClick={() => {
                  void fetchRevisions();
                  setOptionsMenuOpen(false);
                }}
                className="w-full text-left px-3 py-1.5 flex items-center gap-2 hover:bg-[#f0f0f1] dark:hover:bg-[#2c3338] text-slate-700 dark:text-slate-200"
              >
                <History className="w-3.5 h-3.5" />
                <span>Historique des révisions</span>
              </button>
            </div>
          )}
        </div>
      </div>
    </header>
  );
};
