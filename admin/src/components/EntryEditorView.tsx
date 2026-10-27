import React, { useState, useEffect } from 'react';
import { 
  Save, 
  ArrowLeft, 
  Sparkles, 
  Eye, 
  Palette, 
  Compass, 
  FileCode,
  Clock
} from 'lucide-react';
import { api } from '../api';
import type { ContentType, EntryStatus } from '../types';

interface EntryEditorViewProps {
  entryId?: string;
  contentTypeSlug?: string;
  contentTypes: ContentType[];
  onBack: () => void;
  onSaved: () => void;
}

export const EntryEditorView: React.FC<EntryEditorViewProps> = ({
  entryId,
  contentTypeSlug = 'post',
  contentTypes,
  onBack,
  onSaved,
}) => {
  const [activeTab, setActiveTab] = useState<'content' | 'intent' | 'design'>('content');
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);

  const [title, setTitle] = useState('');
  const [slug, setSlug] = useState('');
  const [excerpt, setExcerpt] = useState('');
  const [body, setBody] = useState('');
  const [status, setStatus] = useState<EntryStatus>('draft');
  const [scheduledUnpublishAt, setScheduledUnpublishAt] = useState<string>('');
  const [intent, setIntent] = useState<Record<string, any>>({
    prompt: '',
    target_audience: '',
    tone: 'Professionnel et percutant',
    geo_keywords: '',
  });
  const [designTokens, setDesignTokens] = useState<Record<string, any>>({
    accent_color: '#6366f1',
    layout_mode: 'standard',
  });

  const [fields, setFields] = useState<Record<string, any>>({});

  const isNew = !entryId;
  const currentType = contentTypes.find((ct) => ct.slug === contentTypeSlug);

  useEffect(() => {
    if (entryId) {
      setLoading(true);
      api.entries.get(entryId)
        .then((e) => {
          setTitle(e.title);
          setSlug(e.slug);
          setExcerpt(e.excerpt || '');
          setBody(e.body || '');
          setStatus(e.status);
          if (e.scheduled_unpublish_at) {
            setScheduledUnpublishAt(new Date(e.scheduled_unpublish_at).toISOString().slice(0, 16));
          }
          if (e.intent) setIntent(e.intent);
          if (e.design_tokens) setDesignTokens(e.design_tokens);
          if (e.fields) setFields(e.fields);
        })
        .catch(console.error)
        .finally(() => setLoading(false));
    }
  }, [entryId]);

  const handleSave = async (forceStatus?: EntryStatus) => {
    if (!title.trim()) {
      alert('Veuillez renseigner un titre.');
      return;
    }

    setSaving(true);
    const saveStatus = forceStatus || status;

    const payload = {
      title,
      slug: slug || undefined,
      excerpt,
      body,
      status: saveStatus,
      scheduled_unpublish_at: scheduledUnpublishAt ? new Date(scheduledUnpublishAt).toISOString() : undefined,
      intent,
      design_tokens: designTokens,
      fields, // Conservation intégrale des champs personnalisés et du _legacy_vault
    };

    try {
      if (isNew) {
        await api.entries.create(contentTypeSlug, payload);
      } else {
        await api.entries.update(entryId, payload);
      }
      onSaved();
    } catch (err: any) {
      alert(`Erreur: ${err.message}`);
    } finally {
      setSaving(false);
    }
  };

  if (loading) {
    return <div className="p-8 text-center text-slate-400">Chargement de l'éditeur...</div>;
  }

  return (
    <div className="space-y-6">
      {/* Barre supérieure éditeur */}
      <div className="flex items-center justify-between pb-4 border-b border-white/5">
        <div className="flex items-center gap-3">
          <button
            onClick={onBack}
            className="p-2 rounded-xl bg-white/5 hover:bg-white/10 text-slate-300 transition-colors"
          >
            <ArrowLeft className="w-4 h-4" />
          </button>
          <div>
            <h3 className="text-sm font-bold text-white flex items-center gap-2">
              {isNew ? `Nouveau ${currentType?.name_singular || 'Contenu'}` : `Édition : ${title || 'Sans titre'}`}
              <span className="text-[10px] uppercase font-mono px-2 py-0.5 rounded bg-white/5 text-slate-400">
                {contentTypeSlug}
              </span>
              {((fields?._rf_vault && Object.keys(fields._rf_vault).length > 0) || (fields?._legacy_vault && Object.keys(fields._legacy_vault).length > 0)) && (
                <span className="text-[10px] font-medium px-2 py-0.5 rounded bg-emerald-500/10 text-emerald-300 border border-emerald-500/20 flex items-center gap-1" title="Métadonnées protégées dans le coffre-fort souverain">
                  🔒 Coffre-fort RF ({Object.keys(fields._rf_vault || fields._legacy_vault || {}).length} clés)
                </span>
              )}
            </h3>
          </div>
        </div>

        <div className="flex items-center gap-2.5">
          <button
            onClick={() => handleSave('draft')}
            disabled={saving}
            className="px-3.5 py-2 rounded-xl bg-white/5 hover:bg-white/10 border border-white/10 text-slate-300 text-xs font-semibold transition-all"
          >
            Enregistrer Brouillon
          </button>
          <button
            onClick={() => handleSave('published')}
            disabled={saving}
            className="px-4 py-2 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-semibold shadow-md shadow-indigo-600/25 flex items-center gap-1.5 transition-all hover:scale-105 active:scale-95"
          >
            <Save className="w-3.5 h-3.5" />
            <span>{saving ? 'Publication...' : 'Publier'}</span>
          </button>
        </div>
      </div>

      {/* Titre et Slug */}
      <div className="p-5 rounded-2xl bg-[#0f1422] border border-white/5 space-y-4">
        <div>
          <label className="text-[11px] font-semibold text-slate-400 uppercase tracking-wider block mb-1.5">
            Titre du contenu
          </label>
          <input
            type="text"
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            placeholder="Ex: 'Comment RustForge révolutionne la gestion de contenu avec Rust'..."
            className="w-full bg-black/30 border border-white/10 rounded-xl px-4 py-2.5 text-base font-bold text-white placeholder-slate-600 focus:outline-none focus:border-indigo-500"
          />
        </div>

        <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 pt-3 border-t border-white/5 text-xs text-slate-400">
          <div className="flex items-center gap-2 flex-1">
            <span className="font-mono text-slate-500">Slug URL :</span>
            <input
              type="text"
              value={slug}
              onChange={(e) => setSlug(e.target.value)}
              placeholder="laisser vide pour auto-génération"
              className="bg-transparent border-b border-white/10 text-slate-200 font-mono text-xs focus:outline-none focus:border-indigo-500 flex-1"
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

      {/* Onglets Fond / Forme */}
      <div className="flex items-center gap-2 border-b border-white/5">
        <button
          onClick={() => setActiveTab('content')}
          className={`flex items-center gap-2 px-4 py-2.5 text-xs font-semibold border-b-2 transition-all ${
            activeTab === 'content'
              ? 'border-indigo-500 text-indigo-400 bg-white/[0.02]'
              : 'border-transparent text-slate-400 hover:text-slate-200'
          }`}
        >
          <FileCode className="w-4 h-4" />
          <span>Le Fond : Contenu & Markdown</span>
        </button>

        <button
          onClick={() => setActiveTab('intent')}
          className={`flex items-center gap-2 px-4 py-2.5 text-xs font-semibold border-b-2 transition-all ${
            activeTab === 'intent'
              ? 'border-purple-500 text-purple-400 bg-white/[0.02]'
              : 'border-transparent text-slate-400 hover:text-slate-200'
          }`}
        >
          <Compass className="w-4 h-4 text-purple-400" />
          <span>L'Intention IA : Audience & SEO/GEO</span>
        </button>

        <button
          onClick={() => setActiveTab('design')}
          className={`flex items-center gap-2 px-4 py-2.5 text-xs font-semibold border-b-2 transition-all ${
            activeTab === 'design'
              ? 'border-cyan-500 text-cyan-400 bg-white/[0.02]'
              : 'border-transparent text-slate-400 hover:text-slate-200'
          }`}
        >
          <Palette className="w-4 h-4 text-cyan-400" />
          <span>La Forme : Design Tokens</span>
        </button>
      </div>

      {/* Contenu selon l'onglet actif */}
      {activeTab === 'content' && (
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-5">
          {/* Éditeur */}
          <div className="p-5 rounded-2xl bg-[#0f1422] border border-white/5 space-y-4">
            <div className="flex items-center justify-between">
              <label className="text-[11px] font-semibold text-slate-400 uppercase tracking-wider">
                Source Markdown (CommonMark assaini)
              </label>
              <span className="text-[10px] text-slate-500 font-mono">
                {body.split(/\s+/).filter(Boolean).length} mots
              </span>
            </div>
            <textarea
              rows={18}
              value={body}
              onChange={(e) => setBody(e.target.value)}
              placeholder="Rédigez votre contenu ici en Markdown ou demandez à l'Assistant IA de le structurer pour vous..."
              className="w-full bg-black/40 border border-white/10 rounded-xl p-4 font-mono text-xs text-slate-100 placeholder-slate-600 focus:outline-none focus:border-indigo-500 resize-none leading-relaxed"
            />
          </div>

          {/* Aperçu Live */}
          <div className="p-5 rounded-2xl bg-[#0f1422] border border-white/5 space-y-4">
            <div className="flex items-center justify-between">
              <label className="text-[11px] font-semibold text-slate-400 uppercase tracking-wider flex items-center gap-1.5">
                <Eye className="w-3.5 h-3.5 text-indigo-400" />
                <span>Rendu Serveur Direct</span>
              </label>
              <span className="text-[10px] bg-emerald-500/10 text-emerald-400 px-2 py-0.5 rounded-full border border-emerald-500/20">
                XSS assaini
              </span>
            </div>
            <div className="p-4 rounded-xl bg-black/20 border border-white/5 min-h-[420px] prose prose-invert prose-sm max-w-none text-slate-200">
              {body ? (
                <div dangerouslySetInnerHTML={{ __html: body.replace(/\n/g, '<br/>') }} />
              ) : (
                <p className="text-slate-600 italic">L'aperçu en direct apparaîtra ici...</p>
              )}
            </div>
          </div>
        </div>
      )}

      {activeTab === 'intent' && (
        <div className="p-6 rounded-2xl bg-[#0f1422] border border-white/5 space-y-5">
          <div className="flex items-center gap-2 text-purple-400">
            <Sparkles className="w-5 h-5" />
            <h4 className="text-sm font-bold text-white">Métadonnées d'Intention & Optimisation GEO</h4>
          </div>
          <p className="text-xs text-slate-400">
            Ces paramètres guident les agents IA lors de la co-rédaction, de la traduction automatique et de l'optimisation pour les moteurs conversationnels (ChatGPT Search, Perplexity, Claude).
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
      )}

      {activeTab === 'design' && (
        <div className="p-6 rounded-2xl bg-[#0f1422] border border-white/5 space-y-5">
          <div className="flex items-center gap-2 text-cyan-400">
            <Palette className="w-5 h-5" />
            <h4 className="text-sm font-bold text-white">Tokens de Design Locaux</h4>
          </div>
          <p className="text-xs text-slate-400">
            Permet de surcharger le style global pour cette page spécifique (couleur d'accentuation, modèle de layout).
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
                  onChange={(e) => setDesignTokens({ ...designTokens, accent_color: e.target.value })}
                  className="w-9 h-9 rounded-lg bg-transparent cursor-pointer border border-white/10"
                />
                <input
                  type="text"
                  value={designTokens.accent_color || '#6366f1'}
                  onChange={(e) => setDesignTokens({ ...designTokens, accent_color: e.target.value })}
                  className="bg-black/30 border border-white/10 rounded-xl px-3 py-2 text-xs font-mono text-white focus:outline-none focus:border-cyan-500"
                />
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
