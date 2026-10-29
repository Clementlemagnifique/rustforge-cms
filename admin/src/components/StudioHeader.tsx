import { ExternalLink, Menu, Sparkles, Search, Maximize2, Minimize2 } from 'lucide-react';
import { useStudioStore } from '../stores/useStudioStore';

export function StudioHeader({
  title,
  breadcrumbs = [],
  onOpenAiModal,
  onOpenCommandPalette,
  onToggleNavigation,
  canWrite,
}: {
  title: string;
  breadcrumbs?: string[];
  onOpenAiModal: () => void;
  onOpenCommandPalette?: () => void;
  onToggleNavigation: () => void;
  canWrite: boolean;
}) {
  const zenMode = useStudioStore((s) => s.zenMode);
  const toggleZenMode = useStudioStore((s) => s.toggleZenMode);

  return (
    <header className="min-h-16 border-b border-white/10 px-4 md:px-6 flex gap-3 items-center justify-between bg-[#0a0d14]">
      {/* Côté gauche : menu burger mobile & titre/fil d'ariane */}
      <div className="flex gap-3 items-center min-w-0">
        <button
          className="md:hidden p-1 text-slate-400 hover:text-white"
          onClick={onToggleNavigation}
          aria-label="Ouvrir le menu"
        >
          <Menu size={21} />
        </button>
        <div className="min-w-0">
          {breadcrumbs.length > 1 && (
            <p aria-label="Chemin de navigation" className="text-xs text-slate-500 truncate">
              {breadcrumbs.join(' / ')}
            </p>
          )}
          <h1 className="font-semibold text-sm md:text-base truncate">{title}</h1>
        </div>
      </div>

      {/* Centre : Barre de déclenchement Palette Universelle (Ctrl+K) */}
      {onOpenCommandPalette && (
        <button
          type="button"
          onClick={onOpenCommandPalette}
          aria-label="Recherche et commandes rapides (Ctrl+K)"
          className="hidden md:flex items-center gap-2.5 px-3.5 py-1.5 rounded-xl bg-white/[0.03] hover:bg-white/[0.06] border border-white/10 text-xs text-slate-400 hover:text-slate-200 transition-colors shadow-sm"
        >
          <Search className="w-3.5 h-3.5 text-indigo-400" />
          <span className="text-slate-400 text-xs">Recherche ou commande...</span>
          <kbd className="ml-2 px-1.5 py-0.5 rounded bg-white/5 border border-white/10 text-[10px] text-slate-400 font-mono">
            ⌘K
          </kbd>
        </button>
      )}

      {/* Côté droit : Zen mode, Assistant IA & Voir le site */}
      <div className="flex items-center gap-2">
        {/* Bouton Mode Zen plein écran */}
        <button
          type="button"
          onClick={toggleZenMode}
          title={zenMode ? 'Quitter le mode Zen (Plein écran)' : 'Activer le mode Zen sans distraction'}
          className={`p-2 rounded-lg border transition-colors ${
            zenMode
              ? 'bg-indigo-600/20 border-indigo-500/40 text-indigo-300'
              : 'bg-white/5 border-transparent text-slate-400 hover:text-white hover:bg-white/10'
          }`}
        >
          {zenMode ? <Minimize2 size={16} /> : <Maximize2 size={16} />}
        </button>

        {canWrite && (
          <button
            aria-label="Assistant IA"
            onClick={onOpenAiModal}
            className="flex gap-2 items-center rounded-lg border border-indigo-400/30 bg-indigo-500/10 px-3 py-2 text-sm text-indigo-300 hover:bg-indigo-500/20 transition-colors"
          >
            <Sparkles size={16} />
            <span className="hidden sm:inline">Assistant IA</span>
          </button>
        )}

        <a
          href="/fr"
          target="_blank"
          rel="noreferrer"
          className="flex gap-2 items-center rounded-lg bg-white/5 hover:bg-white/10 px-3 py-2 text-sm text-slate-300 transition-colors"
        >
          <span>Voir le site</span>
          <ExternalLink size={14} />
        </a>
      </div>
    </header>
  );
}
