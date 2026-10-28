/**
 * ============================================================================
 * BLOC TITRE, PERMALIEN (SLUG) & DÉPUBLICATION PROGRAMMÉE (`EditorTitleSlugBar`)
 * ============================================================================
 */

import React from 'react';
import { Clock } from 'lucide-react';
import { useEditorStore } from '../../stores/useEditorStore';

export const EditorTitleSlugBar: React.FC = () => {
  const title = useEditorStore((s) => s.title);
  const slug = useEditorStore((s) => s.slug);
  const scheduledUnpublishAt = useEditorStore((s) => s.scheduledUnpublishAt);
  const setTitle = useEditorStore((s) => s.setTitle);
  const setSlug = useEditorStore((s) => s.setSlug);
  const setScheduledUnpublishAt = useEditorStore((s) => s.setScheduledUnpublishAt);

  return (
    <div className="p-5 rounded-2xl bg-[#0f1422] border border-white/5 space-y-4">
      <div>
        <label className="text-[11px] font-semibold text-slate-400 uppercase tracking-wider block mb-1.5">
          Titre du contenu
        </label>
        <input
          type="text"
          value={title}
          aria-label="Titre du contenu"
          onChange={(e) => setTitle(e.target.value)}
          placeholder="Ex: 'Comment RustForge révolutionne la gestion de contenu avec Rust'..."
          className="w-full bg-black/30 border border-white/10 rounded-xl px-4 py-2.5 text-base font-bold text-white placeholder-slate-600 focus:outline-none focus:border-indigo-500"
        />
      </div>

      <div className="flex flex-col xl:flex-row xl:items-center justify-between gap-3 pt-3 border-t border-white/5 text-xs text-slate-400">
        <div className="flex items-center gap-2 flex-1 min-w-0">
          <span className="font-mono text-slate-500">Slug URL :</span>
          <input
            type="text"
            value={slug}
            onChange={(e) => setSlug(e.target.value)}
            placeholder="laisser vide pour auto-génération"
            className="min-w-0 bg-transparent border-b border-white/10 text-slate-200 font-mono text-xs focus:outline-none focus:border-indigo-500 flex-1"
          />
        </div>

        <div className="flex items-center gap-2 shrink-0 bg-white/[0.02] px-3 py-1.5 rounded-xl border border-white/5">
          <Clock className="w-3.5 h-3.5 text-amber-400 shrink-0" />
          <span className="text-slate-400 font-medium">Dépublication auto :</span>
          <input
            type="datetime-local"
            value={scheduledUnpublishAt}
            onChange={(e) => setScheduledUnpublishAt(e.target.value)}
            className="bg-black/30 border border-white/10 rounded-lg px-2.5 py-1 text-xs text-slate-200 focus:outline-none focus:border-indigo-500"
          />
          {scheduledUnpublishAt && (
            <button
              type="button"
              onClick={() => setScheduledUnpublishAt('')}
              className="text-[10px] text-slate-500 hover:text-slate-300 underline ml-1"
            >
              Effacer
            </button>
          )}
        </div>
      </div>
    </div>
  );
};

