/**
 * ============================================================================
 * BARRE SUPÉRIEURE DE L'ÉDITEUR, AUTOSAUVEGARDE & RÉVISIONS (`EditorHeaderBar`)
 * ============================================================================
 * Responsabilités :
 * - Bandeau de récupération d'autosauvegarde locale (`WP-29`).
 * - En-tête de l'éditeur : retour à la liste, titre courant, badge du coffre
 *   de provenance (`_rf_vault`), boutons Enregistrer Brouillon / Publier.
 * - Tiroir d'historique et de restauration des révisions serveur (`WP-28`).
 */

import React, { useState } from 'react';
import { Save, ArrowLeft, GitCompare } from 'lucide-react';
import { editorMessages } from '../../i18n/messages.fr';
import { useEditorStore } from '../../stores/useEditorStore';
import type { ContentType } from '../../types';

interface EditorHeaderBarProps {
  currentType?: ContentType;
  onBack: (contentType?: string) => void;
  onSaved: (contentType?: string) => void;
}

export const EditorHeaderBar: React.FC<EditorHeaderBarProps> = ({
  currentType,
  onBack,
  onSaved,
}) => {
  const [comparedRevisionId, setComparedRevisionId] = useState<string | null>(null);
  const entryId = useEditorStore((s) => s.entryId);
  const effectiveType = useEditorStore((s) => s.effectiveType);
  const title = useEditorStore((s) => s.title);
  const excerpt = useEditorStore((s) => s.excerpt);
  const body = useEditorStore((s) => s.body);
  const docHistory = useEditorStore((s) => s.docHistory);
  const status = useEditorStore((s) => s.status);
  const saving = useEditorStore((s) => s.saving);
  const converting = useEditorStore((s) => s.converting);
  const fields = useEditorStore((s) => s.fields);

  const recoverableDraft = useEditorStore((s) => s.recoverableDraft);
  const applyRecoveredDraft = useEditorStore((s) => s.applyRecoveredDraft);
  const discardRecoveredDraft = useEditorStore((s) => s.discardRecoveredDraft);

  const revisions = useEditorStore((s) => s.revisions);
  const historyError = useEditorStore((s) => s.historyError);
  const fetchRevisions = useEditorStore((s) => s.fetchRevisions);
  const closeRevisions = useEditorStore((s) => s.closeRevisions);
  const restoreRevision = useEditorStore((s) => s.restoreRevision);
  const saveEntry = useEditorStore((s) => s.saveEntry);

  const isNew = !entryId;
  const isHierarchical = !!currentType?.hierarchical;
  const vaultKeysCount = Object.keys(fields?._rf_vault || fields?._legacy_vault || {}).length;

  return (
    <div className="space-y-4">
      {/* Bandeau de récupération d'autosauvegarde locale (WP-29) */}
      {recoverableDraft && (
        <div
          role="status"
          className="flex flex-wrap items-center justify-between gap-3 rounded-xl border border-amber-400/30 bg-amber-500/10 px-4 py-3 text-xs text-amber-200"
        >
          <span>
            {editorMessages.autosaveFound.replace(
              '{date}',
              new Date(recoverableDraft.saved_at).toLocaleString('fr-FR')
            )}
          </span>
          <div className="flex items-center gap-2">
            <button
              type="button"
              onClick={applyRecoveredDraft}
              className="rounded-lg bg-amber-500/20 border border-amber-400/40 px-3 py-1.5 font-semibold text-amber-100 hover:bg-amber-500/30"
            >
              {editorMessages.autosaveRestore}
            </button>
            <button
              type="button"
              onClick={discardRecoveredDraft}
              className="rounded-lg px-2.5 py-1.5 text-amber-300 hover:bg-white/5"
            >
              {editorMessages.autosaveDiscard}
            </button>
          </div>
        </div>
      )}

      {/* Barre d'actions principale */}
      <div className="flex flex-col gap-3 xl:flex-row xl:items-center xl:justify-between pb-4 border-b border-white/5">
        <div className="flex items-center gap-3 min-w-0">
          <button
            type="button"
            aria-label={editorMessages.backToList}
            onClick={() => onBack(effectiveType)}
            className="shrink-0 p-2 rounded-xl bg-white/5 hover:bg-white/10 text-slate-300 transition-colors"
          >
            <ArrowLeft className="w-4 h-4" />
          </button>
          <div className="min-w-0">
            <h3 className="text-sm font-bold text-white flex flex-wrap items-center gap-2 [overflow-wrap:anywhere]">
              {isNew
                ? effectiveType === 'page'
                  ? 'Nouvelle page'
                  : effectiveType === 'post'
                  ? 'Nouvel article'
                  : `Nouveau ${currentType?.name_singular || 'contenu'}`
                : `Édition : ${title || 'Sans titre'}`}
              <span className="text-[10px] uppercase font-mono px-2 py-0.5 rounded bg-white/5 text-slate-400">
                {effectiveType}
              </span>
              {vaultKeysCount > 0 && (
                <span
                  className="text-[10px] font-medium px-2 py-0.5 rounded bg-emerald-500/10 text-emerald-300 border border-emerald-500/20 flex items-center gap-1"
                  title="Métadonnées protégées dans le coffre-fort souverain"
                >
                  🔒 Coffre-fort RF ({vaultKeysCount} clés)
                </span>
              )}
            </h3>
          </div>
        </div>

        <div className="flex flex-wrap items-center gap-2.5 shrink-0">
          {!isNew && (
            <button
              type="button"
              onClick={() => void fetchRevisions()}
              className="text-xs rounded-lg bg-white/5 px-3 py-2"
            >
              {editorMessages.historyButton}
            </button>
          )}
          <button
            type="button"
            onClick={() => void saveEntry(isHierarchical, onSaved, 'draft')}
            disabled={saving || converting}
            className="px-3.5 py-2 rounded-xl bg-white/5 hover:bg-white/10 border border-white/10 text-slate-300 text-xs font-semibold transition-all"
          >
            {editorMessages.saveDraft}
          </button>
          <button
            type="button"
            onClick={() => void saveEntry(isHierarchical, onSaved, 'published')}
            disabled={saving || converting}
            className="px-4 py-2 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-semibold shadow-md shadow-indigo-600/25 flex items-center gap-1.5 transition-all hover:scale-105 active:scale-95"
          >
            <Save className="w-3.5 h-3.5" />
            <span>
              {saving
                ? editorMessages.saving
                : status === 'published'
                ? editorMessages.updatePublished
                : editorMessages.publish}
            </span>
          </button>
        </div>
      </div>

      {/* Erreur et tiroir d'historique des révisions (WP-28) */}
      {historyError && (
        <p role="alert" className="text-red-300 text-xs">
          {historyError}
        </p>
      )}
      {revisions && (
        <section className="rounded-xl border border-white/10 bg-[#0f1422] p-4 space-y-3">
          <div className="flex justify-between items-center">
            <h4 className="text-xs font-semibold text-slate-200 uppercase tracking-wider">
              {editorMessages.historyTitle} ({revisions.length})
            </h4>
            <button
              type="button"
              onClick={() => {
                setComparedRevisionId(null);
                closeRevisions();
              }}
              className="text-xs text-slate-400 hover:text-white"
            >
              Fermer
            </button>
          </div>
          <div className="space-y-2">
            {revisions.map((r) => {
              const isComparing = comparedRevisionId === r.id;
              const revBlocks = r.snapshot?.blocks?.rf_blocks;
              const currentBlocksCount = docHistory?.present?.rf_blocks.length;
              return (
                <div
                  key={r.id}
                  className="rounded-lg border border-white/5 bg-black/25 p-3 space-y-2.5 text-xs"
                >
                  <div className="flex flex-wrap items-center justify-between gap-2">
                    <div className="space-y-0.5">
                      <span className="font-semibold text-slate-200">
                        {new Date(r.created_at).toLocaleString('fr-FR')} · {r.title}
                      </span>
                      {r.author_name && (
                        <span className="ml-2 text-[11px] text-slate-400">
                          par {r.author_name}
                        </span>
                      )}
                    </div>
                    <div className="flex items-center gap-2">
                      <button
                        type="button"
                        onClick={() => setComparedRevisionId(isComparing ? null : r.id)}
                        className="inline-flex items-center gap-1 rounded-lg border border-white/10 bg-white/5 px-2.5 py-1 text-slate-200 hover:bg-white/10"
                      >
                        <GitCompare className="w-3.5 h-3.5 text-indigo-400" />
                        <span>{isComparing ? 'Masquer diff' : 'Comparer'}</span>
                      </button>
                      <button
                        type="button"
                        disabled={saving}
                        onClick={() => void restoreRevision(r.id, onSaved)}
                        className="rounded-lg bg-indigo-600 hover:bg-indigo-500 px-3 py-1 font-semibold text-white"
                      >
                        {editorMessages.historyRestore}
                      </button>
                    </div>
                  </div>

                  {/* Comparaison visuelle côte-à-côte avant restauration (WP-28) */}
                  {isComparing && (
                    <div className="grid grid-cols-1 md:grid-cols-2 gap-3 pt-2 border-t border-white/10 text-[11px]">
                      <div className="rounded-lg bg-slate-950/80 border border-white/5 p-2.5 space-y-1.5">
                        <div className="font-semibold text-amber-300">
                          Révision du {new Date(r.created_at).toLocaleString('fr-FR')}
                        </div>
                        <div>
                          <span className="text-slate-500">Titre : </span>
                          <span className={r.title !== title ? 'text-amber-200 font-semibold' : 'text-slate-200'}>
                            {r.title}
                          </span>
                        </div>
                        <div>
                          <span className="text-slate-500">Extrait : </span>
                          <span className={(r.excerpt || '') !== excerpt ? 'text-amber-200' : 'text-slate-300'}>
                            {r.excerpt || '—'}
                          </span>
                        </div>
                        <div>
                          <span className="text-slate-500">Structure : </span>
                          <span className="text-slate-300 font-mono">
                            {Array.isArray(revBlocks)
                              ? `rf.document/1 (${revBlocks.length} blocs)`
                              : r.snapshot
                              ? 'Snapshot complet (Markdown)'
                              : 'Révision historique sans snapshot'}
                          </span>
                        </div>
                        <pre className="mt-1 max-h-40 overflow-y-auto rounded bg-black/40 p-2 font-mono text-[10px] text-slate-300 whitespace-pre-wrap">
                          {r.body || '(vide)'}
                        </pre>
                      </div>

                      <div className="rounded-lg bg-slate-950/80 border border-indigo-500/20 p-2.5 space-y-1.5">
                        <div className="font-semibold text-indigo-300">
                          État actuel dans l’éditeur
                        </div>
                        <div>
                          <span className="text-slate-500">Titre : </span>
                          <span className="text-slate-200">{title}</span>
                        </div>
                        <div>
                          <span className="text-slate-500">Extrait : </span>
                          <span className="text-slate-300">{excerpt || '—'}</span>
                        </div>
                        <div>
                          <span className="text-slate-500">Structure : </span>
                          <span className="text-slate-300 font-mono">
                            {typeof currentBlocksCount === 'number'
                              ? `rf.document/1 (${currentBlocksCount} blocs)`
                              : 'Source Markdown'}
                          </span>
                        </div>
                        <pre className="mt-1 max-h-40 overflow-y-auto rounded bg-black/40 p-2 font-mono text-[10px] text-slate-300 whitespace-pre-wrap">
                          {body || '(projection calculée à l’enregistrement)'}
                        </pre>
                      </div>
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        </section>
      )}
    </div>
  );
};

