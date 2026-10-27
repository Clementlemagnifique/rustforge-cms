import React from 'react';
import { 
  LayoutDashboard, 
  FileText, 
  FolderTree, 
  Image, 
  Boxes, 
  Sparkles, 
  Settings, 
  FileCode2,
  Layers,
  LogOut,
  ChevronRight
} from 'lucide-react';
import type { ContentType, User } from '../types';

interface SidebarProps {
  currentView: string;
  onNavigate: (view: string) => void;
  contentTypes: ContentType[];
  user: User | null;
  onLogout: () => void;
}

export const Sidebar: React.FC<SidebarProps> = ({
  currentView,
  onNavigate,
  contentTypes,
  user,
  onLogout,
}) => {
  return (
    <aside className="w-64 border-r border-white/5 bg-[#0d121f] flex flex-col h-screen select-none">
      {/* Brand */}
      <div className="p-5 flex items-center justify-between border-b border-white/5">
        <div className="flex items-center gap-3">
          <div className="w-9 h-9 rounded-xl bg-gradient-to-tr from-indigo-600 via-indigo-500 to-cyan-400 flex items-center justify-center shadow-lg shadow-indigo-500/20 font-black text-white text-sm tracking-tighter">
            RF
          </div>
          <div>
            <h1 className="font-bold text-slate-100 text-sm tracking-wide flex items-center gap-1.5">
              RustForge
              <span className="text-[10px] uppercase font-semibold px-1.5 py-0.5 rounded-full bg-indigo-500/20 text-indigo-300 border border-indigo-500/30">
                AI Core
              </span>
            </h1>
            <p className="text-[11px] text-slate-500">CMS Rust Souverain AI-Native</p>
          </div>
        </div>
      </div>

      {/* Navigation */}
      <div className="flex-1 overflow-y-auto px-3 py-4 space-y-6">
        {/* Core */}
        <div>
          <p className="px-3 text-[10px] font-semibold text-slate-400 uppercase tracking-wider mb-2">
            Vue Globale
          </p>
          <nav className="space-y-1">
            <button
              onClick={() => onNavigate('dashboard')}
              className={`w-full flex items-center gap-3 px-3 py-2 rounded-lg text-xs font-medium transition-all ${
                currentView === 'dashboard'
                  ? 'bg-indigo-600 text-white shadow-md shadow-indigo-600/25'
                  : 'text-slate-400 hover:text-slate-200 hover:bg-white/5'
              }`}
            >
              <LayoutDashboard className="w-4 h-4" />
              <span>Tableau de bord</span>
            </button>
            <button
              onClick={() => onNavigate('media')}
              className={`w-full flex items-center gap-3 px-3 py-2 rounded-lg text-xs font-medium transition-all ${
                currentView === 'media'
                  ? 'bg-indigo-600 text-white shadow-md shadow-indigo-600/25'
                  : 'text-slate-400 hover:text-slate-200 hover:bg-white/5'
              }`}
            >
              <Image className="w-4 h-4" />
              <span>Médiathèque</span>
            </button>
          </nav>
        </div>

        {/* Contenus natifs & Custom Post Types */}
        <div>
          <div className="flex items-center justify-between px-3 mb-2">
            <p className="text-[10px] font-semibold text-slate-400 uppercase tracking-wider">
              Contenus & Types
            </p>
          </div>
          <nav className="space-y-1">
            {contentTypes.map((ct) => (
              <button
                key={ct.slug}
                onClick={() => onNavigate(`entries:${ct.slug}`)}
                className={`w-full flex items-center justify-between px-3 py-2 rounded-lg text-xs font-medium transition-all ${
                  currentView === `entries:${ct.slug}`
                    ? 'bg-indigo-600 text-white shadow-md shadow-indigo-600/25'
                    : 'text-slate-400 hover:text-slate-200 hover:bg-white/5'
                }`}
              >
                <div className="flex items-center gap-3">
                  <FileText className="w-4 h-4 text-slate-400" />
                  <span>{ct.name}</span>
                </div>
                {ct.entry_count > 0 && (
                  <span className="text-[10px] px-1.5 py-0.5 rounded-full bg-white/10 text-slate-300">
                    {ct.entry_count}
                  </span>
                )}
              </button>
            ))}
            <button
              onClick={() => onNavigate('content-types')}
              className={`w-full flex items-center gap-3 px-3 py-2 rounded-lg text-xs font-medium transition-all ${
                currentView === 'content-types'
                  ? 'bg-indigo-600 text-white shadow-md shadow-indigo-600/25'
                  : 'text-slate-400 hover:text-slate-200 hover:bg-white/5'
              }`}
            >
              <FolderTree className="w-4 h-4" />
              <span>Gérer les types</span>
            </button>
          </nav>
        </div>

        {/* Écosystème & IA */}
        <div>
          <p className="px-3 text-[10px] font-semibold text-slate-400 uppercase tracking-wider mb-2">
            Intelligence & Extensibilité
          </p>
          <nav className="space-y-1">
            <button
              onClick={() => onNavigate('ai-studio')}
              className={`w-full flex items-center justify-between px-3 py-2 rounded-lg text-xs font-medium transition-all ${
                currentView === 'ai-studio'
                  ? 'bg-gradient-to-r from-indigo-600 to-purple-600 text-white shadow-lg shadow-purple-600/20'
                  : 'text-slate-300 hover:text-white bg-purple-500/10 border border-purple-500/20'
              }`}
            >
              <div className="flex items-center gap-3">
                <Sparkles className="w-4 h-4 text-purple-400" />
                <span>Studio IA & Voix</span>
              </div>
              <ChevronRight className="w-3.5 h-3.5 opacity-60" />
            </button>
            <button
              onClick={() => onNavigate('converter')}
              className={`w-full flex items-center justify-between px-3 py-2 rounded-lg text-xs font-medium transition-all ${
                currentView === 'converter'
                  ? 'bg-gradient-to-r from-emerald-600 to-teal-600 text-white shadow-md shadow-emerald-600/25'
                  : 'text-slate-300 hover:text-white bg-emerald-500/10 border border-emerald-500/20'
              }`}
            >
              <div className="flex items-center gap-3">
                <FileCode2 className="w-4 h-4 text-emerald-400" />
                <span>Ingestion & DataMorph</span>
              </div>
              <span className="text-[9px] px-1.5 py-0.2 rounded font-bold uppercase bg-emerald-500/20 text-emerald-300">
                Auto
              </span>
            </button>
            <button
              onClick={() => onNavigate('capsules')}
              className={`w-full flex items-center justify-between px-3 py-2 rounded-lg text-xs font-medium transition-all ${
                currentView === 'capsules'
                  ? 'bg-gradient-to-r from-cyan-600 to-indigo-600 text-white shadow-md shadow-cyan-600/25'
                  : 'text-slate-300 hover:text-white bg-cyan-500/10 border border-cyan-500/20'
              }`}
            >
              <div className="flex items-center gap-3">
                <Layers className="w-4 h-4 text-cyan-400" />
                <span>Capsules & Migration</span>
              </div>
              <span className="text-[9px] px-1.5 py-0.2 rounded font-bold uppercase bg-cyan-500/20 text-cyan-300 font-mono">
                .rfcapsule
              </span>
            </button>
            <button
              onClick={() => onNavigate('plugins')}
              className={`w-full flex items-center gap-3 px-3 py-2 rounded-lg text-xs font-medium transition-all ${
                currentView === 'plugins'
                  ? 'bg-indigo-600 text-white shadow-md shadow-indigo-600/25'
                  : 'text-slate-400 hover:text-slate-200 hover:bg-white/5'
              }`}
            >
              <Boxes className="w-4 h-4" />
              <span>Plugins WASM</span>
            </button>
            <button
              onClick={() => onNavigate('settings')}
              className={`w-full flex items-center gap-3 px-3 py-2 rounded-lg text-xs font-medium transition-all ${
                currentView === 'settings'
                  ? 'bg-indigo-600 text-white shadow-md shadow-indigo-600/25'
                  : 'text-slate-400 hover:text-slate-200 hover:bg-white/5'
              }`}
            >
              <Settings className="w-4 h-4" />
              <span>Design System & Réglages</span>
            </button>
          </nav>
        </div>
      </div>

      {/* Profil utilisateur & Déconnexion */}
      <div className="p-3 border-t border-white/5 bg-black/20 flex items-center justify-between">
        <div className="flex items-center gap-2.5 overflow-hidden">
          <div className="w-8 h-8 rounded-full bg-slate-800 border border-white/10 flex items-center justify-center text-xs font-bold text-slate-300">
            {user?.display_name?.charAt(0) || 'U'}
          </div>
          <div className="truncate">
            <p className="text-xs font-semibold text-slate-200 truncate">{user?.display_name || 'Utilisateur'}</p>
            <p className="text-[10px] text-slate-500 capitalize">{user?.role || 'Auteur'}</p>
          </div>
        </div>
        <button
          onClick={onLogout}
          title="Se déconnecter"
          className="p-1.5 rounded-lg text-slate-400 hover:text-red-400 hover:bg-red-500/10 transition-colors"
        >
          <LogOut className="w-4 h-4" />
        </button>
      </div>
    </aside>
  );
};
