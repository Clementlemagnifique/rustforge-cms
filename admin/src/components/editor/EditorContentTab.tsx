/**
 * ============================================================================
 * ONGLET RÉDACTION : BLOCS NATIFS / SOURCE MARKDOWN & APERÇU (`EditorContentTab`)
 * ============================================================================
 * Couvre les exigences :
 * - `WP-23`, `WP-25`, `WP-30` : Édition visuelle par blocs `rf.document/1` avec
 *   historique local `undo/redo`, insertion rapide et médiathèque.
 * - `WP-24` : Conversion bidirectionnelle explicite entre `rf.document/1` et
 *   source Markdown CommonMark.
 * - `WP-33` : Aperçu HTML isolé généré par le même pipeline serveur que le SSR.
 */

import React, { useState } from 'react';
import { Eye, Monitor, Tablet, Smartphone } from 'lucide-react';
import { editorMessages } from '../../i18n/messages.fr';
import { useEditorStore } from '../../stores/useEditorStore';
import { NativeBlockEditor } from '../NativeBlockEditor';

export const EditorContentTab: React.FC = () => {
  const [previewViewport, setPreviewViewport] = useState<'desktop' | 'tablet' | 'mobile'>('desktop');
  const docHistory = useEditorStore((s) => s.docHistory);
  const body = useEditorStore((s) => s.body);
  const saving = useEditorStore((s) => s.saving);
  const converting = useEditorStore((s) => s.converting);
  const mediaLibrary = useEditorStore((s) => s.mediaLibrary);
  const preview = useEditorStore((s) => s.preview);
  const previewError = useEditorStore((s) => s.previewError);

  const setBody = useEditorStore((s) => s.setBody);
  const undo = useEditorStore((s) => s.undo);
  const redo = useEditorStore((s) => s.redo);
  const pushBlocks = useEditorStore((s) => s.pushBlocks);
  const uploadMedia = useEditorStore((s) => s.uploadMedia);
  const convertToSource = useEditorStore((s) => s.convertToSource);
  const convertToBlocks = useEditorStore((s) => s.convertToBlocks);

  const document = docHistory?.present ?? null;

  return (
    <div className="grid grid-cols-1 lg:grid-cols-2 gap-5">
      {/* Zone d'édition (Blocs natifs rf.document/1 ou Source Markdown) */}
      <div className="min-w-0 p-4 sm:p-5 rounded-2xl bg-[#0f1422] border border-white/5 space-y-4">
        <div className="flex items-center justify-between">
          <label className="text-[11px] font-semibold text-slate-400 uppercase tracking-wider">
            {document ? editorMessages.blocks : 'Source Markdown (CommonMark assaini)'}
          </label>
          <span className="text-[10px] text-slate-500 font-mono">
            {document
              ? `${document.rf_blocks.length} blocs`
              : `${body.split(/\s+/).filter(Boolean).length} mots`}
          </span>
        </div>

        {document && docHistory ? (
          <>
            <NativeBlockEditor
              document={document}
              disabled={saving || converting}
              canUndo={docHistory.past.length > 0}
              canRedo={docHistory.future.length > 0}
              onUndo={undo}
              onRedo={redo}
              mediaLibrary={mediaLibrary}
              onUploadMedia={uploadMedia}
              onChange={pushBlocks}
            />
            <button
              type="button"
              disabled={saving || converting}
              onClick={() => void convertToSource()}
              className="text-xs text-slate-400 underline disabled:opacity-40"
            >
              {converting ? editorMessages.converting : editorMessages.source}
            </button>
          </>
        ) : (
          <>
            <div className="flex flex-wrap items-center justify-between gap-2">
              <p className="text-xs text-slate-500">{editorMessages.legacy}</p>
              <button
                type="button"
                disabled={saving || converting}
                onClick={() => void convertToBlocks()}
                className="rounded-lg border border-indigo-400/30 bg-indigo-500/10 px-3 py-1.5 text-xs text-indigo-200 hover:bg-indigo-500/20 disabled:opacity-40"
              >
                {converting ? editorMessages.converting : editorMessages.toBlocks}
              </button>
            </div>
            <textarea
              aria-label="Contenu de la page ou de l’article"
              rows={18}
              value={body}
              onChange={(e) => setBody(e.target.value)}
              placeholder="Rédigez votre contenu ici en Markdown ou demandez à l'Assistant IA de le structurer pour vous..."
              className="w-full bg-black/40 border border-white/10 rounded-xl p-4 font-mono text-xs text-slate-100 placeholder-slate-600 focus:outline-none focus:border-indigo-500 resize-none leading-relaxed"
            />
          </>
        )}
      </div>

      {/* Aperçu Live isolé multi-appareils (WP-33) */}
      <div className="min-w-0 p-4 sm:p-5 rounded-2xl bg-[#0f1422] border border-white/5 space-y-4">
        <div className="flex flex-wrap items-center justify-between gap-2">
          <label className="text-[11px] font-semibold text-slate-400 uppercase tracking-wider flex items-center gap-1.5">
            <Eye className="w-3.5 h-3.5 text-indigo-400" />
            <span>Aperçu du contenu</span>
          </label>
          <div className="flex items-center gap-1 rounded-lg bg-black/30 p-1 border border-white/10">
            <button
              type="button"
              onClick={() => setPreviewViewport('desktop')}
              className={`flex items-center gap-1 px-2 py-1 rounded text-[11px] transition-colors ${
                previewViewport === 'desktop'
                  ? 'bg-indigo-600 text-white'
                  : 'text-slate-400 hover:text-slate-200'
              }`}
              title={editorMessages.viewportDesktop}
            >
              <Monitor className="w-3 h-3" />
              <span>{editorMessages.viewportDesktop}</span>
            </button>
            <button
              type="button"
              onClick={() => setPreviewViewport('tablet')}
              className={`flex items-center gap-1 px-2 py-1 rounded text-[11px] transition-colors ${
                previewViewport === 'tablet'
                  ? 'bg-indigo-600 text-white'
                  : 'text-slate-400 hover:text-slate-200'
              }`}
              title={editorMessages.viewportTablet}
            >
              <Tablet className="w-3 h-3" />
              <span>{editorMessages.viewportTablet}</span>
            </button>
            <button
              type="button"
              onClick={() => setPreviewViewport('mobile')}
              className={`flex items-center gap-1 px-2 py-1 rounded text-[11px] transition-colors ${
                previewViewport === 'mobile'
                  ? 'bg-indigo-600 text-white'
                  : 'text-slate-400 hover:text-slate-200'
              }`}
              title={editorMessages.viewportMobile}
            >
              <Smartphone className="w-3 h-3" />
              <span>{editorMessages.viewportMobile}</span>
            </button>
          </div>
        </div>
        {previewError ? (
          <p role="alert" className="text-amber-300">
            Aperçu indisponible : {previewError}
          </p>
        ) : (
          <div className="bg-black/40 rounded-xl p-2 flex justify-center overflow-x-auto border border-white/5">
            <iframe
              title="Aperçu du contenu"
              sandbox=""
              className={`min-h-[420px] rounded-xl bg-white transition-all duration-200 ${
                previewViewport === 'mobile'
                  ? 'w-[375px] max-w-full shadow-xl'
                  : previewViewport === 'tablet'
                  ? 'w-[768px] max-w-full shadow-xl'
                  : 'w-full'
              }`}
              srcDoc={`<!doctype html><html lang="fr"><meta charset="utf-8"><style>body{font:16px/1.75 system-ui;padding:24px;overflow-wrap:anywhere;color:#172033}img{max-width:100%;height:auto}pre{white-space:pre-wrap}a{color:#4f46e5}</style><body>${
                preview || '<p>Commencez à écrire pour voir votre contenu.</p>'
              }</body></html>`}
            />
          </div>
        )}
      </div>
    </div>
  );
};

