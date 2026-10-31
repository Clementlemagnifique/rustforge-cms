/**
 * ============================================================================
 * SHELL PRINCIPAL DU STUDIO D'ADMINISTRATION (`App`)
 * ============================================================================
 * Architecture :
 * - L'état de session, d'installation, de navigation et les métadonnées
 *   transverses (`contentTypes`, `stats`, `navigation`) sont centralisés dans
 *   `useStudioStore` (Zustand).
 * - Les vues enfants (`EntriesView`, `EntryEditorView`, etc.) sont pilotées par
 *   leurs propres stores dédiés afin d'éviter le prop-drilling et de maintenir
 *   des composants courts et lisibles.
 */

import React, { useEffect } from 'react';
import { RfAdminBar } from './components/RfAdminBar';
import { RfAdminMenu } from './components/RfAdminMenu';
import { RfScreenOptionsDrawer } from './components/RfScreenOptionsDrawer';
import { RfContextualHelpDrawer } from './components/RfContextualHelpDrawer';
import { AppearanceView } from './components/AppearanceView';
import { ToolsView } from './components/ToolsView';
import { StartDashboard as DashboardView } from './components/StartDashboard';
import { EntriesView } from './components/EntriesView';
import { EntryEditorView } from './components/EntryEditorView';
import { MediaView } from './components/MediaView';
import { PluginsView } from './components/PluginsView';
import { ContentTypesView } from './components/ContentTypesView';
import { JobsView } from './components/JobsView';
import { SettingsView } from './components/SettingsView';
import { ConverterView } from './components/ConverterView';
import { CapsulesView } from './components/CapsulesView';
import { InstallerWizardView } from './components/InstallerWizardView';
import { AiAssistantModal } from './components/AiAssistantModal';
import { LoginView } from './components/LoginView';
import { AdminPlanView } from './components/AdminPlanView';
import { NativeModulesView } from './components/NativeModulesView';
import { TaxonomiesView } from './components/TaxonomiesView';
import { UsersView } from './components/UsersView';
import { CommentsView } from './components/CommentsView';
import { FormsView } from './components/FormsView';
import { EcommerceView } from './components/EcommerceView';
import { LmsView } from './components/LmsView';
import { ColumnSettingsView } from './components/ColumnSettingsView';
import { AnalyticsView } from './components/AnalyticsView';
import { TranslationsView } from './components/TranslationsView';
import { CommandPaletteModal } from './components/CommandPaletteModal';
import { studioMessages } from './i18n/messages.fr';
import { useStudioStore } from './stores/useStudioStore';

export const App: React.FC = () => {
  const user = useStudioStore((s) => s.user);
  const isInstalled = useStudioStore((s) => s.isInstalled);
  const loading = useStudioStore((s) => s.loading);
  const connectionError = useStudioStore((s) => s.connectionError);
  const currentView = useStudioStore((s) => s.currentView);
  const navState = useStudioStore((s) => s.navState);
  const navigationError = useStudioStore((s) => s.navigationError);
  const contentTypes = useStudioStore((s) => s.contentTypes);
  const stats = useStudioStore((s) => s.stats);
  const isAiModalOpen = useStudioStore((s) => s.isAiModalOpen);
  const isCommandPaletteOpen = useStudioStore((s) => s.isCommandPaletteOpen);
  const zenMode = useStudioStore((s) => s.zenMode);
  const mobileOpen = useStudioStore((s) => s.mobileNavigation);
  const setMobileOpen = useStudioStore((s) => s.setMobileNavigation);
  const isMenuCollapsed = useStudioStore((s) => s.isMenuCollapsed);
  const toggleMenuCollapsed = useStudioStore((s) => s.toggleMenuCollapsed);
  const isScreenOptionsOpen = useStudioStore((s) => s.isScreenOptionsOpen);
  const toggleScreenOptions = useStudioStore((s) => s.toggleScreenOptions);
  const isHelpOpen = useStudioStore((s) => s.isHelpOpen);
  const toggleHelp = useStudioStore((s) => s.toggleHelp);
  const darkMode = useStudioStore((s) => s.darkMode);
  const toggleDarkMode = useStudioStore((s) => s.toggleDarkMode);

  const setUser = useStudioStore((s) => s.setUser);
  const setInstalledSession = useStudioStore((s) => s.setInstalledSession);
  const setUnsaved = useStudioStore((s) => s.setUnsaved);
  const setAiModalOpen = useStudioStore((s) => s.setAiModalOpen);
  const setCommandPaletteOpen = useStudioStore((s) => s.setCommandPaletteOpen);
  const updateNavigationData = useStudioStore((s) => s.updateNavigationData);
  const syncFromHash = useStudioStore((s) => s.syncFromHash);
  const navigate = useStudioStore((s) => s.navigate);
  const logout = useStudioStore((s) => s.logout);
  const checkAuth = useStudioStore((s) => s.checkAuth);
  const refreshData = useStudioStore((s) => s.refreshData);

  const navigation = navState?.userId === user?.id ? navState?.data : null;

  // 1. Vérification initiale de l'installation et de la session
  useEffect(() => {
    void checkAuth();
  }, [checkAuth]);

  // 2. Synchronisation du routeur sur changement de hash navigateur
  useEffect(() => {
    window.addEventListener('hashchange', syncFromHash);
    return () => window.removeEventListener('hashchange', syncFromHash);
  }, [syncFromHash]);

  // 3. Raccourcis clavier globaux (Ctrl+K / Cmd+K Palette universelle, Ctrl+I Assistant IA)
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') { setMobileOpen(false); }
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k') {
        e.preventDefault();
        setCommandPaletteOpen((prev) => !prev);
      } else if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'i') {
        e.preventDefault();
        setAiModalOpen((prev) => !prev);
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [setCommandPaletteOpen, setAiModalOpen, setMobileOpen]);

  // 4. Titre de document dynamique
  useEffect(() => {
    document.title = `${currentView} ‹ RustForge CMS`;
  }, [currentView]);

  if (loading) {
    return (
      <div className="min-h-screen bg-[#070a10] flex items-center justify-center text-slate-500 text-xs">
        {studioMessages.booting}
      </div>
    );
  }

  if (connectionError) {
    return (
      <main className="min-h-screen bg-[#0a0d14] text-white grid place-content-center p-8 gap-4">
        <h1 className="text-xl font-semibold">{studioMessages.unreachableTitle}</h1>
        <p role="alert" className="text-slate-400">
          {connectionError}
        </p>
        <button
          className="rounded-xl bg-indigo-600 p-3"
          onClick={() => void checkAuth()}
        >
          {studioMessages.retry}
        </button>
      </main>
    );
  }

  if (isInstalled === false) {
    return (
      <InstallerWizardView
        onInstalled={(token, u) => {
          setInstalledSession(token, u);
        }}
      />
    );
  }

  if (!user) {
    return (
      <LoginView
        onSuccess={(u) => {
          setUser(u);
          void refreshData();
        }}
      />
    );
  }

  // Détermination de la route éditoriale active
  const isEntriesView = currentView.startsWith('entries:');
  const isEditorView = currentView.startsWith('editor:');
  const isNewView = currentView.startsWith('new:');
  const isTaxonomyView = currentView.startsWith('taxonomy:');

  const activeContentTypeSlug = isEntriesView
    ? currentView.split(':')[1]
    : isNewView
    ? currentView.split(':')[1]
    : isEditorView
    ? currentView.split(':')[2] || 'post'
    : 'post';

  const activeEntryId = isEditorView ? currentView.split(':')[1] : undefined;
  const menuItems = navigation?.items.filter((item) => item.roles.includes(user.role)) || [];
  const outlineItems = navigation?.menu_items || menuItems;
  const planNode = currentView.startsWith('plan:') ? outlineItems.find(item => item.id === currentView.slice(5) && item.route === currentView) : undefined;
  const canOpenView =
    !!planNode ||
    !navigation ||
    !!menuItems.find((item) => item.route === currentView) ||
    ((isEntriesView || isNewView || isEditorView) &&
      user.role !== 'subscriber' &&
      contentTypes.some((ct) => ct.slug === activeContentTypeSlug)) ||
    currentView.startsWith('rf_') ||
    currentView.startsWith('settings') ||
    currentView.startsWith('users') ||
    currentView.startsWith('comments') ||
    currentView.startsWith('ecommerce') ||
    currentView.startsWith('lms') ||
    currentView.startsWith('analytics') ||
    currentView.startsWith('translations') ||
    currentView.startsWith('tools') ||
    currentView.startsWith('appearance') ||
    currentView === 'site-health' ||
    currentView === 'site-editor' ||
    currentView === 'menus' ||
    currentView === 'migration' ||
    currentView === 'capsules' ||
    currentView === 'converter';

  return (
    <div
      data-mobile-open={mobileOpen}
      data-menu-collapsed={isMenuCollapsed}
      data-zen={zenMode}
      className={`rf-admin rf-admin-transition min-h-screen ${
        darkMode ? 'dark bg-[#101520] text-[#f1f5f9]' : 'bg-[#f0f0f1] text-[#2c3338]'
      } font-sans antialiased selection:bg-[#2271b1] selection:text-white flex flex-col`}
    >
      {/* 1. Barre supérieure souveraine (#rfadminbar) */}
      <RfAdminBar
        user={user}
        mobileOpen={mobileOpen}
        onToggleMobile={() => setMobileOpen((open) => !open)}
        siteTitle="RustForge Site"
        onNavigate={navigate}
        onLogout={logout}
        onOpenCommandPalette={() => setCommandPaletteOpen(true)}
        darkMode={darkMode}
        onToggleDarkMode={toggleDarkMode}
      />

      {/* 3. Navigation latérale souveraine (#rfadminmenu) */}
      {!zenMode && (
        <RfAdminMenu
          currentView={currentView}
          onNavigate={navigate}
          contentTypes={contentTypes}
          items={outlineItems}
          isCollapsed={isMenuCollapsed}
          onToggleCollapsed={toggleMenuCollapsed}
        />
      )}

      {/* 4. Corps de page (#rfbody-content) */}
      <div
        id="rfbody-content"
        className={`min-h-screen pt-8 transition-all duration-150 ${
          zenMode ? 'ml-0' : isMenuCollapsed ? 'ml-14' : 'ml-44 md:ml-48'
        }`}
      >
        <div className="rf-screen-meta">
          <button type="button" aria-expanded={isScreenOptionsOpen} aria-controls="rf-screen-options-wrap" onClick={toggleScreenOptions}>Options de l’écran <span aria-hidden="true">▾</span></button>
          <button type="button" aria-expanded={isHelpOpen} aria-controls="rf-contextual-help-wrap" onClick={toggleHelp}>Aide <span aria-hidden="true">▾</span></button>
        </div>
      {/* 2. Tiroirs rétractables */}
      <RfScreenOptionsDrawer
        isOpen={isScreenOptionsOpen}
        onClose={toggleScreenOptions}
        currentView={currentView}
      />
      <RfContextualHelpDrawer
        isOpen={isHelpOpen}
        onClose={toggleHelp}
        currentView={currentView}
      />

        <main className="p-4 md:p-6 min-h-[calc(100vh-32px)]">
          {!navigation ? (
            <div className="space-y-3">
              <p role={navigationError ? 'alert' : 'status'}>
                {navigationError || studioMessages.loadingMenu}
              </p>
              {navigationError && (
                <button
                  onClick={() => void refreshData()}
                  className="rounded bg-[#2271b1] text-white px-3 py-1.5 text-xs font-medium"
                >
                  {studioMessages.retry}
                </button>
              )}
            </div>
          ) : !canOpenView ? (
            <div className="space-y-3">
              <h2 className="text-xl font-bold">{studioMessages.unavailableSectionTitle}</h2>
              <p className="text-slate-400">{studioMessages.unavailableSectionBody}</p>
              <button
                onClick={() => navigate('dashboard')}
                className="rounded bg-[#2271b1] text-white px-3 py-1.5 text-xs font-medium"
              >
                {studioMessages.backToDashboard}
              </button>
            </div>
          ) : (
            <>
              {planNode && <AdminPlanView navigation={navigation} nodeId={planNode.id} />}
              {currentView === 'modules' && (
                <NativeModulesView
                  navigation={navigation}
                  onChanged={(data) => updateNavigationData(data)}
                />
              )}
              {currentView === 'admin-map' && <AdminPlanView navigation={navigation} />}
              {currentView.startsWith('module:') && !['module:lms','module:ecommerce','module:forms','module:analytics','module:translations'].includes(currentView) && (
                <AdminPlanView
                  key={currentView}
                  navigation={navigation}
                  module={currentView.slice(7)}
                />
              )}
              {currentView === 'dashboard' && (
                <DashboardView
                  stats={stats}
                  onNavigate={navigate}
                  user={user}
                  onOpenAi={() => setAiModalOpen(true)}
                />
              )}
              {currentView === 'media' && <MediaView />}
              {currentView === 'plugins' && <PluginsView />}
              {currentView === 'content-types' && (
                <ContentTypesView contentTypes={contentTypes} onRefresh={refreshData} />
              )}
              {currentView === 'jobs' && <JobsView />}
              {(currentView === 'settings:columns' ||
                currentView === 'rf_column_settings' ||
                currentView === 'columns') && (
                <ColumnSettingsView key="column-settings" onNavigate={navigate} />
              )}
              {currentView === 'settings' && <SettingsView key="settings-general" initialTab="general" />}
              {currentView === 'settings:reading' && <SettingsView key="settings-reading" initialTab="reading" />}
              {currentView === 'settings:writing' && <SettingsView key="settings-writing" initialTab="writing" />}
              {currentView === 'settings:discussion' && <SettingsView key="settings-discussion" initialTab="discussion" />}
              {currentView === 'settings:media' && <SettingsView key="settings-media" initialTab="media" />}
              {currentView === 'settings:permalinks' && <SettingsView key="settings-permalinks" initialTab="permalinks" />}
              {(currentView === 'comments' || currentView === 'comments:list') && <CommentsView key="comments-view" />}
              {currentView === 'users' && <UsersView key="users-list" initialTab="list" />}
              {currentView === 'users:new' && <UsersView key="users-new" initialTab="new" />}
              {currentView === 'profile' && <UsersView key="users-profile" initialTab="profile" />}
              {(currentView === 'tools' || currentView === 'tools:available') && (
                <ToolsView key="tools-available" initialTab="available" stats={stats} onNavigate={navigate} />
              )}
              {currentView === 'tools:import' && (
                <ToolsView key="tools-import" initialTab="import" stats={stats} onNavigate={navigate} />
              )}
              {currentView === 'tools:export' && (
                <ToolsView key="tools-export" initialTab="export" stats={stats} onNavigate={navigate} />
              )}
              {(currentView === 'tools:health' || currentView === 'site-health') && (
                <ToolsView key="tools-health" initialTab="site-health" stats={stats} onNavigate={navigate} />
              )}
              {currentView === 'converter' && (
                <ConverterView onOpenMigration={() => navigate('tools:import')} />
              )}
              {currentView === 'migration' && (
                <ToolsView key="tools-migration" initialTab="import" stats={stats} onNavigate={navigate} />
              )}
              {(currentView === 'appearance' || currentView === 'themes') && (
                <AppearanceView key="appearance-themes" initialTab="themes" />
              )}
              {currentView === 'appearance:styles' && (
                <AppearanceView key="appearance-styles" initialTab="styles" />
              )}
              {currentView === 'site-editor' && (
                <AppearanceView key="site-editor" initialTab="site-editor" />
              )}
              {currentView === 'menus' && <AppearanceView key="menus" initialTab="menus" />}
              {currentView === 'capsules' && <CapsulesView stats={stats} />}
              {(currentView === 'forms' ||
                currentView === 'module:forms' ||
                currentView.startsWith('forms') ||
                currentView === 'rf_forms' ||
                currentView === 'rf_form_list' ||
                currentView === 'rf_submissions' ||
                currentView === 'rf_form_settings') && (
                <FormsView key="forms-view" />
              )}
              {(currentView === 'ecommerce' ||
                currentView === 'module:ecommerce' ||
                currentView.startsWith('ecommerce') ||
                currentView === 'rf_commerce' ||
                currentView === 'rf_products' ||
                currentView === 'rf_orders' ||
                currentView === 'rf_discounts' ||
                currentView === 'rf_commerce_settings') && (
                <EcommerceView
                  key={currentView}
                  initialTab={
                    currentView === 'rf_orders' || currentView === 'ecommerce:orders'
                      ? 'orders'
                      : currentView === 'rf_discounts' || currentView === 'ecommerce:discounts'
                      ? 'discounts'
                      : currentView === 'rf_commerce_settings' || currentView === 'ecommerce:settings'
                      ? 'settings'
                      : 'products'
                  }
                />
              )}

              {(currentView === 'lms' ||
                currentView === 'module:lms' ||
                currentView.startsWith('lms') ||
                currentView === 'rf_lms' ||
                currentView === 'rf_lms_courses' ||
                currentView === 'rf_lms_lessons' ||
                currentView === 'rf_lms_learners' ||
                currentView === 'rf_lms_settings') && (
                <LmsView
                  key={currentView}
                  initialTab={
                    currentView === 'rf_lms_lessons' || currentView === 'lms:lessons'
                      ? 'curriculum'
                      : currentView === 'rf_lms_learners' || currentView === 'lms:learners'
                      ? 'learners'
                      : currentView === 'rf_lms_settings' || currentView === 'lms:settings'
                      ? 'settings'
                      : 'courses'
                  }
                />
              )}

              {(currentView === 'analytics' ||
                currentView === 'module:analytics' ||
                currentView.startsWith('analytics') ||
                currentView === 'rf_analytics' ||
                currentView === 'rf_traffic' ||
                currentView === 'rf_analytics_settings') && (
                <AnalyticsView
                  key={currentView}
                  initialTab={
                    currentView === 'analytics:settings' || currentView === 'rf_analytics_settings'
                      ? 'settings'
                      : 'dashboard'
                  }
                />
              )}

              {(currentView === 'translations' ||
                currentView === 'module:translations' ||
                currentView.startsWith('translations') ||
                currentView === 'rf_translations' ||
                currentView === 'rf_catalogs' ||
                currentView === 'rf_localization' ||
                currentView === 'rf_language_settings') && (
                <TranslationsView
                  key={currentView}
                  initialTab={
                    currentView === 'translations:settings' || currentView === 'rf_language_settings'
                      ? 'settings'
                      : currentView === 'translations:catalogs' || currentView === 'rf_catalogs' || currentView === 'rf_localization'
                      ? 'catalogs'
                      : 'matrix'
                  }
                  onEditEntry={(id) => {
                    navigate(`editor:${id}`);
                  }}
                />
              )}

              {currentView === 'ai-studio' && (
                <div className="p-8 text-center space-y-4 max-w-xl mx-auto">
                  <div className="w-16 h-16 rounded-3xl bg-purple-500/10 border border-purple-500/20 text-purple-400 flex items-center justify-center mx-auto shadow-xl">
                    <span className="text-2xl">🎙️</span>
                  </div>
                  <h3 className="text-lg font-bold text-white">Votre assistant de rédaction</h3>
                  <p className="text-xs text-slate-400 leading-relaxed">
                    L’assistant vous aide à préparer des contenus et des modifications de design, à
                    vérifier avant application. Appuyez sur le bouton ci-dessous ou sur{' '}
                    <kbd className="px-1.5 py-0.5 rounded bg-white/10 text-slate-300 font-mono text-[10px]">
                      Ctrl+K
                    </kbd>{' '}
                    pour dicter un ordre.
                  </p>
                  <button
                    onClick={() => setAiModalOpen(true)}
                    className="px-5 py-2.5 rounded-xl bg-purple-600 hover:bg-purple-500 text-white text-xs font-semibold shadow-lg shadow-purple-600/30 transition-all hover:scale-105 active:scale-95"
                  >
                    Ouvrir l’assistant
                  </button>
                </div>
              )}

              {isTaxonomyView && (
                <TaxonomiesView
                  key={currentView}
                  taxonomySlug={currentView.slice(9) || 'category'}
                  onNavigate={navigate}
                />
              )}

              {isEntriesView && (
                <EntriesView
                  key={activeContentTypeSlug}
                  contentTypeSlug={activeContentTypeSlug}
                  contentTypes={contentTypes}
                  onNavigate={navigate}
                />
              )}

              {(isNewView || isEditorView) && (
                <EntryEditorView
                  key={currentView}
                  entryId={activeEntryId}
                  contentTypeSlug={activeContentTypeSlug}
                  contentTypes={contentTypes}
                  onDirtyChange={setUnsaved}
                  onBack={(type) => navigate(`entries:${type || activeContentTypeSlug}`)}
                  onSaved={(type) => {
                    void refreshData();
                    navigate(`entries:${type || activeContentTypeSlug}`, true);
                  }}
                />
              )}
            </>
          )}
        </main>
      </div>

      {/* Palette de commande universelle (Ctrl+K / Cmd+K) */}
      <CommandPaletteModal
        isOpen={isCommandPaletteOpen}
        onClose={() => setCommandPaletteOpen(false)}
        onNavigate={navigate}
        onOpenAi={() => setAiModalOpen(true)}
      />

      {/* Modale Assistant IA (Prompt & Voix) */}
      <AiAssistantModal
        isOpen={isAiModalOpen}
        onClose={() => setAiModalOpen(false)}
        onRefresh={refreshData}
      />
    </div>
  );
};

export default App;
