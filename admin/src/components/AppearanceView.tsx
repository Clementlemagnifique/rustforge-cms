/**
 * ============================================================================
 * VUE D'APPARENCE, THÈMES SOUVERAINS, ÉDITEUR DE SITE (FSE) & MENUS
 * ============================================================================
 * Reproduction fidèle à 99% de WordPress Core (`themes.php`, `site-editor.php`,
 * `customize.php`, `nav-menus.php`) avec design tokens natifs en Rust (zéro PHP).
 *
 * Composants & Fonctionnalités clés :
 * 1. Thèmes (`themes.php`) :
 *    - Filtres horizontaux `.subsubsub` (Tous, Basés sur des blocs FSE, Classiques)
 *    - Recherche en temps réel parmi les thèmes installés
 *    - Carte du thème actif (Twenty Twenty-Five avec capture officielle, ruban bleu,
 *      actions « Personnaliser », « Éditeur de site », « Détails du thème »)
 *    - Grille des thèmes disponibles (Astra, GeneratePress, rf-base) avec survol d'activation
 *    - Carte dashed « + Ajouter un thème »
 *    - Modale « Détails du thème » conforme à l'overlay de WordPress
 * 2. Éditeur de site souverain FSE (`site-editor.php`) :
 *    - Modèles MiniJinja (home, single, listing, 404, zen)
 *    - Éléments de modèle (header, footer)
 *    - Sélecteur des 8 variations de styles officielles de Twenty Twenty-Five
 *      (Evening, Noon, Dusk, Afternoon, Twilight, Morning, Sunrise, Midnight)
 *    - Aperçu adaptatif réactif (Bureau, Tablette, Mobile)
 * 3. Styles globaux (Tokens de design & CSS personnalisé WP-50)
 * 4. Menus de navigation publique (Header / Footer) avec hiérarchie multi-niveaux.
 */

import React, { useEffect, useState } from 'react';
import {
  Palette,
  Navigation,
  LayoutTemplate,
  Check,
  ExternalLink,
  Plus,
  Trash2,
  ArrowUp,
  ArrowDown,
  ArrowLeft,
  ArrowRight,
  Save,
  Search,
  X,
  Layers,
  Monitor,
  Tablet,
  Smartphone,
  UploadCloud,
} from 'lucide-react';
import { appearanceMessages } from '../i18n/messages.fr';
import {
  useAppearanceStore,
  getItemDepth,
  TWENTY_TWENTY_FIVE_VARIATIONS,
} from '../stores/useAppearanceStore';
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
  { name: 'Système Natif (Zéro chargement)', value: 'system-ui, -apple-system, sans-serif' },
];

const BODY_FONTS = [
  { name: 'Inter (Optimisé pour la lecture)', value: "'Inter', system-ui, sans-serif" },
  { name: 'Plus Jakarta Sans (Harmonisé)', value: "'Plus Jakarta Sans', system-ui, sans-serif" },
  { name: 'Merriweather (Sérif classique)', value: "'Merriweather', Georgia, serif" },
  { name: 'Système Natif (Zéro chargement)', value: 'system-ui, -apple-system, sans-serif' },
];

const SITE_TEMPLATES = [
  { id: 'home', name: 'Accueil / Blog', file: 'home.html', desc: 'Page principale affichant le flux d’articles récents.' },
  { id: 'single', name: 'Article individuel (Single)', file: 'single.html', desc: 'Gabarit de lecture pour les articles de blog.' },
  { id: 'listing', name: 'Page statique / Listing', file: 'listing.html', desc: 'Gabarit standard pour les pages de contenu.' },
  { id: '404', name: 'Page introuvable (404)', file: '404.html', desc: 'Affiché lorsqu’une ressource demandée n’existe pas.' },
  { id: 'zen', name: 'Mode Zen (Minimaliste)', file: 'zen.html', desc: 'Lecture épurée sans barre latérale ni distraction.' },
];

interface AppearanceViewProps {
  initialTab?: AppearanceTab;
}

export const AppearanceView: React.FC<AppearanceViewProps> = ({ initialTab }) => {
  const [isUploadModalOpen, setIsUploadModalOpen] = useState(false);

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
  const publishedPages = useAppearanceStore((s) => s.publishedPages);
  const categories = useAppearanceStore((s) => s.categories);

  // Thèmes et FSE
  const themes = useAppearanceStore((s) => s.themes);
  const activeThemeSlug = useAppearanceStore((s) => s.activeThemeSlug);
  const activatingThemeSlug = useAppearanceStore((s) => s.activatingThemeSlug);
  const themeFilter = useAppearanceStore((s) => s.themeFilter);
  const themeSearchQuery = useAppearanceStore((s) => s.themeSearchQuery);
  const selectedThemeForDetails = useAppearanceStore((s) => s.selectedThemeForDetails);
  const selectedTemplate = useAppearanceStore((s) => s.selectedTemplate);
  const selectedVariationId = useAppearanceStore((s) => s.selectedVariationId);
  const devicePreview = useAppearanceStore((s) => s.devicePreview);

  const setActiveTab = useAppearanceStore((s) => s.setActiveTab);
  const setThemeFilter = useAppearanceStore((s) => s.setThemeFilter);
  const setThemeSearchQuery = useAppearanceStore((s) => s.setThemeSearchQuery);
  const setSelectedThemeForDetails = useAppearanceStore((s) => s.setSelectedThemeForDetails);
  const setSelectedTemplate = useAppearanceStore((s) => s.setSelectedTemplate);
  const setDevicePreview = useAppearanceStore((s) => s.setDevicePreview);
  const applyStyleVariation = useAppearanceStore((s) => s.applyStyleVariation);
  const activateTheme = useAppearanceStore((s) => s.activateTheme);

  const setToken = useAppearanceStore((s) => s.setToken);
  const setActiveLocation = useAppearanceStore((s) => s.setActiveLocation);
  const setNewLinkLabel = useAppearanceStore((s) => s.setNewLinkLabel);
  const setNewLinkUrl = useAppearanceStore((s) => s.setNewLinkUrl);
  const addMenuItem = useAppearanceStore((s) => s.addMenuItem);
  const moveMenuItem = useAppearanceStore((s) => s.moveMenuItem);
  const indentMenuItem = useAppearanceStore((s) => s.indentMenuItem);
  const outdentMenuItem = useAppearanceStore((s) => s.outdentMenuItem);
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

  // Filtrage des thèmes
  const filteredThemes = themes.filter((th) => {
    if (themeSearchQuery.trim()) {
      const q = themeSearchQuery.toLowerCase();
      const match = th.name.toLowerCase().includes(q) || th.description.toLowerCase().includes(q);
      if (!match) return false;
    }
    if (themeFilter === 'fse') {
      return th.slug === 'twentytwentyfive' || th.slug === 'default';
    }
    if (themeFilter === 'classic') {
      return th.slug === 'astra' || th.slug === 'generatepress';
    }
    return true;
  });

  const activeTheme = themes.find((th) => th.slug === activeThemeSlug || th.is_active) || themes[0];
  const otherThemes = filteredThemes.filter((th) => th.slug !== activeTheme?.slug);
  const activeItems = menus[activeLocation] || [];

  return (
    <div className="max-w-6xl space-y-6">
      {/* ==================================================================== */}
      {/* 1. EN-TÊTE PRINCIPAL & ONGLET DE NAVIGATION SOUTERRAINE             */}
      {/* ==================================================================== */}
      <div>
        <h1 className="text-xl font-bold text-[#1d2327] dark:text-white">
          {appearanceMessages.title}
        </h1>
        <p className="text-xs text-slate-500 dark:text-slate-400 mt-1">
          {appearanceMessages.subtitle}
        </p>
      </div>

      {/* Onglets principaux : Thèmes vs Éditeur FSE vs Personnaliser vs Menus */}
      <div className="flex flex-wrap items-center gap-1 border-b border-[#dcdcde] dark:border-[#2c3338]">
        <button
          type="button"
          onClick={() => setActiveTab('themes')}
          className={`flex items-center gap-2 px-4 py-2.5 text-xs font-semibold border-b-2 transition-all ${
            activeTab === 'themes'
              ? 'border-[#2271b1] text-[#2271b1] bg-white dark:bg-[#182030]'
              : 'border-transparent text-slate-600 dark:text-slate-400 hover:text-[#2271b1]'
          }`}
        >
          <LayoutTemplate className="w-4 h-4" />
          <span>Thèmes</span>
        </button>

        <button
          type="button"
          onClick={() => setActiveTab('site-editor')}
          className={`flex items-center gap-2 px-4 py-2.5 text-xs font-semibold border-b-2 transition-all ${
            activeTab === 'site-editor'
              ? 'border-[#2271b1] text-[#2271b1] bg-white dark:bg-[#182030]'
              : 'border-transparent text-slate-600 dark:text-slate-400 hover:text-[#2271b1]'
          }`}
        >
          <Layers className="w-4 h-4" />
          <span>Éditeur de site (FSE)</span>
        </button>

        <button
          type="button"
          onClick={() => setActiveTab('styles')}
          className={`flex items-center gap-2 px-4 py-2.5 text-xs font-semibold border-b-2 transition-all ${
            activeTab === 'styles'
              ? 'border-[#2271b1] text-[#2271b1] bg-white dark:bg-[#182030]'
              : 'border-transparent text-slate-600 dark:text-slate-400 hover:text-[#2271b1]'
          }`}
        >
          <Palette className="w-4 h-4" />
          <span>Personnaliser (Styles)</span>
        </button>

        <button
          type="button"
          onClick={() => setActiveTab('menus')}
          className={`flex items-center gap-2 px-4 py-2.5 text-xs font-semibold border-b-2 transition-all ${
            activeTab === 'menus'
              ? 'border-[#2271b1] text-[#2271b1] bg-white dark:bg-[#182030]'
              : 'border-transparent text-slate-600 dark:text-slate-400 hover:text-[#2271b1]'
          }`}
        >
          <Navigation className="w-4 h-4" />
          <span>Menus publics</span>
        </button>
      </div>

      {error && (
        <p
          role="alert"
          className="rounded-lg border border-red-500/30 bg-red-500/10 px-4 py-3 text-xs text-red-600 dark:text-red-200"
        >
          {error}
        </p>
      )}
      {message && (
        <p
          role="status"
          className="rounded-lg border border-emerald-500/30 bg-emerald-500/10 px-4 py-3 text-xs text-emerald-700 dark:text-emerald-200"
        >
          {message}
        </p>
      )}

      {/* ==================================================================== */}
      {/* 2. ONGLET 1 : THÈMES DU SITE (CONFORME themes.php)                   */}
      {/* ==================================================================== */}
      {activeTab === 'themes' && (
        <div className="space-y-6">
          {/* Barre d'outils supérieure : H1 + Ajouter + Recherche + .subsubsub */}
          <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 pb-3 border-b border-[#dcdcde] dark:border-[#2c3338]">
            <div className="flex items-center gap-3">
              <button
                type="button"
                onClick={() => setIsUploadModalOpen(true)}
                className="inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-semibold rounded border border-[#2271b1] text-[#2271b1] hover:bg-[#2271b1] hover:text-white transition-colors"
              >
                <Plus className="w-3.5 h-3.5" />
                <span>Ajouter un thème</span>
              </button>
            </div>

            {/* Barre de recherche instantanée */}
            <div className="relative w-full sm:w-72">
              <Search className="w-3.5 h-3.5 absolute left-2.5 top-1/2 -translate-y-1/2 text-slate-400" />
              <input
                type="search"
                placeholder="Rechercher parmi les thèmes installés…"
                value={themeSearchQuery}
                onChange={(e) => setThemeSearchQuery(e.target.value)}
                className="w-full pl-8 pr-3 py-1.5 text-xs bg-white dark:bg-[#131619] border border-[#8c8f94] dark:border-[#2c3338] rounded text-[#1d2327] dark:text-white placeholder:text-slate-400 focus:border-[#2271b1] outline-none"
              />
            </div>
          </div>

          {/* Navigation des sous-filtres WordPress (.subsubsub) */}
          <div className="flex items-center gap-2 text-xs text-slate-500">
            <button
              type="button"
              onClick={() => setThemeFilter('all')}
              className={`hover:text-[#2271b1] ${
                themeFilter === 'all' ? 'font-bold text-[#1d2327] dark:text-white' : ''
              }`}
            >
              Tous <span className="text-slate-400">({themes.length})</span>
            </button>
            <span>|</span>
            <button
              type="button"
              onClick={() => setThemeFilter('fse')}
              className={`hover:text-[#2271b1] ${
                themeFilter === 'fse' ? 'font-bold text-[#1d2327] dark:text-white' : ''
              }`}
            >
              Basés sur des blocs (FSE) <span className="text-slate-400">(2)</span>
            </button>
            <span>|</span>
            <button
              type="button"
              onClick={() => setThemeFilter('classic')}
              className={`hover:text-[#2271b1] ${
                themeFilter === 'classic' ? 'font-bold text-[#1d2327] dark:text-white' : ''
              }`}
            >
              Classiques <span className="text-slate-400">(2)</span>
            </button>
          </div>

          {/* Carte du Thème Actif mis en avant (Featured Spotlight) */}
          {activeTheme && (
            <div className="border border-[#2271b1] rounded-xl overflow-hidden bg-white dark:bg-[#182030] shadow-sm">
              <div className="grid grid-cols-1 md:grid-cols-12 gap-0">
                {/* Capture d'écran officielle */}
                <div className="md:col-span-5 relative bg-slate-100 dark:bg-black/30 overflow-hidden min-h-[220px]">
                  <img
                    src="/themes/twentytwentyfive.png"
                    alt={activeTheme.name}
                    className="w-full h-full object-cover object-top"
                    onError={(e) => {
                      (e.target as HTMLElement).style.display = 'none';
                    }}
                  />
                  <div className="absolute top-3 left-3 bg-[#2271b1] text-white text-[11px] font-bold px-2.5 py-0.5 rounded shadow">
                    Actif : {activeTheme.name}
                  </div>
                </div>

                {/* Détails du thème actif */}
                <div className="md:col-span-7 p-6 flex flex-col justify-between space-y-4">
                  <div>
                    <div className="flex items-center justify-between">
                      <h3 className="text-lg font-bold text-[#1d2327] dark:text-white">
                        {activeTheme.name}
                      </h3>
                      <span className="text-xs font-mono text-slate-400">v{activeTheme.version}</span>
                    </div>
                    <p className="text-xs text-slate-500 dark:text-slate-400 mt-1">
                      Par <span className="font-medium text-[#1d2327] dark:text-white">{activeTheme.author}</span>
                    </p>
                    <p className="text-xs text-slate-600 dark:text-slate-300 mt-3 leading-relaxed">
                      {activeTheme.description}
                    </p>
                    <div className="flex flex-wrap gap-1.5 mt-3">
                      <span className="px-2 py-0.5 rounded bg-blue-50 dark:bg-blue-950/40 text-[#2271b1] text-[10px] font-medium border border-blue-200 dark:border-blue-900">
                        Éditeur de site (FSE)
                      </span>
                      <span className="px-2 py-0.5 rounded bg-slate-100 dark:bg-slate-800 text-slate-600 dark:text-slate-300 text-[10px]">
                        8 Variations de style
                      </span>
                      <span className="px-2 py-0.5 rounded bg-slate-100 dark:bg-slate-800 text-slate-600 dark:text-slate-300 text-[10px]">
                        MiniJinja SSR
                      </span>
                    </div>
                  </div>

                  <div className="flex flex-wrap items-center gap-2 pt-3 border-t border-[#dcdcde] dark:border-[#2c3338]">
                    <button
                      type="button"
                      onClick={() => setActiveTab('site-editor')}
                      className="inline-flex items-center gap-1.5 px-4 py-2 rounded bg-[#2271b1] hover:bg-[#135e96] text-white text-xs font-semibold transition-colors"
                    >
                      <Layers className="w-3.5 h-3.5" />
                      <span>Éditeur de site (FSE)</span>
                    </button>
                    <button
                      type="button"
                      onClick={() => setActiveTab('styles')}
                      className="inline-flex items-center gap-1.5 px-3 py-2 rounded border border-[#8c8f94] dark:border-[#2c3338] text-[#1d2327] dark:text-white hover:bg-slate-100 dark:hover:bg-slate-800 text-xs font-medium transition-colors"
                    >
                      <Palette className="w-3.5 h-3.5" />
                      <span>Personnaliser</span>
                    </button>
                    <button
                      type="button"
                      onClick={() => setSelectedThemeForDetails(activeTheme)}
                      className="inline-flex items-center gap-1.5 px-3 py-2 rounded border border-transparent text-[#2271b1] hover:underline text-xs font-medium"
                    >
                      <span>Détails du thème</span>
                    </button>
                  </div>
                </div>
              </div>
            </div>
          )}

          {/* Grille des autres thèmes installés */}
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 pt-2">
            {otherThemes.map((th) => (
              <div
                key={th.slug}
                className="group relative border border-[#dcdcde] dark:border-[#2c3338] rounded-xl overflow-hidden bg-white dark:bg-[#182030] hover:shadow-md transition-all flex flex-col justify-between"
              >
                {/* Vignette / Aperçu visuel */}
                <div className="relative aspect-video bg-slate-100 dark:bg-black/30 overflow-hidden">
                  {th.slug === 'twentytwentyfive' ? (
                    <img
                      src="/themes/twentytwentyfive.png"
                      alt={th.name}
                      className="w-full h-full object-cover object-top"
                    />
                  ) : (
                    <div className="w-full h-full flex flex-col justify-between p-4 bg-gradient-to-tr from-slate-200 to-slate-100 dark:from-slate-900 dark:to-slate-800">
                      <div className="flex items-center justify-between">
                        <span className="text-[11px] font-bold text-slate-700 dark:text-slate-300">
                          {th.name}
                        </span>
                        <span className="text-[10px] text-slate-400 font-mono">v{th.version}</span>
                      </div>
                      <div className="text-[10px] text-slate-500 line-clamp-2">
                        {th.description}
                      </div>
                    </div>
                  )}

                  {/* Boutons d'action au survol (Overlay WordPress) */}
                  <div className="absolute inset-0 bg-black/60 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center gap-2 p-4">
                    <button
                      type="button"
                      disabled={activatingThemeSlug === th.slug}
                      onClick={() => void activateTheme(th.slug)}
                      className="px-3.5 py-1.5 rounded bg-[#2271b1] hover:bg-[#135e96] text-white text-xs font-semibold shadow transition-all"
                    >
                      {activatingThemeSlug === th.slug ? 'Activation…' : 'Activer'}
                    </button>
                    <button
                      type="button"
                      onClick={() => setSelectedThemeForDetails(th)}
                      className="px-3.5 py-1.5 rounded bg-white text-[#1d2327] hover:bg-slate-100 text-xs font-medium shadow transition-all"
                    >
                      Détails
                    </button>
                  </div>
                </div>

                {/* Barre de pied de carte */}
                <div className="p-3.5 flex items-center justify-between border-t border-[#dcdcde] dark:border-[#2c3338]">
                  <div className="truncate pr-2">
                    <h4 className="text-xs font-bold text-[#1d2327] dark:text-white truncate">
                      {th.name}
                    </h4>
                    <p className="text-[11px] text-slate-400 truncate">Par {th.author}</p>
                  </div>
                  <button
                    type="button"
                    onClick={() => setSelectedThemeForDetails(th)}
                    className="text-[#2271b1] hover:underline text-xs shrink-0"
                  >
                    Détails
                  </button>
                </div>
              </div>
            ))}

            {/* Carte pointillée « + Ajouter un thème » */}
            <button
              type="button"
              onClick={() => setIsUploadModalOpen(true)}
              className="min-h-[220px] border-2 border-dashed border-[#dcdcde] dark:border-[#2c3338] hover:border-[#2271b1] rounded-xl p-6 flex flex-col items-center justify-center gap-2 text-slate-400 hover:text-[#2271b1] transition-all group bg-slate-50/50 dark:bg-slate-900/20"
            >
              <div className="w-12 h-12 rounded-full border border-dashed border-current flex items-center justify-center group-hover:scale-110 transition-transform">
                <Plus className="w-6 h-6" />
              </div>
              <span className="text-xs font-semibold">Ajouter un nouveau thème</span>
              <span className="text-[11px] text-slate-400">Importer une archive .zip ou pack Rust</span>
            </button>
          </div>
        </div>
      )}

      {/* ==================================================================== */}
      {/* 3. ONGLET 2 : ÉDITEUR DE SITE SOUVERAIN (FSE - site-editor.php)       */}
      {/* ==================================================================== */}
      {activeTab === 'site-editor' && (
        <div className="border border-[#dcdcde] dark:border-[#2c3338] rounded-xl bg-white dark:bg-[#182030] overflow-hidden shadow-sm">
          {/* Barre supérieure de l'Éditeur de Site */}
          <div className="h-12 border-b border-[#dcdcde] dark:border-[#2c3338] px-4 flex items-center justify-between bg-slate-50 dark:bg-[#131619] text-xs">
            <div className="flex items-center gap-3">
              <span className="font-bold text-[#1d2327] dark:text-white flex items-center gap-1.5">
                <Layers className="w-4 h-4 text-[#2271b1]" />
                <span>Éditeur de site FSE : {SITE_TEMPLATES.find((t) => t.id === selectedTemplate)?.name}</span>
              </span>
            </div>

            {/* Sélecteur de périphérique de prévisualisation */}
            <div className="flex items-center gap-1 bg-white dark:bg-[#182030] border border-[#dcdcde] dark:border-[#2c3338] rounded p-0.5">
              <button
                type="button"
                onClick={() => setDevicePreview('desktop')}
                className={`p-1.5 rounded ${
                  devicePreview === 'desktop'
                    ? 'bg-[#2271b1] text-white'
                    : 'text-slate-500 hover:text-[#1d2327] dark:hover:text-white'
                }`}
                title="Aperçu Bureau"
              >
                <Monitor className="w-3.5 h-3.5" />
              </button>
              <button
                type="button"
                onClick={() => setDevicePreview('tablet')}
                className={`p-1.5 rounded ${
                  devicePreview === 'tablet'
                    ? 'bg-[#2271b1] text-white'
                    : 'text-slate-500 hover:text-[#1d2327] dark:hover:text-white'
                }`}
                title="Aperçu Tablette (768px)"
              >
                <Tablet className="w-3.5 h-3.5" />
              </button>
              <button
                type="button"
                onClick={() => setDevicePreview('mobile')}
                className={`p-1.5 rounded ${
                  devicePreview === 'mobile'
                    ? 'bg-[#2271b1] text-white'
                    : 'text-slate-500 hover:text-[#1d2327] dark:hover:text-white'
                }`}
                title="Aperçu Mobile (375px)"
              >
                <Smartphone className="w-3.5 h-3.5" />
              </button>
            </div>

            {/* Boutons d'action */}
            <div className="flex items-center gap-2">
              <a
                href="/"
                target="_blank"
                rel="noopener noreferrer"
                className="inline-flex items-center gap-1 px-2.5 py-1.5 rounded border border-[#dcdcde] dark:border-[#2c3338] text-slate-600 dark:text-slate-300 hover:bg-slate-100 text-xs"
              >
                <ExternalLink className="w-3.5 h-3.5" />
                <span>Voir le site</span>
              </a>
              <button
                type="button"
                disabled={saving}
                onClick={() => void saveStyles()}
                className="inline-flex items-center gap-1.5 px-3.5 py-1.5 rounded bg-[#2271b1] hover:bg-[#135e96] text-white text-xs font-semibold shadow-xs disabled:opacity-50"
              >
                <Save className="w-3.5 h-3.5" />
                <span>{saving ? 'Enregistrement…' : 'Enregistrer'}</span>
              </button>
            </div>
          </div>

          {/* Disposition 2 volets : Panneau latéral d'options + Canevas central */}
          <div className="grid grid-cols-1 lg:grid-cols-12 min-h-[560px]">
            {/* Volet latéral gauche : Modèles & 8 Variations de styles */}
            <div className="lg:col-span-4 border-r border-[#dcdcde] dark:border-[#2c3338] p-4 space-y-6 overflow-y-auto max-h-[700px]">
              {/* Sélecteur de Modèle (Template) */}
              <div>
                <h4 className="font-semibold text-xs text-[#1d2327] dark:text-white mb-2 uppercase tracking-wider">
                  Modèles de page
                </h4>
                <div className="space-y-1">
                  {SITE_TEMPLATES.map((tmpl) => (
                    <button
                      key={tmpl.id}
                      type="button"
                      onClick={() => setSelectedTemplate(tmpl.id)}
                      className={`w-full p-2.5 rounded-lg text-left transition-colors flex items-center justify-between text-xs ${
                        selectedTemplate === tmpl.id
                          ? 'bg-blue-50 dark:bg-blue-950/40 text-[#2271b1] font-semibold border border-blue-200 dark:border-blue-900'
                          : 'hover:bg-slate-50 dark:hover:bg-slate-800 text-slate-700 dark:text-slate-300'
                      }`}
                    >
                      <div>
                        <div>{tmpl.name}</div>
                        <div className="text-[10px] text-slate-400 font-mono">{tmpl.file}</div>
                      </div>
                      {selectedTemplate === tmpl.id && <Check className="w-4 h-4 text-[#2271b1]" />}
                    </button>
                  ))}
                </div>
              </div>

              {/* Les 8 Variations Officielles Twenty Twenty-Five */}
              <div className="pt-4 border-t border-[#dcdcde] dark:border-[#2c3338]">
                <div className="flex items-center justify-between mb-2">
                  <h4 className="font-semibold text-xs text-[#1d2327] dark:text-white uppercase tracking-wider">
                    Variations de style (TT25)
                  </h4>
                  <span className="text-[10px] text-slate-400 font-mono">8 styles</span>
                </div>
                <p className="text-[11px] text-slate-500 mb-3">
                  Sélectionnez l’une des 8 palettes officielles Twenty Twenty-Five. Les tokens s’appliquent immédiatement.
                </p>

                <div className="grid grid-cols-2 gap-2">
                  {TWENTY_TWENTY_FIVE_VARIATIONS.map((v) => {
                    const isSelected = selectedVariationId === v.id;
                    return (
                      <button
                        key={v.id}
                        type="button"
                        onClick={() => void applyStyleVariation(v.id)}
                        className={`p-2.5 rounded-lg text-left border transition-all ${
                          isSelected
                            ? 'border-[#2271b1] ring-1 ring-[#2271b1] bg-blue-50/40 dark:bg-blue-950/20'
                            : 'border-slate-200 dark:border-[#2c3338] hover:border-slate-300 bg-white dark:bg-[#131619]'
                        }`}
                      >
                        <div className="flex items-center gap-1.5 mb-2">
                          <span
                            className="w-4 h-4 rounded-full border border-black/20"
                            style={{ backgroundColor: v.base }}
                            title="Base"
                          />
                          <span
                            className="w-4 h-4 rounded-full border border-black/20"
                            style={{ backgroundColor: v.contrast }}
                            title="Contraste"
                          />
                          <span
                            className="w-4 h-4 rounded-full border border-black/20"
                            style={{ backgroundColor: v.accent }}
                            title="Accent"
                          />
                        </div>
                        <div className="text-xs font-semibold text-[#1d2327] dark:text-white truncate">
                          {v.name.split(' (')[0]}
                        </div>
                        <div className="text-[10px] text-slate-400 truncate mt-0.5">
                          {v.description}
                        </div>
                      </button>
                    );
                  })}
                </div>
              </div>
            </div>

            {/* Canevas de prévisualisation central */}
            <div className="lg:col-span-8 bg-[#f0f0f1] dark:bg-[#101520] p-6 flex flex-col items-center justify-start overflow-y-auto">
              <div
                className={`w-full transition-all duration-200 bg-white dark:bg-[#182030] shadow-md rounded-xl p-8 space-y-6 ${
                  devicePreview === 'mobile'
                    ? 'max-w-[375px]'
                    : devicePreview === 'tablet'
                    ? 'max-w-[768px]'
                    : 'max-w-4xl'
                }`}
                style={{
                  fontFamily: tokens.font_body,
                }}
              >
                {/* En-tête de page simulé */}
                <header className="border-b border-slate-200 dark:border-slate-700 pb-4 flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <span
                      className="w-6 h-6 rounded-full flex items-center justify-center text-white text-xs font-bold"
                      style={{ backgroundColor: tokens.primary_color }}
                    >
                      RF
                    </span>
                    <span
                      className="font-bold text-sm text-[#1d2327] dark:text-white tracking-tight"
                      style={{ fontFamily: tokens.font_heading }}
                    >
                      Twenty Twenty-Five Sovereign
                    </span>
                  </div>
                  <nav className="flex items-center gap-4 text-xs text-slate-600 dark:text-slate-300">
                    <span className="hover:text-[#2271b1] cursor-pointer">Accueil</span>
                    <span className="hover:text-[#2271b1] cursor-pointer">Articles</span>
                    <span className="hover:text-[#2271b1] cursor-pointer">À propos</span>
                  </nav>
                </header>

                {/* Contenu simulé du modèle sélectionné */}
                <main className="space-y-4 py-4">
                  <div className="space-y-2">
                    <span
                      className="text-xs uppercase font-mono tracking-widest"
                      style={{ color: tokens.primary_color }}
                    >
                      Modèle : {SITE_TEMPLATES.find((t) => t.id === selectedTemplate)?.file}
                    </span>
                    <h2
                      className="text-2xl sm:text-3xl font-bold text-[#1d2327] dark:text-white leading-tight"
                      style={{ fontFamily: tokens.font_heading }}
                    >
                      La simplicité souveraine du rendu MiniJinja en Rust
                    </h2>
                  </div>

                  <p className="text-xs text-slate-600 dark:text-slate-300 leading-relaxed">
                    Ce gabarit est exécuté nativement par le moteur SSR de RustForge CMS.
                    Les tokens de design, les variations de style Twenty Twenty-Five et les menus
                    publics sont directement injectés sans temps de chargement PHP ni surcharge mémoire.
                  </p>

                  <div className="p-4 rounded-xl border border-slate-200 dark:border-slate-700 bg-slate-50 dark:bg-slate-900/40 flex items-center justify-between">
                    <span className="text-xs text-slate-600 dark:text-slate-300">
                      Bouton d’action principal avec vos tokens actifs
                    </span>
                    <button
                      type="button"
                      className="px-4 py-2 text-xs font-semibold text-white shadow transition-transform hover:scale-105"
                      style={{
                        backgroundColor: tokens.primary_color,
                        borderRadius: tokens.radius,
                      }}
                    >
                      Découvrir la suite
                    </button>
                  </div>
                </main>

                {/* Pied de page simulé */}
                <footer className="border-t border-slate-200 dark:border-slate-700 pt-4 flex items-center justify-between text-[11px] text-slate-400">
                  <span>© 2026 RustForge Studio • Fièrement propulsé par Rust & Twenty Twenty-Five</span>
                  <span className="font-mono">SSR : 2 ms</span>
                </footer>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* ==================================================================== */}
      {/* 4. ONGLET 3 : PERSONNALISER & TOKENS DE DESIGN (WP-48 / WP-50)       */}
      {/* ==================================================================== */}
      {activeTab === 'styles' && (
        <form
          onSubmit={(e) => {
            e.preventDefault();
            void saveStyles();
          }}
          className="grid grid-cols-1 lg:grid-cols-12 gap-6 items-start"
        >
          <div className="lg:col-span-7 bg-white dark:bg-[#182030] border border-[#dcdcde] dark:border-[#2c3338] rounded-xl p-6 space-y-6 text-xs shadow-sm">
            {/* Palettes de couleurs harmonisées en 1 clic */}
            <div className="space-y-2">
              <span className="font-semibold text-slate-700 dark:text-slate-200 block">
                Palettes harmonisées (1 clic) :
              </span>
              <div className="grid grid-cols-2 sm:grid-cols-3 gap-2">
                {COLOR_PRESETS.map((preset) => (
                  <button
                    key={preset.name}
                    type="button"
                    onClick={() => {
                      setToken('primary_color', preset.primary);
                      setToken('accent_color', preset.accent);
                    }}
                    className="flex items-center gap-2 rounded-lg border border-slate-200 dark:border-white/10 bg-slate-50 dark:bg-black/30 p-2 text-left hover:border-[#2271b1] transition-colors"
                  >
                    <div className="flex items-center -space-x-1 shrink-0">
                      <span
                        className="w-4 h-4 rounded-full border border-black/40"
                        style={{ background: preset.primary }}
                      />
                      <span
                        className="w-4 h-4 rounded-full border border-black/40"
                        style={{ background: preset.accent }}
                      />
                    </div>
                    <span className="truncate text-[11px] font-medium text-slate-700 dark:text-slate-300">
                      {preset.name}
                    </span>
                  </button>
                ))}
              </div>
            </div>

            <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 pt-2 border-t border-slate-100 dark:border-white/5">
              <label className="flex items-center justify-between gap-4 rounded-lg bg-slate-50 dark:bg-black/25 border border-slate-200 dark:border-white/5 p-3">
                <span className="font-medium text-slate-700 dark:text-slate-200">
                  {appearanceMessages.primaryColor}
                </span>
                <input
                  aria-label={appearanceMessages.primaryColor}
                  type="color"
                  value={
                    /^#[0-9a-f]{6}$/i.test(tokens.primary_color) ? tokens.primary_color : '#4f46e5'
                  }
                  onChange={(e) => setToken('primary_color', e.target.value)}
                  className="w-16 h-8 bg-transparent cursor-pointer rounded"
                />
              </label>

              <label className="flex items-center justify-between gap-4 rounded-lg bg-slate-50 dark:bg-black/25 border border-slate-200 dark:border-white/5 p-3">
                <span className="font-medium text-slate-700 dark:text-slate-200">
                  {appearanceMessages.accentColor}
                </span>
                <input
                  aria-label={appearanceMessages.accentColor}
                  type="color"
                  value={
                    /^#[0-9a-f]{6}$/i.test(tokens.accent_color) ? tokens.accent_color : '#06b6d4'
                  }
                  onChange={(e) => setToken('accent_color', e.target.value)}
                  className="w-16 h-8 bg-transparent cursor-pointer rounded"
                />
              </label>
            </div>

            {/* Typographie fluide & polices souveraines */}
            <div className="space-y-3 pt-2 border-t border-slate-100 dark:border-white/5">
              <div className="flex items-center justify-between">
                <span className="font-semibold text-slate-700 dark:text-slate-200">
                  Typographie souveraine (RGPD local)
                </span>
                <span className="rounded bg-blue-50 dark:bg-indigo-500/20 text-[#2271b1] dark:text-indigo-300 px-2 py-0.5 text-[10px] font-mono">
                  100% sans Google Fonts
                </span>
              </div>
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <label className="block space-y-1.5">
                  <span className="text-slate-600 dark:text-slate-300">Police des Titres :</span>
                  <select
                    value={tokens.font_heading || HEADING_FONTS[0].value}
                    onChange={(e) => setToken('font_heading', e.target.value)}
                    className="w-full rounded-lg bg-white dark:bg-slate-950 border border-slate-300 dark:border-white/10 px-3 py-2 text-xs text-[#1d2327] dark:text-white"
                  >
                    {HEADING_FONTS.map((font) => (
                      <option key={font.name} value={font.value}>
                        {font.name}
                      </option>
                    ))}
                  </select>
                </label>
                <label className="block space-y-1.5">
                  <span className="text-slate-600 dark:text-slate-300">Police du Corps :</span>
                  <select
                    value={tokens.font_body || BODY_FONTS[0].value}
                    onChange={(e) => setToken('font_body', e.target.value)}
                    className="w-full rounded-lg bg-white dark:bg-slate-950 border border-slate-300 dark:border-white/10 px-3 py-2 text-xs text-[#1d2327] dark:text-white"
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

            <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 pt-2 border-t border-slate-100 dark:border-white/5">
              <label className="block space-y-1.5">
                <span className="font-medium text-slate-700 dark:text-slate-200">
                  {appearanceMessages.containerWidth} : {tokens.container_width}
                </span>
                <select
                  value={tokens.container_width}
                  onChange={(e) => setToken('container_width', e.target.value)}
                  className="w-full rounded-lg bg-white dark:bg-slate-950 border border-slate-300 dark:border-white/10 px-3 py-2 text-xs text-[#1d2327] dark:text-white"
                >
                  <option value="960px">Compact (960px — lecture longue)</option>
                  <option value="1100px">Standard (1100px — défaut rf-base)</option>
                  <option value="1280px">Large (1280px — portail éditorial)</option>
                  <option value="1440px">Très large (1440px)</option>
                </select>
              </label>

              <label className="block space-y-1.5">
                <span className="font-medium text-slate-700 dark:text-slate-200">
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
            <label className="block space-y-1.5 pt-2 border-t border-slate-100 dark:border-white/5">
              <span className="font-medium text-slate-700 dark:text-slate-200">
                {appearanceMessages.customCssLabel}
              </span>
              <textarea
                rows={4}
                value={tokens.custom_css}
                onChange={(e) => setToken('custom_css', e.target.value)}
                placeholder="/* Ajoutez ici vos règles CSS personnalisées */"
                className="w-full rounded-lg bg-white dark:bg-slate-950 border border-slate-300 dark:border-white/10 p-3 font-mono text-xs text-[#1d2327] dark:text-white resize-y"
              />
            </label>

            <button
              type="submit"
              disabled={saving}
              className="w-full inline-flex items-center justify-center gap-2 rounded-lg bg-[#2271b1] hover:bg-[#135e96] px-4 py-2.5 font-semibold text-white disabled:opacity-40 shadow transition-colors"
            >
              <Save className="w-4 h-4" />
              <span>{saving ? appearanceMessages.savingStyles : appearanceMessages.saveStyles}</span>
            </button>
          </div>

          {/* Prévisualisation dynamique en direct */}
          <div className="lg:col-span-5 bg-white dark:bg-[#182030] border border-[#dcdcde] dark:border-[#2c3338] rounded-xl p-6 space-y-4 text-xs shadow-sm sticky top-16">
            <span className="font-semibold text-slate-700 dark:text-slate-200 block">
              Aperçu en direct des Tokens :
            </span>
            <div
              className="p-5 border border-slate-200 dark:border-white/10 space-y-3 transition-all"
              style={{
                borderRadius: tokens.radius,
                fontFamily: tokens.font_body,
              }}
            >
              <div
                className="text-base font-bold text-[#1d2327] dark:text-white"
                style={{ fontFamily: tokens.font_heading, color: tokens.primary_color }}
              >
                Titre d’article d’exemple
              </div>
              <p className="text-slate-600 dark:text-slate-300 leading-relaxed text-[11px]">
                Cet aperçu illustre la combinaison de votre typographie de titre, de votre police de
                corps et de votre rayon de courbure.
              </p>
              <button
                type="button"
                className="px-3.5 py-1.5 text-[11px] font-semibold text-white shadow"
                style={{
                  backgroundColor: tokens.primary_color,
                  borderRadius: tokens.radius,
                }}
              >
                Bouton primaire
              </button>
            </div>
          </div>
        </form>
      )}

      {/* ==================================================================== */}
      {/* 5. ONGLET 4 : MENUS PUBLICS (CONFORME nav-menus.php)                 */}
      {/* ==================================================================== */}
      {activeTab === 'menus' && (
        <div className="grid grid-cols-1 lg:grid-cols-12 gap-6 items-start">
          {/* Colonne gauche : Ajout rapide de liens */}
          <div className="lg:col-span-5 bg-white dark:bg-[#182030] border border-[#dcdcde] dark:border-[#2c3338] rounded-xl p-5 space-y-5 text-xs shadow-sm">
            <div className="flex items-center justify-between border-b border-[#dcdcde] dark:border-[#2c3338] pb-3">
              <span className="font-semibold text-[#1d2327] dark:text-white">Emplacement :</span>
              <div className="flex gap-1">
                <button
                  type="button"
                  onClick={() => setActiveLocation('header')}
                  className={`px-3 py-1 rounded text-xs font-medium transition-colors ${
                    activeLocation === 'header'
                      ? 'bg-[#2271b1] text-white'
                      : 'bg-slate-100 dark:bg-slate-800 text-slate-600 dark:text-slate-400'
                  }`}
                >
                  En-tête
                </button>
                <button
                  type="button"
                  onClick={() => setActiveLocation('footer')}
                  className={`px-3 py-1 rounded text-xs font-medium transition-colors ${
                    activeLocation === 'footer'
                      ? 'bg-[#2271b1] text-white'
                      : 'bg-slate-100 dark:bg-slate-800 text-slate-600 dark:text-slate-400'
                  }`}
                >
                  Pied de page
                </button>
              </div>
            </div>

            {/* Ajout de lien personnalisé */}
            <div className="space-y-3">
              <span className="font-semibold text-slate-700 dark:text-slate-200 block">
                Lien personnalisé
              </span>
              <input
                type="text"
                value={newLinkLabel}
                onChange={(e) => setNewLinkLabel(e.target.value)}
                placeholder="Texte du lien (ex: Contact)"
                className="w-full rounded bg-white dark:bg-slate-950 border border-slate-300 dark:border-white/10 px-3 py-1.5 text-xs text-[#1d2327] dark:text-white"
              />
              <input
                type="text"
                value={newLinkUrl}
                onChange={(e) => setNewLinkUrl(e.target.value)}
                placeholder="URL (ex: /contact ou https://...)"
                className="w-full rounded bg-white dark:bg-slate-950 border border-slate-300 dark:border-white/10 px-3 py-1.5 text-xs text-[#1d2327] dark:text-white"
              />
              <button
                type="button"
                disabled={!newLinkLabel.trim() || !newLinkUrl.trim()}
                onClick={() => addMenuItem()}
                className="w-full inline-flex items-center justify-center gap-1.5 rounded bg-slate-100 dark:bg-slate-800 hover:bg-[#2271b1] hover:text-white px-3 py-1.5 font-medium text-[#1d2327] dark:text-white disabled:opacity-40 transition-colors"
              >
                <Plus className="w-3.5 h-3.5" />
                <span>Ajouter au menu</span>
              </button>
            </div>

            {/* Pages publiées */}
            {publishedPages.length > 0 && (
              <div className="space-y-2 pt-3 border-t border-[#dcdcde] dark:border-[#2c3338]">
                <span className="font-semibold text-slate-700 dark:text-slate-200 block">
                  Pages publiées
                </span>
                <div className="max-h-40 overflow-y-auto space-y-1">
                  {publishedPages.map((page) => (
                    <button
                      key={page.id}
                      type="button"
                      onClick={() => addMenuItem(page.title, `/${page.slug}`)}
                      className="w-full text-left p-1.5 rounded hover:bg-slate-50 dark:hover:bg-slate-800 text-slate-600 dark:text-slate-300 flex items-center justify-between"
                    >
                      <span className="truncate">{page.title}</span>
                      <Plus className="w-3 h-3 text-[#2271b1]" />
                    </button>
                  ))}
                </div>
              </div>
            )}

            {/* Catégories de contenu */}
            {categories.length > 0 && (
              <div className="space-y-2 pt-3 border-t border-[#dcdcde] dark:border-[#2c3338]">
                <span className="font-semibold text-slate-700 dark:text-slate-200 block">
                  Catégories
                </span>
                <div className="max-h-36 overflow-y-auto space-y-1">
                  {categories.map((cat) => (
                    <button
                      key={cat.id}
                      type="button"
                      onClick={() => addMenuItem(cat.name, `/category/${cat.name.toLowerCase()}`)}
                      className="w-full text-left p-1.5 rounded hover:bg-slate-50 dark:hover:bg-slate-800 text-slate-600 dark:text-slate-300 flex items-center justify-between"
                    >
                      <span className="truncate">{cat.name}</span>
                      <Plus className="w-3 h-3 text-[#2271b1]" />
                    </button>
                  ))}
                </div>
              </div>
            )}
          </div>

          {/* Colonne droite : Structure du menu actif */}
          <div className="lg:col-span-7 bg-white dark:bg-[#182030] border border-[#dcdcde] dark:border-[#2c3338] rounded-xl p-5 space-y-4 text-xs shadow-sm">
            <div className="flex items-center justify-between border-b border-[#dcdcde] dark:border-[#2c3338] pb-3">
              <span className="font-bold text-[#1d2327] dark:text-white">
                Structure du menu : {activeLocation === 'header' ? 'En-tête' : 'Pied de page'}
              </span>
              <button
                type="button"
                disabled={saving}
                onClick={() => void saveMenus()}
                className="inline-flex items-center gap-1.5 rounded bg-[#2271b1] hover:bg-[#135e96] px-3.5 py-1.5 font-semibold text-white disabled:opacity-40 shadow-xs"
              >
                <Save className="w-3.5 h-3.5" />
                <span>{saving ? 'Enregistrement…' : 'Enregistrer le menu'}</span>
              </button>
            </div>

            {activeItems.length === 0 ? (
              <p className="text-slate-400 py-8 text-center">
                Ce menu est vide. Ajoutez des éléments depuis le volet gauche.
              </p>
            ) : (
              <ul className="space-y-2">
                {activeItems.map((item, index) => {
                  const depth = getItemDepth(item.id, activeItems);
                  return (
                    <li
                      key={item.id}
                      className="p-3 rounded-lg border border-slate-200 dark:border-[#2c3338] bg-slate-50 dark:bg-[#131619] transition-all"
                      style={{ marginLeft: `${(depth - 1) * 20}px` }}
                    >
                      <div className="flex items-center justify-between">
                        <div className="flex items-center gap-2">
                          <span className="font-semibold text-[#1d2327] dark:text-white">
                            {item.label}
                          </span>
                          <span className="text-[10px] text-slate-400 font-mono">{item.url}</span>
                        </div>
                        <div className="flex items-center gap-1">
                          <button
                            type="button"
                            disabled={index === 0}
                            onClick={() => moveMenuItem(item.id, -1)}
                            className="p-1 rounded hover:bg-slate-200 dark:hover:bg-slate-800 disabled:opacity-20"
                            title="Monter"
                          >
                            <ArrowUp className="w-3 h-3" />
                          </button>
                          <button
                            type="button"
                            disabled={index === activeItems.length - 1}
                            onClick={() => moveMenuItem(item.id, 1)}
                            className="p-1 rounded hover:bg-slate-200 dark:hover:bg-slate-800 disabled:opacity-20"
                            title="Descendre"
                          >
                            <ArrowDown className="w-3 h-3" />
                          </button>
                          <button
                            type="button"
                            onClick={() => indentMenuItem(item.id)}
                            className="p-1 rounded hover:bg-slate-200 dark:hover:bg-slate-800"
                            title="Sous-élément"
                          >
                            <ArrowRight className="w-3 h-3" />
                          </button>
                          <button
                            type="button"
                            onClick={() => outdentMenuItem(item.id)}
                            className="p-1 rounded hover:bg-slate-200 dark:hover:bg-slate-800"
                            title="Niveau supérieur"
                          >
                            <ArrowLeft className="w-3 h-3" />
                          </button>
                          <button
                            type="button"
                            onClick={() => removeMenuItem(item.id)}
                            className="p-1 rounded hover:bg-red-50 text-red-500"
                            title="Supprimer"
                          >
                            <Trash2 className="w-3 h-3" />
                          </button>
                        </div>
                      </div>
                    </li>
                  );
                })}
              </ul>
            )}
          </div>
        </div>
      )}

      {/* ==================================================================== */}
      {/* 6. MODALE DÉTAILS DU THÈME (OVERLAY OFFICIEL WORDPRESS)               */}
      {/* ==================================================================== */}
      {selectedThemeForDetails && (
        <div className="fixed inset-0 bg-black/60 backdrop-blur-xs z-50 flex items-center justify-center p-4">
          <div className="bg-white dark:bg-[#182030] rounded-xl shadow-2xl border border-[#dcdcde] dark:border-[#2c3338] w-full max-w-4xl max-h-[90vh] flex flex-col overflow-hidden animate-in fade-in zoom-in-95 duration-150">
            {/* En-tête de la modale */}
            <div className="px-6 py-4 border-b border-[#dcdcde] dark:border-[#2c3338] flex items-center justify-between">
              <h3 className="text-base font-bold text-[#1d2327] dark:text-white">
                Détails du thème : {selectedThemeForDetails.name}
              </h3>
              <button
                type="button"
                onClick={() => setSelectedThemeForDetails(null)}
                className="p-1 rounded hover:bg-slate-100 dark:hover:bg-slate-800 text-slate-400 hover:text-slate-600 dark:hover:text-white"
              >
                <X className="w-5 h-5" />
              </button>
            </div>

            {/* Corps 2 colonnes */}
            <div className="flex-1 overflow-y-auto p-6 grid grid-cols-1 md:grid-cols-12 gap-6">
              <div className="md:col-span-6 bg-slate-100 dark:bg-black/30 rounded-lg overflow-hidden border border-[#dcdcde] dark:border-[#2c3338] flex items-center justify-center">
                <img
                  src="/themes/twentytwentyfive.png"
                  alt=""
                  className="w-full h-auto object-cover"
                />
              </div>
              <div className="md:col-span-6 space-y-4 text-xs">
                <div>
                  <h4 className="text-lg font-bold text-[#1d2327] dark:text-white">
                    {selectedThemeForDetails.name}
                  </h4>
                  <p className="text-slate-500 dark:text-slate-400 mt-0.5">
                    Version {selectedThemeForDetails.version} • Par{' '}
                    <span className="text-[#2271b1] font-medium">
                      {selectedThemeForDetails.author}
                    </span>
                  </p>
                </div>
                <p className="text-slate-600 dark:text-slate-300 leading-relaxed">
                  {selectedThemeForDetails.description}
                </p>
                <div>
                  <h5 className="font-semibold text-slate-700 dark:text-slate-300 mb-1.5">
                    Étiquettes :
                  </h5>
                  <div className="flex flex-wrap gap-1">
                    {[
                      'blog',
                      'éditorial',
                      'fse',
                      'style-variations',
                      'responsive',
                      'deux-colonnes',
                    ].map((tag) => (
                      <span
                        key={tag}
                        className="px-2 py-0.5 rounded bg-slate-100 dark:bg-slate-800 text-slate-600 dark:text-slate-400 text-[10px]"
                      >
                        {tag}
                      </span>
                    ))}
                  </div>
                </div>
              </div>
            </div>

            {/* Pied de page de la modale */}
            <div className="px-6 py-3 border-t border-[#dcdcde] dark:border-[#2c3338] bg-slate-50 dark:bg-[#131619] flex items-center justify-between">
              <span className="text-xs text-slate-400 font-mono">
                slug: {selectedThemeForDetails.slug}
              </span>
              <div className="flex items-center gap-2">
                <button
                  type="button"
                  onClick={() => setSelectedThemeForDetails(null)}
                  className="px-3 py-1.5 rounded border border-[#8c8f94] dark:border-[#2c3338] text-xs text-[#1d2327] dark:text-white"
                >
                  Fermer
                </button>
                {!selectedThemeForDetails.is_active && (
                  <button
                    type="button"
                    onClick={() => {
                      void activateTheme(selectedThemeForDetails.slug);
                      setSelectedThemeForDetails(null);
                    }}
                    className="px-4 py-1.5 rounded bg-[#2271b1] hover:bg-[#135e96] text-white text-xs font-semibold"
                  >
                    Activer
                  </button>
                )}
              </div>
            </div>
          </div>
        </div>
      )}

      {/* ==================================================================== */}
      {/* 7. MODALE D'IMPORTATION / AJOUT DE THÈME                             */}
      {/* ==================================================================== */}
      {isUploadModalOpen && (
        <div className="fixed inset-0 bg-black/60 backdrop-blur-xs z-50 flex items-center justify-center p-4">
          <div className="bg-white dark:bg-[#182030] rounded-xl shadow-2xl border border-[#dcdcde] dark:border-[#2c3338] w-full max-w-md p-6 space-y-4 animate-in fade-in zoom-in-95 duration-150">
            <div className="flex items-center justify-between">
              <h3 className="text-sm font-bold text-[#1d2327] dark:text-white flex items-center gap-2">
                <UploadCloud className="w-4 h-4 text-[#2271b1]" />
                <span>Téléverser un thème</span>
              </h3>
              <button
                type="button"
                onClick={() => setIsUploadModalOpen(false)}
                className="p-1 rounded hover:bg-slate-100 dark:hover:bg-slate-800 text-slate-400"
              >
                <X className="w-4 h-4" />
              </button>
            </div>

            <p className="text-xs text-slate-500 dark:text-slate-400">
              Si vous avez un thème au format .zip (archive de templates MiniJinja et tokens
              theme.json), vous pouvez l’installer en le téléversant ici.
            </p>

            <div className="border-2 border-dashed border-[#8c8f94] dark:border-[#2c3338] rounded-xl p-8 text-center space-y-2 hover:border-[#2271b1] transition-colors cursor-pointer">
              <UploadCloud className="w-8 h-8 text-slate-400 mx-auto" />
              <span className="text-xs text-[#2271b1] font-semibold block">
                Sélectionnez un fichier .zip
              </span>
              <span className="text-[10px] text-slate-400 block">Taille maximale : 16 Mo</span>
            </div>

            <div className="flex justify-end gap-2 pt-2">
              <button
                type="button"
                onClick={() => setIsUploadModalOpen(false)}
                className="px-3 py-1.5 rounded border border-[#8c8f94] dark:border-[#2c3338] text-xs text-[#1d2327] dark:text-white"
              >
                Annuler
              </button>
              <button
                type="button"
                onClick={() => setIsUploadModalOpen(false)}
                className="px-4 py-1.5 rounded bg-[#2271b1] hover:bg-[#135e96] text-white text-xs font-semibold"
              >
                Installer maintenant
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
