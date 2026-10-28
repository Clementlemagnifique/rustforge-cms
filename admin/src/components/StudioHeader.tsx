import { ExternalLink, Menu, Sparkles } from 'lucide-react';
export function StudioHeader({ title, breadcrumbs = [], onOpenAiModal, onToggleNavigation, canWrite }: { title: string; breadcrumbs?: string[]; onOpenAiModal: () => void; onToggleNavigation: () => void; canWrite: boolean }) {
  return <header className="min-h-16 border-b border-white/10 px-4 md:px-6 flex gap-3 items-center justify-between bg-[#0a0d14]">
    <div className="flex gap-3 items-center min-w-0"><button className="md:hidden" onClick={onToggleNavigation} aria-label="Ouvrir le menu"><Menu size={21} /></button><div className="min-w-0">{breadcrumbs.length > 1 && <p aria-label="Chemin de navigation" className="text-xs text-slate-500 truncate">{breadcrumbs.join(' / ')}</p>}<h1 className="font-semibold text-sm md:text-base">{title}</h1></div></div>
    <div className="flex items-center gap-2">{canWrite && <button aria-label="Assistant IA" onClick={onOpenAiModal} className="flex gap-2 items-center rounded-lg border border-indigo-400/30 bg-indigo-500/10 px-3 py-2 text-sm text-indigo-300"><Sparkles size={16} /><span className="hidden sm:inline">Assistant IA</span></button>}<a href="/fr" target="_blank" rel="noreferrer" className="flex gap-2 items-center rounded-lg bg-white/5 px-3 py-2 text-sm"><span>Voir le site</span><ExternalLink size={14} /></a></div>
  </header>;
}
