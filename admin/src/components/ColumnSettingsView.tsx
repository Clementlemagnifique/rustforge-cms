import { useStudioStore } from '../stores/useStudioStore';
/**
 * ============================================================================
 * ÉCRAN DE CONFIGURATION DES COLONNES & VUES (`ColumnSettingsView`)
 * ============================================================================
 * Inspiré d'Admin Columns Pro (https://www.admincolumns.com/) & Adminify :
 * - `MOD-COL-01` : Contrat et types de colonnes (standard, médias, taxonomies, mots).
 * - `MOD-COL-02` : Sélection, ordre, largeur et visibilité.
 * - `MOD-COL-03` : Rendu riche et personnalisation des étiquettes.
 * - `MOD-COL-04` : Tri et préréglages (Presets).
 * - `MOD-COL-05` : Prévisualisation interactive et sauvegarde globale de l'agence.
 */

import React, { useState } from 'react';
import {
  Columns3,
  Eye,
  EyeOff,
  ArrowUp,
  ArrowDown,
  RotateCcw,
  Check,
  Save,
  SlidersHorizontal,
  FileText,
  Tag,
  Calendar,
  User,
  Image as ImageIcon,
  Sparkles,
} from 'lucide-react';
import {
  useColumnSettingsStore, useColumns,
} from '../stores/useColumnSettingsStore';

interface ColumnSettingsViewProps {
  onNavigate: (view: string) => void;
}

export const ColumnSettingsView: React.FC<ColumnSettingsViewProps> = ({ onNavigate }) => {
  const [activeType, setActiveType] = useState<string>('post');
  const [saveSuccess, setSaveSuccess] = useState<boolean>(false);

  const toggleColumn = useColumnSettingsStore((s) => s.toggleColumn);
  const moveColumn = useColumnSettingsStore((s) => s.moveColumn);
  const updateColumnLabel = useColumnSettingsStore((s) => s.updateColumnLabel);
  const applyPreset = useColumnSettingsStore((s) => s.applyPreset);
  const resetToDefault = useColumnSettingsStore((s) => s.resetToDefault);
  const saveGlobalPreferences = useColumnSettingsStore((s) => s.saveGlobalPreferences);
  const saving = useColumnSettingsStore((s) => s.saving);
  const density = useStudioStore((s) => s.studioDensity);
  const setDensity = useStudioStore((s) => s.setStudioDensity);

  const columns = useColumns(activeType);
  const visibleColumns = columns.filter((c) => c.visible);

  const handleSaveGlobal = async () => {
    const ok = await saveGlobalPreferences(activeType);
    if (ok) {
      setSaveSuccess(true);
      setTimeout(() => setSaveSuccess(false), 3000);
    }
  };

  const getColumnIcon = (id: string) => {
    switch (id) {
      case 'image':
        return <ImageIcon className="w-3.5 h-3.5 text-pink-400" />;
      case 'title':
        return <FileText className="w-3.5 h-3.5 text-indigo-400" />;
      case 'status':
        return <SlidersHorizontal className="w-3.5 h-3.5 text-emerald-400" />;
      case 'author':
        return <User className="w-3.5 h-3.5 text-sky-400" />;
      case 'categories':
      case 'tags':
        return <Tag className="w-3.5 h-3.5 text-amber-400" />;
      case 'date':
        return <Calendar className="w-3.5 h-3.5 text-purple-400" />;
      case 'words':
        return <Sparkles className="w-3.5 h-3.5 text-cyan-400" />;
      default:
        return <Columns3 className="w-3.5 h-3.5 text-slate-400" />;
    }
  };

  return (
    <div className="space-y-6 max-w-6xl mx-auto p-4 sm:p-6 text-slate-100">
      {/* En-tête */}
      <div className="flex flex-wrap items-center justify-between gap-4 border-b border-white/5 pb-4">
        <div>
          <div className="flex items-center gap-2">
            <span className="p-2 rounded-xl bg-indigo-500/10 border border-indigo-500/20 text-indigo-400">
              <Columns3 className="w-5 h-5" />
            </span>
            <h1 className="text-xl font-bold text-white tracking-tight">
              Colonnes & Vues de Données
            </h1>
          </div>
          <p className="text-xs text-slate-400 mt-1">
            Personnalisez l’affichage tabulaire des contenus dans le Studio, inspiré d’Admin Columns Pro.
          </p>
        </div>

        <div className="flex items-center gap-2">
          <button
            onClick={() => onNavigate(`entries:${activeType}`)}
            className="px-3.5 py-1.5 rounded-xl bg-white/5 hover:bg-white/10 text-slate-300 text-xs font-medium transition-colors"
          >
            Voir la liste {activeType === 'post' ? 'Articles' : 'Pages'}
          </button>
          <button
            disabled={saving}
            onClick={handleSaveGlobal}
            className="px-4 py-1.5 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-semibold flex items-center gap-2 shadow-lg shadow-indigo-600/20 transition-all disabled:opacity-50"
          >
            {saving ? (
              <span className="w-3.5 h-3.5 border-2 border-white/30 border-t-white rounded-full animate-spin" />
            ) : saveSuccess ? (
              <Check className="w-4 h-4 text-emerald-300" />
            ) : (
              <Save className="w-4 h-4" />
            )}
            <span>{saveSuccess ? 'Enregistré !' : 'Enregistrer pour le site'}</span>
          </button>
        </div>
      </div>

      {/* Sélecteur de type de contenu */}
      <div className="flex flex-wrap items-center justify-between gap-3 bg-[#0f1422] border border-white/5 rounded-2xl p-2">
        <div className="flex items-center gap-1.5">
          <button
            onClick={() => setActiveType('post')}
            className={`px-3.5 py-1.5 rounded-xl text-xs font-semibold transition-all ${
              activeType === 'post'
                ? 'bg-indigo-600 text-white shadow'
                : 'text-slate-400 hover:text-white hover:bg-white/5'
            }`}
          >
            Articles (post)
          </button>
          <button
            onClick={() => setActiveType('page')}
            className={`px-3.5 py-1.5 rounded-xl text-xs font-semibold transition-all ${
              activeType === 'page'
                ? 'bg-indigo-600 text-white shadow'
                : 'text-slate-400 hover:text-white hover:bg-white/5'
            }`}
          >
            Pages (page)
          </button>
        </div>

        {/* Densité & Presets */}
        <div className="flex items-center gap-2 text-xs">
          <span className="text-slate-400 font-medium">Densité :</span>
          <div className="inline-flex rounded-lg bg-black/40 p-0.5 border border-white/10">
            <button
              onClick={() => setDensity('comfortable')}
              className={`px-2.5 py-1 rounded-md text-[11px] font-medium transition-colors ${
                density === 'comfortable'
                  ? 'bg-indigo-600 text-white'
                  : 'text-slate-400 hover:text-white'
              }`}
            >
              Confortable
            </button>
            <button
              onClick={() => setDensity('compact')}
              className={`px-2.5 py-1 rounded-md text-[11px] font-medium transition-colors ${
                density === 'compact'
                  ? 'bg-indigo-600 text-white'
                  : 'text-slate-400 hover:text-white'
              }`}
            >
              Compact
            </button>
          </div>
        </div>
      </div>

      {/* Barre des Préréglages (Presets Admin Columns Pro) */}
      <div className="flex flex-wrap items-center justify-between gap-3 bg-white/[0.02] border border-white/5 rounded-2xl p-3">
        <div className="flex flex-wrap items-center gap-2">
          <span className="text-xs text-slate-400 font-medium">Vues prédéfinies :</span>
          <button
            onClick={() => applyPreset(activeType, 'standard')}
            className="px-2.5 py-1 rounded-lg bg-white/5 hover:bg-indigo-500/20 text-slate-300 hover:text-indigo-200 text-xs font-medium border border-white/5 transition-colors"
          >
            Standard
          </button>
          <button
            onClick={() => applyPreset(activeType, 'editorial')}
            className="px-2.5 py-1 rounded-lg bg-white/5 hover:bg-indigo-500/20 text-slate-300 hover:text-indigo-200 text-xs font-medium border border-white/5 transition-colors"
          >
            Éditoriale (mots & lecture)
          </button>
          <button
            onClick={() => applyPreset(activeType, 'seo')}
            className="px-2.5 py-1 rounded-lg bg-white/5 hover:bg-indigo-500/20 text-slate-300 hover:text-indigo-200 text-xs font-medium border border-white/5 transition-colors"
          >
            SEO & Technique
          </button>
          <button
            onClick={() => applyPreset(activeType, 'compact')}
            className="px-2.5 py-1 rounded-lg bg-white/5 hover:bg-indigo-500/20 text-slate-300 hover:text-indigo-200 text-xs font-medium border border-white/5 transition-colors"
          >
            Minimaliste
          </button>
        </div>

        <button
          onClick={() => resetToDefault(activeType)}
          className="text-xs text-slate-400 hover:text-rose-300 flex items-center gap-1 transition-colors"
        >
          <RotateCcw className="w-3.5 h-3.5" />
          <span>Réinitialiser par défaut</span>
        </button>
      </div>

      {/* Éditeur de Colonnes (Ordre, Visibilité, Libellé) */}
      <div className="grid grid-cols-1 lg:grid-cols-12 gap-6">
        <div className="lg:col-span-6 space-y-3">
          <h2 className="text-xs font-semibold text-slate-300 uppercase tracking-wider">
            Configuration des Colonnes ({columns.length})
          </h2>

          <div className="space-y-2">
            {columns.map((col, index) => (
              <div
                key={col.id}
                className={`flex items-center justify-between gap-3 p-3 rounded-xl border transition-all ${
                  col.visible
                    ? 'bg-[#0f1422] border-white/10 shadow-sm'
                    : 'bg-white/[0.01] border-dashed border-white/5 opacity-60'
                }`}
              >
                {/* Icône & Réordonnancement */}
                <div className="flex items-center gap-2">
                  <div className="flex flex-col gap-0.5">
                    <button
                      type="button"
                      disabled={index === 0}
                      onClick={() => moveColumn(activeType, index, index - 1)}
                      className="p-0.5 text-slate-500 hover:text-indigo-300 disabled:opacity-20"
                      title="Monter"
                    >
                      <ArrowUp className="w-3 h-3" />
                    </button>
                    <button
                      type="button"
                      disabled={index === columns.length - 1}
                      onClick={() => moveColumn(activeType, index, index + 1)}
                      className="p-0.5 text-slate-500 hover:text-indigo-300 disabled:opacity-20"
                      title="Descendre"
                    >
                      <ArrowDown className="w-3 h-3" />
                    </button>
                  </div>

                  <span className="p-1.5 rounded-lg bg-black/40 border border-white/5">
                    {getColumnIcon(col.id)}
                  </span>
                </div>

                {/* Libellé éditable */}
                <div className="flex-1 min-w-0">
                  <input
                    type="text"
                    value={col.label}
                    onChange={(e) => updateColumnLabel(activeType, col.id, e.target.value)}
                    className="w-full bg-slate-950/60 border border-white/10 rounded-lg px-2.5 py-1 text-xs text-white focus:outline-none focus:border-indigo-500"
                  />
                  <div className="flex items-center gap-2 mt-1">
                    <span className="text-[10px] text-slate-500 font-mono">clef: {col.id}</span>
                    {col.sortable && (
                      <span className="text-[9px] px-1.5 py-0.2 rounded bg-indigo-500/10 text-indigo-300 font-medium">
                        Triable
                      </span>
                    )}
                  </div>
                </div>

                {/* Bascule Visibilité */}
                <button
                  type="button"
                  onClick={() => toggleColumn(activeType, col.id)}
                  className={`p-2 rounded-xl transition-colors ${
                    col.visible
                      ? 'bg-emerald-500/10 text-emerald-400 hover:bg-emerald-500/20'
                      : 'bg-white/5 text-slate-500 hover:bg-white/10'
                  }`}
                  title={col.visible ? 'Masquer la colonne' : 'Afficher la colonne'}
                >
                  {col.visible ? <Eye className="w-4 h-4" /> : <EyeOff className="w-4 h-4" />}
                </button>
              </div>
            ))}
          </div>
        </div>

        {/* Aperçu interactif en direct */}
        <div className="lg:col-span-6 space-y-3">
          <div className="flex items-center justify-between">
            <h2 className="text-xs font-semibold text-slate-300 uppercase tracking-wider">
              Aperçu en Direct ({visibleColumns.length} colonnes affichées)
            </h2>
            <span className="text-[11px] text-slate-500">Données simulées</span>
          </div>

          <div className="rounded-2xl bg-[#0f1422] border border-white/10 overflow-hidden shadow-xl">
            <div className="overflow-x-auto">
              <table className={`w-full text-left text-xs ${density === 'compact' ? 'py-1' : ''}`}>
                <thead>
                  <tr className="border-b border-white/5 bg-white/[0.02] text-slate-400 font-semibold uppercase text-[10px]">
                    <th className="py-2.5 px-3 w-8">
                      <input type="checkbox" disabled className="rounded bg-slate-900 border-white/20" />
                    </th>
                    {visibleColumns.map((col) => (
                      <th key={col.id} className="py-2.5 px-3 whitespace-nowrap">
                        {col.label}
                      </th>
                    ))}
                    <th className="py-2.5 px-3 text-right">Actions</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-white/5">
                  <tr className="hover:bg-white/[0.02]">
                    <td className="py-3 px-3">
                      <input type="checkbox" disabled className="rounded bg-slate-900 border-white/20" />
                    </td>
                    {visibleColumns.map((col) => (
                      <td key={col.id} className="py-3 px-3">
                        {col.id === 'image' ? (
                          <div className="w-9 h-9 rounded-lg bg-indigo-500/20 border border-indigo-500/30 flex items-center justify-center text-indigo-300 font-bold text-xs">
                            📷
                          </div>
                        ) : col.id === 'title' ? (
                          <div>
                            <p className="font-semibold text-white">Guide Complet Rust 2026</p>
                            <span className="text-[10px] text-slate-500 font-mono">/guide-rust-2026</span>
                          </div>
                        ) : col.id === 'status' ? (
                          <span className="px-2 py-0.5 rounded-full text-[10px] bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
                            Publié
                          </span>
                        ) : col.id === 'author' ? (
                          <span className="text-slate-300">Clément L.</span>
                        ) : col.id === 'categories' ? (
                          <span className="inline-block px-1.5 py-0.5 rounded bg-white/5 text-[10px] text-indigo-300">
                            Technologie
                          </span>
                        ) : col.id === 'tags' ? (
                          <span className="inline-block px-1.5 py-0.5 rounded bg-white/5 text-[10px] text-slate-300">
                            #rust
                          </span>
                        ) : col.id === 'date' ? (
                          <span className="text-slate-400">29/10/2026</span>
                        ) : col.id === 'words' ? (
                          <span className="text-cyan-300 font-mono text-[11px]">1 450 mots • 6 min</span>
                        ) : col.id === 'id' ? (
                          <span className="text-slate-500 font-mono text-[10px]">art_8f91</span>
                        ) : (
                          <span className="text-slate-400">—</span>
                        )}
                      </td>
                    ))}
                    <td className="py-3 px-3 text-right text-slate-500">
                      <span className="text-xs">⋯</span>
                    </td>
                  </tr>

                  <tr className="hover:bg-white/[0.02]">
                    <td className="py-3 px-3">
                      <input type="checkbox" disabled className="rounded bg-slate-900 border-white/20" />
                    </td>
                    {visibleColumns.map((col) => (
                      <td key={col.id} className="py-3 px-3">
                        {col.id === 'image' ? (
                          <div className="w-9 h-9 rounded-lg bg-emerald-500/20 border border-emerald-500/30 flex items-center justify-center text-emerald-300 font-bold text-xs">
                            🖼️
                          </div>
                        ) : col.id === 'title' ? (
                          <div>
                            <p className="font-semibold text-white">Souveraineté Numérique</p>
                            <span className="text-[10px] text-slate-500 font-mono">/souverainete-numerique</span>
                          </div>
                        ) : col.id === 'status' ? (
                          <span className="px-2 py-0.5 rounded-full text-[10px] bg-amber-500/10 text-amber-400 border border-amber-500/20">
                            Brouillon
                          </span>
                        ) : col.id === 'author' ? (
                          <span className="text-slate-300">Alex R.</span>
                        ) : col.id === 'categories' ? (
                          <span className="inline-block px-1.5 py-0.5 rounded bg-white/5 text-[10px] text-indigo-300">
                            Sécurité
                          </span>
                        ) : col.id === 'tags' ? (
                          <span className="inline-block px-1.5 py-0.5 rounded bg-white/5 text-[10px] text-slate-300">
                            #rgpd
                          </span>
                        ) : col.id === 'date' ? (
                          <span className="text-slate-400">28/10/2026</span>
                        ) : col.id === 'words' ? (
                          <span className="text-cyan-300 font-mono text-[11px]">820 mots • 3 min</span>
                        ) : col.id === 'id' ? (
                          <span className="text-slate-500 font-mono text-[10px]">art_7a22</span>
                        ) : (
                          <span className="text-slate-400">—</span>
                        )}
                      </td>
                    ))}
                    <td className="py-3 px-3 text-right text-slate-500">
                      <span className="text-xs">⋯</span>
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>

          <div className="p-4 rounded-xl bg-indigo-950/20 border border-indigo-500/20 text-xs text-indigo-200">
            <p className="font-semibold mb-1">💡 Astuce productivité (Admin Columns Pro) :</p>
            <p className="text-slate-400 text-[11px] leading-relaxed">
              Dans la liste de vos contenus, le bouton <strong className="text-indigo-300">Colonnes</strong> vous permet d’ajuster l’affichage en 1 clic sans quitter votre travail. Vous pouvez également cliquer sur <strong className="text-indigo-300">Exporter CSV</strong> pour extraire uniquement les colonnes sélectionnées.
            </p>
          </div>
        </div>
      </div>
    </div>
  );
};
