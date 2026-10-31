import React, { useState, useEffect, useCallback } from 'react';
import { api } from '../api';
import { translationsMessages as msg } from '../i18n/messages.fr';
import type {
  LanguagesConfig,
  TranslationMatrixRow,
  TranslationCatalog,
  CatalogItem,
} from '../types';

interface TranslationsViewProps {
  initialTab?: 'matrix' | 'catalogs' | 'settings';
  onEditEntry?: (id: string) => void;
}

export const TranslationsView: React.FC<TranslationsViewProps> = ({
  initialTab = 'matrix',
  onEditEntry,
}) => {
  const [activeTab, setActiveTab] = useState<'matrix' | 'catalogs' | 'settings'>(initialTab);
  const [loading, setLoading] = useState(false);
  const [toast, setToast] = useState<{ type: 'success' | 'error'; message: string } | null>(null);

  // Configuration des langues
  const [languages, setLanguages] = useState<LanguagesConfig>({
    available: [],
    active: [],
    default: 'fr',
  });
  const [selectedDefaultLang, setSelectedDefaultLang] = useState('fr');
  const [selectedActiveCodes, setSelectedActiveCodes] = useState<string[]>(['fr']);

  // Matrice de traduction
  const [matrixItems, setMatrixItems] = useState<TranslationMatrixRow[]>([]);
  const [matrixTotal, setMatrixTotal] = useState(0);
  const [matrixPage, setMatrixPage] = useState(1);
  const [contentTypeFilter, setContentTypeFilter] = useState('');
  const [matrixSearch, setMatrixSearch] = useState('');
  const [translatingId, setTranslatingId] = useState<string | null>(null);

  // Éditeur de catalogues
  const [selectedCatalogLocale, setSelectedCatalogLocale] = useState('en');
  const [catalog, setCatalog] = useState<TranslationCatalog | null>(null);
  const [catalogSearch, setCatalogSearch] = useState('');
  const [catalogFilter, setCatalogFilter] = useState<'all' | 'untranslated' | 'translated'>('all');
  const [savingCatalog, setSavingCatalog] = useState(false);
  const [showImportModal, setShowImportModal] = useState(false);
  const [importContent, setImportContent] = useState('');
  const [importFormat, setImportFormat] = useState<'po' | 'json'>('po');

  const showToast = (type: 'success' | 'error', message: string) => {
    setToast({ type, message });
    setTimeout(() => setToast(null), 4000);
  };

  // Chargement des langues
  const loadLanguages = useCallback(() => {
    api.translations
      .getLanguages()
      .then((cfg) => {
        setLanguages(cfg);
        setSelectedDefaultLang(cfg.default);
        setSelectedActiveCodes(cfg.active.map((l) => l.code));
        if (cfg.active.length > 0 && !cfg.active.some((l) => l.code === selectedCatalogLocale)) {
          setSelectedCatalogLocale(cfg.active[0].code);
        }
      })
      .catch((err: unknown) => {
        showToast('error', err instanceof Error ? err.message : 'Erreur de chargement des langues');
      });
  }, [selectedCatalogLocale]);

  // Chargement de la matrice
  const loadMatrix = useCallback(() => {
    setLoading(true);
    api.translations
      .getMatrix({
        content_type: contentTypeFilter || undefined,
        q: matrixSearch || undefined,
        page: matrixPage,
        per_page: 15,
      })
      .then((res) => {
        setMatrixItems(res.items);
        setMatrixTotal(res.total);
      })
      .catch((err: unknown) => {
        showToast('error', err instanceof Error ? err.message : 'Erreur de chargement de la matrice');
      })
      .finally(() => setLoading(false));
  }, [contentTypeFilter, matrixSearch, matrixPage]);



  useEffect(() => {
    loadLanguages();
  }, [loadLanguages]);

  useEffect(() => {
    let active = true;
    if (activeTab === 'matrix') {
      api.translations
        .getMatrix({
          content_type: contentTypeFilter || undefined,
          q: matrixSearch || undefined,
          page: matrixPage,
          per_page: 15,
        })
        .then((res) => {
          if (active) {
            setMatrixItems(res.items);
            setMatrixTotal(res.total);
          }
        })
        .catch((err: unknown) => {
          if (active) {
            showToast('error', err instanceof Error ? err.message : 'Erreur de chargement de la matrice');
          }
        });
    } else if (activeTab === 'catalogs') {
      api.translations
        .getCatalog(selectedCatalogLocale)
        .then((cat) => {
          if (active) {
            setCatalog(cat);
          }
        })
        .catch((err: unknown) => {
          if (active) {
            showToast('error', err instanceof Error ? err.message : 'Erreur de chargement du catalogue');
          }
        });
    }
    return () => {
      active = false;
    };
  }, [activeTab, contentTypeFilter, matrixSearch, matrixPage, selectedCatalogLocale]);

  // Action : Créer une traduction pour une entrée
  const handleCreateTranslation = (sourceId: string, targetLocale: string) => {
    setTranslatingId(`${sourceId}-${targetLocale}`);
    api.translations
      .createTranslation(sourceId, targetLocale)
      .then((newEntry) => {
        showToast('success', `Traduction en '${targetLocale.toUpperCase()}' créée avec succès.`);
        loadMatrix();
        if (onEditEntry) {
          onEditEntry(newEntry.id);
        }
      })
      .catch((err: unknown) => {
        showToast('error', err instanceof Error ? err.message : 'Échec de la traduction');
      })
      .finally(() => setTranslatingId(null));
  };

  // Action : Délier une entrée
  const handleUnlink = (entryId: string) => {
    if (!window.confirm(msg.confirmUnlink)) return;
    api.translations
      .unlink(entryId)
      .then(() => {
        showToast('success', 'Entrée déliée avec succès.');
        loadMatrix();
      })
      .catch((err: unknown) => {
        showToast('error', err instanceof Error ? err.message : 'Échec de la déliaison');
      });
  };

  // Action : Sauvegarder les langues
  const handleSaveLanguages = () => {
    setLoading(true);
    api.translations
      .updateLanguages({
        active: selectedActiveCodes,
        default: selectedDefaultLang,
      })
      .then((cfg) => {
        setLanguages(cfg);
        showToast('success', msg.languagesSavedSuccess);
      })
      .catch((err: unknown) => {
        showToast('error', err instanceof Error ? err.message : 'Échec de la sauvegarde');
      })
      .finally(() => setLoading(false));
  };

  // Action : Sauvegarder le catalogue
  const handleSaveCatalog = () => {
    if (!catalog) return;
    setSavingCatalog(true);
    api.translations
      .updateCatalog(catalog.locale, catalog.items)
      .then((updated) => {
        setCatalog(updated);
        showToast('success', msg.catalogSavedSuccess);
      })
      .catch((err: unknown) => {
        showToast('error', err instanceof Error ? err.message : 'Échec de la sauvegarde');
      })
      .finally(() => setSavingCatalog(false));
  };

  // Action : Synchroniser avec le socle
  const handleSyncCore = () => {
    if (!catalog) return;
    setLoading(true);
    api.translations
      .syncCatalog(catalog.locale)
      .then((synced) => {
        setCatalog(synced);
        showToast('success', msg.syncSuccess);
      })
      .catch((err: unknown) => {
        showToast('error', err instanceof Error ? err.message : 'Échec de la synchronisation');
      })
      .finally(() => setLoading(false));
  };

  // Action : Importer un fichier PO
  const handleImportSubmit = () => {
    if (!importContent.trim()) return;
    setLoading(true);
    api.translations
      .importCatalog(selectedCatalogLocale, importContent, importFormat)
      .then((imported) => {
        setCatalog(imported);
        setShowImportModal(false);
        setImportContent('');
        showToast('success', msg.importSuccess);
      })
      .catch((err: unknown) => {
        showToast('error', err instanceof Error ? err.message : 'Échec de l’import');
      })
      .finally(() => setLoading(false));
  };

  // Filtrage du catalogue dans la vue
  const filteredCatalogItems = (catalog?.items || []).filter((item: CatalogItem) => {
    if (catalogFilter === 'untranslated' && item.msgstr.trim() !== '') return false;
    if (catalogFilter === 'translated' && item.msgstr.trim() === '') return false;
    if (catalogSearch.trim()) {
      const q = catalogSearch.toLowerCase();
      const inId = item.msgid.toLowerCase().includes(q);
      const inStr = item.msgstr.toLowerCase().includes(q);
      const inCtx = item.context?.toLowerCase().includes(q);
      return inId || inStr || inCtx;
    }
    return true;
  });

  const translatedCount = (catalog?.items || []).filter((i: CatalogItem) => i.msgstr.trim() !== '').length;
  const totalCatalogCount = catalog?.items.length || 0;
  const progressPct = totalCatalogCount > 0 ? Math.round((translatedCount / totalCatalogCount) * 100) : 0;

  return (
    <div className="space-y-6">
      {/* En-tête */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4 pb-4 border-b border-neutral-200 dark:border-neutral-800">
        <div>
          <div className="flex items-center gap-3">
            <h1 className="text-2xl font-bold tracking-tight text-neutral-900 dark:text-neutral-100">
              {msg.title}
            </h1>
            <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-indigo-100 text-indigo-800 dark:bg-indigo-950 dark:text-indigo-300">
              Souverain GNU Gettext & Grappes
            </span>
          </div>
          <p className="mt-1 text-sm text-neutral-500 dark:text-neutral-400">
            {msg.subtitle}
          </p>
        </div>

        {/* Onglets */}
        <div className="flex items-center bg-neutral-100 dark:bg-neutral-800 p-1 rounded-lg">
          <button
            onClick={() => setActiveTab('matrix')}
            className={`px-3 py-1.5 text-xs font-medium rounded-md transition-colors ${
              activeTab === 'matrix'
                ? 'bg-white dark:bg-neutral-900 text-neutral-900 dark:text-neutral-100 shadow-sm'
                : 'text-neutral-600 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-200'
            }`}
          >
            {msg.tabMatrix}
          </button>
          <button
            onClick={() => setActiveTab('catalogs')}
            className={`px-3 py-1.5 text-xs font-medium rounded-md transition-colors ${
              activeTab === 'catalogs'
                ? 'bg-white dark:bg-neutral-900 text-neutral-900 dark:text-neutral-100 shadow-sm'
                : 'text-neutral-600 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-200'
            }`}
          >
            {msg.tabCatalogs}
          </button>
          <button
            onClick={() => setActiveTab('settings')}
            className={`px-3 py-1.5 text-xs font-medium rounded-md transition-colors ${
              activeTab === 'settings'
                ? 'bg-white dark:bg-neutral-900 text-neutral-900 dark:text-neutral-100 shadow-sm'
                : 'text-neutral-600 dark:text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-200'
            }`}
          >
            {msg.tabSettings}
          </button>
        </div>
      </div>

      {/* Toast Notification */}
      {toast && (
        <div
          className={`p-3 rounded-lg text-sm flex items-center justify-between ${
            toast.type === 'success'
              ? 'bg-emerald-50 text-emerald-800 border border-emerald-200 dark:bg-emerald-950/40 dark:text-emerald-300 dark:border-emerald-800'
              : 'bg-rose-50 text-rose-800 border border-rose-200 dark:bg-rose-950/40 dark:text-rose-300 dark:border-rose-800'
          }`}
        >
          <span>{toast.message}</span>
          <button onClick={() => setToast(null)} className="ml-3 font-semibold hover:opacity-75">
            ✕
          </button>
        </div>
      )}

      {/* =========================================================================
          ONGLET 1 : MATRICE DES CONTENUS (MOD-I18N-07)
      ========================================================================= */}
      {activeTab === 'matrix' && (
        <div className="space-y-4">
          {/* Barre de filtre */}
          <div className="flex flex-col sm:flex-row gap-3 items-center justify-between bg-white dark:bg-neutral-900 p-4 rounded-xl border border-neutral-200 dark:border-neutral-800">
            <div className="flex flex-1 items-center gap-3 w-full">
              <input
                type="text"
                placeholder={msg.matrixSearchPlaceholder}
                value={matrixSearch}
                onChange={(e) => {
                  setMatrixSearch(e.target.value);
                  setMatrixPage(1);
                }}
                className="w-full max-w-sm px-3 py-2 text-sm bg-neutral-50 dark:bg-neutral-800 border border-neutral-200 dark:border-neutral-700 rounded-lg focus:outline-none focus:ring-2 focus:ring-indigo-500"
              />
              <select
                value={contentTypeFilter}
                onChange={(e) => {
                  setContentTypeFilter(e.target.value);
                  setMatrixPage(1);
                }}
                className="px-3 py-2 text-sm bg-neutral-50 dark:bg-neutral-800 border border-neutral-200 dark:border-neutral-700 rounded-lg focus:outline-none focus:ring-2 focus:ring-indigo-500"
              >
                <option value="">{msg.allTypes}</option>
                <option value="post">Articles (post)</option>
                <option value="page">Pages (page)</option>
              </select>
            </div>
            <button
              onClick={loadMatrix}
              disabled={loading}
              className="px-3 py-2 text-xs font-medium text-neutral-700 dark:text-neutral-300 bg-neutral-100 dark:bg-neutral-800 hover:bg-neutral-200 dark:hover:bg-neutral-700 rounded-lg transition-colors"
            >
              {loading ? 'Actualisation…' : 'Actualiser'}
            </button>
          </div>

          {/* Tableau de la matrice */}
          <div className="bg-white dark:bg-neutral-900 rounded-xl border border-neutral-200 dark:border-neutral-800 overflow-hidden shadow-sm">
            <div className="overflow-x-auto">
              <table className="w-full text-left text-sm">
                <thead className="bg-neutral-50 dark:bg-neutral-800/60 border-b border-neutral-200 dark:border-neutral-800 text-xs text-neutral-500 dark:text-neutral-400 uppercase">
                  <tr>
                    <th className="py-3 px-4">Publication originale</th>
                    <th className="py-3 px-4">Type</th>
                    <th className="py-3 px-4">Langue source</th>
                    {languages.active.map((lang) => (
                      <th key={lang.code} className="py-3 px-4 text-center">
                        <span className="inline-flex items-center gap-1 font-semibold">
                          {lang.label} ({lang.code.toUpperCase()})
                          {lang.is_rtl && (
                            <span className="text-[10px] px-1 bg-amber-100 text-amber-800 dark:bg-amber-950 dark:text-amber-300 rounded">
                              RTL
                            </span>
                          )}
                        </span>
                      </th>
                    ))}
                    <th className="py-3 px-4 text-right">Grappe</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-neutral-200 dark:divide-neutral-800">
                  {matrixItems.length === 0 ? (
                    <tr>
                      <td
                        colSpan={languages.active.length + 4}
                        className="py-8 text-center text-neutral-500 dark:text-neutral-400"
                      >
                        Aucun contenu correspondant.
                      </td>
                    </tr>
                  ) : (
                    matrixItems.map((item) => (
                      <tr key={item.id} className="hover:bg-neutral-50/50 dark:hover:bg-neutral-800/40">
                        <td className="py-3 px-4">
                          <button
                            onClick={() => onEditEntry && onEditEntry(item.id)}
                            className="font-medium text-neutral-900 dark:text-neutral-100 hover:text-indigo-600 dark:hover:text-indigo-400 transition-colors text-left"
                          >
                            {item.title}
                          </button>
                          <div className="text-xs text-neutral-400 font-mono mt-0.5">
                            /{item.slug}
                          </div>
                        </td>
                        <td className="py-3 px-4">
                          <span className="text-xs px-2 py-0.5 rounded bg-neutral-100 dark:bg-neutral-800 text-neutral-700 dark:text-neutral-300">
                            {item.content_type}
                          </span>
                        </td>
                        <td className="py-3 px-4">
                          <span className="font-mono text-xs px-2 py-0.5 rounded bg-neutral-200 dark:bg-neutral-700 text-neutral-800 dark:text-neutral-200 uppercase font-semibold">
                            {item.locale}
                          </span>
                        </td>
                        {languages.active.map((lang) => {
                          const sibling = item.translations.find((s) => s.locale === lang.code);
                          const isSource = item.locale === lang.code;
                          const isTranslating = translatingId === `${item.id}-${lang.code}`;

                          return (
                            <td key={lang.code} className="py-3 px-4 text-center">
                              {sibling ? (
                                <button
                                  onClick={() => onEditEntry && onEditEntry(sibling.id)}
                                  className={`inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium transition-colors ${
                                    sibling.status === 'published'
                                      ? 'bg-emerald-100 text-emerald-800 dark:bg-emerald-950/60 dark:text-emerald-300 hover:bg-emerald-200'
                                      : 'bg-amber-100 text-amber-800 dark:bg-amber-950/60 dark:text-amber-300 hover:bg-amber-200'
                                  }`}
                                  title={`Ouvrir la traduction (${sibling.title})`}
                                >
                                  <span>✓ {sibling.locale.toUpperCase()}</span>
                                  <span className="text-[10px] opacity-75">
                                    {sibling.status === 'published' ? 'Publié' : 'Brouillon'}
                                  </span>
                                </button>
                              ) : isSource ? (
                                <span className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-neutral-100 dark:bg-neutral-800 text-neutral-600 dark:text-neutral-400">
                                  Source
                                </span>
                              ) : (
                                <button
                                  onClick={() => handleCreateTranslation(item.id, lang.code)}
                                  disabled={isTranslating}
                                  className="inline-flex items-center gap-1 px-2.5 py-1 rounded-full text-xs font-medium bg-indigo-50 text-indigo-700 hover:bg-indigo-100 dark:bg-indigo-950/40 dark:text-indigo-300 dark:hover:bg-indigo-900/50 transition-colors disabled:opacity-50"
                                >
                                  {isTranslating ? '…' : `+ Traduire (${lang.code.toUpperCase()})`}
                                </button>
                              )}
                            </td>
                          );
                        })}
                        <td className="py-3 px-4 text-right">
                          {item.translation_group_id ? (
                            <button
                              onClick={() => handleUnlink(item.id)}
                              className="text-xs text-rose-600 dark:text-rose-400 hover:underline"
                              title="Détacher de la grappe de traduction"
                            >
                              Délier
                            </button>
                          ) : (
                            <span className="text-xs text-neutral-400">Indépendant</span>
                          )}
                        </td>
                      </tr>
                    ))
                  )}
                </tbody>
              </table>
            </div>

            {/* Pagination */}
            {matrixTotal > 15 && (
              <div className="flex items-center justify-between px-4 py-3 border-t border-neutral-200 dark:border-neutral-800 bg-neutral-50 dark:bg-neutral-800/40 text-xs text-neutral-500">
                <span>
                  Total : {matrixTotal} publications
                </span>
                <div className="flex gap-2">
                  <button
                    disabled={matrixPage <= 1}
                    onClick={() => setMatrixPage((p) => Math.max(1, p - 1))}
                    className="px-2.5 py-1 rounded bg-white dark:bg-neutral-700 border border-neutral-200 dark:border-neutral-600 disabled:opacity-50"
                  >
                    Précédent
                  </button>
                  <span className="self-center font-medium text-neutral-700 dark:text-neutral-300">
                    Page {matrixPage} / {Math.ceil(matrixTotal / 15)}
                  </span>
                  <button
                    disabled={matrixPage >= Math.ceil(matrixTotal / 15)}
                    onClick={() => setMatrixPage((p) => p + 1)}
                    className="px-2.5 py-1 rounded bg-white dark:bg-neutral-700 border border-neutral-200 dark:border-neutral-600 disabled:opacity-50"
                  >
                    Suivant
                  </button>
                </div>
              </div>
            )}
          </div>
        </div>
      )}

      {/* =========================================================================
          ONGLET 2 : ÉDITEUR DE CATALOGUES GNU GETTEXT (MOD-I18N-02 à 05)
      ========================================================================= */}
      {activeTab === 'catalogs' && (
        <div className="space-y-4">
          {/* Barre d'action supérieure */}
          <div className="flex flex-col sm:flex-row items-stretch sm:items-center justify-between gap-4 bg-white dark:bg-neutral-900 p-4 rounded-xl border border-neutral-200 dark:border-neutral-800">
            <div className="flex flex-wrap items-center gap-3">
              <label className="text-xs font-semibold text-neutral-500 dark:text-neutral-400 uppercase">
                {msg.selectLanguage}
              </label>
              <select
                value={selectedCatalogLocale}
                onChange={(e) => setSelectedCatalogLocale(e.target.value)}
                className="px-3 py-1.5 text-sm bg-neutral-50 dark:bg-neutral-800 border border-neutral-200 dark:border-neutral-700 rounded-lg focus:outline-none focus:ring-2 focus:ring-indigo-500 font-medium"
              >
                {languages.available.map((l) => (
                  <option key={l.code} value={l.code}>
                    {l.label} ({l.code.toUpperCase()})
                  </option>
                ))}
              </select>

              {/* Jauge de complétion */}
              <div className="flex items-center gap-2 pl-3 border-l border-neutral-200 dark:border-neutral-700">
                <div className="w-24 bg-neutral-200 dark:bg-neutral-700 rounded-full h-2">
                  <div
                    className="bg-indigo-600 h-2 rounded-full transition-all"
                    style={{ width: `${progressPct}%` }}
                  />
                </div>
                <span className="text-xs font-mono font-medium text-neutral-600 dark:text-neutral-300">
                  {translatedCount}/{totalCatalogCount} ({progressPct}%)
                </span>
              </div>
            </div>

            {/* Boutons d'import/export */}
            <div className="flex flex-wrap items-center gap-2">
              <a
                href={api.translations.exportPoUrl(selectedCatalogLocale)}
                download
                className="px-3 py-1.5 text-xs font-medium text-neutral-700 dark:text-neutral-300 bg-neutral-100 dark:bg-neutral-800 hover:bg-neutral-200 dark:hover:bg-neutral-700 rounded-lg transition-colors flex items-center gap-1.5"
              >
                📥 {msg.exportPo}
              </a>
              <button
                onClick={() => setShowImportModal(true)}
                className="px-3 py-1.5 text-xs font-medium text-neutral-700 dark:text-neutral-300 bg-neutral-100 dark:bg-neutral-800 hover:bg-neutral-200 dark:hover:bg-neutral-700 rounded-lg transition-colors flex items-center gap-1.5"
              >
                📤 {msg.importPo}
              </button>
              <button
                onClick={handleSyncCore}
                disabled={loading}
                className="px-3 py-1.5 text-xs font-medium text-indigo-700 dark:text-indigo-300 bg-indigo-50 dark:bg-indigo-950/40 hover:bg-indigo-100 rounded-lg transition-colors"
                title="Ajoute les nouvelles chaînes du socle sans écraser vos traductions"
              >
                🔄 {msg.syncCore}
              </button>
              <button
                onClick={handleSaveCatalog}
                disabled={savingCatalog}
                className="px-4 py-1.5 text-xs font-semibold text-white bg-indigo-600 hover:bg-indigo-700 rounded-lg shadow-sm transition-colors disabled:opacity-50"
              >
                {savingCatalog ? msg.savingCatalog : msg.saveCatalog}
              </button>
            </div>
          </div>

          {/* Filtres de recherche */}
          <div className="flex flex-col sm:flex-row gap-3 items-center justify-between bg-white dark:bg-neutral-900 p-3 rounded-xl border border-neutral-200 dark:border-neutral-800">
            <input
              type="text"
              placeholder={msg.searchKeysPlaceholder}
              value={catalogSearch}
              onChange={(e) => setCatalogSearch(e.target.value)}
              className="w-full max-w-md px-3 py-1.5 text-sm bg-neutral-50 dark:bg-neutral-800 border border-neutral-200 dark:border-neutral-700 rounded-lg focus:outline-none focus:ring-2 focus:ring-indigo-500"
            />
            <div className="flex items-center gap-2">
              <button
                onClick={() => setCatalogFilter('all')}
                className={`px-2.5 py-1 text-xs rounded-md font-medium transition-colors ${
                  catalogFilter === 'all'
                    ? 'bg-neutral-900 text-white dark:bg-neutral-100 dark:text-neutral-900'
                    : 'text-neutral-600 dark:text-neutral-400 hover:bg-neutral-100 dark:hover:bg-neutral-800'
                }`}
              >
                {msg.allStrings}
              </button>
              <button
                onClick={() => setCatalogFilter('untranslated')}
                className={`px-2.5 py-1 text-xs rounded-md font-medium transition-colors ${
                  catalogFilter === 'untranslated'
                    ? 'bg-amber-600 text-white'
                    : 'text-amber-600 dark:text-amber-400 hover:bg-amber-50 dark:hover:bg-amber-950/30'
                }`}
              >
                {msg.untranslatedOnly}
              </button>
              <button
                onClick={() => setCatalogFilter('translated')}
                className={`px-2.5 py-1 text-xs rounded-md font-medium transition-colors ${
                  catalogFilter === 'translated'
                    ? 'bg-emerald-600 text-white'
                    : 'text-emerald-600 dark:text-emerald-400 hover:bg-emerald-50 dark:hover:bg-emerald-950/30'
                }`}
              >
                {msg.translatedOnly}
              </button>
            </div>
          </div>

          {/* Liste des chaînes du catalogue */}
          <div className="bg-white dark:bg-neutral-900 rounded-xl border border-neutral-200 dark:border-neutral-800 overflow-hidden shadow-sm divide-y divide-neutral-200 dark:divide-neutral-800">
            {filteredCatalogItems.length === 0 ? (
              <div className="py-12 text-center text-sm text-neutral-500">
                Aucune chaîne trouvée pour ce filtre.
              </div>
            ) : (
              filteredCatalogItems.map((item, idx) => {
                const originalIndex = catalog?.items.findIndex(
                  (i) => i.msgid === item.msgid && i.context === item.context
                );
                return (
                  <div
                    key={`${item.msgid}-${idx}`}
                    className="p-4 grid grid-cols-1 lg:grid-cols-2 gap-4 hover:bg-neutral-50/50 dark:hover:bg-neutral-800/30"
                  >
                    {/* Colonne source (msgid) */}
                    <div className="space-y-1.5">
                      <div className="flex items-center gap-2">
                        {item.context && (
                          <span className="text-[10px] font-semibold uppercase tracking-wider px-2 py-0.5 rounded bg-indigo-50 text-indigo-700 dark:bg-indigo-950 dark:text-indigo-300">
                            {item.context}
                          </span>
                        )}
                        {item.comments && item.comments.length > 0 && (
                          <span className="text-xs text-neutral-400">
                            {item.comments.join(' • ')}
                          </span>
                        )}
                      </div>
                      <div className="text-sm font-medium text-neutral-900 dark:text-neutral-100 select-all">
                        {item.msgid}
                      </div>
                      {item.references && item.references.length > 0 && (
                        <div className="text-[11px] font-mono text-neutral-400">
                          {item.references.join(', ')}
                        </div>
                      )}
                    </div>

                    {/* Colonne cible (msgstr) */}
                    <div>
                      <input
                        type="text"
                        value={item.msgstr}
                        placeholder={`Traduction en ${selectedCatalogLocale.toUpperCase()}…`}
                        onChange={(e) => {
                          const val = e.target.value;
                          if (catalog && originalIndex !== undefined && originalIndex >= 0) {
                            const newItems = [...catalog.items];
                            newItems[originalIndex] = {
                              ...newItems[originalIndex],
                              msgstr: val,
                            };
                            setCatalog({ ...catalog, items: newItems });
                          }
                        }}
                        className={`w-full px-3 py-2 text-sm rounded-lg border focus:outline-none focus:ring-2 focus:ring-indigo-500 transition-colors ${
                          item.msgstr.trim() === ''
                            ? 'bg-amber-50/40 border-amber-200 dark:bg-amber-950/20 dark:border-amber-800'
                            : 'bg-white dark:bg-neutral-800 border-neutral-200 dark:border-neutral-700'
                        }`}
                      />
                    </div>
                  </div>
                );
              })
            )}
          </div>
        </div>
      )}

      {/* =========================================================================
          ONGLET 3 : CONFIGURATION DES LANGUES (MOD-I18N-01)
      ========================================================================= */}
      {activeTab === 'settings' && (
        <div className="space-y-6 max-w-3xl">
          {/* Bannière souveraine explicative */}
          <div className="p-4 rounded-xl bg-indigo-50/80 dark:bg-indigo-950/40 border border-indigo-100 dark:border-indigo-900 text-sm text-indigo-950 dark:text-indigo-200 space-y-2">
            <h3 className="font-semibold text-base flex items-center gap-2">
              <span>🌐</span> {msg.languagesSettingsTitle}
            </h3>
            <p className="text-xs leading-relaxed text-indigo-800 dark:text-indigo-300">
              {msg.languagesSettingsSubtitle}
            </p>
            <div className="grid grid-cols-1 md:grid-cols-3 gap-2 mt-2 pt-2 border-t border-indigo-200/50 dark:border-indigo-800/50 text-xs">
              <div className="p-2 bg-white/60 dark:bg-neutral-900/60 rounded">
                <strong>1. Interface UI :</strong> Dictionnaires Gettext PO/JSON sans altération du code.
              </div>
              <div className="p-2 bg-white/60 dark:bg-neutral-900/60 rounded">
                <strong>2. Grappes de contenus :</strong> Entrées distinctes liées par translation_group_id.
              </div>
              <div className="p-2 bg-white/60 dark:bg-neutral-900/60 rounded">
                <strong>3. Routage SEO :</strong> Négociation HTTP q-factor, préfixes /&#123;locale&#125;/ et balises hreflang.
              </div>
            </div>
          </div>

          {/* Formulaire des réglages */}
          <div className="bg-white dark:bg-neutral-900 p-6 rounded-xl border border-neutral-200 dark:border-neutral-800 space-y-6">
            {/* Langue principale par défaut */}
            <div>
              <label className="block text-sm font-semibold text-neutral-900 dark:text-neutral-100 mb-1">
                {msg.defaultLanguageLabel}
              </label>
              <select
                value={selectedDefaultLang}
                onChange={(e) => setSelectedDefaultLang(e.target.value)}
                className="w-full max-w-sm px-3 py-2 text-sm bg-neutral-50 dark:bg-neutral-800 border border-neutral-200 dark:border-neutral-700 rounded-lg focus:outline-none focus:ring-2 focus:ring-indigo-500 font-medium"
              >
                {languages.available.map((l) => (
                  <option key={l.code} value={l.code}>
                    {l.label} ({l.code.toUpperCase()})
                  </option>
                ))}
              </select>
            </div>

            {/* Langues actives multi-sélection */}
            <div>
              <label className="block text-sm font-semibold text-neutral-900 dark:text-neutral-100 mb-1">
                {msg.activeLanguagesLabel}
              </label>
              <p className="text-xs text-neutral-500 dark:text-neutral-400 mb-3">
                {msg.activeLanguagesHint}
              </p>
              <div className="grid grid-cols-2 sm:grid-cols-3 gap-3">
                {languages.available.map((lang) => {
                  const isChecked = selectedActiveCodes.includes(lang.code);
                  const isDefault = selectedDefaultLang === lang.code;

                  return (
                    <label
                      key={lang.code}
                      className={`flex items-center gap-3 p-3 rounded-lg border transition-all cursor-pointer ${
                        isChecked
                          ? 'bg-indigo-50/50 border-indigo-200 dark:bg-indigo-950/20 dark:border-indigo-800'
                          : 'bg-neutral-50 dark:bg-neutral-800/40 border-neutral-200 dark:border-neutral-700 opacity-60'
                      }`}
                    >
                      <input
                        type="checkbox"
                        checked={isChecked}
                        disabled={isDefault}
                        onChange={(e) => {
                          if (e.target.checked) {
                            setSelectedActiveCodes([...selectedActiveCodes, lang.code]);
                          } else {
                            if (isDefault) return;
                            setSelectedActiveCodes(selectedActiveCodes.filter((c) => c !== lang.code));
                          }
                        }}
                        className="rounded text-indigo-600 focus:ring-indigo-500"
                      />
                      <div className="text-xs">
                        <span className="font-medium text-neutral-900 dark:text-neutral-100">
                          {lang.label}
                        </span>
                        <div className="text-neutral-400 font-mono">
                          {lang.code.toUpperCase()}
                          {lang.is_rtl && ' • RTL'}
                          {isDefault && ' (défaut)'}
                        </div>
                      </div>
                    </label>
                  );
                })}
              </div>
            </div>

            {/* Bouton de sauvegarde */}
            <div className="pt-4 border-t border-neutral-200 dark:border-neutral-800">
              <button
                onClick={handleSaveLanguages}
                disabled={loading}
                className="px-5 py-2.5 text-sm font-semibold text-white bg-indigo-600 hover:bg-indigo-700 rounded-lg shadow-sm transition-colors disabled:opacity-50"
              >
                {loading ? msg.savingLanguages : msg.saveLanguagesBtn}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Modal d'importation PO/JSON */}
      {showImportModal && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/50 backdrop-blur-sm">
          <div className="bg-white dark:bg-neutral-900 rounded-2xl max-w-xl w-full p-6 shadow-2xl border border-neutral-200 dark:border-neutral-800 space-y-4">
            <div className="flex items-center justify-between">
              <h3 className="text-lg font-bold text-neutral-900 dark:text-neutral-100">
                {msg.importModalTitle}
              </h3>
              <button
                onClick={() => setShowImportModal(false)}
                className="text-neutral-400 hover:text-neutral-600"
              >
                ✕
              </button>
            </div>

            <div className="space-y-3">
              <div>
                <label className="block text-xs font-semibold text-neutral-700 dark:text-neutral-300 mb-1">
                  {msg.importFormatLabel}
                </label>
                <div className="flex gap-4">
                  <label className="flex items-center gap-1.5 text-xs">
                    <input
                      type="radio"
                      name="importFormat"
                      checked={importFormat === 'po'}
                      onChange={() => setImportFormat('po')}
                    />
                    Format GNU Gettext (.po)
                  </label>
                  <label className="flex items-center gap-1.5 text-xs">
                    <input
                      type="radio"
                      name="importFormat"
                      checked={importFormat === 'json'}
                      onChange={() => setImportFormat('json')}
                    />
                    Format JSON
                  </label>
                </div>
              </div>

              <div>
                <label className="block text-xs font-semibold text-neutral-700 dark:text-neutral-300 mb-1">
                  {msg.importContentLabel}
                </label>
                <textarea
                  rows={8}
                  value={importContent}
                  onChange={(e) => setImportContent(e.target.value)}
                  placeholder={`msgid "..."\nmsgstr "..."`}
                  className="w-full p-3 font-mono text-xs bg-neutral-50 dark:bg-neutral-800 border border-neutral-200 dark:border-neutral-700 rounded-lg focus:outline-none focus:ring-2 focus:ring-indigo-500"
                />
              </div>
            </div>

            <div className="flex justify-end gap-3 pt-2">
              <button
                onClick={() => setShowImportModal(false)}
                className="px-4 py-2 text-xs font-medium text-neutral-700 dark:text-neutral-300 bg-neutral-100 dark:bg-neutral-800 rounded-lg hover:bg-neutral-200"
              >
                Annuler
              </button>
              <button
                onClick={handleImportSubmit}
                disabled={loading || !importContent.trim()}
                className="px-4 py-2 text-xs font-semibold text-white bg-indigo-600 hover:bg-indigo-700 rounded-lg shadow-sm disabled:opacity-50"
              >
                {loading ? 'Importation…' : msg.importBtn}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
