import React, { useState, useEffect } from 'react';
import { Sidebar } from './components/Sidebar';
import { Header } from './components/Header';
import { DashboardView } from './components/DashboardView';
import { EntriesView } from './components/EntriesView';
import { EntryEditorView } from './components/EntryEditorView';
import { MediaView } from './components/MediaView';
import { PluginsView } from './components/PluginsView';
import { ContentTypesView } from './components/ContentTypesView';
import { SettingsView } from './components/SettingsView';
import { ConverterView } from './components/ConverterView';
import { CapsulesView } from './components/CapsulesView';
import { InstallerWizardView } from './components/InstallerWizardView';
import { AiAssistantModal } from './components/AiAssistantModal';
import { LoginView } from './components/LoginView';
import { api, setToken } from './api';
import type { User, ContentType, DashboardStats } from './types';

export const App: React.FC = () => {
  const [user, setUser] = useState<User | null>(null);
  const [isInstalled, setIsInstalled] = useState<boolean | null>(null);
  const [loading, setLoading] = useState(true);
  const [currentView, setCurrentView] = useState('dashboard');
  const [contentTypes, setContentTypes] = useState<ContentType[]>([]);
  const [stats, setStats] = useState<DashboardStats | null>(null);
  const [isAiModalOpen, setIsAiModalOpen] = useState(false);

  // Vérification de la session et de l'état d'installation
  const checkAuth = async () => {
    try {
      const statusRes = await api.setup.status().catch(() => ({ is_installed: true }));
      setIsInstalled(statusRes.is_installed);

      if (statusRes.is_installed) {
        const me = await api.auth.me();
        setUser(me);
      } else {
        setUser(null);
      }
    } catch {
      setUser(null);
    } finally {
      setLoading(false);
    }
  };

  const refreshData = async () => {
    try {
      const [cts, st] = await Promise.all([
        api.contentTypes.list(),
        api.stats.dashboard(),
      ]);
      setContentTypes(cts);
      setStats(st);
    } catch (err) {
      console.error(err);
    }
  };

  useEffect(() => {
    checkAuth();
  }, []);

  useEffect(() => {
    if (user) {
      refreshData();
    }
  }, [user]);

  // Raccourci Ctrl+K / Cmd+K pour ouvrir l'Assistant IA
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k') {
        e.preventDefault();
        setIsAiModalOpen((prev) => !prev);
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);

  const handleLogout = () => {
    setToken(null);
    setUser(null);
  };

  if (loading) {
    return (
      <div className="min-h-screen bg-[#070a10] flex items-center justify-center text-slate-500 text-xs">
        Démarrage de RustForge Studio...
      </div>
    );
  }

  if (isInstalled === false) {
    return (
      <InstallerWizardView
        onInstalled={(token, u) => {
          setToken(token);
          setUser(u);
          setIsInstalled(true);
        }}
      />
    );
  }

  if (!user) {
    return <LoginView onSuccess={(u) => setUser(u)} />;
  }

  // Détermination de la vue active
  const isEntriesView = currentView.startsWith('entries:');
  const isEditorView = currentView.startsWith('editor:');
  const isNewView = currentView.startsWith('new:');

  const activeContentTypeSlug = isEntriesView
    ? currentView.split(':')[1]
    : isNewView
    ? currentView.split(':')[1]
    : 'post';

  const activeEntryId = isEditorView ? currentView.split(':')[1] : undefined;

  const getPageTitle = () => {
    if (currentView === 'dashboard') return 'Tableau de bord';
    if (currentView === 'media') return 'Médiathèque';
    if (currentView === 'plugins') return 'Plugins WebAssembly';
    if (currentView === 'content-types') return 'Types de Contenu';
    if (currentView === 'settings') return 'Réglages & Design System';
    if (currentView === 'converter') return 'Ingestion & Transmutation Universelle';
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
      {/* Sidebar */}
      <Sidebar
        currentView={currentView}
        onNavigate={setCurrentView}
        contentTypes={contentTypes}
        user={user}
        onLogout={handleLogout}
      />

      {/* Main Content Area */}
      <div className="flex-1 flex flex-col h-screen overflow-hidden">
        <Header
          title={getPageTitle()}
          onOpenAiModal={() => setIsAiModalOpen(true)}
        />

        <main className="flex-1 overflow-y-auto p-6">
          {currentView === 'dashboard' && (
            <DashboardView
              stats={stats}
              onNavigate={setCurrentView}
              onOpenAi={() => setIsAiModalOpen(true)}
            />
          )}

          {currentView === 'media' && <MediaView />}

          {currentView === 'plugins' && <PluginsView />}

          {currentView === 'content-types' && (
            <ContentTypesView
              contentTypes={contentTypes}
              onRefresh={refreshData}
            />
          )}

          {currentView === 'settings' && <SettingsView />}
          {currentView === 'converter' && <ConverterView />}
          {currentView === 'capsules' && <CapsulesView stats={stats} />}

          {currentView === 'ai-studio' && (
            <div className="p-8 text-center space-y-4 max-w-xl mx-auto">
              <div className="w-16 h-16 rounded-3xl bg-purple-500/10 border border-purple-500/20 text-purple-400 flex items-center justify-center mx-auto shadow-xl">
                <span className="text-2xl">🎙️</span>
              </div>
              <h3 className="text-lg font-bold text-white">Studio de Commande Vocale & Sémantique</h3>
              <p className="text-xs text-slate-400 leading-relaxed">
                RustForge expose l'intégralité de son modèle et de ses designs sous forme d'outils déterministes.
                Appuyez sur le bouton ci-dessous ou sur <kbd className="px-1.5 py-0.5 rounded bg-white/10 text-slate-300 font-mono text-[10px]">Ctrl+K</kbd> pour dicter un ordre.
              </p>
              <button
                onClick={() => setIsAiModalOpen(true)}
                className="px-5 py-2.5 rounded-xl bg-purple-600 hover:bg-purple-500 text-white text-xs font-semibold shadow-lg shadow-purple-600/30 transition-all hover:scale-105 active:scale-95"
              >
                Lancer la commande vocale
              </button>
            </div>
          )}

          {isEntriesView && (
            <EntriesView
              contentTypeSlug={activeContentTypeSlug}
              contentTypes={contentTypes}
              onNavigate={setCurrentView}
            />
          )}

          {(isNewView || isEditorView) && (
            <EntryEditorView
              entryId={activeEntryId}
              contentTypeSlug={activeContentTypeSlug}
              contentTypes={contentTypes}
              onBack={() => setCurrentView(`entries:${activeContentTypeSlug}`)}
              onSaved={() => {
                refreshData();
                setCurrentView(`entries:${activeContentTypeSlug}`);
              }}
            />
          )}
        </main>
      </div>

      {/* Assistant Modal Prompt & Voix */}
      <AiAssistantModal
        isOpen={isAiModalOpen}
        onClose={() => setIsAiModalOpen(false)}
        onRefresh={refreshData}
      />
    </div>
  );
};

export default App;
