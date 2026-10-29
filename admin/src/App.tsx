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

import React, { useCallback, useEffect } from 'react';
import { StudioNavigationPanel } from './components/StudioNavigationPanel';
import { StudioNavigation as Sidebar } from './components/StudioNavigation';
import { MigrationView } from './components/MigrationView';
import { AppearanceView } from './components/AppearanceView';
import { StudioHeader as Header } from './components/StudioHeader';
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
import { studioMessages } from './i18n/messages.fr';
import { currentMenu, menuTrail } from './navigation';
import { useStudioStore } from './stores/useStudioStore';

export const App: React.FC = () => {
  const user = useStudioStore((s) => s.user);
  const isInstalled = useStudioStore((s) => s.isInstalled);
  const loading = useStudioStore((s) => s.loading);
  const connectionError = useStudioStore((s) => s.connectionError);
  const currentView = useStudioStore((s) => s.currentView);
  const mobileNavigation = useStudioStore((s) => s.mobileNavigation);
  const navState = useStudioStore((s) => s.navState);
  const navigationError = useStudioStore((s) => s.navigationError);
  const contentTypes = useStudioStore((s) => s.contentTypes);
  const stats = useStudioStore((s) => s.stats);
  const isAiModalOpen = useStudioStore((s) => s.isAiModalOpen);

  const setUser = useStudioStore((s) => s.setUser);
  const setInstalledSession = useStudioStore((s) => s.setInstalledSession);
  const setUnsaved = useStudioStore((s) => s.setUnsaved);
  const setMobileNavigation = useStudioStore((s) => s.setMobileNavigation);
  const closeMobileNavigation = useCallback(() => setMobileNavigation(false), [setMobileNavigation]);
  const setAiModalOpen = useStudioStore((s) => s.setAiModalOpen);
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

  // 3. Raccourci clavier global Ctrl+K / Cmd+K pour l'Assistant IA
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k') {
        e.preventDefault();
        setAiModalOpen((prev) => !prev);
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [setAiModalOpen]);

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
  const selectedMenu = currentMenu(menuItems, currentView);
  const canOpenView =
    !!navigation &&
    (!!menuItems.find((item) => item.route === currentView) ||
      ((isEntriesView || isNewView || isEditorView) &&
        user.role !== 'subscriber' &&
        contentTypes.some((ct) => ct.slug === activeContentTypeSlug)));

  const getPageTitle = () => {
    if (selectedMenu) return selectedMenu.label;
    if (currentView === 'dashboard') return 'Tableau de bord';
    if (currentView === 'media') return 'Médiathèque';
    if (currentView === 'plugins') return 'Plugins WebAssembly';
    if (currentView === 'content-types') return 'Types de Contenu';
    if (currentView === 'forms' || currentView === 'module:forms' || currentView.startsWith('forms')) return 'Formulaires & Messages';
    if (currentView === 'jobs') return 'Tâches et diagnostic';
    if (currentView === 'settings') return 'Réglages';
    if (currentView === 'converter') return 'Conversion de thèmes · avancé';
    if (currentView === 'migration') return 'Importer WordPress';
    if (currentView === 'appearance') return 'Apparence';
    if (currentView === 'capsules') return 'Sauvegardes';
    if (currentView === 'ai-studio') return 'Studio IA & Pilote Vocal';
    if (isEntriesView) {
      const ct = contentTypes.find((c) => c.slug === activeContentTypeSlug);
      return ct?.name || 'Contenus';
    }
    if (isNewView || isEditorView) return 'Éditeur de Contenu';
    return 'Studio';
  };

  return (
    <div className="flex h-screen bg-[#0a0d14] text-slate-100 overflow-hidden font-sans">
      {/* Navigation latérale responsive */}
      <StudioNavigationPanel open={mobileNavigation} onClose={closeMobileNavigation}>
        <Sidebar
          key={user.id}
          items={menuItems}
          currentView={currentView}
          onNavigate={navigate}
          contentTypes={contentTypes}
          user={user}
          onLogout={logout}
        />
      </StudioNavigationPanel>

      {/* Zone de contenu principale */}
      <div inert={mobileNavigation || undefined} className="flex-1 min-w-0 flex flex-col h-screen overflow-hidden">
        <Header
          title={getPageTitle()}
          breadcrumbs={menuTrail(menuItems, currentView).map((item) => item.label)}
          onToggleNavigation={() => setMobileNavigation((open) => !open)}
          canWrite={user.role !== 'subscriber'}
          onOpenAiModal={() => setAiModalOpen(true)}
        />

        <main className="flex-1 overflow-y-auto p-4 md:p-6">
          {!navigation ? (
            <div className="space-y-3">
              <p role={navigationError ? 'alert' : 'status'}>
                {navigationError || studioMessages.loadingMenu}
              </p>
              {navigationError && (
                <button
                  onClick={() => void refreshData()}
                  className="rounded-lg bg-indigo-600 px-4 py-2"
                >
                  {studioMessages.retry}
                </button>
              )}
            </div>
          ) : !canOpenView ? (
            <div className="space-y-3">
              <h2 className="text-xl">{studioMessages.unavailableSectionTitle}</h2>
              <p className="text-slate-400">{studioMessages.unavailableSectionBody}</p>
              <button
                onClick={() => navigate('dashboard')}
                className="rounded-lg bg-indigo-600 px-4 py-2"
              >
                {studioMessages.backToDashboard}
              </button>
            </div>
          ) : (
            <>
              {currentView === 'modules' && (
                <NativeModulesView
                  navigation={navigation}
                  onChanged={(data) => updateNavigationData(data)}
                />
              )}
              {currentView === 'admin-map' && <AdminPlanView navigation={navigation} />}
              {currentView.startsWith('module:') && (
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
              {currentView === 'settings' && <SettingsView key="settings-general" initialTab="general" />}
              {currentView === 'settings:reading' && <SettingsView key="settings-reading" initialTab="reading" />}
              {currentView === 'settings:writing' && <SettingsView key="settings-writing" initialTab="writing" />}
              {currentView === 'settings:discussion' && <SettingsView key="settings-discussion" initialTab="discussion" />}
              {(currentView === 'comments' || currentView === 'comments:list') && <CommentsView key="comments-view" />}
              {currentView === 'users' && <UsersView key="users-list" initialTab="list" />}
              {currentView === 'users:new' && <UsersView key="users-new" initialTab="new" />}
              {currentView === 'profile' && <UsersView key="users-profile" initialTab="profile" />}
              {currentView === 'converter' && (
                <ConverterView onOpenMigration={() => navigate('migration')} />
              )}
              {currentView === 'migration' && (
                <MigrationView onNavigate={navigate} onImported={refreshData} />
              )}
              {currentView === 'appearance' && <AppearanceView key="appearance" initialTab="styles" />}
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
