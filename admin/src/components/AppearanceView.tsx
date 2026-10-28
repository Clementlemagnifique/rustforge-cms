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

import React, { useEffect } from 'react';
import { Palette, Navigation, Plus, Trash2, ArrowUp, ArrowDown, Save } from 'lucide-react';
import { appearanceMessages } from '../i18n/messages.fr';
import { useAppearanceStore } from '../stores/useAppearanceStore';
import type { AppearanceTab } from '../stores/useAppearanceStore';

interface AppearanceViewProps {
  initialTab?: AppearanceTab;
}

export const AppearanceView: React.FC<AppearanceViewProps> = ({ initialTab }) => {
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
  const publishedPages = useAppearanceStore((s) => s.publishedPages);
  const categories = useAppearanceStore((s) => s.categories);

  const setActiveTab = useAppearanceStore((s) => s.setActiveTab);
  const setToken = useAppearanceStore((s) => s.setToken);
  const setActiveLocation = useAppearanceStore((s) => s.setActiveLocation);
  const setNewLinkLabel = useAppearanceStore((s) => s.setNewLinkLabel);
  const setNewLinkUrl = useAppearanceStore((s) => s.setNewLinkUrl);
  const setNewLinkTarget = useAppearanceStore((s) => s.setNewLinkTarget);
  const addMenuItem = useAppearanceStore((s) => s.addMenuItem);
  const moveMenuItem = useAppearanceStore((s) => s.moveMenuItem);
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
          <div className="lg:col-span-7 bg-[#0f1422] border border-white/10 rounded-2xl p-6 space-y-5 text-xs">
            <label className="flex items-center justify-between gap-4">
              <span className="font-medium text-slate-200">{appearanceMessages.primaryColor}</span>
              <input
                aria-label={appearanceMessages.primaryColor}
                type="color"
                value={/^#[0-9a-f]{6}$/i.test(tokens.primary_color) ? tokens.primary_color : '#4f46e5'}
                onChange={(e) => setToken('primary_color', e.target.value)}
                className="w-20 h-9 bg-transparent cursor-pointer"
              />
            </label>

            <label className="flex items-center justify-between gap-4">
              <span className="font-medium text-slate-200">{appearanceMessages.accentColor}</span>
              <input
                aria-label={appearanceMessages.accentColor}
                type="color"
                value={/^#[0-9a-f]{6}$/i.test(tokens.accent_color) ? tokens.accent_color : '#06b6d4'}
                onChange={(e) => setToken('accent_color', e.target.value)}
                className="w-20 h-9 bg-transparent cursor-pointer"
              />
            </label>

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
                className="block w-full"
              />
            </label>

            {/* CSS personnalisé borné (WP-50) */}
            <label className="block space-y-1.5 pt-2 border-t border-white/5">
              <span className="font-medium text-slate-200">
                {appearanceMessages.customCssLabel}
              </span>
              <textarea
                rows={5}
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
              className="inline-flex items-center gap-2 rounded-xl bg-indigo-600 hover:bg-indigo-500 px-5 py-2.5 text-xs font-semibold text-white disabled:opacity-40"
            >
              <Save className="w-4 h-4" />
              <span>
                {saving ? appearanceMessages.savingStyles : appearanceMessages.saveStyles}
              </span>
            </button>
          </div>

          {/* Aperçu immédiat de la palette */}
          <div
            className="lg:col-span-5 rounded-2xl p-6 border bg-[#0b0f19] space-y-4"
            style={{
              borderRadius: tokens.radius,
              borderColor: tokens.primary_color,
            }}
          >
            <h3 className="font-bold text-white text-base">Aperçu de la palette `rf-base`</h3>
            <p className="text-xs text-slate-400 leading-relaxed">
              Un titre, un lien d’accentuation et un bouton d’action utilisant vos tokens CSS
              publics (`--tp-primary`, `--tp-accent`, `--tp-radius`, `--tp-container`).
            </p>
            <div className="flex flex-wrap items-center gap-4 pt-2">
              <span className="text-xs font-semibold" style={{ color: tokens.accent_color }}>
                Découvrir mes contenus →
              </span>
              <span
                className="inline-block px-4 py-2 text-xs font-semibold text-white"
                style={{
                  background: tokens.primary_color,
                  borderRadius: tokens.radius,
                }}
              >
                En savoir plus
              </span>
            </div>
          </div>
        </form>
      ) : (
        /* Onglet Menus publics & Emplacements (WP-46) */
        <div className="grid grid-cols-1 lg:grid-cols-12 gap-6 items-start">
          {/* Colonne gauche : Sources d'ajout rapide (Pages, Catégories, Lien libre) */}
          <div className="lg:col-span-5 space-y-4">
            <div className="rounded-2xl bg-[#0f1422] border border-white/10 p-5 space-y-4 text-xs">
              <h3 className="font-bold text-white">{appearanceMessages.addMenuItemTitle}</h3>

              {/* Formulaire de lien personnalisé */}
              <div className="space-y-3">
                <label className="block space-y-1">
                  <span className="text-slate-400">{appearanceMessages.customLinkLabel}</span>
                  <input
                    type="text"
                    value={newLinkLabel}
                    onChange={(e) => setNewLinkLabel(e.target.value)}
                    placeholder="Ex: À propos, Contact, Boutique…"
                    className="w-full rounded-xl bg-slate-950 border border-white/10 px-3 py-2 text-xs text-white"
                  />
                </label>
                <label className="block space-y-1">
                  <span className="text-slate-400">{appearanceMessages.customLinkUrl}</span>
                  <input
                    type="text"
                    value={newLinkUrl}
                    onChange={(e) => setNewLinkUrl(e.target.value)}
                    placeholder="/fr/page/a-propos ou https://…"
                    className="w-full rounded-xl bg-slate-950 border border-white/10 px-3 py-2 font-mono text-xs text-white"
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
                        onClick={() => addMenuItem(p.title, `/fr/page/${p.slug}`, '_self')}
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
                        onClick={() => addMenuItem(c.name, `/fr?term=${c.slug}`, '_self')}
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

          {/* Colonne droite : Arbre du menu de l'emplacement actif */}
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
                Aucun lien configuré pour cet emplacement. Le thème utilise les liens par défaut.
              </p>
            ) : (
              <ul className="space-y-2">
                {activeItems.map((item, idx) => (
                  <li
                    key={item.id}
                    className="flex items-center justify-between gap-3 rounded-xl bg-black/30 border border-white/5 px-3.5 py-2.5"
                  >
                    <div className="min-w-0">
                      <div className="font-semibold text-slate-100">{item.label}</div>
                      <div className="font-mono text-[11px] text-slate-400 truncate">
                        {item.url}{' '}
                        {item.target === '_blank' ? '· (nouvel onglet)' : ''}
                      </div>
                    </div>
                    <div className="flex items-center gap-1 shrink-0">
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
                        onClick={() => removeMenuItem(item.id)}
                        className="p-1.5 rounded-lg text-red-300 hover:bg-red-500/10"
                        title="Retirer du menu"
                      >
                        <Trash2 className="w-3.5 h-3.5" />
                      </button>
                    </div>
                  </li>
                ))}
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
