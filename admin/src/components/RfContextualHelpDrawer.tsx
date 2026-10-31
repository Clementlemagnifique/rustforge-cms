/**
 * ============================================================================
 * TIROIR D'AIDE CONTEXTUELLE SOUVERAIN (`RfContextualHelpDrawer`)
 * ============================================================================
 * Reproduit le tiroir d'aide (#contextual-help-wrap) :
 * - Documentation contextuelle de l'écran affiché.
 * - Liens vers les ressources souveraines RustForge.
 */

import React, { useState } from 'react';
import { ExternalLink } from 'lucide-react';

interface RfContextualHelpDrawerProps {
  isOpen: boolean;
  onClose: () => void;
  currentView: string;
}

export const RfContextualHelpDrawer: React.FC<RfContextualHelpDrawerProps> = ({
  isOpen,
  onClose,
  currentView,
}) => {
  const [activeTab, setActiveTab] = useState<'overview' | 'actions' | 'shortcuts'>('overview');

  if (!isOpen) return null;

  return (
    <div
      id="rf-contextual-help-wrap"
      role="region"
      aria-label="Aide contextuelle"
      className="fixed top-8 left-0 right-0 z-40 bg-[#1d2327] border-b-2 border-[#2271b1] text-[#c3c4c7] p-5 shadow-2xl transition-all animate-in slide-in-from-top-2 duration-150 text-xs font-sans"
    >
      <div className="max-w-7xl mx-auto grid grid-cols-1 md:grid-cols-12 gap-6">
        {/* Navigation gauche des onglets d'aide */}
        <div className="md:col-span-3 border-r border-[#2c3338] pr-4 space-y-1">
          <h5 className="font-semibold text-white uppercase tracking-wider text-[11px] mb-2">
            Rubriques d’aide
          </h5>
          <button
            type="button"
            onClick={() => setActiveTab('overview')}
            className={`w-full text-left px-3 py-1.5 rounded transition-colors ${
              activeTab === 'overview'
                ? 'bg-[#2271b1] text-white font-medium'
                : 'text-[#c3c4c7] hover:bg-[#2c3338] hover:text-white'
            }`}
          >
            Vue d’ensemble
          </button>
          <button
            type="button"
            onClick={() => setActiveTab('actions')}
            className={`w-full text-left px-3 py-1.5 rounded transition-colors ${
              activeTab === 'actions'
                ? 'bg-[#2271b1] text-white font-medium'
                : 'text-[#c3c4c7] hover:bg-[#2c3338] hover:text-white'
            }`}
          >
            Actions groupées & Filtres
          </button>
          <button
            type="button"
            onClick={() => setActiveTab('shortcuts')}
            className={`w-full text-left px-3 py-1.5 rounded transition-colors ${
              activeTab === 'shortcuts'
                ? 'bg-[#2271b1] text-white font-medium'
                : 'text-[#c3c4c7] hover:bg-[#2c3338] hover:text-white'
            }`}
          >
            Raccourcis clavier
          </button>
        </div>

        {/* Contenu central */}
        <div className="md:col-span-6 space-y-2 text-slate-300">
          <h4 className="text-sm font-semibold text-white">
            {activeTab === 'overview' && 'Navigation et gestion de l’écran'}
            {activeTab === 'actions' && 'Actions groupées et modification rapide'}
            {activeTab === 'shortcuts' && 'Raccourcis de productivité RustForge'}
          </h4>

          {activeTab === 'overview' && (
            <p className="leading-relaxed">
              Cet écran vous permet d’organiser, filtrer et modifier rapidement vos données. Les
              colonnes affichées peuvent être personnalisées en cliquant sur l’onglet « Options de
              l’écran » situé en haut à droite. Vue active : <span className="font-mono text-[#72aee6]">{currentView}</span>.
            </p>
          )}

          {activeTab === 'actions' && (
            <p className="leading-relaxed">
              Sélectionnez une ou plusieurs lignes pour appliquer une action groupée (corbeille,
              publication, duplication). En survolant une ligne, vous accédez directement aux actions
              rapides : Modifier, Modification rapide, Mettre à la corbeille, et Afficher.
            </p>
          )}

          {activeTab === 'shortcuts' && (
            <div className="grid grid-cols-2 gap-2 pt-1 font-mono text-[11px]">
              <div className="flex items-center justify-between p-2 rounded bg-[#131619] border border-[#2c3338]">
                <span>Palette de commandes</span>
                <kbd className="px-1.5 py-0.5 rounded bg-[#2c3338] text-white">Ctrl + K</kbd>
              </div>
              <div className="flex items-center justify-between p-2 rounded bg-[#131619] border border-[#2c3338]">
                <span>Assistant IA & Voix</span>
                <kbd className="px-1.5 py-0.5 rounded bg-[#2c3338] text-white">Ctrl + I</kbd>
              </div>
            </div>
          )}
        </div>

        {/* Colonne droite : Liens souverains & fermeture */}
        <div className="md:col-span-3 border-l border-[#2c3338] pl-4 space-y-3">
          <h5 className="font-semibold text-white uppercase tracking-wider text-[11px]">
            Pour plus d’informations
          </h5>
          <ul className="space-y-1.5">
            <li>
              <a
                href="https://github.com/rustforge"
                target="_blank"
                rel="noreferrer"
                className="inline-flex items-center gap-1 text-[#72aee6] hover:underline"
              >
                <span>Documentation RustForge</span>
                <ExternalLink className="w-3 h-3" />
              </a>
            </li>
            <li>
              <a
                href="#jobs"
                className="inline-flex items-center gap-1 text-[#72aee6] hover:underline"
              >
                <span>Santé du site & Tâches</span>
              </a>
            </li>
          </ul>

          <div className="pt-2">
            <button
              type="button"
              onClick={onClose}
              className="px-3 py-1 rounded border border-[#3c434a] bg-[#2c3338] hover:bg-[#3c434a] text-white text-xs transition-colors"
            >
              Fermer l’aide
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};
