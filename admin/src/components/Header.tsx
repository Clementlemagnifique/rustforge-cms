import React from 'react';
import { Mic, ExternalLink } from 'lucide-react';

interface HeaderProps {
  title: string;
  subtitle?: string;
  onOpenAiModal: () => void;
}

export const Header: React.FC<HeaderProps> = ({ title, subtitle, onOpenAiModal }) => {
  return (
    <header className="h-16 border-b border-white/5 bg-[#0a0d14]/80 backdrop-blur-md px-6 flex items-center justify-between sticky top-0 z-10">
      <div>
        <h2 className="text-base font-bold text-slate-100 tracking-tight">{title}</h2>
        {subtitle && <p className="text-xs text-slate-400">{subtitle}</p>}
      </div>

      <div className="flex items-center gap-3">
        {/* Assistant Vocal / Prompt IA */}
        <button
          onClick={onOpenAiModal}
          className="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-gradient-to-r from-indigo-500/10 via-purple-500/10 to-pink-500/10 hover:from-indigo-500/20 hover:to-pink-500/20 border border-purple-500/30 text-purple-300 text-xs font-semibold shadow-sm transition-all hover:scale-[1.02] active:scale-[0.98]"
        >
          <Mic className="w-3.5 h-3.5 text-purple-400 animate-pulse" />
          <span>Piloter à la Voix / Prompt</span>
          <span className="text-[10px] bg-purple-500/20 text-purple-200 px-1 py-0.2 rounded font-mono">
            Ctrl+K
          </span>
        </button>

        {/* Lien public */}
        <a
          href="/api/v1/public/entries"
          target="_blank"
          rel="noreferrer"
          className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-white/5 hover:bg-white/10 text-slate-300 text-xs font-medium transition-colors border border-white/5"
        >
          <span>API Publique</span>
          <ExternalLink className="w-3 h-3 text-slate-400" />
        </a>

        {/* Badge Système */}
        <div className="hidden sm:flex items-center gap-2 pl-3 border-l border-white/10 text-xs text-emerald-400 font-medium">
          <span className="w-2 h-2 rounded-full bg-emerald-400 animate-ping"></span>
          <span className="text-[11px] text-slate-300">Rust 1.99 Engine</span>
        </div>
      </div>
    </header>
  );
};
