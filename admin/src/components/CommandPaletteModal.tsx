/**
 * ============================================================================
 * PALETTE DE COMMANDE UNIVERSELLE (`CommandPaletteModal`)
 * ============================================================================
 * Inspirée d'Adminify, Raycast et WordPress 6.x :
 * - Raccourci universel `Ctrl+K` / `Cmd+K`.
 * - Recherche floue instantanée de vues, d'actions et de créations de contenus.
 * - Pilotage au clavier intégral (`↑`, `↓`, `Entrée`, `Échap`).
 * - Bascule immédiate des modes Zen et Densité (Sobriété d'interface).
 */

import React, { useState, useEffect, useMemo, useRef } from 'react';
import {
  Search,
  LayoutDashboard,
  FileText,
  File,
  Image as ImageIcon,
  MessageSquare,
  ShoppingBag,
  GraduationCap,
  Palette,
  Settings,
  Users,
  Activity,
  Archive,
  PlusCircle,
  Maximize2,
  Minimize2,
  Sparkles,
  SlidersHorizontal,
  ExternalLink,
  Layers,
  BarChart3,
  Languages,
  Globe,
} from 'lucide-react';
import { useStudioStore } from '../stores/useStudioStore';

interface CommandItem {
  id: string;
  title: string;
  section: 'Navigation' | 'Création Rapide' | 'Actions & Sobriété';
  icon: React.ElementType;
  keywords: string[];
  shortcut?: string;
  action: () => void;
}

interface CommandPaletteModalProps {
  isOpen: boolean;
  onClose: () => void;
  onNavigate: (view: string) => void;
  onOpenAi: () => void;
}

export const CommandPaletteModal: React.FC<CommandPaletteModalProps> = ({
  isOpen,
  onClose,
  onNavigate,
  onOpenAi,
}) => {
  const [query, setQuery] = useState('');
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const listRef = useRef<HTMLDivElement>(null);

  const user = useStudioStore(s => s.user);
  const navigation = useStudioStore(s => s.navState);
  const zenMode = useStudioStore((s) => s.zenMode);
  const toggleZenMode = useStudioStore((s) => s.toggleZenMode);
  const studioDensity = useStudioStore((s) => s.studioDensity);
  const setStudioDensity = useStudioStore((s) => s.setStudioDensity);

  // Définition de l'inventaire exhaustif des commandes
  const commands: CommandItem[] = useMemo(
    () => [
      // Section : Navigation
      {
        id: 'nav:dashboard',
        title: 'Tableau de bord',
        section: 'Navigation',
        icon: LayoutDashboard,
        keywords: ['accueil', 'start', 'stats', 'dashboard'],
        action: () => {
          onNavigate('dashboard');
          onClose();
        },
      },
      {
        id: 'nav:posts',
        title: 'Articles',
        section: 'Navigation',
        icon: FileText,
        keywords: ['billets', 'blog', 'posts', 'actualites'],
        action: () => {
          onNavigate('entries:post');
          onClose();
        },
      },
      {
        id: 'nav:pages',
        title: 'Pages',
        section: 'Navigation',
        icon: File,
        keywords: ['pages', 'statique', 'arborescence'],
        action: () => {
          onNavigate('entries:page');
          onClose();
        },
      },
      {
        id: 'nav:media',
        title: 'Médiathèque & Dossiers',
        section: 'Navigation',
        icon: ImageIcon,
        keywords: ['images', 'photos', 'fichiers', 'filebird', 'medias'],
        action: () => {
          onNavigate('media');
          onClose();
        },
      },
      {
        id: 'nav:forms',
        title: 'Formulaires & Soumissions',
        section: 'Navigation',
        icon: MessageSquare,
        keywords: ['contact', 'messages', 'leads', 'inbox'],
        action: () => {
          onNavigate('module:forms');
          onClose();
        },
      },
      {
        id: 'nav:ecommerce',
        title: 'Boutique & Commandes',
        section: 'Navigation',
        icon: ShoppingBag,
        keywords: ['produits', 'achats', 'ventes', 'paniers', 'telechargements'],
        action: () => {
          onNavigate('ecommerce:products');
          onClose();
        },
      },
      {
        id: 'nav:lms',
        title: 'Formations & Curriculum',
        section: 'Navigation',
        icon: GraduationCap,
        keywords: ['cours', 'lecons', 'eleves', 'apprenants', 'formation'],
        action: () => {
          onNavigate('lms:courses');
          onClose();
        },
      },
      {
        id: 'nav:columns',
        title: 'Colonnes & Vues de données',
        section: 'Navigation',
        icon: SlidersHorizontal,
        keywords: ['admin columns', 'tableau', 'champs', 'export', 'colonnes'],
        action: () => {
          onNavigate('settings:columns');
          onClose();
        },
      },
      {
        id: 'nav:appearance',
        title: 'Apparence & Styles',
        section: 'Navigation',
        icon: Palette,
        keywords: ['themes', 'couleurs', 'polices', 'design', 'tokens'],
        action: () => {
          onNavigate('appearance');
          onClose();
        },
      },
      {
        id: 'nav:settings',
        title: 'Réglages Généraux',
        section: 'Navigation',
        icon: Settings,
        keywords: ['configuration', 'options', 'permalinks', 'site'],
        action: () => {
          onNavigate('settings');
          onClose();
        },
      },
      {
        id: 'nav:users',
        title: 'Utilisateurs & Profils',
        section: 'Navigation',
        icon: Users,
        keywords: ['comptes', 'auteurs', 'roles', 'membres'],
        action: () => {
          onNavigate('users');
          onClose();
        },
      },
      {
        id: 'nav:jobs',
        title: 'Tâches & Diagnostic de fond',
        section: 'Navigation',
        icon: Activity,
        keywords: ['file', 'queue', 'workers', 'status', 'logs'],
        action: () => {
          onNavigate('jobs');
          onClose();
        },
      },
      {
        id: 'nav:analytics',
        title: 'Statistiques & Mesure d’Audience',
        section: 'Navigation',
        icon: BarChart3,
        keywords: ['stats', 'visites', 'audience', 'analytics', 'trafic', 'visiteurs', 'cnil', 'rgpd'],
        action: () => {
          onNavigate('analytics');
          onClose();
        },
      },
      {
        id: 'nav:analytics_settings',
        title: 'Réglages de mesure d’audience',
        section: 'Navigation',
        icon: SlidersHorizontal,
        keywords: ['purge', 'retention', 'dnt', 'analytics settings', 'confidentialite'],
        action: () => {
          onNavigate('analytics:settings');
          onClose();
        },
      },
      {
        id: 'nav:translations',
        title: 'Traductions & Matrice des contenus',
        section: 'Navigation',
        icon: Languages,
        keywords: ['traductions', 'translations', 'multilingue', 'langues', 'i18n', 'locales'],
        action: () => {
          onNavigate('translations');
          onClose();
        },
      },
      {
        id: 'nav:catalogs',
        title: 'Éditeur de Catalogues UI (PO/JSON)',
        section: 'Navigation',
        icon: Globe,
        keywords: ['catalogues', 'gettext', 'po', 'traductions interface', 'loco'],
        action: () => {
          onNavigate('translations:catalogs');
          onClose();
        },
      },
      {
        id: 'nav:language_settings',
        title: 'Configuration des langues du site',
        section: 'Navigation',
        icon: SlidersHorizontal,
        keywords: ['langues', 'languages settings', 'hreflang', 'i18n'],
        action: () => {
          onNavigate('translations:settings');
          onClose();
        },
      },
      {
        id: 'nav:capsules',
        title: 'Sauvegardes & Restauration',
        section: 'Navigation',
        icon: Archive,
        keywords: ['backups', 'export', 'snapshots', 'capsules'],
        action: () => {
          onNavigate('capsules');
          onClose();
        },
      },

      // Section : Création Rapide
      {
        id: 'new:post',
        title: 'Rédiger un nouvel Article',
        section: 'Création Rapide',
        icon: PlusCircle,
        keywords: ['ecrire', 'nouveau', 'article', 'post'],
        shortcut: 'N',
        action: () => {
          onNavigate('new:post');
          onClose();
        },
      },
      {
        id: 'new:page',
        title: 'Créer une nouvelle Page',
        section: 'Création Rapide',
        icon: PlusCircle,
        keywords: ['creer', 'nouvelle', 'page'],
        action: () => {
          onNavigate('new:page');
          onClose();
        },
      },
      {
        id: 'new:product',
        title: 'Ouvrir les produits numériques',
        section: 'Création Rapide',
        icon: ShoppingBag,
        keywords: ['vendre', 'produit', 'tarifs'],
        action: () => {
          onNavigate('ecommerce:products');
          onClose();
        },
      },
      {
        id: 'new:course',
        title: 'Ouvrir les cours de formation',
        section: 'Création Rapide',
        icon: GraduationCap,
        keywords: ['lms', 'formation', 'cours', 'curriculum'],
        action: () => {
          onNavigate('lms:courses');
          onClose();
        },
      },

      // Section : Actions & Sobriété
      {
        id: 'action:zen',
        title: zenMode ? 'Désactiver le Mode Zen' : 'Activer le Mode Zen (Plein écran)',
        section: 'Actions & Sobriété',
        icon: zenMode ? Minimize2 : Maximize2,
        keywords: ['plein ecran', 'distraction free', 'focus', 'zen'],
        shortcut: 'F11',
        action: () => {
          toggleZenMode();
          onClose();
        },
      },
      {
        id: 'action:density',
        title:
          studioDensity === 'comfortable'
            ? 'Passer en Densité Compacte (Vue tableur épurée)'
            : 'Passer en Densité Confortable (Espacée)',
        section: 'Actions & Sobriété',
        icon: Layers,
        keywords: ['tableur', 'compact', 'adminify', 'hauteur', 'densite'],
        action: () => {
          setStudioDensity(studioDensity === 'comfortable' ? 'compact' : 'comfortable');
          onClose();
        },
      },
      {
        id: 'action:ai',
        title: "Ouvrir l'Assistant de Pilotage IA",
        section: 'Actions & Sobriété',
        icon: Sparkles,
        keywords: ['ai', 'copilote', 'gemini', 'assistant', 'prompt'],
        shortcut: '⌘I',
        action: () => {
          onClose();
          onOpenAi();
        },
      },
      {
        id: 'action:view_site',
        title: 'Ouvrir le site public dans un nouvel onglet',
        section: 'Actions & Sobriété',
        icon: ExternalLink,
        keywords: ['front', 'visiteur', 'apercu', 'site'],
        action: () => {
          window.open('/fr', '_blank');
          onClose();
        },
      },
    ],
    [onNavigate, onClose, onOpenAi, zenMode, toggleZenMode, studioDensity, setStudioDensity]
  );

  const allowedCommands = useMemo(() => {
    const routes = new Set(navigation?.userId === user?.id ? navigation?.data.items.map(n => n.route).filter(Boolean) : []);
    const targets: Record<string, string> = {
      'nav:dashboard':'dashboard','nav:posts':'entries:post','nav:pages':'entries:page','nav:media':'media',
      'nav:forms':'module:forms','nav:ecommerce':'ecommerce:products','nav:lms':'lms:courses','nav:columns':'settings:columns',
      'nav:appearance':'appearance','nav:settings':'settings','nav:users':'users','nav:jobs':'jobs','nav:capsules':'capsules',
      'new:post':'new:post','new:page':'new:page','new:product':'ecommerce:products','new:course':'lms:courses','action:ai':'ai-studio',
    };
    return commands.filter(c => targets[c.id] ? routes.has(targets[c.id]) : c.id.startsWith('action:'));
  }, [commands, navigation, user?.id]);

  // Filtrage réactif selon la saisie
  const filteredCommands = useMemo(() => {
    if (!query.trim()) return allowedCommands;
    const clean = query.toLowerCase().trim();
    return allowedCommands.filter((c) => {
      const matchTitle = c.title.toLowerCase().includes(clean);
      const matchSection = c.section.toLowerCase().includes(clean);
      const matchKeywords = c.keywords.some((k) => k.toLowerCase().includes(clean));
      return matchTitle || matchSection || matchKeywords;
    });
  }, [allowedCommands, query]);

  // Focus automatique du champ et initialisation à l'ouverture
  useEffect(() => {
    if (isOpen) {
      const timer = window.setTimeout(() => {
        setQuery('');
        setSelectedIndex(0);
        inputRef.current?.focus();
      }, 30);
      return () => window.clearTimeout(timer);
    }
  }, [isOpen]);

  // Gestion du clavier
  useEffect(() => {
    if (!isOpen) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'ArrowDown') {
        e.preventDefault();
        setSelectedIndex((prev) => (prev + 1 < filteredCommands.length ? prev + 1 : 0));
      } else if (e.key === 'ArrowUp') {
        e.preventDefault();
        setSelectedIndex((prev) => (prev - 1 >= 0 ? prev - 1 : filteredCommands.length - 1));
      } else if (e.key === 'Enter') {
        e.preventDefault();
        const active = filteredCommands[selectedIndex];
        if (active) {
          active.action();
        }
      } else if (e.key === 'Escape') {
        e.preventDefault();
        onClose();
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isOpen, filteredCommands, selectedIndex, onClose]);

  // Scroll automatique vers l'élément sélectionné
  useEffect(() => {
    if (listRef.current) {
      const activeEl = listRef.current.querySelector('[data-selected="true"]');
      if (activeEl) {
        activeEl.scrollIntoView({ block: 'nearest' });
      }
    }
  }, [selectedIndex]);

  if (!isOpen) return null;

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-label="Palette de commande universelle"
      className="fixed inset-0 z-50 flex items-start justify-center pt-20 px-4 bg-black/60 backdrop-blur-sm transition-opacity"
      onClick={onClose}
    >
      <div
        className="w-full max-w-xl rounded-2xl bg-[#0f1422] border border-white/10 shadow-2xl overflow-hidden flex flex-col text-slate-100 divide-y divide-white/5 animate-in fade-in zoom-in-95 duration-150"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Champ de recherche */}
        <div className="flex items-center gap-3 px-4 py-3 bg-white/[0.02]">
          <Search className="w-5 h-5 text-indigo-400 shrink-0" />
          <input
            ref={inputRef}
            type="text"
            value={query}
            onChange={(e) => {
              setQuery(e.target.value);
              setSelectedIndex(0);
            }}
            placeholder="Rechercher une vue, un contenu ou exécuter une action..."
            className="w-full bg-transparent border-0 outline-none text-sm placeholder:text-slate-500 text-white"
          />
          <kbd className="px-1.5 py-0.5 rounded bg-white/5 border border-white/10 text-[10px] text-slate-400 font-mono shrink-0">
            ÉCHAP
          </kbd>
        </div>

        {/* Liste des résultats */}
        <div ref={listRef} className="max-h-80 overflow-y-auto p-2 space-y-1">
          {filteredCommands.length === 0 ? (
            <div className="py-10 text-center text-xs text-slate-500">
              Aucune commande ou page ne correspond à « {query} »
            </div>
          ) : (
            filteredCommands.map((cmd, idx) => {
              const isSelected = idx === selectedIndex;
              const Icon = cmd.icon;
              return (
                <div
                  key={cmd.id}
                  data-selected={isSelected}
                  onClick={() => cmd.action()}
                  onMouseEnter={() => setSelectedIndex(idx)}
                  className={`flex items-center justify-between px-3 py-2 rounded-xl text-xs cursor-pointer transition-colors ${
                    isSelected
                      ? 'bg-indigo-600/20 text-white border border-indigo-500/30'
                      : 'hover:bg-white/5 text-slate-300 border border-transparent'
                  }`}
                >
                  <div className="flex items-center gap-2.5 min-w-0">
                    <div
                      className={`p-1.5 rounded-lg ${
                        isSelected ? 'bg-indigo-500 text-white' : 'bg-white/5 text-slate-400'
                      }`}
                    >
                      <Icon className="w-4 h-4" />
                    </div>
                    <div className="min-w-0">
                      <p className="font-medium truncate">{cmd.title}</p>
                      <p className="text-[10px] text-slate-500 truncate">{cmd.section}</p>
                    </div>
                  </div>

                  <div className="flex items-center gap-2 shrink-0">
                    {cmd.shortcut && (
                      <kbd className="px-1.5 py-0.5 rounded bg-white/5 border border-white/10 text-[10px] font-mono text-slate-400">
                        {cmd.shortcut}
                      </kbd>
                    )}
                    {isSelected && (
                      <span className="text-[10px] text-indigo-400 font-medium">↵ Entrée</span>
                    )}
                  </div>
                </div>
              );
            })
          )}
        </div>

        {/* Pied de palette avec rappels de raccourcis */}
        <div className="px-4 py-2.5 bg-white/[0.01] flex items-center justify-between text-[11px] text-slate-500">
          <div className="flex items-center gap-3">
            <span>
              <kbd className="font-mono text-slate-400">↑</kbd>{' '}
              <kbd className="font-mono text-slate-400">↓</kbd> Naviguer
            </span>
            <span>
              <kbd className="font-mono text-slate-400">↵</kbd> Sélectionner
            </span>
            <span>
              <kbd className="font-mono text-slate-400">Échap</kbd> Fermer
            </span>
          </div>
          <div className="flex items-center gap-1.5 text-indigo-400 font-medium">
            <Sparkles className="w-3.5 h-3.5" />
            <span>RustForge Sovereign</span>
          </div>
        </div>
      </div>
    </div>
  );
};
