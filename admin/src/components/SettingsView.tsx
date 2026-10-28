/**
 * ============================================================================
 * VUE RÉGLAGES DU SITE : GÉNÉRAL, LECTURE & ÉCRITURE (`SettingsView`)
 * ============================================================================
 * Tâches : `WP-69` à `WP-81` (Titre, slogan, URL, e-mail d'administration,
 * langue/fuseau/formats de date, inscription et rôle par défaut borné,
 * accueil statique ou derniers articles, pagination et visibilité moteurs).
 */

import React, { useEffect } from 'react';
import {
  Save,
  CheckCircle2,
  Globe,
  BookOpen,
  PenTool,
  Link2,
  AlertTriangle,
} from 'lucide-react';
import { settingsMessages } from '../i18n/messages.fr';
import { useSettingsStore, type SettingsTab } from '../stores/useSettingsStore';

interface SettingsViewProps {
  initialTab?: SettingsTab;
}

export const SettingsView: React.FC<SettingsViewProps> = ({ initialTab = 'general' }) => {
  const activeTab = useSettingsStore((s) => s.activeTab);
  const settings = useSettingsStore((s) => s.settings);
  const pages = useSettingsStore((s) => s.pages);
  const categories = useSettingsStore((s) => s.categories);
  const loading = useSettingsStore((s) => s.loading);
  const saving = useSettingsStore((s) => s.saving);
  const saved = useSettingsStore((s) => s.saved);
  const error = useSettingsStore((s) => s.error);

  const setActiveTab = useSettingsStore((s) => s.setActiveTab);
  const loadAll = useSettingsStore((s) => s.loadAll);
  const setSetting = useSettingsStore((s) => s.setSetting);
  const saveSettings = useSettingsStore((s) => s.saveSettings);

  useEffect(() => {
    setActiveTab(initialTab);
    void loadAll();
  }, [initialTab, setActiveTab, loadAll]);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    void saveSettings();
  };

  if (loading) {
    return (
      <div className="p-8 text-center text-slate-500 text-xs">
        Chargement des réglages…
      </div>
    );
  }

  const showOnFront = settings['reading.show_on_front'] || 'posts';
  const blogPublic = settings['reading.blog_public'] !== false;
  const canRegister = settings['users.can_register'] === true;

  return (
    <form onSubmit={handleSubmit} className="space-y-6 max-w-4xl">
      {/* En-tête & bouton d'enregistrement */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h3 className="text-base font-bold text-white">{settingsMessages.title}</h3>
          <p className="text-xs text-slate-400">{settingsMessages.subtitle}</p>
        </div>

        <button
          type="submit"
          disabled={saving}
          className="px-4 py-2 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-semibold flex items-center gap-2 shadow-md shadow-indigo-600/20 transition-all"
        >
          {saved ? (
            <CheckCircle2 className="w-4 h-4 text-emerald-300" />
          ) : (
            <Save className="w-4 h-4" />
          )}
          <span>
            {saving
              ? settingsMessages.savingButton
              : saved
              ? settingsMessages.savedFeedback
              : settingsMessages.saveButton}
          </span>
        </button>
      </div>

      {/* Sous-onglets Général / Lecture / Écriture */}
      <div className="flex items-center gap-2 bg-[#0f1422] p-1 rounded-xl border border-white/5 w-fit">
        <button
          type="button"
          onClick={() => setActiveTab('general')}
          className={`px-3 py-1.5 rounded-lg text-xs font-medium flex items-center gap-1.5 transition-colors ${
            activeTab === 'general'
              ? 'bg-indigo-600 text-white'
              : 'text-slate-400 hover:text-white'
          }`}
        >
          <Globe className="w-3.5 h-3.5" />
          <span>{settingsMessages.tabGeneral}</span>
        </button>

        <button
          type="button"
          onClick={() => setActiveTab('reading')}
          className={`px-3 py-1.5 rounded-lg text-xs font-medium flex items-center gap-1.5 transition-colors ${
            activeTab === 'reading'
              ? 'bg-indigo-600 text-white'
              : 'text-slate-400 hover:text-white'
          }`}
        >
          <BookOpen className="w-3.5 h-3.5" />
          <span>{settingsMessages.tabReading}</span>
        </button>

        <button
          type="button"
          onClick={() => setActiveTab('writing')}
          className={`px-3 py-1.5 rounded-lg text-xs font-medium flex items-center gap-1.5 transition-colors ${
            activeTab === 'writing'
              ? 'bg-indigo-600 text-white'
              : 'text-slate-400 hover:text-white'
          }`}
        >
          <PenTool className="w-3.5 h-3.5" />
          <span>{settingsMessages.tabWriting}</span>
        </button>

        <button
          type="button"
          onClick={() => setActiveTab('permalinks')}
          className={`px-3 py-1.5 rounded-lg text-xs font-medium flex items-center gap-1.5 transition-colors ${
            activeTab === 'permalinks'
              ? 'bg-indigo-600 text-white'
              : 'text-slate-400 hover:text-white'
          }`}
        >
          <Link2 className="w-3.5 h-3.5" />
          <span>{settingsMessages.tabPermalinks}</span>
        </button>
      </div>

      {error && (
        <div
          role="alert"
          className="p-3.5 rounded-xl bg-rose-500/10 border border-rose-500/30 text-rose-300 text-xs flex items-center gap-2"
        >
          <AlertTriangle className="w-4 h-4 shrink-0" />
          <span>{error}</span>
        </div>
      )}

      {/* ONGLET 1 : GÉNÉRAL (`WP-69` à `WP-74`) */}
      {activeTab === 'general' && (
        <div className="space-y-6">
          <div className="p-6 rounded-2xl bg-[#0f1422] border border-white/5 space-y-4">
            <h4 className="text-xs font-bold text-white uppercase tracking-wider">
              Identité & Adresses du site (WP-69, WP-70, WP-71)
            </h4>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div>
                <label className="text-[11px] text-slate-400 block mb-1">Titre du site</label>
                <input
                  type="text"
                  value={settings['site.title'] || ''}
                  onChange={(e) => setSetting('site.title', e.target.value)}
                  className="w-full bg-black/30 border border-white/10 rounded-xl px-3 py-2 text-xs text-white focus:outline-none focus:border-indigo-500"
                />
              </div>

              <div>
                <label className="text-[11px] text-slate-400 block mb-1">Slogan</label>
                <input
                  type="text"
                  value={settings['site.tagline'] || ''}
                  onChange={(e) => setSetting('site.tagline', e.target.value)}
                  className="w-full bg-black/30 border border-white/10 rounded-xl px-3 py-2 text-xs text-white focus:outline-none focus:border-indigo-500"
                />
              </div>

              <div>
                <label className="text-[11px] text-slate-400 block mb-1">
                  Adresse publique du site (URL)
                </label>
                <input
                  type="url"
                  value={settings['site.url'] || ''}
                  onChange={(e) => setSetting('site.url', e.target.value)}
                  placeholder="https://exemple.fr"
                  className="w-full bg-black/30 border border-white/10 rounded-xl px-3 py-2 text-xs text-white focus:outline-none focus:border-indigo-500"
                />
              </div>

              <div>
                <label className="text-[11px] text-slate-400 block mb-1">
                  Adresse e-mail d’administration
                </label>
                <input
                  type="email"
                  value={settings['site.admin_email'] || ''}
                  onChange={(e) => setSetting('site.admin_email', e.target.value)}
                  placeholder="admin@exemple.fr"
                  className="w-full bg-black/30 border border-white/10 rounded-xl px-3 py-2 text-xs text-white focus:outline-none focus:border-indigo-500"
                />
              </div>
            </div>
          </div>

          <div className="p-6 rounded-2xl bg-[#0f1422] border border-white/5 space-y-4">
            <h4 className="text-xs font-bold text-white uppercase tracking-wider">
              Inscription, Langue & Fuseau horaire (WP-72, WP-73)
            </h4>
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
              <div>
                <label className="text-[11px] text-slate-400 block mb-1">
                  Langue principale du site
                </label>
                <select
                  value={settings['site.language'] || 'fr'}
                  onChange={(e) => setSetting('site.language', e.target.value)}
                  className="w-full bg-black/30 border border-white/10 rounded-xl px-3 py-2 text-xs text-white focus:outline-none focus:border-indigo-500"
                >
                  <option value="fr">Français (fr)</option>
                </select>
              </div>

              <div>
                <label className="text-[11px] text-slate-400 block mb-1">Fuseau horaire</label>
                <select
                  value={settings['site.timezone'] || 'Europe/Paris'}
                  onChange={(e) => setSetting('site.timezone', e.target.value)}
                  className="w-full bg-black/30 border border-white/10 rounded-xl px-3 py-2 text-xs text-white focus:outline-none focus:border-indigo-500"
                >
                  <option value="Europe/Paris">Europe/Paris (UTC+1/+2)</option>
                  <option value="UTC">UTC</option>
                  <option value="America/Montreal">America/Montreal</option>
                  <option value="Europe/Brussels">Europe/Brussels</option>
                  <option value="Europe/Zurich">Europe/Zurich</option>
                </select>
              </div>

              <div>
                <label className="text-[11px] text-slate-400 block mb-1">
                  Rôle par défaut des nouveaux inscrits
                </label>
                <select
                  value={settings['users.default_role'] || 'subscriber'}
                  onChange={(e) => setSetting('users.default_role', e.target.value)}
                  className="w-full bg-black/30 border border-white/10 rounded-xl px-3 py-2 text-xs text-white focus:outline-none focus:border-indigo-500"
                >
                  <option value="subscriber">Abonné / Lecteur</option>
                  <option value="author">Auteur</option>
                  <option value="editor">Éditeur</option>
                </select>
              </div>
            </div>

            <label className="flex items-center gap-2.5 text-xs text-slate-300 pt-2 cursor-pointer">
              <input
                type="checkbox"
                checked={canRegister}
                onChange={(e) => setSetting('users.can_register', e.target.checked)}
                className="rounded border-white/20 bg-black/30 text-indigo-600 focus:ring-indigo-500"
              />
              <span>Autoriser l’inscription publique de nouveaux membres (WP-72)</span>
            </label>
          </div>
        </div>
      )}

      {/* ONGLET 2 : LECTURE & ACCUEIL (`WP-78` à `WP-81`) */}
      {activeTab === 'reading' && (
        <div className="p-6 rounded-2xl bg-[#0f1422] border border-white/5 space-y-5">
          <h4 className="text-xs font-bold text-white uppercase tracking-wider">
            Affichage de la page d’accueil & Pagination (WP-78, WP-79, WP-81)
          </h4>

          <div className="space-y-3">
            <label className="text-[11px] text-slate-400 block">
              La page d’accueil affiche :
            </label>
            <div className="flex flex-col sm:flex-row gap-4 text-xs text-slate-200">
              <label className="flex items-center gap-2 cursor-pointer">
                <input
                  type="radio"
                  name="show_on_front"
                  checked={showOnFront === 'posts'}
                  onChange={() => setSetting('reading.show_on_front', 'posts')}
                />
                <span>Les derniers articles publiés</span>
              </label>
              <label className="flex items-center gap-2 cursor-pointer">
                <input
                  type="radio"
                  name="show_on_front"
                  checked={showOnFront === 'page'}
                  onChange={() => setSetting('reading.show_on_front', 'page')}
                />
                <span>Une page statique dédiée</span>
              </label>
            </div>
          </div>

          {showOnFront === 'page' && (
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 pt-2">
              <div>
                <label className="text-[11px] text-slate-400 block mb-1">
                  Page d’accueil statique
                </label>
                <select
                  value={settings['reading.page_on_front'] || ''}
                  onChange={(e) => setSetting('reading.page_on_front', e.target.value)}
                  className="w-full bg-black/30 border border-white/10 rounded-xl px-3 py-2 text-xs text-white focus:outline-none focus:border-indigo-500"
                >
                  <option value="">— Sélectionner une page publiée —</option>
                  {settings['reading.page_on_front'] && !pages.some((p) => p.id === settings['reading.page_on_front']) && (
                    <option disabled value={settings['reading.page_on_front']}>{settingsMessages.unavailablePage}</option>
                  )}
                  {pages.map((p) => (
                    <option key={p.id} value={p.id}>
                      {p.title} (/{p.slug})
                    </option>
                  ))}
                </select>
              </div>
              <div>
                <label htmlFor="rf-posts-page" className="text-[11px] text-slate-400 block mb-1">{settingsMessages.postsPage}</label>
                <select id="rf-posts-page" value={settings['reading.page_for_posts'] || ''}
                  onChange={(e) => setSetting('reading.page_for_posts', e.target.value)}
                  className="w-full bg-black/30 border border-white/10 rounded-xl px-3 py-2 text-xs text-white">
                  <option value="">{settingsMessages.noPostsPage}</option>
                  {settings['reading.page_for_posts'] && (!pages.some((p) => p.id === settings['reading.page_for_posts']) || settings['reading.page_for_posts'] === settings['reading.page_on_front']) && (
                    <option disabled value={settings['reading.page_for_posts']}>{settingsMessages.unavailablePage}</option>
                  )}
                  {pages.filter((p) => p.id !== settings['reading.page_on_front']).map((p) => <option key={p.id} value={p.id}>{p.title}</option>)}
                </select>
                <p className="text-xs text-slate-400 mt-2">{settingsMessages.postsPageHint}</p>
              </div>
            </div>
          )}

          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 pt-2">
            <div>
              <label className="text-[11px] text-slate-400 block mb-1">
                Nombre maximal d?articles par page publique (1 ? 100)
              </label>
              <input
                type="number"
                min={1}
                max={100}
                value={settings['reading.per_page'] ?? 10}
                onChange={(e) =>
                  setSetting('reading.per_page', Math.max(1, Math.min(100, Number(e.target.value) || 10)))
                }
                className="w-full bg-black/30 border border-white/10 rounded-xl px-3 py-2 text-xs text-white focus:outline-none focus:border-indigo-500"
              />
            </div>

            <div>
              <label className="text-[11px] text-slate-400 block mb-1">
                {settingsMessages.postsPerRss}
              </label>
              <input
                type="number"
                min={1}
                max={100}
                value={settings['reading.posts_per_rss'] ?? 10}
                onChange={(e) =>
                  setSetting('reading.posts_per_rss', Math.max(1, Math.min(100, Number(e.target.value) || 10)))
                }
                className="w-full bg-black/30 border border-white/10 rounded-xl px-3 py-2 text-xs text-white focus:outline-none focus:border-indigo-500"
              />
            </div>
          </div>

          <div className="pt-2">
            <label className="text-[11px] text-slate-400 block mb-1">
              {settingsMessages.rssUseExcerpt}
            </label>
            <select
              value={settings['reading.rss_use_excerpt'] ? 'excerpt' : 'full'}
              onChange={(e) =>
                setSetting('reading.rss_use_excerpt', e.target.value === 'excerpt')
              }
              className="w-full sm:w-1/2 bg-black/30 border border-white/10 rounded-xl px-3 py-2 text-xs text-white focus:outline-none focus:border-indigo-500"
            >
              <option value="full">{settingsMessages.rssFull}</option>
              <option value="excerpt">{settingsMessages.rssExcerpt}</option>
            </select>
          </div>

          <div className="pt-2 border-t border-white/5">
            <label className="flex items-center gap-2.5 text-xs text-slate-300 cursor-pointer">
              <input
                type="checkbox"
                checked={!blogPublic}
                onChange={(e) => setSetting('reading.blog_public', !e.target.checked)}
                className="rounded border-white/20 bg-black/30 text-indigo-600 focus:ring-indigo-500"
              />
              <span>
                Demander aux moteurs de recherche de ne pas indexer ce site (ajoute{' '}
                <code className="text-amber-300">noindex, nofollow</code>)
              </span>
            </label>
          </div>
        </div>
      )}

      {/* ONGLET 3 : ÉCRITURE & DÉFAUTS (`WP-75`) */}
      {activeTab === 'writing' && (
        <div className="p-6 rounded-2xl bg-[#0f1422] border border-white/5 space-y-4">
          <h4 className="text-xs font-bold text-white uppercase tracking-wider">
            Options de rédaction par défaut (WP-75)
          </h4>

          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div>
              <label className="text-[11px] text-slate-400 block mb-1">
                Catégorie par défaut des nouveaux articles
              </label>
              <select
                value={settings['writing.default_category'] || 'non-classe'}
                onChange={(e) => setSetting('writing.default_category', e.target.value)}
                className="w-full bg-black/30 border border-white/10 rounded-xl px-3 py-2 text-xs text-white focus:outline-none focus:border-indigo-500"
              >
                {categories.map((cat) => (
                  <option key={cat.id} value={cat.slug}>
                    {cat.name} ({cat.slug})
                  </option>
                ))}
              </select>
            </div>

            <div>
              <label className="text-[11px] text-slate-400 block mb-1">
                Mode d’édition par défaut
              </label>
              <select
                value={settings['writing.default_editor_mode'] || 'blocks'}
                onChange={(e) => setSetting('writing.default_editor_mode', e.target.value)}
                className="w-full bg-black/30 border border-white/10 rounded-xl px-3 py-2 text-xs text-white focus:outline-none focus:border-indigo-500"
              >
                <option value="blocks">Éditeur visuel par blocs (rf.document/1)</option>
                <option value="markdown">Source Markdown</option>
              </select>
            </div>
          </div>
        </div>
      )}

      {/* ONGLET 4 : PERMALIENS & URLS (`WP-90`, `WP-91`) */}
      {activeTab === 'permalinks' && (
        <div className="space-y-6">
          <div className="p-6 rounded-2xl bg-[#0f1422] border border-white/5 space-y-4">
            <div>
              <h4 className="text-xs font-bold text-white uppercase tracking-wider">
                {settingsMessages.permalinksTitle} (WP-90)
              </h4>
              <p className="text-[11px] text-slate-400 mt-0.5">
                {settingsMessages.permalinksSubtitle}
              </p>
            </div>

            <div className="space-y-3 pt-2">
              {[
                { id: 'plain', label: settingsMessages.permalinkPlain, example: '/fr/post/article-exemple' },
                { id: 'day_and_name', label: settingsMessages.permalinkDayAndName, example: '/fr/2026/10/28/article-exemple' },
                { id: 'month_and_name', label: settingsMessages.permalinkMonthAndName, example: '/fr/2026/10/article-exemple' },
                { id: 'numeric', label: settingsMessages.permalinkNumeric, example: '/fr/archives/a0000000-0000-0000-0000-000000000001' },
                { id: 'post_name', label: settingsMessages.permalinkPostName, example: '/fr/post/article-exemple' },
              ].map((opt) => (
                <label
                  key={opt.id}
                  className="flex items-start gap-3 p-3 rounded-xl bg-black/20 border border-white/5 hover:border-white/15 cursor-pointer transition-colors"
                >
                  <input
                    type="radio"
                    name="permalink_structure"
                    value={opt.id}
                    checked={(settings['permalink.structure'] || 'plain') === opt.id}
                    onChange={() => setSetting('permalink.structure', opt.id)}
                    className="mt-0.5 text-indigo-600 focus:ring-indigo-500"
                  />
                  <div className="flex-1 min-w-0">
                    <span className="text-xs font-medium text-white block">{opt.label}</span>
                    <code className="text-[11px] text-slate-400 font-mono block mt-0.5 truncate">
                      {opt.example}
                    </code>
                  </div>
                </label>
              ))}

              {/* Structure personnalisée */}
              <div className="p-3 rounded-xl bg-black/20 border border-white/5 space-y-2">
                <label className="flex items-center gap-3 cursor-pointer">
                  <input
                    type="radio"
                    name="permalink_structure"
                    value="custom"
                    checked={
                      Boolean(settings['permalink.structure']) &&
                      !['plain', 'day_and_name', 'month_and_name', 'numeric', 'post_name'].includes(
                        settings['permalink.structure']
                      )
                    }
                    onChange={() => {
                      if (!settings['permalink.structure'] || ['plain', 'day_and_name', 'month_and_name', 'numeric', 'post_name'].includes(settings['permalink.structure'])) {
                        setSetting('permalink.structure', '/%year%/%monthnum%/%postname%/');
                      }
                    }}
                    className="text-indigo-600 focus:ring-indigo-500"
                  />
                  <span className="text-xs font-medium text-white">{settingsMessages.permalinkCustom}</span>
                </label>

                {Boolean(settings['permalink.structure']) &&
                  !['plain', 'day_and_name', 'month_and_name', 'numeric', 'post_name'].includes(
                    settings['permalink.structure']
                  ) && (
                    <div className="pl-6 space-y-2 pt-1">
                      <input
                        type="text"
                        value={settings['permalink.structure'] || ''}
                        onChange={(e) => setSetting('permalink.structure', e.target.value)}
                        placeholder={settingsMessages.permalinkCustomPlaceholder}
                        className="w-full bg-black/40 border border-white/10 rounded-xl px-3 py-2 text-xs font-mono text-white focus:outline-none focus:border-indigo-500"
                      />
                      <div className="flex flex-wrap items-center gap-1.5 pt-1">
                        <span className="text-[10px] text-slate-400 mr-1">{settingsMessages.permalinkAvailableTags}</span>
                        {['%year%', '%monthnum%', '%day%', '%postname%', '%post_id%', '%category%', '%author%'].map(
                          (tag) => (
                            <button
                              key={tag}
                              type="button"
                              onClick={() => {
                                const cur = settings['permalink.structure'] || '';
                                if (!cur.includes(tag)) {
                                  setSetting('permalink.structure', cur.replace(/\/$/, '') + '/' + tag + '/');
                                }
                              }}
                              className="px-2 py-0.5 rounded bg-white/5 hover:bg-white/10 text-[10px] font-mono text-indigo-300 border border-white/10 transition-colors"
                            >
                              {tag}
                            </button>
                          )
                        )}
                      </div>
                    </div>
                  )}
              </div>
            </div>
          </div>

          {/* Préfixes des taxonomies (`WP-91`) */}
          <div className="p-6 rounded-2xl bg-[#0f1422] border border-white/5 space-y-4">
            <div>
              <h4 className="text-xs font-bold text-white uppercase tracking-wider">
                {settingsMessages.taxonomyBasesTitle} (WP-91)
              </h4>
              <p className="text-[11px] text-slate-400 mt-0.5">
                {settingsMessages.taxonomyBasesSubtitle}
              </p>
            </div>

            <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 pt-1">
              <div>
                <label className="text-[11px] text-slate-400 block mb-1">
                  {settingsMessages.categoryBaseLabel}
                </label>
                <input
                  type="text"
                  value={settings['permalink.category_base'] || ''}
                  onChange={(e) => setSetting('permalink.category_base', e.target.value)}
                  placeholder={settingsMessages.categoryBasePlaceholder}
                  className="w-full bg-black/30 border border-white/10 rounded-xl px-3 py-2 text-xs font-mono text-white focus:outline-none focus:border-indigo-500"
                />
              </div>

              <div>
                <label className="text-[11px] text-slate-400 block mb-1">
                  {settingsMessages.tagBaseLabel}
                </label>
                <input
                  type="text"
                  value={settings['permalink.tag_base'] || ''}
                  onChange={(e) => setSetting('permalink.tag_base', e.target.value)}
                  placeholder={settingsMessages.tagBasePlaceholder}
                  className="w-full bg-black/30 border border-white/10 rounded-xl px-3 py-2 text-xs font-mono text-white focus:outline-none focus:border-indigo-500"
                />
              </div>
            </div>
          </div>
        </div>
      )}
    </form>
  );
};
