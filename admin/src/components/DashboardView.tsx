import React from 'react';
import { 
  FileText, 
  Image, 
  Sparkles, 
  ArrowUpRight, 
  Clock, 
  TrendingUp,
  Cpu
} from 'lucide-react';
import type { DashboardStats } from '../types';

interface DashboardViewProps {
  stats: DashboardStats | null;
  onNavigate: (view: string) => void;
  onOpenAi: () => void;
}

export const DashboardView: React.FC<DashboardViewProps> = ({ stats, onNavigate, onOpenAi }) => {
  const formatBytes = (bytes: number) => {
    if (bytes === 0) return '0 o';
    const k = 1024;
    const sizes = ['o', 'Ko', 'Mo', 'Go'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  };

  return (
    <div className="space-y-6">
      {/* Banner AI Engine */}
      <div className="relative overflow-hidden rounded-2xl bg-gradient-to-r from-indigo-900/40 via-purple-900/30 to-black/60 border border-purple-500/20 p-6 flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4 shadow-xl">
        <div className="space-y-1.5 z-10">
          <div className="inline-flex items-center gap-2 px-2.5 py-0.5 rounded-full bg-purple-500/20 text-purple-300 text-[11px] font-semibold border border-purple-500/30">
            <Sparkles className="w-3 h-3 text-purple-400" />
            <span>Moteur d'Intentions Actif</span>
          </div>
          <h3 className="text-xl font-bold text-white tracking-tight">
            Pilotez votre CMS par la voix ou le prompt
          </h3>
          <p className="text-xs text-slate-300 max-w-xl">
            Modifiez le fond (création d'articles, génération d'arbres de blocs sémantiques) ou la forme (palette, typographie, design tokens) en langage naturel.
          </p>
        </div>

        <button
          onClick={onOpenAi}
          className="shrink-0 z-10 px-4 py-2.5 rounded-xl bg-gradient-to-r from-indigo-600 to-purple-600 hover:from-indigo-500 hover:to-purple-500 text-white text-xs font-semibold shadow-lg shadow-indigo-600/30 flex items-center gap-2 transition-all hover:scale-105 active:scale-95"
        >
          <Sparkles className="w-4 h-4" />
          <span>Ouvrir l'Assistant IA</span>
        </button>

        {/* Effet visuel flou */}
        <div className="absolute right-0 top-0 -mr-16 -mt-16 w-64 h-64 bg-purple-600/10 rounded-full blur-3xl pointer-events-none" />
      </div>

      {/* Cartes métriques */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        {stats?.content_types?.map((ct) => (
          <div
            key={ct.slug}
            onClick={() => onNavigate(`entries:${ct.slug}`)}
            className="p-5 rounded-2xl bg-[#0f1422] border border-white/5 hover:border-indigo-500/30 transition-all cursor-pointer group shadow-sm hover:shadow-indigo-500/5"
          >
            <div className="flex items-center justify-between">
              <div className="w-9 h-9 rounded-xl bg-indigo-500/10 border border-indigo-500/20 flex items-center justify-center text-indigo-400 group-hover:scale-110 transition-transform">
                <FileText className="w-4 h-4" />
              </div>
              <span className="text-[11px] font-medium text-emerald-400 flex items-center gap-1">
                <TrendingUp className="w-3 h-3" />
                <span>{ct.published} publiés</span>
              </span>
            </div>
            <div className="mt-4">
              <p className="text-2xl font-bold text-white tracking-tight">{ct.published + ct.drafts}</p>
              <p className="text-xs text-slate-400 mt-0.5">{ct.name} au total</p>
            </div>
          </div>
        )) || (
          <div className="p-5 rounded-2xl bg-[#0f1422] border border-white/5 animate-pulse h-28" />
        )}

        {/* Médias */}
        <div 
          onClick={() => onNavigate('media')}
          className="p-5 rounded-2xl bg-[#0f1422] border border-white/5 hover:border-cyan-500/30 transition-all cursor-pointer group shadow-sm"
        >
          <div className="flex items-center justify-between">
            <div className="w-9 h-9 rounded-xl bg-cyan-500/10 border border-cyan-500/20 flex items-center justify-center text-cyan-400 group-hover:scale-110 transition-transform">
              <Image className="w-4 h-4" />
            </div>
            <span className="text-[11px] font-medium text-slate-400">
              {formatBytes(stats?.media_bytes || 0)}
            </span>
          </div>
          <div className="mt-4">
            <p className="text-2xl font-bold text-white tracking-tight">{stats?.media || 0}</p>
            <p className="text-xs text-slate-400 mt-0.5">Fichiers & Images</p>
          </div>
        </div>

        {/* Moteur Rust */}
        <div className="p-5 rounded-2xl bg-[#0f1422] border border-white/5 shadow-sm">
          <div className="flex items-center justify-between">
            <div className="w-9 h-9 rounded-xl bg-emerald-500/10 border border-emerald-500/20 flex items-center justify-center text-emerald-400">
              <Cpu className="w-4 h-4" />
            </div>
            <span className="text-[10px] font-mono uppercase px-2 py-0.5 rounded-full bg-emerald-500/10 text-emerald-300 border border-emerald-500/20">
              0.86s build
            </span>
          </div>
          <div className="mt-4">
            <p className="text-2xl font-bold text-white tracking-tight">WASM Sandbox</p>
            <p className="text-xs text-slate-400 mt-0.5">Composants WASIp2 Isolés</p>
          </div>
        </div>
      </div>

      {/* Contenus récents */}
      <div className="p-5 rounded-2xl bg-[#0f1422] border border-white/5 shadow-sm">
        <div className="flex items-center justify-between mb-4">
          <div>
            <h4 className="text-sm font-bold text-white">Derniers Contenus Édités</h4>
            <p className="text-xs text-slate-400">Tous types de contenus confondus</p>
          </div>
          <button
            onClick={() => onNavigate('entries:post')}
            className="text-xs text-indigo-400 hover:text-indigo-300 font-medium flex items-center gap-1"
          >
            <span>Voir tout</span>
            <ArrowUpRight className="w-3.5 h-3.5" />
          </button>
        </div>

        <div className="divide-y divide-white/5">
          {stats?.recent && stats.recent.length > 0 ? (
            stats.recent.map((entry) => (
              <div
                key={entry.id}
                onClick={() => onNavigate(`editor:${entry.id}`)}
                className="py-3 flex items-center justify-between hover:bg-white/[0.02] px-2 rounded-lg transition-colors cursor-pointer"
              >
                <div className="space-y-0.5">
                  <div className="flex items-center gap-2">
                    <span className="text-[10px] font-mono uppercase font-semibold px-1.5 py-0.5 rounded bg-white/5 text-slate-300">
                      {entry.content_type}
                    </span>
                    <p className="text-xs font-semibold text-slate-100 hover:text-indigo-400 transition-colors">
                      {entry.title}
                    </p>
                  </div>
                  <p className="text-[11px] text-slate-500 truncate max-w-md">
                    {entry.excerpt || 'Aucun extrait'}
                  </p>
                </div>

                <div className="flex items-center gap-3 shrink-0">
                  <span
                    className={`text-[10px] px-2 py-0.5 rounded-full font-medium ${
                      entry.status === 'published'
                        ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20'
                        : 'bg-amber-500/10 text-amber-400 border border-amber-500/20'
                    }`}
                  >
                    {entry.status === 'published' ? 'Publié' : 'Brouillon'}
                  </span>
                  <span className="text-[11px] text-slate-500 flex items-center gap-1">
                    <Clock className="w-3 h-3" />
                    {new Date(entry.updated_at).toLocaleDateString('fr-FR')}
                  </span>
                </div>
              </div>
            ))
          ) : (
            <div className="py-8 text-center text-xs text-slate-500">
              Aucun contenu pour l'instant. Utilisez l'Assistant IA ou le bouton Nouveau pour commencer !
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
