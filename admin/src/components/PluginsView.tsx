/**
 * ============================================================================
 * VUE DE GESTION DES PLUGINS WEBASSEMBLY (`PluginsView`)
 * ============================================================================
 */

import React, { useState, useEffect } from 'react';
import { Boxes, RefreshCw, ShieldCheck, Power, AlertTriangle } from 'lucide-react';
import { api } from '../api';
import type { PluginInfo } from '../types';

export const PluginsView: React.FC = () => {
  const [plugins, setPlugins] = useState<PluginInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [scanning, setScanning] = useState(false);

  const fetchPlugins = async () => {
    setLoading(true);
    try {
      const list = await api.plugins.list();
      setPlugins(list);
    } catch (err) {
      console.error(err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    let active = true;
    api.plugins
      .list()
      .then((list) => {
        if (active) setPlugins(list);
      })
      .catch((err) => console.error(err))
      .finally(() => {
        if (active) setLoading(false);
      });
    return () => {
      active = false;
    };
  }, []);

  const handleRescan = async () => {
    setScanning(true);
    try {
      const list = await api.plugins.rescan();
      setPlugins(list);
    } catch (err: any) {
      alert(`Erreur de scan : ${err.message}`);
    } finally {
      setScanning(false);
    }
  };

  const handleToggle = async (plugin: PluginInfo) => {
    try {
      if (plugin.enabled) {
        await api.plugins.disable(plugin.id);
      } else {
        await api.plugins.enable(plugin.id);
      }
      await fetchPlugins();
    } catch (err: any) {
      alert(`Erreur : ${err.message}`);
    }
  };

  return (
    <div className="space-y-6">
      {/* Explication WASM */}
      <div className="p-6 rounded-2xl bg-gradient-to-r from-emerald-950/30 via-slate-900 to-[#0f1422] border border-emerald-500/20 flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
        <div className="space-y-1">
          <div className="inline-flex items-center gap-2 px-2.5 py-0.5 rounded-full bg-emerald-500/10 text-emerald-400 text-[11px] font-semibold border border-emerald-500/20">
            <ShieldCheck className="w-3.5 h-3.5" />
            <span>Architecture Zero-Trust WebAssembly</span>
          </div>
          <h3 className="text-base font-bold text-white">
            Plugins Sandboxés Wasmtime (WASIp2)
          </h3>
          <p className="text-xs text-slate-400 max-w-xl">
            Contrairement aux runtimes dynamiques historiques non sécurisés, chaque micro-plugin
            RustForge s'exécute dans un bac à sable mémoire étanche avec quotas stricts de CPU
            (fuel) et sans accès réseau ou disque non autorisé.
          </p>
        </div>

        <button
          onClick={handleRescan}
          disabled={scanning}
          className="shrink-0 px-4 py-2 rounded-xl bg-white/5 hover:bg-white/10 border border-white/10 text-slate-200 text-xs font-semibold flex items-center gap-2 transition-all"
        >
          <RefreshCw className={`w-3.5 h-3.5 ${scanning ? 'animate-spin' : ''}`} />
          <span>Scanner plugins/*.wasm</span>
        </button>
      </div>

      {/* Liste des plugins */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {loading ? (
          <div className="col-span-full py-12 text-center text-xs text-slate-500">
            Chargement des plugins WASM...
          </div>
        ) : plugins.length === 0 ? (
          <div className="col-span-full py-12 text-center text-xs text-slate-500">
            Aucun plugin détecté dans le répertoire plugins/. Cliquez sur "Scanner" après avoir
            déposé un fichier .wasm.
          </div>
        ) : (
          plugins.map((plugin) => (
            <div
              key={plugin.id}
              className={`p-5 rounded-2xl bg-[#0f1422] border transition-all ${
                plugin.enabled
                  ? 'border-emerald-500/30 shadow-md shadow-emerald-500/5'
                  : 'border-white/5 opacity-80'
              }`}
            >
              <div className="flex items-start justify-between gap-3">
                <div className="flex items-center gap-3">
                  <div className="w-10 h-10 rounded-xl bg-indigo-500/10 border border-indigo-500/20 flex items-center justify-center text-indigo-400">
                    <Boxes className="w-5 h-5" />
                  </div>
                  <div>
                    <h4 className="text-sm font-bold text-white flex items-center gap-2">
                      {plugin.name}
                      <span className="text-[10px] font-mono px-1.5 py-0.2 rounded bg-white/5 text-slate-400">
                        v{plugin.version}
                      </span>
                    </h4>
                    <p className="text-[11px] text-slate-500 font-mono">{plugin.file}</p>
                  </div>
                </div>

                <button
                  onClick={() => handleToggle(plugin)}
                  disabled={!!plugin.error}
                  className={`px-3 py-1.5 rounded-lg text-xs font-semibold flex items-center gap-1.5 transition-all ${
                    plugin.enabled
                      ? 'bg-emerald-500/20 text-emerald-300 border border-emerald-500/40 hover:bg-emerald-500/30'
                      : 'bg-white/5 text-slate-400 border border-white/10 hover:bg-white/10 hover:text-white'
                  }`}
                >
                  <Power className="w-3.5 h-3.5" />
                  <span>{plugin.enabled ? 'Actif' : 'Désactivé'}</span>
                </button>
              </div>

              <p className="text-xs text-slate-300 mt-3 leading-relaxed">
                {plugin.description || 'Aucune description fournie dans le manifeste.'}
              </p>

              {/* Hooks & Permissions */}
              <div className="mt-4 pt-3 border-t border-white/5 flex flex-wrap items-center gap-2">
                <span className="text-[10px] text-slate-500 uppercase font-semibold">Hooks :</span>
                {plugin.hooks.map((h) => (
                  <span
                    key={h}
                    className="text-[10px] font-mono px-2 py-0.5 rounded-full bg-indigo-500/10 text-indigo-300 border border-indigo-500/20"
                  >
                    {h}
                  </span>
                ))}
              </div>

              {plugin.error && (
                <div className="mt-3 p-2.5 rounded-xl bg-red-500/10 border border-red-500/30 text-red-300 text-[11px] flex items-center gap-2">
                  <AlertTriangle className="w-4 h-4 shrink-0" />
                  <span className="truncate">{plugin.error}</span>
                </div>
              )}
            </div>
          ))
        )}
      </div>
    </div>
  );
};
