import React, { useState, useEffect } from 'react';
import { Palette, Save, CheckCircle2 } from 'lucide-react';
import { api } from '../api';

export const SettingsView: React.FC = () => {
  const [settings, setSettings] = useState<Record<string, any>>({});
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    api.settings.getAll()
      .then(setSettings)
      .catch(console.error)
      .finally(() => setLoading(false));
  }, []);

  const handleSave = async (e: React.FormEvent) => {
    e.preventDefault();
    setSaving(true);
    setSaved(false);
    try {
      const res = await api.settings.update(settings);
      setSettings(res);
      setSaved(true);
      setTimeout(() => setSaved(false), 3000);
    } catch (err: any) {
      alert(`Erreur: ${err.message}`);
    } finally {
      setSaving(false);
    }
  };

  if (loading) {
    return <div className="p-8 text-center text-slate-500">Chargement des réglages...</div>;
  }

  return (
    <form onSubmit={handleSave} className="space-y-6 max-w-4xl">
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-base font-bold text-white">Réglages Généraux & Design System</h3>
          <p className="text-xs text-slate-400">
            Contrôlez les métadonnées globales du site et les variables de style.
          </p>
        </div>

        <button
          type="submit"
          disabled={saving}
          className="px-4 py-2 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-semibold flex items-center gap-2 shadow-md shadow-indigo-600/20 transition-all hover:scale-105 active:scale-95"
        >
          {saved ? <CheckCircle2 className="w-4 h-4 text-emerald-300" /> : <Save className="w-4 h-4" />}
          <span>{saving ? 'Enregistrement...' : saved ? 'Enregistré !' : 'Enregistrer'}</span>
        </button>
      </div>

      {/* Identité du site */}
      <div className="p-6 rounded-2xl bg-[#0f1422] border border-white/5 space-y-4">
        <h4 className="text-xs font-bold text-white uppercase tracking-wider">Identité du site</h4>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div>
            <label className="text-[11px] text-slate-400 block mb-1">Titre du site</label>
            <input
              type="text"
              value={settings['site.title'] || ''}
              onChange={(e) => setSettings({ ...settings, 'site.title': e.target.value })}
              className="w-full bg-black/30 border border-white/10 rounded-xl px-3 py-2 text-xs text-white focus:outline-none focus:border-indigo-500"
            />
          </div>

          <div>
            <label className="text-[11px] text-slate-400 block mb-1">Slogan</label>
            <input
              type="text"
              value={settings['site.tagline'] || ''}
              onChange={(e) => setSettings({ ...settings, 'site.tagline': e.target.value })}
              className="w-full bg-black/30 border border-white/10 rounded-xl px-3 py-2 text-xs text-white focus:outline-none focus:border-indigo-500"
            />
          </div>
        </div>
      </div>

      {/* Design System & Tokens */}
      <div className="p-6 rounded-2xl bg-[#0f1422] border border-white/5 space-y-4">
        <div className="flex items-center gap-2 text-indigo-400">
          <Palette className="w-4 h-4" />
          <h4 className="text-xs font-bold text-white uppercase tracking-wider">Tokens Globaux du Design System</h4>
        </div>
        <p className="text-xs text-slate-400">
          Ces tokens sont modifiables par simple commande vocale ou prompt dans l'Assistant IA.
        </p>

        <div className="grid grid-cols-1 sm:grid-cols-3 gap-4 pt-2">
          <div>
            <label className="text-[11px] text-slate-400 block mb-1">Couleur Primaire</label>
            <input
              type="text"
              value={settings['site.design_tokens']?.primary_color || '#6366f1'}
              onChange={(e) =>
                setSettings({
                  ...settings,
                  'site.design_tokens': {
                    ...(settings['site.design_tokens'] || {}),
                    primary_color: e.target.value,
                  },
                })
              }
              className="w-full bg-black/30 border border-white/10 rounded-xl px-3 py-2 text-xs font-mono text-white focus:outline-none focus:border-indigo-500"
            />
          </div>

          <div>
            <label className="text-[11px] text-slate-400 block mb-1">Police des Titres</label>
            <input
              type="text"
              value={settings['site.design_tokens']?.font_heading || 'Plus Jakarta Sans'}
              onChange={(e) =>
                setSettings({
                  ...settings,
                  'site.design_tokens': {
                    ...(settings['site.design_tokens'] || {}),
                    font_heading: e.target.value,
                  },
                })
              }
              className="w-full bg-black/30 border border-white/10 rounded-xl px-3 py-2 text-xs text-white focus:outline-none focus:border-indigo-500"
            />
          </div>

          <div>
            <label className="text-[11px] text-slate-400 block mb-1">Rayon des Bordures</label>
            <input
              type="text"
              value={settings['site.design_tokens']?.radius || '12px'}
              onChange={(e) =>
                setSettings({
                  ...settings,
                  'site.design_tokens': {
                    ...(settings['site.design_tokens'] || {}),
                    radius: e.target.value,
                  },
                })
              }
              className="w-full bg-black/30 border border-white/10 rounded-xl px-3 py-2 text-xs text-white focus:outline-none focus:border-indigo-500"
            />
          </div>
        </div>
      </div>
    </form>
  );
};
