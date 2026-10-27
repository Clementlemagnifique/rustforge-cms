import React, { useState } from 'react';
import {
  Download,
  UploadCloud,
  Layers,
  Database,
  Image,
  FileText,
  ShieldCheck,
  CheckCircle2,
  AlertCircle,
  RefreshCw,
  Globe
} from 'lucide-react';
import { api } from '../api';
import type { DashboardStats } from '../types';

interface CapsulesViewProps {
  stats: DashboardStats | null;
}

export const CapsulesView: React.FC<CapsulesViewProps> = ({ stats }) => {
  // État Export
  const [exporting, setExporting] = useState(false);
  const [exportSuccess, setExportSuccess] = useState(false);
  const [exportError, setExportError] = useState<string | null>(null);

  // État Import / Restauration
  const [capsuleFile, setCapsuleFile] = useState<File | null>(null);
  const [dragActive, setDragActive] = useState(false);
  const [inspecting, setInspecting] = useState(false);
  const [manifest, setManifest] = useState<any | null>(null);
  const [targetSiteUrl, setTargetSiteUrl] = useState(window.location.origin);
  const [restoring, setRestoring] = useState(false);
  const [restoreError, setRestoreError] = useState<string | null>(null);
  const [restoreReport, setRestoreReport] = useState<any | null>(null);

  const handleExport = async () => {
    setExportError(null);
    setExportSuccess(false);
    setExporting(true);
    try {
      await api.capsules.exportBlob();
      setExportSuccess(true);
    } catch (err: any) {
      setExportError(err.message || "Erreur lors de l'exportation de la capsule.");
    } finally {
      setExporting(false);
    }
  };

  const handleDrag = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    if (e.type === 'dragenter' || e.type === 'dragover') {
      setDragActive(true);
    } else if (e.type === 'dragleave') {
      setDragActive(false);
    }
  };

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setDragActive(false);
    if (e.dataTransfer.files && e.dataTransfer.files[0]) {
      handleFileSelected(e.dataTransfer.files[0]);
    }
  };

  const handleFileSelected = async (file: File) => {
    if (!file.name.endsWith('.rfcapsule') && !file.name.endsWith('.zip')) {
      setRestoreError("Le fichier doit comporter l'extension .rfcapsule ou .zip");
      return;
    }

    setCapsuleFile(file);
    setRestoreError(null);
    setRestoreReport(null);
    setInspecting(true);

    try {
      const res = await api.capsules.inspect(file);
      setManifest(res.manifest);
    } catch (err: any) {
      setRestoreError(err.message || "Archive capsule invalide ou corrompue.");
      setManifest(null);
    } finally {
      setInspecting(false);
    }
  };

  const handleRestore = async () => {
    if (!capsuleFile) return;
    setRestoreError(null);
    setRestoring(true);

    try {
      const report = await api.capsules.restore(capsuleFile, targetSiteUrl);
      setRestoreReport(report);
    } catch (err: any) {
      setRestoreError(err.message || "Erreur lors de la restauration de la capsule.");
    } finally {
      setRestoring(false);
    }
  };

  return (
    <div className="p-8 max-w-6xl mx-auto space-y-8 select-none">
      {/* Header */}
      <div>
        <div className="flex items-center gap-3 mb-2">
          <div className="w-10 h-10 rounded-xl bg-gradient-to-tr from-cyan-500 via-indigo-500 to-purple-600 flex items-center justify-center text-white shadow-lg shadow-indigo-500/20">
            <Layers className="w-5 h-5" />
          </div>
          <div>
            <h1 className="text-xl font-bold text-white tracking-tight flex items-center gap-2">
              Capsules Souveraines & Migration
              <span className="text-[10px] uppercase font-bold px-2 py-0.5 rounded-full bg-cyan-500/20 text-cyan-300 border border-cyan-500/30">
                .rfcapsule
              </span>
            </h1>
            <p className="text-xs text-slate-400">
              L'alter ego haute performance d'All-in-One WP Migration : encapsulez, clonez et restaurez votre site complet en un clic.
            </p>
          </div>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Colonne 1 : Exporter la Capsule */}
        <div className="bg-[#0e1424]/90 border border-white/5 rounded-2xl p-6 flex flex-col justify-between shadow-xl">
          <div className="space-y-4">
            <div className="flex items-center justify-between pb-3 border-b border-white/5">
              <div className="flex items-center gap-2 text-white font-semibold text-sm">
                <Download className="w-4 h-4 text-indigo-400" />
                <span>Exporter le site complet</span>
              </div>
              <span className="text-[11px] px-2 py-0.5 rounded-full bg-indigo-500/10 text-indigo-300 border border-indigo-500/20">
                1 Clic
              </span>
            </div>

            <p className="text-xs text-slate-300 leading-relaxed">
              Génère une archive autonome <span className="text-indigo-300 font-mono">.rfcapsule</span> contenant l'intégralité de la base de données relationnelle, les schémas de types, les utilisateurs et tous les fichiers médias stockés dans le dossier uploads.
            </p>

            {/* Statistiques incluses dans l'archive */}
            <div className="grid grid-cols-3 gap-3 p-3 rounded-xl bg-black/30 border border-white/5">
              <div className="flex items-center gap-2">
                <FileText className="w-3.5 h-3.5 text-slate-400" />
                <div>
                  <p className="text-[10px] text-slate-500">Articles</p>
                  <p className="text-xs font-bold text-slate-200">
                    {stats ? stats.content_types.reduce((acc, ct) => acc + ct.published + ct.drafts + ct.scheduled, 0) : '—'}
                  </p>
                </div>
              </div>
              <div className="flex items-center gap-2">
                <Image className="w-3.5 h-3.5 text-slate-400" />
                <div>
                  <p className="text-[10px] text-slate-500">Médias</p>
                  <p className="text-xs font-bold text-slate-200">{stats?.media ?? '—'}</p>
                </div>
              </div>
              <div className="flex items-center gap-2">
                <Database className="w-3.5 h-3.5 text-slate-400" />
                <div>
                  <p className="text-[10px] text-slate-500">Types</p>
                  <p className="text-xs font-bold text-slate-200">{stats?.content_types.length ?? '—'}</p>
                </div>
              </div>
            </div>

            {exportSuccess && (
              <div className="p-3 bg-emerald-500/10 border border-emerald-500/20 rounded-xl text-emerald-300 text-xs flex items-center gap-2">
                <CheckCircle2 className="w-4 h-4 shrink-0 text-emerald-400" />
                <span>La capsule a été générée et téléchargée avec succès !</span>
              </div>
            )}

            {exportError && (
              <div className="p-3 bg-red-500/10 border border-red-500/20 rounded-xl text-red-300 text-xs flex items-center gap-2">
                <AlertCircle className="w-4 h-4 shrink-0" />
                <span>{exportError}</span>
              </div>
            )}
          </div>

          <button
            onClick={handleExport}
            disabled={exporting}
            className="w-full mt-6 py-3 rounded-xl bg-gradient-to-r from-indigo-600 via-indigo-500 to-cyan-500 hover:from-indigo-500 hover:to-cyan-400 text-white text-xs font-bold shadow-lg shadow-indigo-600/25 transition-all flex items-center justify-center gap-2 disabled:opacity-50"
          >
            {exporting ? (
              <>
                <RefreshCw className="w-4 h-4 animate-spin" />
                <span>Création de la capsule ZIP & Compression...</span>
              </>
            ) : (
              <>
                <Download className="w-4 h-4" />
                <span>Télécharger la Capsule (.rfcapsule)</span>
              </>
            )}
          </button>
        </div>

        {/* Colonne 2 : Restaurer / Cloner une Capsule */}
        <div className="bg-[#0e1424]/90 border border-white/5 rounded-2xl p-6 flex flex-col justify-between shadow-xl">
          <div className="space-y-4">
            <div className="flex items-center justify-between pb-3 border-b border-white/5">
              <div className="flex items-center gap-2 text-white font-semibold text-sm">
                <UploadCloud className="w-4 h-4 text-cyan-400" />
                <span>Restaurer ou cloner une capsule</span>
              </div>
              <span className="text-[11px] px-2 py-0.5 rounded-full bg-cyan-500/10 text-cyan-300 border border-cyan-500/20">
                Import & Rewrite
              </span>
            </div>

            {/* Zone Drop */}
            <div
              onDragEnter={handleDrag}
              onDragLeave={handleDrag}
              onDragOver={handleDrag}
              onDrop={handleDrop}
              className={`border-2 border-dashed rounded-xl p-5 text-center transition-all ${
                dragActive
                  ? 'border-indigo-400 bg-indigo-500/10'
                  : capsuleFile
                  ? 'border-emerald-500/40 bg-emerald-500/5'
                  : 'border-white/10 hover:border-white/20 bg-black/20'
              }`}
            >
              <input
                type="file"
                id="admin-capsule-upload"
                accept=".rfcapsule,.zip"
                onChange={(e) => {
                  if (e.target.files && e.target.files[0]) {
                    handleFileSelected(e.target.files[0]);
                  }
                }}
                className="hidden"
              />

              {capsuleFile ? (
                <div className="space-y-2">
                  <div className="w-8 h-8 mx-auto rounded-full bg-emerald-500/20 border border-emerald-500/30 flex items-center justify-center text-emerald-400">
                    <CheckCircle2 className="w-4 h-4" />
                  </div>
                  <p className="text-xs font-semibold text-white">{capsuleFile.name}</p>
                  <p className="text-[10px] text-slate-400">
                    {(capsuleFile.size / (1024 * 1024)).toFixed(2)} Mo
                  </p>
                  <label
                    htmlFor="admin-capsule-upload"
                    className="inline-block text-[10px] text-indigo-400 hover:underline cursor-pointer"
                  >
                    Changer de capsule
                  </label>
                </div>
              ) : (
                <label htmlFor="admin-capsule-upload" className="cursor-pointer space-y-1.5 block">
                  <UploadCloud className="w-8 h-8 mx-auto text-cyan-400/80" />
                  <p className="text-xs font-medium text-slate-200">
                    Glissez votre archive <span className="text-cyan-300 font-mono">.rfcapsule</span>
                  </p>
                  <p className="text-[10px] text-slate-500">ou cliquez pour choisir un fichier</p>
                </label>
              )}
            </div>

            {/* Inspecteur de métadonnées de la capsule */}
            {inspecting && (
              <div className="p-3 bg-black/30 rounded-xl text-center text-xs text-slate-400 flex items-center justify-center gap-2">
                <RefreshCw className="w-3.5 h-3.5 animate-spin" />
                <span>Inspection du manifeste capsule.json...</span>
              </div>
            )}

            {manifest && (
              <div className="p-3.5 rounded-xl bg-indigo-500/10 border border-indigo-500/20 space-y-2 text-xs">
                <div className="flex items-center justify-between text-indigo-300 font-semibold">
                  <span>Manifeste valide</span>
                  <span className="text-[10px] text-slate-400 font-mono">{manifest.version}</span>
                </div>
                <div className="grid grid-cols-3 gap-2 text-[11px] text-slate-300">
                  <div>
                    <span className="text-slate-500 block text-[9px]">Articles</span>
                    <span className="font-bold">{manifest.entry_count}</span>
                  </div>
                  <div>
                    <span className="text-slate-500 block text-[9px]">Médias</span>
                    <span className="font-bold">{manifest.media_count}</span>
                  </div>
                  <div>
                    <span className="text-slate-500 block text-[9px]">Termes</span>
                    <span className="font-bold">{manifest.term_count}</span>
                  </div>
                </div>
                {manifest.source_site_url && (
                  <p className="text-[10px] text-slate-400 truncate">
                    Origine : <span className="text-slate-300 font-mono">{manifest.source_site_url}</span>
                  </p>
                )}
              </div>
            )}

            {/* Champ Réécriture d'URL */}
            {capsuleFile && (
              <div className="space-y-1 pt-1">
                <label className="text-xs font-medium text-slate-300 flex items-center gap-1.5">
                  <Globe className="w-3.5 h-3.5 text-cyan-400" />
                  URL de destination (Remplacement automatique des URLs)
                </label>
                <input
                  type="text"
                  value={targetSiteUrl}
                  onChange={(e) => setTargetSiteUrl(e.target.value)}
                  className="w-full px-3 py-2 rounded-xl bg-black/30 border border-white/10 text-white text-xs font-mono focus:outline-none focus:border-cyan-500"
                />
              </div>
            )}

            {restoreReport && (
              <div className="p-3.5 bg-emerald-500/10 border border-emerald-500/30 rounded-xl text-emerald-300 text-xs space-y-1">
                <p className="font-semibold flex items-center gap-1.5">
                  <CheckCircle2 className="w-4 h-4 text-emerald-400" />
                  <span>{restoreReport.message}</span>
                </p>
                <p className="text-[11px] text-slate-300">
                  {restoreReport.entries_restored} articles restaurés • {restoreReport.media_extracted} fichiers extraits en {restoreReport.execution_time_ms} ms.
                </p>
              </div>
            )}

            {restoreError && (
              <div className="p-3 bg-red-500/10 border border-red-500/20 rounded-xl text-red-300 text-xs flex items-center gap-2">
                <AlertCircle className="w-4 h-4 shrink-0" />
                <span>{restoreError}</span>
              </div>
            )}
          </div>

          <button
            onClick={handleRestore}
            disabled={!capsuleFile || restoring}
            className="w-full mt-6 py-3 rounded-xl bg-gradient-to-r from-teal-600 via-cyan-500 to-indigo-600 hover:from-teal-500 hover:to-indigo-500 text-white text-xs font-bold shadow-lg shadow-cyan-500/20 transition-all flex items-center justify-center gap-2 disabled:opacity-50"
          >
            {restoring ? (
              <>
                <RefreshCw className="w-4 h-4 animate-spin" />
                <span>Restauration des médias & Injection BDD...</span>
              </>
            ) : (
              <>
                <UploadCloud className="w-4 h-4" />
                <span>Déployer cette Capsule</span>
              </>
            )}
          </button>
        </div>
      </div>

      {/* Guide & Spécifications du format .rfcapsule */}
      <div className="p-6 rounded-2xl bg-black/20 border border-white/5 space-y-4">
        <h3 className="text-xs font-semibold text-slate-300 uppercase tracking-wider flex items-center gap-2">
          <ShieldCheck className="w-4 h-4 text-emerald-400" />
          Anatomie d'une Capsule RustForge (.rfcapsule)
        </h3>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4 text-xs text-slate-400">
          <div className="p-3 rounded-xl bg-white/[0.02] border border-white/5 space-y-1">
            <span className="font-mono text-indigo-300 font-semibold block">1. capsule.json</span>
            <p className="text-[11px] leading-relaxed">
              Manifeste JSON contenant les métadonnées, la date, la signature cryptographique et l'URL source pour la réécriture.
            </p>
          </div>
          <div className="p-3 rounded-xl bg-white/[0.02] border border-white/5 space-y-1">
            <span className="font-mono text-cyan-300 font-semibold block">2. database.rfir.json</span>
            <p className="text-[11px] leading-relaxed">
              Document pivot RFIR complet (articles, métadonnées Vault, taxonomies, réglages, utilisateurs).
            </p>
          </div>
          <div className="p-3 rounded-xl bg-white/[0.02] border border-white/5 space-y-1">
            <span className="font-mono text-purple-300 font-semibold block">3. uploads/</span>
            <p className="text-[11px] leading-relaxed">
              Arborescence physique des médias (WebP, JPG, SVG, PDF) extraite directement dans le dossier du serveur.
            </p>
          </div>
        </div>
      </div>
    </div>
  );
};
