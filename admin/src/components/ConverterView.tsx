import React, { useState } from 'react';
import {
  UploadCloud,
  FileCode2,
  CheckCircle2,
  AlertTriangle,
  ArrowRight,
  Palette,
  RefreshCw,
  Check,
  ShieldCheck,
  Eye,
  Sparkles,
  Layers,
  Database,
  Lock,
  Tag,
  Clock,
  ExternalLink
} from 'lucide-react';
import { api } from '../api';

export const ConverterView: React.FC<{ onOpenMigration?: () => void }> = ({ onOpenMigration }) => {
  const [activeTab, setActiveTab] = useState<'packages' | 'datamorph'>('packages');

  // État onglet Extensions & Thèmes (.zip)
  const [dragActive, setDragActive] = useState(false);
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [analyzing, setAnalyzing] = useState(false);
  const [manifest, setManifest] = useState<any | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [convertingTheme, setConvertingTheme] = useState(false);
  const [convertedThemeResult, setConvertedThemeResult] = useState<any | null>(null);
  const [previewHtml, setPreviewHtml] = useState<string | null>(null);
  const [applyingPlan, setApplyingPlan] = useState(false);
  const [appliedResult, setAppliedResult] = useState<any | null>(null);

  // État onglet DataMorph WXR (.xml)
  const [wxrDragActive, setWxrDragActive] = useState(false);
  const [wxrFile, setWxrFile] = useState<File | null>(null);
  const [importingWxr, setImportingWxr] = useState(false);
  const [wxrReport, setWxrReport] = useState<any | null>(null);
  const [wxrError, setWxrError] = useState<string | null>(null);

  // Handlers pour .zip
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

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files && e.target.files[0]) {
      handleFileSelected(e.target.files[0]);
    }
  };

  const handleFileSelected = async (file: File) => {
    if (!file.name.toLowerCase().endsWith('.zip')) {
      setError("Veuillez sélectionner une archive ZIP valide (.zip).");
      return;
    }
    setError(null);
    setSelectedFile(file);
    setManifest(null);
    setConvertedThemeResult(null);
    setAppliedResult(null);

    setAnalyzing(true);
    try {
      const res = await api.converter.analyze(file);
      setManifest(res);
    } catch (err: any) {
      setError(err.message || "Erreur lors de l'analyse du package ZIP.");
    } finally {
      setAnalyzing(false);
    }
  };

  const handleConvertTheme = async () => {
    if (!selectedFile) return;
    setConvertingTheme(true);
    setError(null);
    try {
      const res = await api.converter.convertTheme(selectedFile);
      setConvertedThemeResult(res);
    } catch (err: any) {
      setError(err.message || "Échec de la conversion du thème FSE.");
    } finally {
      setConvertingTheme(false);
    }
  };

  const handleApplyPlan = async () => {
    if (!manifest) return;
    setApplyingPlan(true);
    setError(null);
    try {
      const modules = manifest.activation_plan?.enable_modules || [];
      const themeSlug = convertedThemeResult?.theme_slug || manifest.activation_plan?.deploy_theme_slug;
      const res = await api.converter.applyPlan(modules, themeSlug);
      setAppliedResult(res);
    } catch (err: any) {
      setError(err.message || "Échec de l'application du plan.");
    } finally {
      setApplyingPlan(false);
    }
  };

  // Handlers pour DataMorph .xml
  const handleWxrDrag = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    if (e.type === 'dragenter' || e.type === 'dragover') {
      setWxrDragActive(true);
    } else if (e.type === 'dragleave') {
      setWxrDragActive(false);
    }
  };

  const handleWxrDrop = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setWxrDragActive(false);
    if (e.dataTransfer.files && e.dataTransfer.files[0]) {
      handleWxrFileSelected(e.dataTransfer.files[0]);
    }
  };

  const handleWxrFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files && e.target.files[0]) {
      handleWxrFileSelected(e.target.files[0]);
    }
  };

  const handleWxrFileSelected = async (file: File) => {
    if (!file.name.toLowerCase().endsWith('.xml')) {
      setWxrError("Veuillez sélectionner un fichier d'export XML valide (.xml).");
      return;
    }
    setWxrError(null);
    setWxrFile(file);
    setWxrReport(null);
    setImportingWxr(true);

    try {
      const res = await api.converter.importWxr(file);
      setWxrReport(res);
    } catch (err: any) {
      setWxrError(err.message || "Erreur lors de l'importation DataMorph WXR.");
    } finally {
      setImportingWxr(false);
    }
  };

  return (
    <div className="max-w-6xl mx-auto space-y-8 pb-16">
      {/* En-tête */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4 border-b border-white/5 pb-6">
        <div>
          <div className="flex items-center gap-2 mb-1">
            <span className="px-2.5 py-0.5 rounded-full text-[10px] font-bold tracking-wide uppercase bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
              Zéro Exécution PHP
            </span>
            <span className="px-2.5 py-0.5 rounded-full text-[10px] font-bold tracking-wide uppercase bg-indigo-500/10 text-indigo-400 border border-indigo-500/20">
              Moteur Souverain Rust
            </span>
          </div>
          <h2 className="text-xl font-black text-white flex items-center gap-2">
            Moteur d'Ingestion & Transmutation Universelle
          </h2>
          <p className="text-xs text-slate-400">
            Transmutez vos extensions & thèmes en modules natifs RustForge et absorbez l'intégralité de vos contenus sans perte.
          </p>
        </div>

        {/* Sélecteur d'onglet */}
        <div className="flex items-center p-1 bg-[#0d121f] rounded-xl border border-white/10 self-start md:self-auto">
          <button
            onClick={() => setActiveTab('packages')}
            className={`flex items-center gap-2 px-4 py-2 rounded-lg text-xs font-bold transition-all ${
              activeTab === 'packages'
                ? 'bg-indigo-600 text-white shadow-md shadow-indigo-600/30'
                : 'text-slate-400 hover:text-white'
            }`}
          >
            <Layers className="w-4 h-4" />
            <span>Extensions & Thèmes (.zip)</span>
          </button>
          <button
            onClick={() => onOpenMigration?.()}
            className={`flex items-center gap-2 px-4 py-2 rounded-lg text-xs font-bold transition-all ${
              activeTab === 'datamorph'
                ? 'bg-indigo-600 text-white shadow-md shadow-indigo-600/30'
                : 'text-slate-400 hover:text-white'
            }`}
          >
            <Database className="w-4 h-4" />
            <span>DataMorph Migration (.xml RFIR)</span>
          </button>
        </div>
      </div>

      {/* ============================================================== */}
      {/* VUE 1 : CONVERTISSEUR D'EXTENSIONS & THÈMES (.ZIP)            */}
      {/* ============================================================== */}
      {activeTab === 'packages' && (
        <div className="space-y-6">
          {previewHtml !== null && <section className="space-y-2">
            <button onClick={() => setPreviewHtml(null)}>Fermer l’aperçu</button>
            <iframe title="Aperçu isolé du thème" sandbox="" srcDoc={previewHtml} className="w-full h-[600px] bg-white rounded-xl" />
          </section>}
          {/* Erreur */}
          {error && (
            <div className="p-4 rounded-xl bg-red-500/10 border border-red-500/20 text-red-400 text-xs flex items-center gap-3">
              <AlertTriangle className="w-4 h-4 shrink-0" />
              <span>{error}</span>
            </div>
          )}

          {/* Étape 1 : Zone de Dépôt Drag & Drop */}
          <div
            onDragEnter={handleDrag}
            onDragLeave={handleDrag}
            onDragOver={handleDrag}
            onDrop={handleDrop}
            className={`relative border-2 border-dashed rounded-2xl p-10 text-center transition-all cursor-pointer ${
              dragActive
                ? 'border-indigo-500 bg-indigo-500/10 scale-[1.01]'
                : 'border-white/10 hover:border-indigo-500/50 bg-[#0d121f]/50'
            }`}
          >
            <input
              type="file"
              accept=".zip"
              onChange={handleFileChange}
              className="absolute inset-0 w-full h-full opacity-0 cursor-pointer"
            />

            <div className="max-w-md mx-auto space-y-4 pointer-events-none">
              <div className="w-16 h-16 rounded-2xl bg-indigo-600/10 border border-indigo-500/20 text-indigo-400 flex items-center justify-center mx-auto shadow-inner">
                {analyzing ? (
                  <RefreshCw className="w-8 h-8 animate-spin" />
                ) : (
                  <UploadCloud className="w-8 h-8" />
                )}
              </div>
              <div>
                <h3 className="text-sm font-bold text-white">
                  {analyzing
                    ? "Analyse lexicale et vérification des quotas en cours..."
                    : selectedFile
                    ? selectedFile.name
                    : "Glissez-déposez une archive ZIP d'anciens CMS ici"}
                </h3>
                <p className="text-xs text-slate-500 mt-1">
                  Thèmes déclaratifs, thèmes classiques ou extensions métier.
                </p>
              </div>
              {!analyzing && (
                <button className="px-4 py-2 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-semibold shadow-md shadow-indigo-600/30 transition-all">
                  Parcourir mes fichiers ZIP
                </button>
              )}
            </div>
          </div>

          {/* Résultat d'Analyse (Étapes 2, 3, 4, 5) */}
          {manifest && (
            <div className="space-y-6">
              {/* Étape 2 : Inventaire & Détection */}
              <div className="p-6 rounded-2xl bg-[#0d121f] border border-white/5 space-y-6">
                <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 pb-4 border-b border-white/5">
                  <div className="flex items-center gap-3">
                    <div className="w-12 h-12 rounded-xl bg-indigo-500/10 border border-indigo-500/20 text-indigo-400 flex items-center justify-center font-bold text-base">
                      {manifest.package.kind === 'theme_fse' ? (
                        <Palette className="w-6 h-6 text-pink-400" />
                      ) : manifest.package.kind === 'plugin' ? (
                        <Layers className="w-6 h-6 text-indigo-400" />
                      ) : (
                        <FileCode2 className="w-6 h-6 text-amber-400" />
                      )}
                    </div>
                    <div>
                      <div className="flex items-center gap-2">
                        <h3 className="text-base font-bold text-white">{manifest.package.name}</h3>
                        <span className="px-2 py-0.5 rounded text-[10px] font-semibold bg-white/10 text-slate-300">
                          v{manifest.package.version}
                        </span>
                        <span className="px-2 py-0.5 rounded text-[10px] font-bold uppercase bg-indigo-500/20 text-indigo-300 border border-indigo-500/30">
                          {manifest.package.kind}
                        </span>
                      </div>
                      <p className="text-xs text-slate-400 mt-0.5">
                        Par {manifest.package.author} &bull; {manifest.package.total_files} fichiers inspectés &bull;{' '}
                        {(manifest.package.uncompressed_size_bytes / 1024).toFixed(1)} Ko
                      </p>
                    </div>
                  </div>

                  {/* Score de compatibilité */}
                  <div className="flex items-center gap-3 bg-white/5 px-4 py-2.5 rounded-xl border border-white/10">
                    <div className="text-right">
                      <div className="text-[10px] uppercase font-bold text-slate-400">Score Compatibilité</div>
                      <div className="text-base font-black text-emerald-400">
                        {manifest.compatibility.score} %
                      </div>
                    </div>
                    <div className="w-10 h-10 rounded-full border-2 border-emerald-500/30 flex items-center justify-center text-emerald-400 font-bold text-xs bg-emerald-500/10">
                      <ShieldCheck className="w-5 h-5" />
                    </div>
                  </div>
                </div>

                {/* Stratégie Retenue */}
                <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                  <div className="p-4 rounded-xl bg-white/[0.02] border border-white/5">
                    <div className="text-[10px] uppercase font-bold text-slate-500 mb-1">Stratégie Adoptée</div>
                    <div className="text-sm font-semibold text-slate-200 capitalize flex items-center gap-2">
                      <Sparkles className="w-4 h-4 text-indigo-400" />
                      {manifest.strategy.replace('_', ' ')}
                    </div>
                  </div>
                  <div className="p-4 rounded-xl bg-white/[0.02] border border-white/5">
                    <div className="text-[10px] uppercase font-bold text-slate-500 mb-1">Cible RustForge</div>
                    <div className="text-sm font-semibold text-indigo-300">
                      {manifest.target_native_module
                        ? `Module Natif : ${manifest.target_native_module}`
                        : manifest.package.kind === 'theme_fse'
                        ? 'Moteur SSR MiniJinja'
                        : 'Coffre-fort _rf_vault'}
                    </div>
                  </div>
                  <div className="p-4 rounded-xl bg-white/[0.02] border border-white/5">
                    <div className="text-[10px] uppercase font-bold text-slate-500 mb-1">Hooks & Points d'entrée</div>
                    <div className="text-sm font-semibold text-slate-300">
                      {manifest.detected_hooks?.length || 0} hooks PHP interceptés
                    </div>
                  </div>
                </div>

                {/* Étape 3 : Proposition de Conversion détaillée */}
                <div className="space-y-4">
                  <h4 className="text-xs font-bold uppercase tracking-wider text-slate-400">
                    Diagnostic & Transposition des Fonctionnalités
                  </h4>

                  <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                    {/* Pris en charge / Migré */}
                    <div className="p-4 rounded-xl bg-emerald-500/5 border border-emerald-500/20 space-y-2">
                      <div className="flex items-center gap-2 text-emerald-400 text-xs font-bold">
                        <CheckCircle2 className="w-4 h-4" />
                        <span>Fonctionnalités Migrées Nativement</span>
                      </div>
                      <ul className="space-y-1.5 text-xs text-slate-300">
                        {manifest.compatibility.migrated_features?.map((f: string, i: number) => (
                          <li key={i} className="flex items-start gap-2">
                            <span className="text-emerald-500 mt-0.5">•</span>
                            <span>{f}</span>
                          </li>
                        ))}
                      </ul>
                    </div>

                    {/* Remplacé / Modernisé */}
                    <div className="p-4 rounded-xl bg-indigo-500/5 border border-indigo-500/20 space-y-2">
                      <div className="flex items-center gap-2 text-indigo-400 text-xs font-bold">
                        <Sparkles className="w-4 h-4" />
                        <span>Remplacé par la Suite RustForge</span>
                      </div>
                      <ul className="space-y-1.5 text-xs text-slate-300">
                        {manifest.compatibility.replaced_features?.map((f: string, i: number) => (
                          <li key={i} className="flex items-start gap-2">
                            <span className="text-indigo-400 mt-0.5">•</span>
                            <span>{f}</span>
                          </li>
                        ))}
                      </ul>
                    </div>
                  </div>

                  {/* Incompatibilités / Avertissements s'il y en a */}
                  {manifest.compatibility.incompatible_features?.length > 0 && (
                    <div className="p-4 rounded-xl bg-amber-500/5 border border-amber-500/20 space-y-2">
                      <div className="flex items-center gap-2 text-amber-400 text-xs font-bold">
                        <AlertTriangle className="w-4 h-4" />
                        <span>Limites Documentées & Données Préservées</span>
                      </div>
                      <ul className="space-y-1 text-xs text-slate-400">
                        {manifest.compatibility.incompatible_features.map((f: string, i: number) => (
                          <li key={i} className="flex items-start gap-2">
                            <span className="text-amber-500 mt-0.5">&bull;</span>
                            <span>{f}</span>
                          </li>
                        ))}
                      </ul>
                    </div>
                  )}
                </div>

                {/* Étape 4 & 5 : Actions de Conversion et Activation */}
                <div className="pt-4 border-t border-white/5 flex flex-wrap items-center justify-between gap-4">
                  <div className="text-xs text-slate-400">
                    {convertedThemeResult ? (
                      <span className="text-emerald-400 font-semibold flex items-center gap-1.5">
                        <Check className="w-4 h-4" /> Thème converti dans {convertedThemeResult.target_directory}
                      </span>
                    ) : appliedResult ? (
                      <span className="text-emerald-400 font-semibold flex items-center gap-1.5">
                        <Check className="w-4 h-4" /> {appliedResult.message}
                      </span>
                    ) : (
                      <span>Prêt pour validation et activation sans impact sur la production.</span>
                    )}
                  </div>

                  <div className="flex items-center gap-3">
                    {manifest.package.kind === 'theme_fse' && !convertedThemeResult && (
                      <button
                        onClick={handleConvertTheme}
                        disabled={convertingTheme}
                        className="px-4 py-2 rounded-xl bg-pink-600 hover:bg-pink-500 disabled:opacity-50 text-white text-xs font-bold flex items-center gap-2 shadow-lg shadow-pink-600/20 transition-all"
                      >
                        {convertingTheme ? (
                          <RefreshCw className="w-4 h-4 animate-spin" />
                        ) : (
                          <Palette className="w-4 h-4" />
                        )}
                        <span>Transmuter vers MiniJinja</span>
                      </button>
                    )}

                    {convertedThemeResult && (
                      <a
                        href="#" onClick={async e => {
                          e.preventDefault();
                          const response=await fetch(convertedThemeResult.preview_url,{headers:{Authorization:`Bearer ${localStorage.getItem('rustforge_token') || localStorage.getItem('trustpress_token') || ''}`}});
                          if(!response.ok){alert('Aperçu indisponible');return;}
                          setPreviewHtml(await response.text());
                        }}
                        target="_blank"
                        rel="noreferrer"
                        className="px-4 py-2 rounded-xl bg-white/10 hover:bg-white/20 text-white text-xs font-bold flex items-center gap-2 transition-all"
                      >
                        <Eye className="w-4 h-4" />
                        <span>Prévisualiser en direct</span>
                      </a>
                    )}

                    <button
                      onClick={handleApplyPlan}
                      disabled={applyingPlan || !!appliedResult}
                      className="px-5 py-2 rounded-xl bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 text-white text-xs font-bold flex items-center gap-2 shadow-lg shadow-indigo-600/25 transition-all"
                    >
                      {applyingPlan ? (
                        <RefreshCw className="w-4 h-4 animate-spin" />
                      ) : appliedResult ? (
                        <Check className="w-4 h-4" />
                      ) : (
                        <ArrowRight className="w-4 h-4" />
                      )}
                      <span>
                        {appliedResult ? "Activé avec succès" : "Valider & Activer les Modules"}
                      </span>
                    </button>
                  </div>
                </div>
              </div>
            </div>
          )}
        </div>
      )}

      {/* ============================================================== */}
      {/* VUE 2 : DATAMORPH MIGRATION WXR (.XML)                        */}
      {/* ============================================================== */}
      {activeTab === 'datamorph' && (
        <div className="space-y-6">
          {/* Bannière explicative */}
          <div className="p-5 rounded-2xl bg-[#0d121f] border border-white/5 flex flex-col md:flex-row md:items-center justify-between gap-4">
            <div className="flex items-start gap-3">
              <div className="w-10 h-10 rounded-xl bg-indigo-600/10 border border-indigo-500/20 text-indigo-400 flex items-center justify-center shrink-0">
                <Database className="w-5 h-5" />
              </div>
              <div>
                <h3 className="text-sm font-bold text-white">Ingestion Universelle Sans Perte (DataMorph & RFIR)</h3>
                <p className="text-xs text-slate-400 mt-0.5">
                  Importation directe depuis des exports XML d'anciens CMS.
                  Désérialisation automatique des structures sérialisées vers PostgreSQL JSONB.
                </p>
              </div>
            </div>
            <div className="flex items-center gap-2 self-start md:self-auto shrink-0">
              <span className="px-3 py-1 rounded-full text-[10px] font-bold tracking-wide uppercase bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 flex items-center gap-1.5">
                <Lock className="w-3 h-3" /> Coffre-Fort _rf_vault Actif
              </span>
            </div>
          </div>

          {/* Erreur WXR */}
          {wxrError && (
            <div className="p-4 rounded-xl bg-red-500/10 border border-red-500/20 text-red-400 text-xs flex items-center gap-3">
              <AlertTriangle className="w-4 h-4 shrink-0" />
              <span>{wxrError}</span>
            </div>
          )}

          {/* Zone de Dépôt Drag & Drop WXR */}
          <div
            onDragEnter={handleWxrDrag}
            onDragLeave={handleWxrDrag}
            onDragOver={handleWxrDrag}
            onDrop={handleWxrDrop}
            className={`relative border-2 border-dashed rounded-2xl p-10 text-center transition-all cursor-pointer ${
              wxrDragActive
                ? 'border-indigo-500 bg-indigo-500/10 scale-[1.01]'
                : 'border-white/10 hover:border-indigo-500/50 bg-[#0d121f]/50'
            }`}
          >
            <input
              type="file"
              accept=".xml"
              onChange={handleWxrFileChange}
              className="absolute inset-0 w-full h-full opacity-0 cursor-pointer"
            />

            <div className="max-w-md mx-auto space-y-4 pointer-events-none">
              <div className="w-16 h-16 rounded-2xl bg-indigo-600/10 border border-indigo-500/20 text-indigo-400 flex items-center justify-center mx-auto shadow-inner">
                {importingWxr ? (
                  <RefreshCw className="w-8 h-8 animate-spin" />
                ) : (
                  <UploadCloud className="w-8 h-8" />
                )}
              </div>
              <div>
                <h3 className="text-sm font-bold text-white">
                  {importingWxr
                    ? "Streaming XML & Ingestion PostgreSQL en cours..."
                    : wxrFile
                    ? wxrFile.name
                    : "Glissez-déposez votre fichier XML d'export ici"}
                </h3>
                <p className="text-xs text-slate-500 mt-1">
                  Format XML universel. Traitement ultra-rapide par le moteur streaming natif en Rust.
                </p>
              </div>
              {!importingWxr && (
                <button className="px-4 py-2 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-semibold shadow-md shadow-indigo-600/30 transition-all">
                  Parcourir mon export .xml
                </button>
              )}
            </div>
          </div>

          {/* Rapport d'importation DataMorph */}
          {wxrReport && (
            <div className="p-6 rounded-2xl bg-[#0d121f] border border-white/5 space-y-6">
              <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 pb-4 border-b border-white/5">
                <div className="flex items-center gap-3">
                  <div className="w-12 h-12 rounded-xl bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 flex items-center justify-center">
                    <CheckCircle2 className="w-6 h-6" />
                  </div>
                  <div>
                    <h3 className="text-base font-bold text-white">Ingestion Réussie avec Succès</h3>
                    <p className="text-xs text-slate-400 mt-0.5">
                      Les données historiques ont été absorbées et intégrées au standard souverain de RustForge.
                    </p>
                  </div>
                </div>

                <div className="flex items-center gap-2 text-xs font-bold text-slate-300 bg-white/5 px-3 py-1.5 rounded-xl border border-white/10">
                  <Clock className="w-4 h-4 text-indigo-400" />
                  <span>Temps d'exécution : {wxrReport.execution_time_ms} ms</span>
                </div>
              </div>

              {/* 4 Métriques Clés */}
              <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
                <div className="p-4 rounded-xl bg-white/[0.02] border border-white/5">
                  <div className="text-[10px] uppercase font-bold text-slate-400 mb-1">Entrées Importées</div>
                  <div className="text-2xl font-black text-emerald-400">
                    {wxrReport.entries_imported}
                  </div>
                  <div className="text-[10px] text-slate-500 mt-0.5">
                    {wxrReport.entries_skipped} ignorées / doublons
                  </div>
                </div>

                <div className="p-4 rounded-xl bg-white/[0.02] border border-white/5">
                  <div className="text-[10px] uppercase font-bold text-slate-400 mb-1">Termes & Taxonomies</div>
                  <div className="text-2xl font-black text-indigo-400">
                    {wxrReport.terms_imported}
                  </div>
                  <div className="text-[10px] text-slate-500 mt-0.5">
                    Catégories & Étiquettes
                  </div>
                </div>

                <div className="p-4 rounded-xl bg-white/[0.02] border border-white/5">
                  <div className="text-[10px] uppercase font-bold text-slate-400 mb-1">Coffre-Fort Numérique</div>
                  <div className="text-2xl font-black text-amber-400">
                    {wxrReport.legacy_fields_stored_in_vault}
                  </div>
                  <div className="text-[10px] text-slate-500 mt-0.5">
                    Champs _rf_vault
                  </div>
                </div>

                <div className="p-4 rounded-xl bg-white/[0.02] border border-white/5">
                  <div className="text-[10px] uppercase font-bold text-slate-400 mb-1">Médias Détectés</div>
                  <div className="text-2xl font-black text-slate-200">
                    {wxrReport.media_detected}
                  </div>
                  <div className="text-[10px] text-slate-500 mt-0.5">
                    Pièces jointes prêtes
                  </div>
                </div>
              </div>

              {/* Types de Contenus créés si applicable */}
              {wxrReport.created_content_types?.length > 0 && (
                <div className="p-4 rounded-xl bg-indigo-500/5 border border-indigo-500/20 space-y-2">
                  <div className="flex items-center gap-2 text-indigo-400 text-xs font-bold">
                    <Tag className="w-4 h-4" />
                    <span>Nouveaux Types de Contenus Déclarés Automatiquement</span>
                  </div>
                  <div className="flex flex-wrap gap-2">
                    {wxrReport.created_content_types.map((ct: string, i: number) => (
                      <span key={i} className="px-2.5 py-1 rounded bg-indigo-500/20 text-indigo-300 font-semibold text-xs border border-indigo-500/30">
                        {ct}
                      </span>
                    ))}
                  </div>
                </div>
              )}

              {/* Avertissements si applicable */}
              {wxrReport.warnings?.length > 0 && (
                <div className="p-4 rounded-xl bg-amber-500/5 border border-amber-500/20 space-y-2">
                  <div className="flex items-center gap-2 text-amber-400 text-xs font-bold">
                    <AlertTriangle className="w-4 h-4" />
                    <span>Avertissements lors de l'importation</span>
                  </div>
                  <ul className="space-y-1 text-xs text-slate-400">
                    {wxrReport.warnings.map((w: string, i: number) => (
                      <li key={i} className="flex items-start gap-2">
                        <span className="text-amber-500 mt-0.5">&bull;</span>
                        <span>{w}</span>
                      </li>
                    ))}
                  </ul>
                </div>
              )}

              {/* Actions suivantes */}
              <div className="pt-4 border-t border-white/5 flex items-center justify-between">
                <span className="text-xs text-slate-400">
                  Toutes les données sont désormais indexées et servies en &lt; 1 ms.
                </span>
                <a
                  href="/fr"
                  target="_blank"
                  rel="noreferrer"
                  className="px-4 py-2 rounded-xl bg-white/10 hover:bg-white/20 text-white text-xs font-bold flex items-center gap-2 transition-all"
                >
                  <ExternalLink className="w-4 h-4" />
                  <span>Voir le Site Public</span>
                </a>
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
};
