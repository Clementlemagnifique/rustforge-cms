/**
 * ============================================================================
 * VUE D'APPARENCE, STYLES GLOBAUX & MENUS PUBLICS (`AppearanceView`)
 * ============================================================================
 * Pilotée par `useAppearanceStore` (Zustand) :
 * - `WP-48`, `TH-12`, `TH-27` : Personnalisation des tokens du thème `rf-base`
 *   (`primary_color`, `accent_color`, `radius`, `container_width`).
 * - `WP-50` : Éditeur de CSS personnalisé borné (max 16 Kio, sans `@import` ni HTML).
 * - `WP-46` : Gestionnaire visuel des menus de navigation publique par emplacement
 *   (`header`, `footer`) avec ajout rapide depuis les pages publiées et catégories.
 */

import React, { useEffect, useState } from 'react';
import {
  Palette,
  Navigation,
  Plus,
  Trash2,
  ArrowUp,
  ArrowDown,
  ArrowLeft,
  ArrowRight,
  Copy,
  Edit3,
  Save,
} from 'lucide-react';
import { appearanceMessages } from '../i18n/messages.fr';
import { useAppearanceStore, getItemDepth } from '../stores/useAppearanceStore';
import type { AppearanceTab } from '../stores/useAppearanceStore';

const COLOR_PRESETS = [
  { name: 'Moderne Indigo', primary: '#4f46e5', accent: '#06b6d4' },
  { name: 'Émeraude & Ambre', primary: '#059669', accent: '#f59e0b' },
  { name: 'Ardoise & Océan', primary: '#0284c7', accent: '#38bdf8' },
  { name: 'Rose & Solaire', primary: '#e11d48', accent: '#f97316' },
  { name: 'Violet Royal', primary: '#7c3aed', accent: '#d946ef' },
];

const HEADING_FONTS = [
  { name: 'Plus Jakarta Sans (Moderne & Géométrique)', value: "'Plus Jakarta Sans', system-ui, sans-serif" },
  { name: 'Inter (Neutre & Polyvalent)', value: "'Inter', system-ui, sans-serif" },
  { name: 'Playfair Display (Éditorial & Élégant)', value: "'Playfair Display', Georgia, serif" },
  { name: 'Space Grotesk (Technique & Avant-garde)', value: "'Space Grotesk', system-ui, monospace" },
  { name: 'Système Natif (Zéro chargement)', value: "system-ui, -apple-system, sans-serif" },
];

const BODY_FONTS = [
  { name: 'Inter (Optimisé pour la lecture)', value: "'Inter', system-ui, sans-serif" },
  { name: 'Plus Jakarta Sans (Harmonisé)', value: "'Plus Jakarta Sans', system-ui, sans-serif" },
  { name: 'Merriweather (Sérif classique)', value: "'Merriweather', Georgia, serif" },
  { name: 'Système Natif (Zéro chargement)', value: "system-ui, -apple-system, sans-serif" },
];

interface AppearanceViewProps {
  initialTab?: AppearanceTab;
}

export const AppearanceView: React.FC<AppearanceViewProps> = ({ initialTab }) => {
  const [editingId, setEditingId] = useState<string | null>(null);

  const activeTab = useAppearanceStore((s) => s.activeTab);
  const loading = useAppearanceStore((s) => s.loading);
  const saving = useAppearanceStore((s) => s.saving);
  const error = useAppearanceStore((s) => s.error);
  const message = useAppearanceStore((s) => s.message);

  const tokens = useAppearanceStore((s) => s.tokens);
  const menus = useAppearanceStore((s) => s.menus);
  const activeLocation = useAppearanceStore((s) => s.activeLocation);
  const newLinkLabel = useAppearanceStore((s) => s.newLinkLabel);
  const newLinkUrl = useAppearanceStore((s) => s.newLinkUrl);
  const newLinkTarget = useAppearanceStore((s) => s.newLinkTarget);
  const newLinkBadge = useAppearanceStore((s) => s.newLinkBadge);
  const newLinkSubtitle = useAppearanceStore((s) => s.newLinkSubtitle);
  const publishedPages = useAppearanceStore((s) => s.publishedPages);
  const categories = useAppearanceStore((s) => s.categories);

  const setActiveTab = useAppearanceStore((s) => s.setActiveTab);
  const setToken = useAppearanceStore((s) => s.setToken);
  const setActiveLocation = useAppearanceStore((s) => s.setActiveLocation);
  const setNewLinkLabel = useAppearanceStore((s) => s.setNewLinkLabel);
  const setNewLinkUrl = useAppearanceStore((s) => s.setNewLinkUrl);
  const setNewLinkTarget = useAppearanceStore((s) => s.setNewLinkTarget);
  const setNewLinkBadge = useAppearanceStore((s) => s.setNewLinkBadge);
  const setNewLinkSubtitle = useAppearanceStore((s) => s.setNewLinkSubtitle);
  const addMenuItem = useAppearanceStore((s) => s.addMenuItem);
  const moveMenuItem = useAppearanceStore((s) => s.moveMenuItem);
  const indentMenuItem = useAppearanceStore((s) => s.indentMenuItem);
  const outdentMenuItem = useAppearanceStore((s) => s.outdentMenuItem);
  const duplicateMenuItem = useAppearanceStore((s) => s.duplicateMenuItem);
  const updateMenuItem = useAppearanceStore((s) => s.updateMenuItem);
  const removeMenuItem = useAppearanceStore((s) => s.removeMenuItem);
  const loadAppearance = useAppearanceStore((s) => s.loadAppearance);
  const saveStyles = useAppearanceStore((s) => s.saveStyles);
  const saveMenus = useAppearanceStore((s) => s.saveMenus);

  useEffect(() => {
    void loadAppearance(initialTab);
  }, [initialTab, loadAppearance]);

  if (loading) {
    return <p role="status" className="text-xs text-slate-400">Chargement de l’apparence…</p>;
  }

  const activeItems = menus[activeLocation] || [];

  return (
    <div className="max-w-5xl space-y-6">
      <div>
        <h2 className="text-xl font-bold text-white">{appearanceMessages.title}</h2>
        <p className="text-xs text-slate-400 mt-1">{appearanceMessages.subtitle}</p>
      </div>

      {/* Onglets : Styles globaux (WP-48/WP-50) vs Menus publics (WP-46) */}
      <div className="flex flex-wrap items-center gap-2 border-b border-white/10">
        <button
          type="button"
          onClick={() => setActiveTab('styles')}
          className={`flex items-center gap-2 px-4 py-2.5 text-xs font-semibold border-b-2 transition-all ${
            activeTab === 'styles'
              ? 'border-indigo-500 text-indigo-400 bg-white/[0.02]'
              : 'border-transparent text-slate-400 hover:text-slate-200'
          }`}
        >
          <Palette className="w-4 h-4" />
          <span>{appearanceMessages.tabStyles}</span>
        </button>

        <button
          type="button"
          onClick={() => setActiveTab('menus')}
          className={`flex items-center gap-2 px-4 py-2.5 text-xs font-semibold border-b-2 transition-all ${
            activeTab === 'menus'
              ? 'border-emerald-500 text-emerald-400 bg-white/[0.02]'
              : 'border-transparent text-slate-400 hover:text-slate-200'
          }`}
        >
          <Navigation className="w-4 h-4" />
          <span>{appearanceMessages.tabMenus}</span>
        </button>
      </div>

      {error && (
        <p
          role="alert"
          className="rounded-xl border border-red-500/30 bg-red-500/10 px-4 py-3 text-xs text-red-200"
        >
          {error}
        </p>
      )}
      {message && (
        <p
          role="status"
          className="rounded-xl border border-emerald-500/30 bg-emerald-500/10 px-4 py-3 text-xs text-emerald-200"
        >
          {message}
        </p>
      )}

      {activeTab === 'styles' ? (
        <form
          onSubmit={(e) => {
            e.preventDefault();
            void saveStyles();
          }}
          className="grid grid-cols-1 lg:grid-cols-12 gap-6 items-start"
        >
          <div className="lg:col-span-7 bg-[#0f1422] border border-white/10 rounded-2xl p-6 space-y-6 text-xs">
            {/* Palettes de couleurs harmonisées en 1 clic (Astra / GP) */}
            <div className="space-y-2">
              <span className="font-semibold text-slate-200 block">Palettes harmonisées (1 clic) :</span>
              <div className="grid grid-cols-2 sm:grid-cols-3 gap-2">
                {COLOR_PRESETS.map((preset) => (
                  <button
                    key={preset.name}
                    type="button"
                    onClick={() => {
                      setToken('primary_color', preset.primary);
                      setToken('accent_color', preset.accent);
                    }}
                    className="flex items-center gap-2 rounded-xl border border-white/10 bg-black/30 p-2 text-left hover:border-indigo-400 transition-colors"
                  >
                    <div className="flex items-center -space-x-1 shrink-0">
                      <span className="w-4 h-4 rounded-full border border-black/40" style={{ background: preset.primary }} />
                      <span className="w-4 h-4 rounded-full border border-black/40" style={{ background: preset.accent }} />
                    </div>
                    <span className="truncate text-[11px] font-medium text-slate-300">{preset.name}</span>
                  </button>
                ))}
              </div>
            </div>

            <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 pt-2 border-t border-white/5">
              <label className="flex items-center justify-between gap-4 rounded-xl bg-black/25 border border-white/5 p-3">
                <span className="font-medium text-slate-200">{appearanceMessages.primaryColor}</span>
                <input
                  aria-label={appearanceMessages.primaryColor}
                  type="color"
                  value={/^#[0-9a-f]{6}$/i.test(tokens.primary_color) ? tokens.primary_color : '#4f46e5'}
                  onChange={(e) => setToken('primary_color', e.target.value)}
                  className="w-16 h-8 bg-transparent cursor-pointer rounded"
                />
              </label>

              <label className="flex items-center justify-between gap-4 rounded-xl bg-black/25 border border-white/5 p-3">
                <span className="font-medium text-slate-200">{appearanceMessages.accentColor}</span>
                <input
                  aria-label={appearanceMessages.accentColor}
                  type="color"
                  value={/^#[0-9a-f]{6}$/i.test(tokens.accent_color) ? tokens.accent_color : '#06b6d4'}
                  onChange={(e) => setToken('accent_color', e.target.value)}
                  className="w-16 h-8 bg-transparent cursor-pointer rounded"
                />
              </label>
            </div>

            {/* Typographie fluide & polices souveraines (TT25 / Astra) */}
            <div className="space-y-3 pt-2 border-t border-white/5">
              <div className="flex items-center justify-between">
                <span className="font-semibold text-slate-200">Typographie souveraine (RGPD local)</span>
                <span className="rounded bg-indigo-500/20 text-indigo-300 px-2 py-0.5 text-[10px] font-mono">
                  100% sans Google Fonts
                </span>
              </div>
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <label className="block space-y-1.5">
                  <span className="text-slate-300">Police des Titres :</span>
                  <select
                    value={tokens.font_heading || HEADING_FONTS[0].value}
                    onChange={(e) => setToken('font_heading', e.target.value)}
                    className="w-full rounded-xl bg-slate-950 border border-white/10 px-3 py-2 text-xs text-white"
                  >
                    {HEADING_FONTS.map((font) => (
                      <option key={font.name} value={font.value}>
                        {font.name}
                      </option>
                    ))}
                  </select>
                </label>
                <label className="block space-y-1.5">
                  <span className="text-slate-300">Police du Corps :</span>
                  <select
                    value={tokens.font_body || BODY_FONTS[0].value}
                    onChange={(e) => setToken('font_body', e.target.value)}
                    className="w-full rounded-xl bg-slate-950 border border-white/10 px-3 py-2 text-xs text-white"
                  >
                    {BODY_FONTS.map((font) => (
                      <option key={font.name} value={font.value}>
                        {font.name}
                      </option>
                    ))}
                  </select>
                </label>
              </div>
            </div>

            <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 pt-2 border-t border-white/5">
              <label className="block space-y-1.5">
                <span className="font-medium text-slate-200">
                  {appearanceMessages.containerWidth} : {tokens.container_width}
                </span>
                <select
                  value={tokens.container_width}
                  onChange={(e) => setToken('container_width', e.target.value)}
                  className="w-full rounded-xl bg-slate-950 border border-white/10 px-3 py-2 text-xs text-white"
                >
                  <option value="960px">Compact (960px — lecture longue)</option>
                  <option value="1100px">Standard (1100px — défaut rf-base)</option>
                  <option value="1280px">Large (1280px — portail éditorial)</option>
                  <option value="1440px">Très large (1440px)</option>
                </select>
              </label>

              <label className="block space-y-1.5">
                <span className="font-medium text-slate-200">
                  {appearanceMessages.borderRadius} : {tokens.radius}
                </span>
                <input
                  aria-label={appearanceMessages.borderRadius}
                  type="range"
                  min={0}
                  max={28}
                  value={Number.parseInt(tokens.radius) || 0}
                  onChange={(e) => setToken('radius', `${e.target.value}px`)}
                  className="block w-full mt-2"
                />
              </label>
            </div>

            {/* CSS personnalisé borné (WP-50) */}
            <label className="block space-y-1.5 pt-2 border-t border-white/5">
              <span className="font-medium text-slate-200">
                {appearanceMessages.customCssLabel}
              </span>
              <textarea
                rows={4}
                maxLength={16384}
                value={tokens.custom_css}
                onChange={(e) => setToken('custom_css', e.target.value)}
                placeholder=".glass-nav { border-bottom-width: 2px; }"
                className="w-full rounded-xl bg-slate-950 border border-white/10 p-3 font-mono text-xs text-slate-100"
              />
            </label>

            <button
              type="submit"
              disabled={saving}
              className="inline-flex items-center gap-2 rounded-xl bg-indigo-600 hover:bg-indigo-500 px-5 py-2.5 text-xs font-semibold text-white disabled:opacity-40 shadow-lg shadow-indigo-600/20"
            >
              <Save className="w-4 h-4" />
              <span>
                {saving ? appearanceMessages.savingStyles : appearanceMessages.saveStyles}
              </span>
            </button>
          </div>

          {/* Aperçu immédiat et interactif des composants du thème */}
          <div
            className="lg:col-span-5 rounded-2xl p-6 border bg-[#0b0f19] space-y-5 shadow-2xl transition-all"
            style={{
              borderRadius: tokens.radius,
              borderColor: tokens.primary_color,
            }}
          >
            <div className="flex items-center justify-between border-b border-white/10 pb-3">
              <span className="font-bold text-white text-xs uppercase tracking-wider">Aperçu Thème en direct</span>
              <span
                className="w-2.5 h-2.5 rounded-full animate-pulse"
                style={{ background: tokens.accent_color }}
              />
            </div>

            {/* Titre et texte avec les polices sélectionnées */}
            <div className="space-y-1.5">
              <h3
                className="text-lg font-bold text-white leading-tight"
                style={{ fontFamily: tokens.font_heading || undefined }}
              >
                Typographie fluide & Harmonisée
              </h3>
              <p
                className="text-xs text-slate-300 leading-relaxed"
                style={{ fontFamily: tokens.font_body || undefined }}
              >
                Rendu textuel en variables CSS natives. Les titres s’adaptent automatiquement à la largeur du viewport.
              </p>
            </div>

            {/* Boutons d'action (Primary, Secondary, Outline) */}
            <div className="space-y-2 pt-2 border-t border-white/5">
              <span className="text-[11px] font-medium text-slate-400 block">Variantes de boutons :</span>
              <div className="flex flex-wrap items-center gap-2">
                <span
                  className="px-3 py-1.5 text-xs font-semibold text-white shadow-sm"
                  style={{
                    background: tokens.primary_color,
                    borderRadius: tokens.radius,
                  }}
                >
                  Principal
                </span>
                <span
                  className="px-3 py-1.5 text-xs font-semibold text-slate-200 border border-white/20 bg-white/5"
                  style={{ borderRadius: tokens.radius }}
                >
                  Secondaire
                </span>
                <span
                  className="px-3 py-1.5 text-xs font-semibold"
                  style={{
                    color: tokens.primary_color,
                    border: `1.5px solid ${tokens.primary_color}`,
                    borderRadius: tokens.radius,
                  }}
                >
                  Contour
                </span>
              </div>
            </div>

            {/* Carte / Groupe conteneur (WP-B036) */}
            <div
              className="p-3.5 bg-white/[0.03] border border-white/10 space-y-1"
              style={{ borderRadius: tokens.radius }}
            >
              <div className="flex items-center justify-between text-[11px]">
                <span className="font-semibold text-white">Carte conteneur</span>
                <span className="text-[10px]" style={{ color: tokens.accent_color }}>
                  Rayon : {tokens.radius}
                </span>
              </div>
              <p className="text-[11px] text-slate-400">
                Structure avec ombre douce et fond contrasté adaptatif.
              </p>
            </div>
          </div>
        </form>
      ) : (
        /* Onglet Menus publics & Emplacements hiérarchiques (WP-46 / NAV-B / QM-01 à QM-03) */
        <div className="grid grid-cols-1 lg:grid-cols-12 gap-6 items-start">
          {/* Colonne gauche : Sources d'ajout rapide (Pages, Catégories, Lien libre avec options QuadMenu) */}
          <div className="lg:col-span-5 space-y-4">
            <div className="rounded-2xl bg-[#0f1422] border border-white/10 p-5 space-y-4 text-xs">
              <h3 className="font-bold text-white">{appearanceMessages.addMenuItemTitle}</h3>

              {/* Formulaire de lien personnalisé avec badge et sous-titre */}
              <div className="space-y-3">
                <label className="block space-y-1">
                  <span className="text-slate-400">{appearanceMessages.customLinkLabel}</span>
                  <input
                    type="text"
                    value={newLinkLabel}
                    onChange={(e) => setNewLinkLabel(e.target.value)}
                    placeholder="Ex: Accueil, Services, Contact…"
                    className="w-full rounded-xl bg-slate-950 border border-white/10 px-3 py-2 text-xs text-white"
                  />
                </label>

                <label className="block space-y-1">
                  <span className="text-slate-400">{appearanceMessages.customLinkUrl}</span>
                  <input
                    type="text"
                    value={newLinkUrl}
                    onChange={(e) => setNewLinkUrl(e.target.value)}
                    placeholder="/fr/services ou https://…"
                    className="w-full rounded-xl bg-slate-950 border border-white/10 px-3 py-2 font-mono text-xs text-white"
                  />
                </label>

                <div className="grid grid-cols-2 gap-2">
                  <label className="block space-y-1">
                    <span className="text-slate-400">{appearanceMessages.badgeLabel}</span>
                    <input
                      type="text"
                      maxLength={20}
                      value={newLinkBadge}
                      onChange={(e) => setNewLinkBadge(e.target.value)}
                      placeholder="Ex: Pro, New"
                      className="w-full rounded-xl bg-slate-950 border border-white/10 px-3 py-2 text-xs text-white"
                    />
                  </label>

                  <label className="block space-y-1">
                    <span className="text-slate-400">{appearanceMessages.customLinkTarget}</span>
                    <select
                      value={newLinkTarget}
                      onChange={(e) => setNewLinkTarget(e.target.value as '_self' | '_blank')}
                      className="w-full rounded-xl bg-slate-950 border border-white/10 px-3 py-2 text-xs text-white"
                    >
                      <option value="_self">{appearanceMessages.targetSelf}</option>
                      <option value="_blank">{appearanceMessages.targetBlank}</option>
                    </select>
                  </label>
                </div>

                <label className="block space-y-1">
                  <span className="text-slate-400">{appearanceMessages.subtitleLabel}</span>
                  <input
                    type="text"
                    maxLength={80}
                    value={newLinkSubtitle}
                    onChange={(e) => setNewLinkSubtitle(e.target.value)}
                    placeholder="Courte description ou phrase d'appel"
                    className="w-full rounded-xl bg-slate-950 border border-white/10 px-3 py-2 text-xs text-white"
                  />
                </label>

                <button
                  type="button"
                  onClick={() => addMenuItem()}
                  className="inline-flex items-center gap-1.5 rounded-xl bg-indigo-600 hover:bg-indigo-500 px-3.5 py-2 text-xs font-semibold text-white"
                >
                  <Plus className="w-3.5 h-3.5" />
                  <span>{appearanceMessages.addLinkButton}</span>
                </button>
              </div>

              {/* Ajout en 1 clic depuis les pages publiées */}
              {publishedPages.length > 0 && (
                <div className="pt-3 border-t border-white/5 space-y-2">
                  <div className="font-semibold text-slate-300">
                    {appearanceMessages.quickSourcePages}
                  </div>
                  <div className="flex flex-wrap gap-1.5">
                    {publishedPages.map((p) => (
                      <button
                        key={p.id}
                        type="button"
                        onClick={() => addMenuItem(p.title, `/fr/page/${p.slug}`, '_self', { type: 'page', object_id: p.id })}
                        className="rounded-lg border border-white/10 bg-white/5 px-2.5 py-1 text-[11px] text-slate-200 hover:border-indigo-400"
                      >
                        + {p.title}
                      </button>
                    ))}
                  </div>
                </div>
              )}

              {/* Ajout en 1 clic depuis les catégories */}
              {categories.length > 0 && (
                <div className="pt-3 border-t border-white/5 space-y-2">
                  <div className="font-semibold text-slate-300">
                    {appearanceMessages.quickSourceCategories}
                  </div>
                  <div className="flex flex-wrap gap-1.5">
                    {categories.map((c) => (
                      <button
                        key={c.id}
                        type="button"
                        onClick={() => addMenuItem(c.name, `/fr?term=${c.slug}`, '_self', { type: 'category', object_id: c.id })}
                        className="rounded-lg border border-white/10 bg-white/5 px-2.5 py-1 text-[11px] text-slate-200 hover:border-emerald-400"
                      >
                        + {c.name}
                      </button>
                    ))}
                  </div>
                </div>
              )}
            </div>
          </div>

          {/* Colonne droite : Arbre hiérarchique du menu de l'emplacement actif */}
          <div className="lg:col-span-7 rounded-2xl bg-[#0f1422] border border-white/10 p-5 space-y-4 text-xs">
            <div className="flex flex-wrap items-center justify-between gap-2">
              <div className="flex items-center gap-2">
                <button
                  type="button"
                  onClick={() => setActiveLocation('header')}
                  className={`px-3 py-1.5 rounded-xl font-semibold transition-colors ${
                    activeLocation === 'header'
                      ? 'bg-indigo-600 text-white'
                      : 'bg-white/5 text-slate-300 hover:bg-white/10'
                  }`}
                >
                  {appearanceMessages.locationHeader} ({menus.header.length})
                </button>
                <button
                  type="button"
                  onClick={() => setActiveLocation('footer')}
                  className={`px-3 py-1.5 rounded-xl font-semibold transition-colors ${
                    activeLocation === 'footer'
                      ? 'bg-indigo-600 text-white'
                      : 'bg-white/5 text-slate-300 hover:bg-white/10'
                  }`}
                >
                  {appearanceMessages.locationFooter} ({menus.footer.length})
                </button>
              </div>
            </div>

            {activeItems.length === 0 ? (
              <p className="py-8 text-center text-slate-500">
                {appearanceMessages.emptyMenuLocation}
              </p>
            ) : (
              <ul className="space-y-2.5">
                {activeItems.map((item, idx) => {
                  const depth = getItemDepth(item.id, activeItems);
                  const canIndent =
                    idx > 0 && getItemDepth(activeItems[idx - 1].id, activeItems) < 3;
                  const canOutdent = Boolean(item.parent_id);
                  const isEditing = editingId === item.id;

                  const indentClass =
                    depth === 1
                      ? 'ml-0'
                      : depth === 2
                        ? 'ml-5 pl-3 border-l-2 border-indigo-500/60 bg-indigo-950/20'
                        : 'ml-10 pl-3 border-l-2 border-emerald-500/60 bg-emerald-950/20';

                  return (
                    <li
                      key={item.id}
                      className={`rounded-xl border border-white/5 transition-all p-3 ${indentClass} ${
                        isEditing ? 'bg-indigo-950/30 border-indigo-500/30' : 'bg-black/30'
                      }`}
                    >
                      <div className="flex items-center justify-between gap-3">
                        <div className="min-w-0 space-y-0.5">
                          <div className="flex items-center gap-2 flex-wrap">
                            {depth > 1 && (
                              <span className="px-1.5 py-0.5 rounded text-[10px] font-semibold bg-white/10 text-slate-300">
                                {appearanceMessages.levelPrefix} {depth}
                              </span>
                            )}
                            <span className="font-semibold text-slate-100">{item.label}</span>
                            {item.badge && (
                              <span className="px-1.5 py-0.5 rounded text-[10px] font-bold bg-cyan-500/20 text-cyan-300">
                                {item.badge}
                              </span>
                            )}
                            {item.type && item.type !== 'custom' && (
                              <span className="px-1.5 py-0.5 rounded text-[10px] font-medium bg-indigo-500/20 text-indigo-300">
                                {item.type === 'page' ? 'Page' : item.type === 'category' ? 'Catégorie' : item.type === 'post' ? 'Article' : 'Tag'} dynamique
                              </span>
                            )}
                          </div>
                          <div className="font-mono text-[11px] text-slate-400 truncate">
                            {item.url}
                            {item.target === '_blank' ? ' · (nouvel onglet)' : ''}
                            {item.subtitle ? ` · ${item.subtitle}` : ''}
                          </div>
                        </div>

                        {/* Barre d'outils d'actions hiérarchiques */}
                        <div className="flex items-center gap-1 shrink-0">
                          <button
                            type="button"
                            disabled={!canOutdent}
                            onClick={() => outdentMenuItem(item.id)}
                            className="p-1.5 rounded-lg hover:bg-white/10 disabled:opacity-20"
                            title={appearanceMessages.outdentItem}
                          >
                            <ArrowLeft className="w-3.5 h-3.5" />
                          </button>
                          <button
                            type="button"
                            disabled={!canIndent}
                            onClick={() => indentMenuItem(item.id)}
                            className="p-1.5 rounded-lg hover:bg-white/10 disabled:opacity-20"
                            title={appearanceMessages.indentItem}
                          >
                            <ArrowRight className="w-3.5 h-3.5" />
                          </button>
                          <button
                            type="button"
                            disabled={idx === 0}
                            onClick={() => moveMenuItem(item.id, -1)}
                            className="p-1.5 rounded-lg hover:bg-white/10 disabled:opacity-30"
                            title="Monter"
                          >
                            <ArrowUp className="w-3.5 h-3.5" />
                          </button>
                          <button
                            type="button"
                            disabled={idx === activeItems.length - 1}
                            onClick={() => moveMenuItem(item.id, 1)}
                            className="p-1.5 rounded-lg hover:bg-white/10 disabled:opacity-30"
                            title="Descendre"
                          >
                            <ArrowDown className="w-3.5 h-3.5" />
                          </button>
                          <button
                            type="button"
                            onClick={() => duplicateMenuItem(item.id)}
                            className="p-1.5 rounded-lg hover:bg-white/10 text-slate-300"
                            title={appearanceMessages.duplicateItem}
                          >
                            <Copy className="w-3.5 h-3.5" />
                          </button>
                          <button
                            type="button"
                            onClick={() => setEditingId(isEditing ? null : item.id)}
                            className={`p-1.5 rounded-lg transition-colors ${
                              isEditing ? 'bg-indigo-600 text-white' : 'hover:bg-white/10 text-slate-300'
                            }`}
                            title={isEditing ? appearanceMessages.closeEditItem : appearanceMessages.editItem}
                          >
                            <Edit3 className="w-3.5 h-3.5" />
                          </button>
                          <button
                            type="button"
                            onClick={() => removeMenuItem(item.id)}
                            className="p-1.5 rounded-lg text-red-300 hover:bg-red-500/10"
                            title="Retirer du menu"
                          >
                            <Trash2 className="w-3.5 h-3.5" />
                          </button>
                        </div>
                      </div>

                      {/* Panneau d'édition inline dépliable (QM-01 à QM-03) */}
                      {isEditing && (
                        <div className="mt-3 pt-3 border-t border-white/10 grid grid-cols-1 md:grid-cols-2 gap-3 text-xs">
                          <label className="block space-y-1">
                            <span className="text-slate-400">{appearanceMessages.customLinkLabel}</span>
                            <input
                              type="text"
                              value={item.label}
                              onChange={(e) => updateMenuItem(item.id, { label: e.target.value })}
                              className="w-full rounded-lg bg-slate-950 border border-white/10 px-2.5 py-1.5 text-xs text-white"
                            />
                          </label>
                          <label className="block space-y-1">
                            <span className="text-slate-400">{appearanceMessages.customLinkUrl}</span>
                            <input
                              type="text"
                              value={item.url}
                              onChange={(e) => updateMenuItem(item.id, { url: e.target.value })}
                              className="w-full rounded-lg bg-slate-950 border border-white/10 px-2.5 py-1.5 font-mono text-xs text-white"
                            />
                          </label>
                          <label className="block space-y-1">
                            <span className="text-slate-400">{appearanceMessages.badgeLabel}</span>
                            <input
                              type="text"
                              maxLength={20}
                              value={item.badge ?? ''}
                              onChange={(e) => updateMenuItem(item.id, { badge: e.target.value })}
                              placeholder="Ex: Pro, New"
                              className="w-full rounded-lg bg-slate-950 border border-white/10 px-2.5 py-1.5 text-xs text-white"
                            />
                          </label>
                          <label className="block space-y-1">
                            <span className="text-slate-400">{appearanceMessages.customLinkTarget}</span>
                            <select
                              value={item.target ?? '_self'}
                              onChange={(e) =>
                                updateMenuItem(item.id, {
                                  target: e.target.value as '_self' | '_blank',
                                })
                              }
                              className="w-full rounded-lg bg-slate-950 border border-white/10 px-2.5 py-1.5 text-xs text-white"
                            >
                              <option value="_self">{appearanceMessages.targetSelf}</option>
                              <option value="_blank">{appearanceMessages.targetBlank}</option>
                            </select>
                          </label>
                          <label className="md:col-span-2 block space-y-1">
                            <span className="text-slate-400">{appearanceMessages.subtitleLabel}</span>
                            <input
                              type="text"
                              maxLength={80}
                              value={item.subtitle ?? ''}
                              onChange={(e) => updateMenuItem(item.id, { subtitle: e.target.value })}
                              placeholder="Courte description ou phrase d'appel"
                              className="w-full rounded-lg bg-slate-950 border border-white/10 px-2.5 py-1.5 text-xs text-white"
                            />
                          </label>
                          <label className="md:col-span-2 block space-y-1">
                            <span className="text-slate-400">Classes CSS personnalisées</span>
                            <input
                              type="text"
                              maxLength={100}
                              value={item.classes ?? ''}
                              onChange={(e) => updateMenuItem(item.id, { classes: e.target.value })}
                              placeholder="Ex: nav-btn highlight menu-cta"
                              className="w-full rounded-lg bg-slate-950 border border-white/10 px-2.5 py-1.5 font-mono text-xs text-white"
                            />
                          </label>
                        </div>
                      )}
                    </li>
                  );
                })}
              </ul>
            )}

            <div className="pt-2 border-t border-white/5 flex justify-end">
              <button
                type="button"
                disabled={saving}
                onClick={() => void saveMenus()}
                className="inline-flex items-center gap-2 rounded-xl bg-emerald-600 hover:bg-emerald-500 px-5 py-2.5 text-xs font-semibold text-white disabled:opacity-40"
              >
                <Save className="w-4 h-4" />
                <span>{appearanceMessages.saveMenus}</span>
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
