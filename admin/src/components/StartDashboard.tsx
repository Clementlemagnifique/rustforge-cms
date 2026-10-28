/**
 * ============================================================================
 * TABLEAU DE BORD ÉDITORIAL UNIFIÉ (`StartDashboard`)
 * ============================================================================
 * Fusionne sans perte :
 * - L'accueil adapté au rôle (`writer`, `admin`) et les raccourcis de démarrage
 *   (`WP-14`).
 * - Les cartes métriques enrichies (compteurs par type, volume média formaté,
 *   bannière d'assistance IA facultative et liste détaillée des dernières
 *   modifications).
 */

import React from 'react';
import {
  FileText,
  Image,
  Sparkles,
  ArrowUpRight,
  Clock,
  TrendingUp,
} from 'lucide-react';
import type { DashboardStats, User } from '../types';

export interface StartDashboardProps {
  stats: DashboardStats | null;
  onNavigate: (view: string) => void;
  user: User;
  onOpenAi?: () => void;
}

/** Formate un volume d'octets en unité lisible (o, Ko, Mo, Go). */
function formatBytes(bytes: number): string {
  if (!bytes || bytes <= 0) return '0 o';
  const k = 1024;
  const sizes = ['o', 'Ko', 'Mo', 'Go'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`;
}

export const StartDashboard: React.FC<StartDashboardProps> = ({
  stats,
  onNavigate,
  user,
  onOpenAi,
}) => {
  const writer = user.role !== 'subscriber';
  const admin = user.role === 'admin';

  return (
    <div className="max-w-5xl mx-auto space-y-6">
      {/* Bannière d'accueil et actions rapides selon le rôle */}
      <section className="p-6 md:p-8 rounded-2xl border border-white/10 bg-[#101728] flex flex-col md:flex-row md:items-center justify-between gap-6">
        <div>
          <p className="text-indigo-300 text-sm">
            Bienvenue, {user.display_name || user.username}
          </p>
          <h2 className="text-2xl font-bold mt-2">Qu’allez-vous publier aujourd’hui ?</h2>
          <p className="text-slate-400 mt-3 text-sm max-w-2xl">
            Retrouvez vos articles, vos pages et vos médias. Commencez simplement ; l’assistant IA
            est là si vous en avez besoin.
          </p>
          {writer && (
            <div className="flex gap-3 flex-wrap mt-6">
              <button
                onClick={() => onNavigate('new:post')}
                className="rounded-xl bg-indigo-600 hover:bg-indigo-500 px-4 py-3 text-sm font-semibold"
              >
                Rédiger un article
              </button>
              <button
                onClick={() => onNavigate('new:page')}
                className="rounded-xl bg-white/5 hover:bg-white/10 px-4 py-3 text-sm"
              >
                Créer une page
              </button>
              {admin && (
                <button
                  onClick={() => onNavigate('migration')}
                  className="rounded-xl bg-white/5 hover:bg-white/10 px-4 py-3 text-sm"
                >
                  Importer mon WordPress
                </button>
              )}
            </div>
          )}
        </div>

        {writer && onOpenAi && (
          <button
            onClick={onOpenAi}
            className="shrink-0 px-4 py-2.5 rounded-xl bg-gradient-to-r from-indigo-600 to-purple-600 hover:from-indigo-500 hover:to-purple-500 text-white text-xs font-semibold shadow-lg shadow-indigo-600/30 flex items-center gap-2 transition-all"
          >
            <Sparkles className="w-4 h-4" />
            <span>Ouvrir l’Assistant IA</span>
          </button>
        )}
      </section>

      {/* Cartes métriques par type de contenu et médiathèque */}
      {writer && (
        <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-4">
          {stats?.content_types
            ?.filter((ct) => ['post', 'page'].includes(ct.slug))
            .map((ct) => (
              <button
                key={ct.slug}
                onClick={() => onNavigate(`entries:${ct.slug}`)}
                className="text-left p-5 rounded-xl border border-white/10 bg-[#101728] hover:border-indigo-500/30 transition-all group"
              >
                <div className="flex items-center justify-between mb-3">
                  <div className="w-9 h-9 rounded-xl bg-indigo-500/10 border border-indigo-500/20 flex items-center justify-center text-indigo-400">
                    <FileText className="w-4 h-4" />
                  </div>
                  <span className="text-[11px] font-medium text-emerald-400 flex items-center gap-1">
                    <TrendingUp className="w-3 h-3" />
                    <span>{ct.published} publiés</span>
                  </span>
                </div>
                <strong className="block text-2xl">
                  {ct.published + ct.drafts + (ct.scheduled || 0)}
                </strong>
                <span className="text-sm text-slate-400">{ct.name}</span>
              </button>
            ))}

          <button
            onClick={() => onNavigate('media')}
            className="text-left p-5 rounded-xl border border-white/10 bg-[#101728] hover:border-cyan-500/30 transition-all"
          >
            <div className="flex items-center justify-between mb-3">
              <div className="w-9 h-9 rounded-xl bg-cyan-500/10 border border-cyan-500/20 flex items-center justify-center text-cyan-400">
                <Image className="w-4 h-4" />
              </div>
              <span className="text-[11px] font-medium text-slate-400">
                {formatBytes(stats?.media_bytes || 0)}
              </span>
            </div>
            <strong className="block text-2xl">{stats?.media ?? 0}</strong>
            <span className="text-sm text-slate-400">Médias</span>
          </button>
        </div>
      )}

      {/* Premiers pas d'administration */}
      {admin && (
        <section className="p-6 rounded-xl border border-white/10 space-y-4">
          <h3 className="font-semibold">Les premiers pas de votre site</h3>
          <div className="grid sm:grid-cols-3 gap-3 text-sm">
            <button
              onClick={() => onNavigate('appearance')}
              className="rounded-lg bg-white/5 hover:bg-white/10 text-left p-4"
            >
              Choisir mes couleurs
            </button>
            <a
              href="/fr"
              target="_blank"
              rel="noreferrer"
              className="rounded-lg bg-white/5 hover:bg-white/10 p-4"
            >
              Ouvrir mon site
            </a>
            <button
              onClick={() => onNavigate('capsules')}
              className="rounded-lg bg-white/5 hover:bg-white/10 text-left p-4"
            >
              Préparer ma sauvegarde
            </button>
          </div>
        </section>
      )}

      {/* Dernières modifications */}
      {writer && (
        <section className="p-6 rounded-xl border border-white/10">
          <div className="flex items-center justify-between mb-4">
            <h3 className="font-semibold">Dernières modifications</h3>
            <button
              onClick={() => onNavigate('entries:post')}
              className="text-xs text-indigo-400 hover:text-indigo-300 font-medium flex items-center gap-1"
            >
              <span>Voir tout</span>
              <ArrowUpRight className="w-3.5 h-3.5" />
            </button>
          </div>

          {stats?.recent?.length ? (
            <ul className="divide-y divide-white/5">
              {stats.recent.map((entry) => (
                <li key={entry.id}>
                  <button
                    onClick={() => onNavigate(`editor:${entry.id}:${entry.content_type}`)}
                    className="w-full text-left py-3 flex justify-between items-center gap-4 text-sm hover:bg-white/[0.02] px-2 rounded-lg transition-colors"
                  >
                    <div className="min-w-0">
                      <span className="font-medium text-slate-100">{entry.title}</span>
                      {entry.excerpt && (
                        <p className="text-xs text-slate-500 truncate max-w-md mt-0.5">
                          {entry.excerpt}
                        </p>
                      )}
                    </div>
                    <div className="flex items-center gap-3 shrink-0">
                      <span className="text-xs text-slate-400">
                        {entry.status === 'published'
                          ? 'Publié'
                          : entry.status === 'draft'
                          ? 'Brouillon'
                          : 'Archivé'}
                      </span>
                      <span className="text-[11px] text-slate-500 flex items-center gap-1">
                        <Clock className="w-3 h-3" />
                        {new Date(entry.updated_at).toLocaleDateString('fr-FR')}
                      </span>
                    </div>
                  </button>
                </li>
              ))}
            </ul>
          ) : (
            <p className="text-sm text-slate-500">Votre premier article apparaîtra ici.</p>
          )}
        </section>
      )}
    </div>
  );
};
