/**
 * ============================================================================
 * BARRE SUPÉRIEURE DE NAVIGATION SOUVERAINE (`RfAdminBar`)
 * ============================================================================
 * Reproduit à 100% l'organisation et l'ergonomie de la barre officielle de
 * WordPress (`#rfadminbar`) avec les compléments sobres d'Adminify :
 * - Hauteur normalisée 32px, fond charcoal `#1d2327`, texte `#c3c4c7`.
 * - Côté gauche : Logo RustForge avec menu contextuel, nom du site avec icône
 *   maison et lien direct public, déclencheur rapide Ctrl+K, bulle de
 *   commentaires, et menu déroulant « + Créer » (Article, Page, Média, Compte).
 * - Côté droit : Commutateur sombre/clair, boutons rétractables « Options de l'écran ▾ »
 *   et « Aide ▾ », et salutation « Bonjour, {utilisateur} » avec avatar.
 */

import React, { useState, useRef, useEffect } from 'react';
import {
  Home,
  MessageSquare,
  Plus,
  ExternalLink,
  Search,
  Sun,
  Moon,
  ChevronDown,
  User as UserIcon,
  LogOut,
  Flame,
} from 'lucide-react';
import type { User } from '../types';

interface RfAdminBarProps {
  user: User;
  mobileOpen: boolean;
  onToggleMobile: () => void;
  siteTitle?: string;
  onNavigate: (route: string) => void;
  onLogout: () => void;
  onOpenCommandPalette: () => void;
  darkMode: boolean;
  onToggleDarkMode: () => void;
}

export const RfAdminBar: React.FC<RfAdminBarProps> = ({
  user,
  mobileOpen,
  onToggleMobile,
  siteTitle = 'RustForge Site',
  onNavigate,
  onLogout,
  onOpenCommandPalette,
  darkMode,
  onToggleDarkMode,
}) => {
  const [rfMenuOpen, setRfMenuOpen] = useState(false);
  const [newMenuOpen, setNewMenuOpen] = useState(false);
  const [userMenuOpen, setUserMenuOpen] = useState(false);

  const rfRef = useRef<HTMLDivElement>(null);
  const newRef = useRef<HTMLDivElement>(null);
  const userRef = useRef<HTMLDivElement>(null);

  // Fermer les menus lors d'un clic extérieur
  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      const target = e.target as Node;
      if (rfRef.current && !rfRef.current.contains(target)) setRfMenuOpen(false);
      if (newRef.current && !newRef.current.contains(target)) setNewMenuOpen(false);
      if (userRef.current && !userRef.current.contains(target)) setUserMenuOpen(false);
    };
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  useEffect(() => {
    const close = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        for (const ref of [rfRef, newRef, userRef]) { if (ref.current?.contains(document.activeElement)) ref.current.querySelector('button')?.focus(); }
        setRfMenuOpen(false); setNewMenuOpen(false); setUserMenuOpen(false);
      }
    };
    document.addEventListener('keydown', close);
    return () => document.removeEventListener('keydown', close);
  }, []);

  return (
    <header
      id="rfadminbar"
      role="banner"
      aria-label="Barre d'administration RustForge"
      className="fixed top-0 left-0 right-0 z-50 h-8 bg-[#1d2327] border-b border-[#2c3338] text-[#c3c4c7] select-none flex items-center justify-between px-2 text-[13px] font-sans antialiased"
    >
      {/* ==================================================================== */}
      {/* CÔTÉ GAUCHE : Logo, Site, Raccourcis, + Créer */}
      {/* ==================================================================== */}
      <button type="button" className="rf-mobile-toggle" aria-label="Menu" aria-expanded={mobileOpen} aria-controls="rfadminmenu" onClick={onToggleMobile}>☰</button>
      <div className="flex items-center h-full">
        {/* 1. Menu Logo RustForge */}
        <div ref={rfRef} className="relative h-full" onMouseLeave={() => setRfMenuOpen(false)}>
          <button
            type="button"
            onClick={() => setRfMenuOpen(true)}
            onMouseEnter={() => setRfMenuOpen(true)}
            title="À propos de RustForge CMS" aria-expanded={rfMenuOpen} aria-haspopup="true"
            className="flex items-center justify-center h-full px-2.5 hover:bg-[#2c3338] hover:text-[#72aee6] transition-colors"
          >
            <div className="rf-brand-mark w-4 h-4 rounded-full flex items-center justify-center shadow-sm">
              <Flame className="w-2.5 h-2.5 text-white" />
            </div>
          </button>

          {rfMenuOpen && (
            <div
              onMouseLeave={() => setRfMenuOpen(false)}
              className="absolute left-0 top-8 w-48 bg-[#2c3338] shadow-2xl py-1 text-xs border border-white/5 z-50"
            >
              <button
                type="button"
                onClick={() => {
                  setRfMenuOpen(false);
                  onNavigate('dashboard');
                }}
                className="w-full text-left px-4 py-2 hover:bg-[#2271b1] hover:text-white text-[#c3c4c7] transition-colors"
              >
                À propos de RustForge
              </button>
              <a
                href="https://github.com/rustforge"
                target="_blank"
                rel="noreferrer"
                className="flex items-center justify-between w-full text-left px-4 py-2 hover:bg-[#2271b1] hover:text-white text-[#c3c4c7] transition-colors"
              >
                <span>Documentation</span>
                <ExternalLink className="w-3 h-3 text-slate-400" />
              </a>
              <button
                type="button"
                onClick={() => {
                  setRfMenuOpen(false);
                  onNavigate('ai-studio');
                }}
                className="w-full text-left px-4 py-2 hover:bg-[#2271b1] hover:text-white text-[#c3c4c7] transition-colors"
              >
                Assistant IA Souverain
              </button>
            </div>
          )}
        </div>

        {/* 2. Nom du site + icône Maison */}
        <a
          href="/"
          target="_blank"
          rel="noopener noreferrer"
          title={`Voir le site « ${siteTitle} »`}
          className="flex items-center gap-1.5 h-full px-3 hover:bg-[#2c3338] hover:text-[#72aee6] transition-colors text-white font-medium"
        >
          <Home className="w-3.5 h-3.5 text-slate-400" />
          <span className="truncate max-w-[160px] md:max-w-[220px]">{siteTitle}</span>
        </a>

        {/* 3. Recherche globale Ctrl+K */}
        <button
          type="button"
          onClick={onOpenCommandPalette}
          title="Recherche universelle et commandes (Ctrl+K)"
          className="hidden sm:flex items-center gap-1.5 h-full px-2.5 hover:bg-[#2c3338] hover:text-[#72aee6] transition-colors text-[12px]"
        >
          <Search className="w-3.5 h-3.5 text-slate-400" />
          <span className="text-slate-400">Rechercher…</span>
          <kbd className="px-1 py-0.2 rounded bg-black/40 text-[10px] text-slate-400 font-mono border border-white/10">
            Ctrl+K
          </kbd>
        </button>

        {/* 4. Commentaires avec bulle de compteur */}
        <button
          type="button"
          onClick={() => onNavigate('comments')}
          title="Gérer les commentaires"
          className="flex items-center gap-1 h-full px-2.5 hover:bg-[#2c3338] hover:text-[#72aee6] transition-colors"
        >
          <MessageSquare className="w-3.5 h-3.5 text-slate-400" />
        </button>

        {/* 5. Menu « + Créer » (Article, Page, Média, Compte) */}
        <div ref={newRef} className="relative h-full" onMouseLeave={() => setNewMenuOpen(false)}>
          <button
            type="button"
            onClick={() => setNewMenuOpen(true)}
            onMouseEnter={() => setNewMenuOpen(true)}
            title="Créer un nouveau contenu" aria-expanded={newMenuOpen} aria-haspopup="true"
            className="flex items-center gap-1 h-full px-2.5 hover:bg-[#2c3338] hover:text-[#72aee6] transition-colors"
          >
            <Plus className="w-3.5 h-3.5 text-slate-400" />
            <span className="hidden md:inline font-medium">Créer</span>
            <ChevronDown className="w-3 h-3 text-slate-500" />
          </button>

          {newMenuOpen && (
            <div
              onMouseLeave={() => setNewMenuOpen(false)}
              className="absolute left-0 top-8 w-44 bg-[#2c3338] shadow-2xl py-1 text-xs border border-white/5 z-50"
            >
              <button
                type="button"
                onClick={() => {
                  setNewMenuOpen(false);
                  onNavigate('new:post');
                }}
                className="w-full text-left px-4 py-2 hover:bg-[#2271b1] hover:text-white text-[#c3c4c7] transition-colors"
              >
                Article
              </button>
              <button
                type="button"
                onClick={() => {
                  setNewMenuOpen(false);
                  onNavigate('new:page');
                }}
                className="w-full text-left px-4 py-2 hover:bg-[#2271b1] hover:text-white text-[#c3c4c7] transition-colors"
              >
                Page
              </button>
              <button
                type="button"
                onClick={() => {
                  setNewMenuOpen(false);
                  onNavigate('media');
                }}
                className="w-full text-left px-4 py-2 hover:bg-[#2271b1] hover:text-white text-[#c3c4c7] transition-colors"
              >
                Fichier média
              </button>
              {user.role === 'admin' && (
                <button
                  type="button"
                  onClick={() => {
                    setNewMenuOpen(false);
                    onNavigate('users:new');
                  }}
                  className="w-full text-left px-4 py-2 hover:bg-[#2271b1] hover:text-white text-[#c3c4c7] transition-colors"
                >
                  Compte utilisateur
                </button>
              )}
            </div>
          )}
        </div>
      </div>

      {/* ==================================================================== */}
      {/* CÔTÉ DROIT : Thème, Tiroirs Options / Aide, Compte utilisateur */}
      {/* ==================================================================== */}
      <div className="flex items-center h-full space-x-1">
        {/* Commutateur mode clair / sombre (Inspiration Adminify) */}
        <button
          type="button"
          onClick={onToggleDarkMode}
          title={darkMode ? 'Basculer en mode clair' : 'Basculer en mode sombre'}
          className="flex items-center justify-center w-8 h-full hover:bg-[#2c3338] hover:text-amber-400 transition-colors"
        >
          {darkMode ? (
            <Sun className="w-3.5 h-3.5 text-amber-400" />
          ) : (
            <Moon className="w-3.5 h-3.5 text-slate-300" />
          )}
        </button>

        {/* Coin utilisateur « Bonjour, {user} » */}
        <div ref={userRef} className="relative h-full" onMouseLeave={() => setUserMenuOpen(false)}>
          <button
            type="button"
            aria-expanded={userMenuOpen} aria-haspopup="true" onClick={() => setUserMenuOpen(true)}
            onMouseEnter={() => setUserMenuOpen(true)}
            className="flex items-center gap-2 h-full px-3 hover:bg-[#2c3338] hover:text-[#72aee6] transition-colors text-slate-200"
          >
            <span className="rf-account-name hidden md:inline text-xs">
              Bonjour, <strong className="font-semibold text-white">{user.display_name || user.username}</strong>
            </span>
            <div className="w-5 h-5 rounded-full bg-[#3c434a] border border-white/20 flex items-center justify-center text-[10px] text-white font-bold uppercase overflow-hidden">
              {(user.display_name || user.username).slice(0, 2)}
            </div>
          </button>

          {userMenuOpen && (
            <div
              onMouseLeave={() => setUserMenuOpen(false)}
              className="absolute right-0 top-8 w-48 bg-[#2c3338] shadow-2xl py-1 text-xs border border-white/5 z-50"
            >
              <div className="px-4 py-2 border-b border-white/5 text-[11px] text-slate-400">
                Connecté en tant que :<br />
                <strong className="text-white font-semibold text-xs">{user.email}</strong>
              </div>
              <button
                type="button"
                onClick={() => {
                  setUserMenuOpen(false);
                  onNavigate('profile');
                }}
                className="flex items-center gap-2 w-full text-left px-4 py-2 hover:bg-[#2271b1] hover:text-white text-[#c3c4c7] transition-colors"
              >
                <UserIcon className="w-3.5 h-3.5" />
                <span>Modifier mon profil</span>
              </button>
              <button
                type="button"
                onClick={() => {
                  setUserMenuOpen(false);
                  onLogout();
                }}
                className="flex items-center gap-2 w-full text-left px-4 py-2 hover:bg-red-600 hover:text-white text-red-300 transition-colors border-t border-white/5"
              >
                <LogOut className="w-3.5 h-3.5" />
                <span>Se déconnecter</span>
              </button>
            </div>
          )}
        </div>
      </div>
    </header>
  );
};

