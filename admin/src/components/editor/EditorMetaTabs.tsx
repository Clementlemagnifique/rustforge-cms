/**
 * ============================================================================
 * ONGLETS MÉTADONNÉES IA (`intent`) & TOKENS DE STYLE (`designTokens`)
 * ============================================================================
 */

import React from 'react';
import { Sparkles, Palette } from 'lucide-react';
import { useEditorStore } from '../../stores/useEditorStore';

export const EditorIntentTab: React.FC = () => {
  const intent = useEditorStore((s) => s.intent);
  const setIntent = useEditorStore((s) => s.setIntent);

  return (
    <div className="p-6 rounded-2xl bg-[#0f1422] border border-white/5 space-y-5">
      <div className="flex items-center gap-2 text-purple-400">
        <Sparkles className="w-5 h-5" />
        <h4 className="text-sm font-bold text-white">
          Métadonnées d'Intention & Optimisation GEO
        </h4>
      </div>
      <p className="text-xs text-slate-400">
        Ces paramètres guident les agents IA lors de la co-rédaction, de la traduction automatique
        et de l'optimisation pour les moteurs conversationnels (ChatGPT Search, Perplexity, Claude).
      </p>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4 pt-2">
        <div>
          <label className="text-[11px] font-semibold text-slate-300 block mb-1">
            Audience Cible
          </label>
          <input
            type="text"
            value={intent.target_audience || ''}
            onChange={(e) => setIntent({ ...intent, target_audience: e.target.value })}
            placeholder="Ex: Dirigeants CTO, Développeurs Senior Rust, Équipes Marketing..."
            className="w-full bg-black/30 border border-white/10 rounded-xl px-3.5 py-2 text-xs text-white placeholder-slate-600 focus:outline-none focus:border-purple-500"
          />
        </div>

        <div>
          <label className="text-[11px] font-semibold text-slate-300 block mb-1">
            Tonalité / Voix de marque
          </label>
          <input
            type="text"
            value={intent.tone || ''}
            onChange={(e) => setIntent({ ...intent, tone: e.target.value })}
            placeholder="Ex: Visionnaire, rigoureux, direct et bienveillant..."
            className="w-full bg-black/30 border border-white/10 rounded-xl px-3.5 py-2 text-xs text-white placeholder-slate-600 focus:outline-none focus:border-purple-500"
          />
        </div>
      </div>
    </div>
  );
};

export const EditorDesignTab: React.FC = () => {
  const designTokens = useEditorStore((s) => s.designTokens);
  const setDesignTokens = useEditorStore((s) => s.setDesignTokens);

  return (
    <div className="p-6 rounded-2xl bg-[#0f1422] border border-white/5 space-y-5">
      <div className="flex items-center gap-2 text-cyan-400">
        <Palette className="w-5 h-5" />
        <h4 className="text-sm font-bold text-white">Tokens de Design Locaux</h4>
      </div>
      <p className="text-xs text-slate-400">
        Permet de surcharger le style global pour cette page spécifique (couleur d'accentuation,
        modèle de layout).
      </p>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4 pt-2">
        <div>
          <label className="text-[11px] font-semibold text-slate-300 block mb-1">
            Couleur d'accent locale
          </label>
          <div className="flex items-center gap-2">
            <input
              type="color"
              value={designTokens.accent_color || '#6366f1'}
              onChange={(e) =>
                setDesignTokens({ ...designTokens, accent_color: e.target.value })
              }
              className="w-9 h-9 rounded-lg bg-transparent cursor-pointer border border-white/10"
            />
            <input
              type="text"
              value={designTokens.accent_color || '#6366f1'}
              onChange={(e) =>
                setDesignTokens({ ...designTokens, accent_color: e.target.value })
              }
              className="bg-black/30 border border-white/10 rounded-xl px-3 py-2 text-xs font-mono text-white focus:outline-none focus:border-cyan-500"
            />
          </div>
        </div>
      </div>
    </div>
  );
};

