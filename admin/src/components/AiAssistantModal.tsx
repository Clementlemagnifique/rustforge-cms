import React, { useState, useEffect, useRef } from 'react';
import {
  Sparkles,
  Mic,
  MicOff,
  Send,
  X,
  CheckCircle2,
  AlertCircle,
  Layers,
  Palette,
  FileText,
  Search,
  Wrench,
  RotateCcw,
  Loader2,
  ChevronDown,
} from 'lucide-react';
import { api } from '../api';
import type { AiAction, AiChatTurn, AiStatus } from '../types';

interface AiAssistantModalProps {
  isOpen: boolean;
  onClose: () => void;
  onRefresh: () => void;
}

interface UiMessage extends AiChatTurn {
  actions?: AiAction[];
  meta?: string;
  error?: boolean;
}

const SUGGESTIONS = [
  { icon: FileText, color: 'text-indigo-400', label: 'Rédiger un article d’expert', prompt: 'Rédige un article sur les avantages d’un CMS écrit en Rust pour les PME, ton expert mais accessible. Laisse-le en brouillon.' },
  { icon: Palette, color: 'text-emerald-400', label: 'Ajuster le Design System', prompt: 'Passe la couleur principale en vert émeraude, active le mode sombre et arrondis les bordures à 12px.' },
  { icon: Layers, color: 'text-cyan-400', label: 'Créer un type « Événements »', prompt: 'Crée un type de contenu Événements avec une date (obligatoire), un lieu (obligatoire) et un prix.' },
  { icon: Search, color: 'text-purple-400', label: 'Auditer le dernier contenu', prompt: 'Fais un audit SEO et GEO de mon dernier contenu modifié et donne-moi les 3 priorités.' },
];

const ActionCard: React.FC<{ action: AiAction }> = ({ action }) => {
  const [open, setOpen] = useState(false);
  return (
    <div className={`rounded-lg border text-[11px] ${action.success ? 'border-emerald-500/30 bg-emerald-500/5' : 'border-red-500/30 bg-red-500/5'}`}>
      <button onClick={() => setOpen(!open)} className="w-full flex items-center gap-2 px-2.5 py-1.5 text-left">
        {action.success ? <CheckCircle2 className="w-3.5 h-3.5 text-emerald-400 shrink-0" /> : <AlertCircle className="w-3.5 h-3.5 text-red-400 shrink-0" />}
        <span className="font-mono text-slate-400">{action.tool}</span>
        <span className="text-slate-200 truncate flex-1">{action.message}</span>
        <ChevronDown className={`w-3.5 h-3.5 text-slate-500 transition-transform ${open ? 'rotate-180' : ''}`} />
      </button>
      {open && (
        <pre className="mx-2.5 mb-2 p-2 bg-black/40 rounded text-[10px] text-slate-300 overflow-auto max-h-48">
          {JSON.stringify({ arguments: action.arguments, data: action.data }, null, 2)}
        </pre>
      )}
    </div>
  );
};

export const AiAssistantModal: React.FC<AiAssistantModalProps> = ({ isOpen, onClose, onRefresh }) => {
  const [prompt, setPrompt] = useState('');
  const [isListening, setIsListening] = useState(false);
  const [loading, setLoading] = useState(false);
  const [status, setStatus] = useState<AiStatus | null>(null);
  const [messages, setMessages] = useState<UiMessage[]>([]);
  const scrollRef = useRef<HTMLDivElement>(null);
  const recognitionRef = useRef<any>(null);

  useEffect(() => {
    if (isOpen) {
      api.ai.status().then(setStatus).catch(() => setStatus(null));
    }
  }, [isOpen]);

  useEffect(() => {
    scrollRef.current?.scrollTo({ top: scrollRef.current.scrollHeight, behavior: 'smooth' });
  }, [messages, loading]);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && isOpen) onClose();
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [isOpen, onClose]);

  if (!isOpen) return null;

  // Reconnaissance vocale (Web Speech API)
  const toggleListening = () => {
    const SpeechRecognition = (window as any).SpeechRecognition || (window as any).webkitSpeechRecognition;
    if (!SpeechRecognition) {
      alert('La reconnaissance vocale nécessite un navigateur compatible (Chrome, Edge, Safari).');
      return;
    }
    if (isListening) {
      recognitionRef.current?.stop();
      setIsListening(false);
      return;
    }
    const recognition = new SpeechRecognition();
    recognition.lang = 'fr-FR';
    recognition.continuous = false;
    recognition.onresult = (event: any) => setPrompt(event.results[0][0].transcript);
    recognition.onerror = () => setIsListening(false);
    recognition.onend = () => setIsListening(false);
    recognitionRef.current = recognition;
    recognition.start();
    setIsListening(true);
  };

  const send = async (overridePrompt?: string) => {
    const text = (overridePrompt ?? prompt).trim();
    if (!text || loading) return;

    const history: UiMessage[] = [...messages, { role: 'user', content: text }];
    setMessages(history);
    setPrompt('');
    setLoading(true);

    try {
      const turns: AiChatTurn[] = history.filter((m) => !m.error).map(({ role, content }) => ({ role, content }));
      const res = await api.ai.chat(turns);
      setMessages([
        ...history,
        {
          role: 'assistant',
          content: res.reply,
          actions: res.actions,
          meta: `${res.model} · ${(res.duration_ms / 1000).toFixed(1)} s`,
        },
      ]);
      if (res.actions.some((a) => a.success)) onRefresh();
    } catch (err: any) {
      setMessages([...history, { role: 'assistant', content: err.message || 'Erreur lors de l’appel au modèle.', error: true }]);
    } finally {
      setLoading(false);
    }
  };

  const statusBadge = !status ? (
    <span className="text-[10px] text-slate-500">statut inconnu</span>
  ) : status.reachable && status.model_available ? (
    <span className="flex items-center gap-1.5 text-[10px] text-emerald-300">
      <span className="w-1.5 h-1.5 rounded-full bg-emerald-400 shadow-[0_0_6px] shadow-emerald-400" />
      {status.model}
    </span>
  ) : (
    <span className="flex items-center gap-1.5 text-[10px] text-amber-300" title={status.error}>
      <span className="w-1.5 h-1.5 rounded-full bg-amber-400" />
      {status.reachable ? `${status.model} non installé` : 'Ollama injoignable'}
    </span>
  );

  return (
    <div className="fixed inset-0 z-50 bg-black/70 backdrop-blur-sm flex items-center justify-center p-4" onClick={onClose}>
      <div
        onClick={(e) => e.stopPropagation()}
        className="w-full max-w-2xl h-[80vh] bg-[#0f1422] border border-purple-500/30 rounded-2xl shadow-2xl overflow-hidden flex flex-col"
      >
        {/* En-tête */}
        <div className="p-4 border-b border-white/10 flex items-center justify-between bg-gradient-to-r from-purple-900/30 via-indigo-900/30 to-transparent">
          <div className="flex items-center gap-2.5">
            <div className="w-7 h-7 rounded-lg bg-purple-500/20 border border-purple-500/40 flex items-center justify-center">
              <Sparkles className="w-4 h-4 text-purple-300" />
            </div>
            <div>
              <h3 className="text-sm font-bold text-white flex items-center gap-2">
                Assistant RustForge
                <span className="text-[10px] bg-purple-500/20 text-purple-300 px-1.5 py-0.5 rounded font-mono">Tool Calling</span>
              </h3>
              {statusBadge}
            </div>
          </div>
          <div className="flex items-center gap-1">
            {messages.length > 0 && (
              <button
                onClick={() => setMessages([])}
                title="Nouvelle conversation"
                className="p-1.5 rounded-lg text-slate-400 hover:text-white hover:bg-white/5 transition-colors"
              >
                <RotateCcw className="w-4 h-4" />
              </button>
            )}
            <button onClick={onClose} className="p-1.5 rounded-lg text-slate-400 hover:text-white hover:bg-white/5 transition-colors">
              <X className="w-5 h-5" />
            </button>
          </div>
        </div>

        {/* Conversation */}
        <div ref={scrollRef} className="flex-1 overflow-y-auto p-5 space-y-4">
          {messages.length === 0 && (
            <div className="space-y-3">
              <p className="text-xs text-slate-400">
                Pilotez le fond, la forme et la structure du site en langage naturel. Le modèle choisit et exécute lui-même les outils du CMS.
              </p>
              <p className="text-[11px] font-semibold text-slate-400 uppercase tracking-wider">Suggestions</p>
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-2">
                {SUGGESTIONS.map((s) => (
                  <button
                    key={s.label}
                    onClick={() => send(s.prompt)}
                    className="text-left p-2.5 rounded-lg bg-white/5 hover:bg-white/10 border border-white/5 hover:border-purple-500/30 text-xs text-slate-300 flex items-center gap-2.5 transition-all"
                  >
                    <s.icon className={`w-4 h-4 shrink-0 ${s.color}`} />
                    <span className="truncate">{s.label}</span>
                  </button>
                ))}
              </div>
            </div>
          )}

          {messages.map((m, i) =>
            m.role === 'user' ? (
              <div key={i} className="flex justify-end">
                <div className="max-w-[85%] rounded-2xl rounded-br-sm bg-indigo-600/80 px-3.5 py-2 text-sm text-white whitespace-pre-wrap">{m.content}</div>
              </div>
            ) : (
              <div key={i} className="flex gap-2.5">
                <div className={`w-6 h-6 rounded-md shrink-0 flex items-center justify-center ${m.error ? 'bg-red-500/20' : 'bg-purple-500/20'}`}>
                  {m.error ? <AlertCircle className="w-3.5 h-3.5 text-red-400" /> : <Sparkles className="w-3.5 h-3.5 text-purple-300" />}
                </div>
                <div className="flex-1 min-w-0 space-y-2">
                  {m.actions && m.actions.length > 0 && (
                    <div className="space-y-1.5">
                      <p className="flex items-center gap-1.5 text-[10px] uppercase tracking-wider text-slate-500">
                        <Wrench className="w-3 h-3" /> {m.actions.length} action(s) exécutée(s)
                      </p>
                      {m.actions.map((a, j) => (
                        <ActionCard key={j} action={a} />
                      ))}
                    </div>
                  )}
                  <div className={`text-sm whitespace-pre-wrap ${m.error ? 'text-red-300' : 'text-slate-200'}`}>{m.content}</div>
                  {m.meta && <p className="text-[10px] text-slate-500 font-mono">{m.meta}</p>}
                </div>
              </div>
            ),
          )}

          {loading && (
            <div className="flex items-center gap-2 text-xs text-slate-400">
              <Loader2 className="w-4 h-4 animate-spin text-purple-400" />
              Le modèle réfléchit et exécute les outils…
            </div>
          )}
        </div>

        {/* Saisie */}
        <div className="p-4 border-t border-white/10">
          <div className="relative">
            <textarea
              id="ai-assistant-input"
              rows={2}
              value={prompt}
              autoFocus
              onChange={(e) => setPrompt(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter' && !e.shiftKey) {
                  e.preventDefault();
                  send();
                }
              }}
              placeholder="Ex : « Rédige une page À propos pour une agence web lyonnaise et publie-la »"
              className="w-full rounded-xl bg-black/40 border border-white/10 focus:border-purple-500 focus:ring-1 focus:ring-purple-500 p-3 pr-28 text-sm text-slate-100 placeholder-slate-500 outline-none resize-none transition-all"
            />
            <div className="absolute right-2.5 bottom-3 flex items-center gap-2">
              <button
                type="button"
                onClick={toggleListening}
                title={isListening ? 'Arrêter la dictée' : 'Dicter à la voix'}
                className={`p-2 rounded-lg transition-all ${isListening ? 'bg-red-500 text-white animate-pulse' : 'bg-white/5 hover:bg-white/10 text-slate-300'}`}
              >
                {isListening ? <MicOff className="w-4 h-4" /> : <Mic className="w-4 h-4" />}
              </button>
              <button
                id="ai-assistant-send"
                type="button"
                onClick={() => send()}
                disabled={loading || !prompt.trim()}
                className="px-3 py-2 rounded-lg bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 text-white text-xs font-semibold flex items-center gap-1.5 shadow-md shadow-indigo-600/30 transition-all"
              >
                <Send className="w-3.5 h-3.5" />
                Envoyer
              </button>
            </div>
          </div>
          <p className="mt-1.5 text-[10px] text-slate-500">Entrée pour envoyer · Maj+Entrée pour un saut de ligne · Échap pour fermer</p>
        </div>
      </div>
    </div>
  );
};
