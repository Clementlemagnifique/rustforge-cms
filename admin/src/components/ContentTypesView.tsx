import React, { useState } from 'react';
import { Plus, Trash2, Box } from 'lucide-react';
import { api } from '../api';
import type { ContentType } from '../types';

interface ContentTypesViewProps {
  contentTypes: ContentType[];
  onRefresh: () => void;
}

export const ContentTypesView: React.FC<ContentTypesViewProps> = ({
  contentTypes,
  onRefresh,
}) => {
  const [isCreating, setIsCreating] = useState(false);
  const [name, setName] = useState('');
  const [nameSingular, setNameSingular] = useState('');
  const [slug, setSlug] = useState('');
  const [description, setDescription] = useState('');
  const [loading, setLoading] = useState(false);

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!name.trim() || !nameSingular.trim()) return;

    setLoading(true);
    try {
      await api.contentTypes.create({
        name,
        name_singular: nameSingular,
        slug: slug || undefined,
        description,
        fields: [],
      });
      setName('');
      setNameSingular('');
      setSlug('');
      setDescription('');
      setIsCreating(false);
      onRefresh();
    } catch (err: any) {
      alert(`Erreur: ${err.message}`);
    } finally {
      setLoading(false);
    }
  };

  const handleDelete = async (id: string) => {
    if (!confirm('Supprimer ce type de contenu personnalisé ?')) return;
    try {
      await api.contentTypes.delete(id);
      onRefresh();
    } catch (err: any) {
      alert(err.message);
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-base font-bold text-white">Types de Contenus (Custom Post Types)</h3>
          <p className="text-xs text-slate-400">
            Définissez les entités natives de votre CMS (Produits, Formulaires, Événements, Portfolios).
          </p>
        </div>

        <button
          onClick={() => setIsCreating(!isCreating)}
          className="px-4 py-2 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-semibold flex items-center gap-2 shadow-md shadow-indigo-600/20 transition-all hover:scale-105 active:scale-95"
        >
          <Plus className="w-4 h-4" />
          <span>Nouveau Type</span>
        </button>
      </div>

      {/* Formulaire de création */}
      {isCreating && (
        <form onSubmit={handleCreate} className="p-5 rounded-2xl bg-[#0f1422] border border-indigo-500/30 space-y-4">
          <h4 className="text-xs font-bold text-white uppercase tracking-wider">Créer un Type de Contenu</h4>
          <div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
            <div>
              <label className="text-[11px] text-slate-400 block mb-1">Nom Pluriel (ex: Produits)</label>
              <input
                type="text"
                required
                value={name}
                onChange={(e) => setName(e.target.value)}
                className="w-full bg-black/30 border border-white/10 rounded-xl px-3 py-2 text-xs text-white focus:outline-none focus:border-indigo-500"
              />
            </div>
            <div>
              <label className="text-[11px] text-slate-400 block mb-1">Nom Singulier (ex: Produit)</label>
              <input
                type="text"
                required
                value={nameSingular}
                onChange={(e) => setNameSingular(e.target.value)}
                className="w-full bg-black/30 border border-white/10 rounded-xl px-3 py-2 text-xs text-white focus:outline-none focus:border-indigo-500"
              />
            </div>
            <div>
              <label className="text-[11px] text-slate-400 block mb-1">Slug URL (optionnel)</label>
              <input
                type="text"
                value={slug}
                onChange={(e) => setSlug(e.target.value)}
                placeholder="auto-généré"
                className="w-full bg-black/30 border border-white/10 rounded-xl px-3 py-2 text-xs font-mono text-white focus:outline-none focus:border-indigo-500"
              />
            </div>
          </div>

          <div className="flex justify-end gap-2 pt-2">
            <button
              type="button"
              onClick={() => setIsCreating(false)}
              className="px-3 py-1.5 rounded-lg text-xs text-slate-400 hover:text-white"
            >
              Annuler
            </button>
            <button
              type="submit"
              disabled={loading}
              className="px-4 py-1.5 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-semibold"
            >
              {loading ? 'Création...' : 'Créer le type'}
            </button>
          </div>
        </form>
      )}

      {/* Liste des types */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {contentTypes.map((ct) => (
          <div
            key={ct.slug}
            className="p-5 rounded-2xl bg-[#0f1422] border border-white/5 space-y-3"
          >
            <div className="flex items-start justify-between">
              <div className="flex items-center gap-3">
                <div className="w-9 h-9 rounded-xl bg-indigo-500/10 border border-indigo-500/20 text-indigo-400 flex items-center justify-center">
                  <Box className="w-4 h-4" />
                </div>
                <div>
                  <h4 className="text-sm font-bold text-white flex items-center gap-2">
                    {ct.name}
                    {ct.is_system && (
                      <span className="text-[10px] px-1.5 py-0.2 rounded bg-indigo-500/20 text-indigo-300 font-medium">
                        Système
                      </span>
                    )}
                  </h4>
                  <p className="text-[11px] text-slate-500 font-mono">/{ct.slug}</p>
                </div>
              </div>

              {!ct.is_system && (
                <button
                  onClick={() => handleDelete(ct.id)}
                  title="Supprimer ce type"
                  className="p-1.5 text-slate-500 hover:text-red-400 transition-colors"
                >
                  <Trash2 className="w-4 h-4" />
                </button>
              )}
            </div>

            <p className="text-xs text-slate-400">
              {ct.description || 'Type de contenu personnalisé prêt pour les schémas de champs et templates.'}
            </p>

            <div className="pt-2 border-t border-white/5 flex items-center justify-between text-[11px] text-slate-400">
              <span>{ct.entry_count} contenu(s)</span>
              <span className="text-emerald-400 font-medium">Révisions activées</span>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};
