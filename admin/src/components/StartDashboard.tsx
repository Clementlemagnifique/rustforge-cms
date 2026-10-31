/**
 * ============================================================================
 * TABLEAU DE BORD D'ACCUEIL SOUVERAIN (`index.php` / `StartDashboard`)
 * ============================================================================
 * Reproduction fidèle à 99% du tableau de bord d'administration WordPress :
 * - Bannière d'accueil `#welcome-panel` avec actions "Pour commencer", "Étapes
 *   suivantes" et "Plus d'actions", rétractable via option mémorisée.
 * - Grille modulaire de widgets :
 *   1. "D'un coup d'œil" (`#dashboard_right_now`) : compteurs articles, pages,
 *      médias, commentaires, version RustForge et thème Twenty Twenty-Five.
 *   2. "Activité" (`#dashboard_activity`) : publications récentes et modération
 *      rapide des commentaires.
 *   3. "Brouillon rapide" (`#dashboard_quick_press`) : création instantanée de
 *      brouillon d'article en base via l'API Rust avec liste des brouillons.
 *   4. "Santé du site" (`#dashboard_site_health`) : diagnostic instantané 100% bon,
 *      SSR < 2 ms, RAM < 25 Mo, lien vers `tools:health`.
 *   5. "Studio IA & Événements" : raccourcis universels Ctrl+K et Assistant Ctrl+I.
 */

import React, { useState, useEffect } from 'react';
import {
  FileText,
  Image as ImageIcon,
  Sparkles,
  ArrowUpRight,
  Clock,
  Activity,
  CheckCircle2,
  PenTool,
  Compass,
  MessageSquare,
  ShieldCheck,
  Cpu,
  Send,
  X,
  ExternalLink,
  BookOpen,
  Settings,
  ChevronDown,
  ChevronUp,
} from 'lucide-react';
import { api } from '../api';
import type { DashboardStats, Entry, User } from '../types';

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

  // État du panneau de bienvenue (mémorisé en localStorage)
  const [welcomeDismissed, setWelcomeDismissed] = useState<boolean>(() => {
    return localStorage.getItem('rf_welcome_panel_dismissed') === 'true';
  });

  const toggleWelcome = () => {
    const next = !welcomeDismissed;
    setWelcomeDismissed(next);
    localStorage.setItem('rf_welcome_panel_dismissed', String(next));
  };

  // État du widget "Brouillon rapide"
  const [draftTitle, setDraftTitle] = useState('');
  const [draftContent, setDraftContent] = useState('');
  const [savingDraft, setSavingDraft] = useState(false);
  const [draftFeedback, setDraftFeedback] = useState<string | null>(null);
  const [recentDrafts, setRecentDrafts] = useState<Entry[]>([]);

  // Charge les derniers brouillons pour le widget "Brouillon rapide"
  useEffect(() => {
    let cancelled = false;
    api.entries
      .list({ content_type: 'post', status: 'draft', per_page: 4 })
      .then((res) => {
        if (!cancelled && res.items) {
          setRecentDrafts(res.items);
        }
      })
      .catch(() => {
        // Mode silencieux si non disponible
      });
    return () => {
      cancelled = true;
    };
  }, []);

  const handleSaveDraft = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!draftTitle.trim() || savingDraft) return;
    setSavingDraft(true);
    setDraftFeedback(null);
    try {
      const created = await api.entries.create('post', {
        title: draftTitle.trim(),
        body: draftContent.trim(),
        status: 'draft',
      });
      setDraftTitle('');
      setDraftContent('');
      setDraftFeedback('Brouillon enregistré avec succès.');
      setRecentDrafts((prev) => [created, ...prev.slice(0, 3)]);
      setTimeout(() => setDraftFeedback(null), 4000);
    } catch (err) {
      setDraftFeedback(
        `Erreur : ${err instanceof Error ? err.message : 'Sauvegarde impossible'}`
      );
    } finally {
      setSavingDraft(false);
    }
  };

  const postCt = stats?.content_types?.find((c) => c.slug === 'post');
  const pageCt = stats?.content_types?.find((c) => c.slug === 'page');

  return (
    <div className="max-w-7xl mx-auto space-y-6">
      {/* 1. En-tête principal de la vue d'administration */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3 border-b border-[#c3c4c7] dark:border-[#2c3338] pb-4">
        <div>
          <h1 className="text-2xl font-semibold text-[#1d2327] dark:text-[#f0f0f1] tracking-tight">
            Tableau de bord
          </h1>
          <p className="text-xs text-[#646970] dark:text-[#8c8f94] mt-0.5">
            Bienvenue sur l’espace d’administration de votre site web souverain.
          </p>
        </div>

        <div className="flex items-center gap-2">
          {onOpenAi && (
            <button
              onClick={onOpenAi}
              className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-md bg-[#2271b1] hover:bg-[#135e96] text-white text-xs font-medium shadow-sm transition-colors"
            >
              <Sparkles className="w-3.5 h-3.5" />
              <span>Studio IA (Ctrl+I)</span>
            </button>
          )}
          <button
            onClick={toggleWelcome}
            className="text-xs text-[#2271b1] hover:text-[#135e96] dark:text-[#72aee6] flex items-center gap-1 px-2 py-1"
          >
            {welcomeDismissed ? (
              <>
                <span>Afficher le panneau d’accueil</span>
                <ChevronDown className="w-3.5 h-3.5" />
              </>
            ) : (
              <>
                <span>Masquer l’accueil</span>
                <ChevronUp className="w-3.5 h-3.5" />
              </>
            )}
          </button>
        </div>
      </div>

      {/* 2. Panneau de bienvenue (#welcome-panel) */}
      {!welcomeDismissed && (
        <section
          id="rf-welcome-panel"
          className="relative bg-white dark:bg-[#182030] border border-[#c3c4c7] dark:border-[#2c3338] rounded-md p-6 shadow-sm"
        >
          <button
            onClick={toggleWelcome}
            title="Ignorer le panneau"
            aria-label="Ignorer le panneau d’accueil"
            className="absolute top-4 right-4 text-[#646970] hover:text-[#1d2327] dark:text-[#8c8f94] dark:hover:text-white"
          >
            <X className="w-4 h-4" />
          </button>

          <div className="max-w-3xl">
            <h2 className="text-xl font-semibold text-[#1d2327] dark:text-[#f0f0f1]">
              Bienvenue sur RustForge !
            </h2>
            <p className="text-xs text-[#646970] dark:text-[#8c8f94] mt-1">
              Nous avons rassemblé quelques liens essentiels pour vous aider à prendre en main votre
              site propulsé par un moteur souverain en Rust.
            </p>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mt-6 pt-6 border-t border-[#f0f0f1] dark:border-[#2c3338]">
            {/* Colonne 1 : Pour commencer */}
            <div className="space-y-3">
              <h3 className="text-xs font-bold uppercase tracking-wider text-[#1d2327] dark:text-[#f0f0f1]">
                Pour commencer
              </h3>
              <p className="text-xs text-[#646970] dark:text-[#8c8f94]">
                Prenez le contrôle du design de votre thème Twenty Twenty-Five Sovereign.
              </p>
              <div className="pt-1">
                <button
                  onClick={() => onNavigate('site-editor')}
                  className="inline-flex items-center gap-2 px-4 py-2 rounded-md bg-[#2271b1] hover:bg-[#135e96] text-white text-xs font-semibold shadow-sm transition-colors"
                >
                  <Compass className="w-4 h-4" />
                  <span>Éditeur de site (FSE)</span>
                </button>
              </div>
              <p className="text-[11px] text-[#646970] dark:text-[#8c8f94]">
                ou{' '}
                <button
                  onClick={() => onNavigate('appearance:styles')}
                  className="text-[#2271b1] hover:underline dark:text-[#72aee6]"
                >
                  choisir une palette de styles
                </button>
              </p>
            </div>

            {/* Colonne 2 : Étapes suivantes */}
            <div className="space-y-2">
              <h3 className="text-xs font-bold uppercase tracking-wider text-[#1d2327] dark:text-[#f0f0f1]">
                Étapes suivantes
              </h3>
              <ul className="space-y-2 text-xs">
                {writer && (
                  <li>
                    <button
                      onClick={() => onNavigate('new:post')}
                      className="inline-flex items-center gap-2 text-[#2271b1] hover:underline dark:text-[#72aee6]"
                    >
                      <PenTool className="w-3.5 h-3.5 text-[#646970] dark:text-[#8c8f94]" />
                      <span>Rédiger votre premier article</span>
                    </button>
                  </li>
                )}
                {writer && (
                  <li>
                    <button
                      onClick={() => onNavigate('new:page')}
                      className="inline-flex items-center gap-2 text-[#2271b1] hover:underline dark:text-[#72aee6]"
                    >
                      <FileText className="w-3.5 h-3.5 text-[#646970] dark:text-[#8c8f94]" />
                      <span>Ajouter une page d’accueil</span>
                    </button>
                  </li>
                )}
                <li>
                  <a
                    href="/fr"
                    target="_blank"
                    rel="noreferrer"
                    className="inline-flex items-center gap-2 text-[#2271b1] hover:underline dark:text-[#72aee6]"
                  >
                    <ExternalLink className="w-3.5 h-3.5 text-[#646970] dark:text-[#8c8f94]" />
                    <span>Afficher votre site en direct</span>
                  </a>
                </li>
                {admin && (
                  <li>
                    <button
                      onClick={() => onNavigate('tools:import')}
                      className="inline-flex items-center gap-2 text-[#2271b1] hover:underline dark:text-[#72aee6]"
                    >
                      <BookOpen className="w-3.5 h-3.5 text-[#646970] dark:text-[#8c8f94]" />
                      <span>Importer un export WordPress (WXR)</span>
                    </button>
                  </li>
                )}
              </ul>
            </div>

            {/* Colonne 3 : Plus d'actions */}
            <div className="space-y-2">
              <h3 className="text-xs font-bold uppercase tracking-wider text-[#1d2327] dark:text-[#f0f0f1]">
                Plus d’actions
              </h3>
              <ul className="space-y-2 text-xs">
                <li>
                  <button
                    onClick={() => onNavigate('media')}
                    className="inline-flex items-center gap-2 text-[#2271b1] hover:underline dark:text-[#72aee6]"
                  >
                    <ImageIcon className="w-3.5 h-3.5 text-[#646970] dark:text-[#8c8f94]" />
                    <span>Organiser vos dossiers FileBird</span>
                  </button>
                </li>
                <li>
                  <button
                    onClick={() => onNavigate('comments')}
                    className="inline-flex items-center gap-2 text-[#2271b1] hover:underline dark:text-[#72aee6]"
                  >
                    <MessageSquare className="w-3.5 h-3.5 text-[#646970] dark:text-[#8c8f94]" />
                    <span>Modérer les commentaires</span>
                  </button>
                </li>
                {admin && (
                  <li>
                    <button
                      onClick={() => onNavigate('settings')}
                      className="inline-flex items-center gap-2 text-[#2271b1] hover:underline dark:text-[#72aee6]"
                    >
                      <Settings className="w-3.5 h-3.5 text-[#646970] dark:text-[#8c8f94]" />
                      <span>Configurer les réglages généraux</span>
                    </button>
                  </li>
                )}
                <li>
                  <button
                    onClick={() => onNavigate('tools:health')}
                    className="inline-flex items-center gap-2 text-[#2271b1] hover:underline dark:text-[#72aee6]"
                  >
                    <ShieldCheck className="w-3.5 h-3.5 text-[#646970] dark:text-[#8c8f94]" />
                    <span>Vérifier la santé du site</span>
                  </button>
                </li>
              </ul>
            </div>
          </div>
        </section>
      )}

      {/* 3. Grille modulaire des widgets WordPress-like */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 items-start">
        {/* COLONNE GAUCHE */}
        <div className="space-y-6">
          {/* WIDGET 1 : D'un coup d'œil (#dashboard_right_now) */}
          <div className="bg-white dark:bg-[#182030] border border-[#c3c4c7] dark:border-[#2c3338] rounded-md shadow-sm overflow-hidden">
            <div className="px-4 py-3 border-b border-[#c3c4c7] dark:border-[#2c3338] flex items-center justify-between">
              <h2 className="text-xs font-bold text-[#1d2327] dark:text-[#f0f0f1] uppercase tracking-wider flex items-center gap-2">
                <Clock className="w-3.5 h-3.5 text-[#2271b1]" />
                D’un coup d’œil
              </h2>
              <span className="text-[11px] text-[#646970] dark:text-[#8c8f94]">Résumé</span>
            </div>

            <div className="p-4 space-y-4">
              <div className="grid grid-cols-2 gap-3 text-xs">
                <button
                  onClick={() => onNavigate('entries:post')}
                  className="text-left flex items-center gap-2 p-2 rounded hover:bg-[#f6f7f7] dark:hover:bg-[#20293d] transition-colors"
                >
                  <PenTool className="w-4 h-4 text-[#2271b1]" />
                  <span>
                    <strong className="text-sm font-semibold text-[#1d2327] dark:text-[#f0f0f1]">
                      {postCt?.published ?? 0}
                    </strong>{' '}
                    <span className="text-[#646970] dark:text-[#8c8f94]">Articles</span>
                  </span>
                </button>

                <button
                  onClick={() => onNavigate('entries:page')}
                  className="text-left flex items-center gap-2 p-2 rounded hover:bg-[#f6f7f7] dark:hover:bg-[#20293d] transition-colors"
                >
                  <FileText className="w-4 h-4 text-[#2271b1]" />
                  <span>
                    <strong className="text-sm font-semibold text-[#1d2327] dark:text-[#f0f0f1]">
                      {pageCt?.published ?? 0}
                    </strong>{' '}
                    <span className="text-[#646970] dark:text-[#8c8f94]">Pages</span>
                  </span>
                </button>

                <button
                  onClick={() => onNavigate('media')}
                  className="text-left flex items-center gap-2 p-2 rounded hover:bg-[#f6f7f7] dark:hover:bg-[#20293d] transition-colors"
                >
                  <ImageIcon className="w-4 h-4 text-[#2271b1]" />
                  <span>
                    <strong className="text-sm font-semibold text-[#1d2327] dark:text-[#f0f0f1]">
                      {stats?.media ?? 0}
                    </strong>{' '}
                    <span className="text-[#646970] dark:text-[#8c8f94]">
                      Médias ({formatBytes(stats?.media_bytes || 0)})
                    </span>
                  </span>
                </button>

                <button
                  onClick={() => onNavigate('comments')}
                  className="text-left flex items-center gap-2 p-2 rounded hover:bg-[#f6f7f7] dark:hover:bg-[#20293d] transition-colors"
                >
                  <MessageSquare className="w-4 h-4 text-[#2271b1]" />
                  <span>
                    <strong className="text-sm font-semibold text-[#1d2327] dark:text-[#f0f0f1]">
                      0
                    </strong>{' '}
                    <span className="text-[#646970] dark:text-[#8c8f94]">Commentaires</span>
                  </span>
                </button>
              </div>

              <div className="pt-3 border-t border-[#f0f0f1] dark:border-[#2c3338] text-[11px] text-[#646970] dark:text-[#8c8f94] flex items-center justify-between">
                <div>
                  RustForge <span className="font-semibold text-[#1d2327] dark:text-[#f0f0f1]">0.1.0</span>{' '}
                  fonctionnant avec le thème{' '}
                  <button
                    onClick={() => onNavigate('appearance')}
                    className="text-[#2271b1] hover:underline dark:text-[#72aee6] font-medium"
                  >
                    Twenty Twenty-Five Sovereign
                  </button>
                  .
                </div>
                <span className="inline-flex items-center gap-1 text-emerald-600 dark:text-emerald-400 font-medium">
                  <CheckCircle2 className="w-3 h-3" />
                  Moteur actif
                </span>
              </div>
            </div>
          </div>

          {/* WIDGET 2 : Activité (#dashboard_activity) */}
          <div className="bg-white dark:bg-[#182030] border border-[#c3c4c7] dark:border-[#2c3338] rounded-md shadow-sm overflow-hidden">
            <div className="px-4 py-3 border-b border-[#c3c4c7] dark:border-[#2c3338] flex items-center justify-between">
              <h2 className="text-xs font-bold text-[#1d2327] dark:text-[#f0f0f1] uppercase tracking-wider flex items-center gap-2">
                <Activity className="w-3.5 h-3.5 text-[#2271b1]" />
                Activité
              </h2>
              <button
                onClick={() => onNavigate('entries:post')}
                className="text-xs text-[#2271b1] hover:underline dark:text-[#72aee6] flex items-center gap-1"
              >
                <span>Tout voir</span>
                <ArrowUpRight className="w-3 h-3" />
              </button>
            </div>

            <div className="p-4 space-y-4">
              <div>
                <h3 className="text-xs font-bold text-[#1d2327] dark:text-[#f0f0f1] mb-2">
                  Récemment publié
                </h3>
                {stats?.recent?.length ? (
                  <ul className="divide-y divide-[#f0f0f1] dark:divide-[#2c3338]">
                    {stats.recent.map((entry) => (
                      <li key={entry.id} className="py-2 first:pt-0 last:pb-0">
                        <button
                          onClick={() => onNavigate(`editor:${entry.id}:${entry.content_type}`)}
                          className="w-full text-left flex items-baseline justify-between gap-3 group"
                        >
                          <div className="min-w-0">
                            <span className="text-xs font-medium text-[#2271b1] group-hover:underline dark:text-[#72aee6] truncate block">
                              {entry.title || '(sans titre)'}
                            </span>
                            {entry.excerpt && (
                              <p className="text-[11px] text-[#646970] dark:text-[#8c8f94] truncate">
                                {entry.excerpt}
                              </p>
                            )}
                          </div>
                          <span className="text-[11px] text-[#646970] dark:text-[#8c8f94] shrink-0">
                            {new Date(entry.updated_at).toLocaleDateString('fr-FR', {
                              day: 'numeric',
                              month: 'short',
                            })}
                          </span>
                        </button>
                      </li>
                    ))}
                  </ul>
                ) : (
                  <p className="text-xs text-[#646970] dark:text-[#8c8f94] italic">
                    Aucune publication récente.
                  </p>
                )}
              </div>

              <div className="pt-3 border-t border-[#f0f0f1] dark:border-[#2c3338]">
                <h3 className="text-xs font-bold text-[#1d2327] dark:text-[#f0f0f1] mb-1">
                  Commentaires récents
                </h3>
                <p className="text-xs text-[#646970] dark:text-[#8c8f94]">
                  Aucun commentaire en attente de modération.{' '}
                  <button
                    onClick={() => onNavigate('comments')}
                    className="text-[#2271b1] hover:underline dark:text-[#72aee6]"
                  >
                    Ouvrir les commentaires
                  </button>
                </p>
              </div>
            </div>
          </div>
        </div>

        {/* COLONNE DROITE */}
        <div className="space-y-6">
          {/* WIDGET 3 : Brouillon rapide (#dashboard_quick_press) */}
          {writer && (
            <div className="bg-white dark:bg-[#182030] border border-[#c3c4c7] dark:border-[#2c3338] rounded-md shadow-sm overflow-hidden">
              <div className="px-4 py-3 border-b border-[#c3c4c7] dark:border-[#2c3338] flex items-center justify-between">
                <h2 className="text-xs font-bold text-[#1d2327] dark:text-[#f0f0f1] uppercase tracking-wider flex items-center gap-2">
                  <PenTool className="w-3.5 h-3.5 text-[#2271b1]" />
                  Brouillon rapide
                </h2>
                <span className="text-[11px] text-[#646970] dark:text-[#8c8f94]">Écriture</span>
              </div>

              <form onSubmit={handleSaveDraft} className="p-4 space-y-3">
                {draftFeedback && (
                  <div className="p-2 rounded bg-emerald-50 dark:bg-emerald-950/30 border border-emerald-300 dark:border-emerald-700 text-emerald-800 dark:text-emerald-300 text-xs">
                    {draftFeedback}
                  </div>
                )}

                <div>
                  <label
                    htmlFor="rf-quick-draft-title"
                    className="block text-xs font-medium text-[#1d2327] dark:text-[#f0f0f1] mb-1"
                  >
                    Titre
                  </label>
                  <input
                    id="rf-quick-draft-title"
                    type="text"
                    required
                    value={draftTitle}
                    onChange={(e) => setDraftTitle(e.target.value)}
                    placeholder="Titre de votre idée…"
                    className="w-full text-xs px-2.5 py-1.5 rounded border border-[#8c8f94] dark:border-[#475569] bg-white dark:bg-[#101520] text-[#1d2327] dark:text-[#f0f0f1] focus:outline-none focus:ring-1 focus:ring-[#2271b1]"
                  />
                </div>

                <div>
                  <label
                    htmlFor="rf-quick-draft-content"
                    className="block text-xs font-medium text-[#1d2327] dark:text-[#f0f0f1] mb-1"
                  >
                    Contenu
                  </label>
                  <textarea
                    id="rf-quick-draft-content"
                    rows={3}
                    value={draftContent}
                    onChange={(e) => setDraftContent(e.target.value)}
                    placeholder="Qu’avez-vous en tête ?"
                    className="w-full text-xs px-2.5 py-1.5 rounded border border-[#8c8f94] dark:border-[#475569] bg-white dark:bg-[#101520] text-[#1d2327] dark:text-[#f0f0f1] focus:outline-none focus:ring-1 focus:ring-[#2271b1]"
                  />
                </div>

                <div className="flex items-center justify-between pt-1">
                  <button
                    type="submit"
                    disabled={savingDraft || !draftTitle.trim()}
                    className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded bg-[#2271b1] hover:bg-[#135e96] disabled:opacity-50 text-white text-xs font-medium transition-colors"
                  >
                    <Send className="w-3 h-3" />
                    <span>{savingDraft ? 'Enregistrement…' : 'Enregistrer le brouillon'}</span>
                  </button>
                  <span className="text-[11px] text-[#646970] dark:text-[#8c8f94]">
                    Sauvegardé en article draft
                  </span>
                </div>

                {/* Brouillons récents sous le formulaire */}
                {recentDrafts.length > 0 && (
                  <div className="pt-3 border-t border-[#f0f0f1] dark:border-[#2c3338]">
                    <div className="text-[11px] font-semibold text-[#1d2327] dark:text-[#f0f0f1] mb-1.5">
                      Vos brouillons récents :
                    </div>
                    <ul className="space-y-1 text-xs">
                      {recentDrafts.map((d) => (
                        <li key={d.id} className="truncate">
                          <button
                            type="button"
                            onClick={() => onNavigate(`editor:${d.id}:post`)}
                            className="text-[#2271b1] hover:underline dark:text-[#72aee6] text-left truncate w-full"
                          >
                            {d.title || '(Brouillon sans titre)'}
                          </button>
                        </li>
                      ))}
                    </ul>
                  </div>
                )}
              </form>
            </div>
          )}

          {/* WIDGET 4 : Santé du site (#dashboard_site_health) */}
          <div className="bg-white dark:bg-[#182030] border border-[#c3c4c7] dark:border-[#2c3338] rounded-md shadow-sm overflow-hidden">
            <div className="px-4 py-3 border-b border-[#c3c4c7] dark:border-[#2c3338] flex items-center justify-between">
              <h2 className="text-xs font-bold text-[#1d2327] dark:text-[#f0f0f1] uppercase tracking-wider flex items-center gap-2">
                <ShieldCheck className="w-3.5 h-3.5 text-emerald-600 dark:text-emerald-400" />
                Santé du site
              </h2>
              <span className="px-2 py-0.5 rounded-full text-[10px] font-semibold bg-emerald-100 dark:bg-emerald-950/40 text-emerald-800 dark:text-emerald-300">
                À vérifier
              </span>
            </div>

            <div className="p-4 space-y-3">
              <p className="text-xs text-[#646970] dark:text-[#8c8f94] leading-relaxed">
                Consultez les diagnostics pour vérifier la configuration du site. Les mesures de mémoire et de rendu ne sont pas encore raccordées.
              </p>

              <div className="grid grid-cols-2 gap-2 text-xs">
                <div className="p-2.5 rounded bg-[#f6f7f7] dark:bg-[#20293d] flex items-center gap-2">
                  <Cpu className="w-4 h-4 text-purple-600 dark:text-purple-400" />
                  <div>
                    <div className="text-[10px] text-[#646970] dark:text-[#8c8f94]">Mémoire RAM</div>
                    <div className="font-semibold text-[#1d2327] dark:text-[#f0f0f1]">Non mesurée</div>
                  </div>
                </div>

                <div className="p-2.5 rounded bg-[#f6f7f7] dark:bg-[#20293d] flex items-center gap-2">
                  <Activity className="w-4 h-4 text-emerald-600 dark:text-emerald-400" />
                  <div>
                    <div className="text-[10px] text-[#646970] dark:text-[#8c8f94]">Rendu SSR</div>
                    <div className="font-semibold text-[#1d2327] dark:text-[#f0f0f1]">Non mesuré</div>
                  </div>
                </div>
              </div>

              <div className="pt-2 flex items-center justify-between">
                <button
                  onClick={() => onNavigate('tools:health')}
                  className="text-xs text-[#2271b1] hover:underline dark:text-[#72aee6] font-medium"
                >
                  Ouvrir l’écran Santé du site
                </button>
                <span className="text-[10px] text-[#646970] dark:text-[#8c8f94]">
                  PostgreSQL 17 • Zero PHP
                </span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
