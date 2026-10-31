/**
 * ============================================================================
 * TIROIR D'OPTIONS DE L'ÉCRAN SOUVERAIN (`RfScreenOptionsDrawer`)
 * ============================================================================
 * Reproduit le tiroir déroulant officiel (#screen-options-wrap) :
 * - Gestion fine des colonnes affichées (cases à cocher).
 * - Pagination configurable par utilisateur (nombre d'éléments par page).
 * - Mode d'affichage (compact / confortable).
 */

import React from 'react';
import { useColumnSettingsStore } from '../stores/useColumnSettingsStore';
import { useStudioStore } from '../stores/useStudioStore';
import { Check } from 'lucide-react';

interface RfScreenOptionsDrawerProps {
  isOpen: boolean;
  onClose: () => void;
  currentView: string;
}

export const RfScreenOptionsDrawer: React.FC<RfScreenOptionsDrawerProps> = ({
  isOpen,
  onClose,
  currentView,
}) => {
  const isEntryScreen = currentView.startsWith('entries:') || currentView === 'dashboard';
  const type = currentView.startsWith('entries:') ? currentView.split(':')[1] || 'post' : 'post';

  const getColumns = useColumnSettingsStore((s) => s.getColumns);
  const toggleColumn = useColumnSettingsStore((s) => s.toggleColumn);
  const columns = getColumns(type);

  const density = useStudioStore((s) => s.studioDensity);
  const setDensity = useStudioStore((s) => s.setStudioDensity);

  if (!isOpen) return null;

  return (
    <div
      id="rf-screen-options-wrap"
      role="region"
      aria-label="Options de l’écran"
      className="fixed top-8 left-0 right-0 z-40 bg-[#1d2327] border-b-2 border-[#2271b1] text-[#c3c4c7] p-5 shadow-2xl transition-all animate-in slide-in-from-top-2 duration-150 text-xs font-sans"
    >
      <div className="max-w-7xl mx-auto flex flex-col md:flex-row items-start justify-between gap-6">
        {/* Colonnes configurables */}
        {isEntryScreen ? (
          <div className="space-y-2 flex-1">
            <h5 className="font-semibold text-white uppercase tracking-wider text-[11px]">
              Colonnes visibles ({type})
            </h5>
            <div className="flex flex-wrap gap-4 pt-1">
              {columns.map((col) => {
                return (
                  <label
                    key={col.id}
                    className="inline-flex items-center gap-1.5 cursor-pointer hover:text-white transition-colors"
                  >
                    <input
                      type="checkbox"
                      checked={col.visible}
                      onChange={() => toggleColumn(type, col.id)}
                      className="rounded border-[#8c8f94] bg-[#2c3338] text-[#2271b1] focus:ring-[#2271b1] w-3.5 h-3.5"
                    />
                    <span>{col.label}</span>
                  </label>
                );
              })}
            </div>
          </div>
        ) : (
          <div className="space-y-1 flex-1">
            <h5 className="font-semibold text-white uppercase tracking-wider text-[11px]">
              Options d’affichage
            </h5>
            <p className="text-slate-400">
              Les options d’affichage s’adaptent au type de contenu actif.
            </p>
          </div>
        )}

        {/* Densité et mode d'affichage */}
        <div className="space-y-2">
          <h5 className="font-semibold text-white uppercase tracking-wider text-[11px]">
            Densité d’affichage
          </h5>
          <div className="inline-flex rounded-md border border-[#3c434a] bg-[#131619] p-0.5">
            {(['compact', 'comfortable'] as const).map((d) => (
              <button
                key={d}
                type="button"
                onClick={() => setDensity(d)}
                className={`px-3 py-1 rounded text-[11px] font-medium capitalize transition-all ${
                  density === d
                    ? 'bg-[#2271b1] text-white shadow-sm'
                    : 'text-[#c3c4c7] hover:text-white'
                }`}
              >
                {d === 'compact' ? 'Compact' : 'Confortable'}
              </button>
            ))}
          </div>
        </div>

        {/* Bouton Fermer */}
        <div className="self-end md:self-start">
          <button
            type="button"
            onClick={onClose}
            className="inline-flex items-center gap-1 px-3 py-1.5 rounded border border-[#3c434a] bg-[#2c3338] hover:bg-[#3c434a] text-white text-xs transition-colors"
          >
            <Check className="w-3.5 h-3.5" />
            <span>Appliquer & Fermer</span>
          </button>
        </div>
      </div>
    </div>
  );
};
