import React, { useState } from 'react';
import { 
  Sparkles, 
  Server, 
  CheckCircle2, 
  UploadCloud, 
  ArrowRight, 
  ShieldCheck, 
  AlertCircle, 
  RefreshCw, 
  Layers, 
  Globe 
} from 'lucide-react';
import { api, setToken } from '../api';
import type { User } from '../types';

interface InstallerWizardViewProps {
  onInstalled: (token: string, user: User) => void;
}

export const InstallerWizardView: React.FC<InstallerWizardViewProps> = ({ onInstalled }) => {
  const [mode, setMode] = useState<'fresh' | 'capsule'>('fresh');

  // Formulaire Nouvelle Installation
  const [siteTitle, setSiteTitle] = useState('Mon Site RustForge');
  const [siteTagline, setSiteTagline] = useState('Propulsé par le CMS Rust souverain');
  const [adminUsername, setAdminUsername] = useState('admin');
  const [adminEmail, setAdminEmail] = useState('admin@rustforge.local');
  const [adminPassword, setAdminPassword] = useState('');
  const [installing, setInstalling] = useState(false);
  const [installError, setInstallError] = useState<string | null>(null);

  // Formulaire Restauration Capsule
  const [capsuleFile, setCapsuleFile] = useState<File | null>(null);
  const [targetSiteUrl, setTargetSiteUrl] = useState(window.location.origin);
  const [dragActive, setDragActive] = useState(false);
  const [restoring, setRestoring] = useState(false);
  const [restoreError, setRestoreError] = useState<string | null>(null);
  const [restoreSuccess, setRestoreSuccess] = useState<any | null>(null);

  const handleFreshInstall = async (e: React.FormEvent) => {
    e.preventDefault();
    setInstallError(null);

    if (!siteTitle.trim() || !adminUsername.trim() || !adminEmail.trim()) {
      setInstallError('Veuillez remplir tous les champs obligatoires.');
      return;
    }

    if (adminPassword.length < 8) {
      setInstallError('Le mot de passe administrateur doit contenir au moins 8 caractères.');
      return;
    }

    try {
      setInstalling(true);
      const res = await api.setup.install({
        site_title: siteTitle,
        site_tagline: siteTagline,
        admin_username: adminUsername,
        admin_email: adminEmail,
        admin_password: adminPassword,
      });

      setToken(res.token);
      onInstalled(res.token, res.user);
    } catch (err: any) {
      setInstallError(err.message || "Erreur lors de l'installation de RustForge.");
    } finally {
      setInstalling(false);
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
      const file = e.dataTransfer.files[0];
      if (file.name.endsWith('.rfcapsule') || file.name.endsWith('.zip')) {
        setCapsuleFile(file);
      } else {
        setRestoreError("Le fichier doit être une archive .rfcapsule valide.");
      }
    }
  };

  const handleRestoreCapsule = async () => {
    if (!capsuleFile) return;
    setRestoreError(null);
    setRestoring(true);

    try {
      const res = await api.setup.restoreCapsule(capsuleFile, targetSiteUrl);
      setRestoreSuccess(res);
      // Recharger pour se connecter sur le site restauré
      setTimeout(() => {
        window.location.reload();
      }, 2500);
    } catch (err: any) {
      setRestoreError(err.message || 'Erreur lors de la restauration de la capsule.');
    } finally {
      setRestoring(false);
    }
  };

  return (
    <div className="min-h-screen bg-[#070a13] text-slate-200 flex flex-col justify-center items-center p-4 relative overflow-hidden select-none">
      {/* Glow décoratif */}
      <div className="absolute top-1/4 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[700px] h-[350px] bg-gradient-to-tr from-indigo-600/15 via-purple-600/10 to-teal-500/10 blur-[130px] rounded-full pointer-events-none" />

      <div className="w-full max-w-xl z-10">
        {/* En-tête de marque */}
        <div className="text-center mb-8">
          <div className="inline-flex items-center justify-center w-14 h-14 rounded-2xl bg-gradient-to-tr from-indigo-600 via-indigo-500 to-cyan-400 shadow-xl shadow-indigo-500/25 font-black text-white text-2xl tracking-tight mb-4 border border-white/10">
            RF
          </div>
          <h1 className="text-2xl font-bold text-white tracking-tight flex items-center justify-center gap-2">
            RustForge CMS
            <span className="text-xs uppercase font-semibold px-2 py-0.5 rounded-full bg-indigo-500/20 text-indigo-300 border border-indigo-500/30">
              Web Installer
            </span>
          </h1>
          <p className="text-slate-400 text-xs mt-1">
            L'alter ego haute performance de WordPress, compilé en Rust souverain.
          </p>
        </div>

        {/* Card principale */}
        <div className="bg-[#0e1424]/90 backdrop-blur-xl border border-white/10 rounded-2xl p-6 sm:p-8 shadow-2xl relative">
          {/* Badge statut BDD */}
          <div className="flex items-center justify-between pb-5 mb-6 border-b border-white/5">
            <div className="flex items-center gap-2.5 text-xs text-slate-300">
              <span className="w-2.5 h-2.5 rounded-full bg-emerald-400 animate-pulse" />
              <Server className="w-3.5 h-3.5 text-emerald-400" />
              <span className="font-medium">Serveur Rust & PostgreSQL opérationnels</span>
            </div>
            <span className="text-[10px] text-slate-500 font-mono">v0.1.0-release</span>
          </div>

          {/* Onglets Choix d'installation */}
          <div className="flex rounded-xl bg-black/40 p-1 border border-white/5 mb-6">
            <button
              onClick={() => { setMode('fresh'); setInstallError(null); }}
              className={`flex-1 py-2 text-xs font-semibold rounded-lg transition-all flex items-center justify-center gap-2 ${
                mode === 'fresh'
                  ? 'bg-indigo-600 text-white shadow-md shadow-indigo-600/30'
                  : 'text-slate-400 hover:text-white'
              }`}
            >
              <Sparkles className="w-3.5 h-3.5" />
              Nouvelle Installation (1 min)
            </button>
            <button
              onClick={() => { setMode('capsule'); setRestoreError(null); }}
              className={`flex-1 py-2 text-xs font-semibold rounded-lg transition-all flex items-center justify-center gap-2 ${
                mode === 'capsule'
                  ? 'bg-indigo-600 text-white shadow-md shadow-indigo-600/30'
                  : 'text-slate-400 hover:text-white'
              }`}
            >
              <Layers className="w-3.5 h-3.5 text-cyan-300" />
              Restaurer une Capsule (.rfcapsule)
            </button>
          </div>

          {/* Mode A : Installation Neuve */}
          {mode === 'fresh' && (
            <form onSubmit={handleFreshInstall} className="space-y-4">
              {installError && (
                <div className="p-3 bg-red-500/10 border border-red-500/20 rounded-xl text-red-300 text-xs flex items-center gap-2">
                  <AlertCircle className="w-4 h-4 shrink-0" />
                  <span>{installError}</span>
                </div>
              )}

              <div className="space-y-1">
                <label className="text-xs font-medium text-slate-300">Titre du site</label>
                <input
                  type="text"
                  value={siteTitle}
                  onChange={(e) => setSiteTitle(e.target.value)}
                  placeholder="Ex : Mon Magazine Rust"
                  className="w-full px-3.5 py-2.5 rounded-xl bg-black/30 border border-white/10 text-white text-xs focus:outline-none focus:border-indigo-500 transition-colors"
                  required
                />
              </div>

              <div className="space-y-1">
                <label className="text-xs font-medium text-slate-300">Slogan / Tagline</label>
                <input
                  type="text"
                  value={siteTagline}
                  onChange={(e) => setSiteTagline(e.target.value)}
                  placeholder="Ex : Un site ultra-rapide propulsé par RustForge"
                  className="w-full px-3.5 py-2.5 rounded-xl bg-black/30 border border-white/10 text-white text-xs focus:outline-none focus:border-indigo-500 transition-colors"
                />
              </div>

              <div className="grid grid-cols-2 gap-3 pt-2">
                <div className="space-y-1">
                  <label className="text-xs font-medium text-slate-300">Identifiant admin</label>
                  <input
                    type="text"
                    value={adminUsername}
                    onChange={(e) => setAdminUsername(e.target.value)}
                    className="w-full px-3.5 py-2.5 rounded-xl bg-black/30 border border-white/10 text-white text-xs focus:outline-none focus:border-indigo-500 transition-colors"
                    required
                  />
                </div>
                <div className="space-y-1">
                  <label className="text-xs font-medium text-slate-300">Email admin</label>
                  <input
                    type="email"
                    value={adminEmail}
                    onChange={(e) => setAdminEmail(e.target.value)}
                    className="w-full px-3.5 py-2.5 rounded-xl bg-black/30 border border-white/10 text-white text-xs focus:outline-none focus:border-indigo-500 transition-colors"
                    required
                  />
                </div>
              </div>

              <div className="space-y-1 pt-1">
                <div className="flex items-center justify-between">
                  <label className="text-xs font-medium text-slate-300">Mot de passe de sécurité</label>
                  <span className="text-[10px] text-slate-500">Min. 8 caractères</span>
                </div>
                <input
                  type="password"
                  value={adminPassword}
                  onChange={(e) => setAdminPassword(e.target.value)}
                  placeholder="••••••••••••"
                  className="w-full px-3.5 py-2.5 rounded-xl bg-black/30 border border-white/10 text-white text-xs focus:outline-none focus:border-indigo-500 transition-colors"
                  required
                />
              </div>

              <button
                type="submit"
                disabled={installing}
                className="w-full mt-6 py-3 rounded-xl bg-gradient-to-r from-indigo-600 via-indigo-500 to-cyan-500 hover:from-indigo-500 hover:to-cyan-400 text-white text-xs font-bold shadow-lg shadow-indigo-600/30 transition-all flex items-center justify-center gap-2 disabled:opacity-50"
              >
                {installing ? (
                  <>
                    <RefreshCw className="w-4 h-4 animate-spin" />
                    <span>Création des tables & Configuration en cours...</span>
                  </>
                ) : (
                  <>
                    <span>Installer RustForge en 1 Clic</span>
                    <ArrowRight className="w-4 h-4" />
                  </>
                )}
              </button>
            </form>
          )}

          {/* Mode B : Restauration d'une Capsule */}
          {mode === 'capsule' && (
            <div className="space-y-4">
              <p className="text-xs text-slate-400">
                Déployez un site complet instantanément (base de données, contenus et médias) à partir d'une archive{' '}
                <span className="text-indigo-300 font-mono">.rfcapsule</span>.
              </p>

              {restoreError && (
                <div className="p-3 bg-red-500/10 border border-red-500/20 rounded-xl text-red-300 text-xs flex items-center gap-2">
                  <AlertCircle className="w-4 h-4 shrink-0" />
                  <span>{restoreError}</span>
                </div>
              )}

              {restoreSuccess && (
                <div className="p-4 bg-emerald-500/10 border border-emerald-500/30 rounded-xl text-emerald-300 text-xs space-y-2">
                  <div className="flex items-center gap-2 font-semibold">
                    <CheckCircle2 className="w-4 h-4 text-emerald-400" />
                    <span>Restauration réussie ! Redirection en cours...</span>
                  </div>
                  <div className="text-[11px] text-slate-300">
                    Restauré : {restoreSuccess.entries_restored} articles, {restoreSuccess.media_extracted} médias.
                  </div>
                </div>
              )}

              {/* Zone Drag & Drop */}
              <div
                onDragEnter={handleDrag}
                onDragLeave={handleDrag}
                onDragOver={handleDrag}
                onDrop={handleDrop}
                className={`border-2 border-dashed rounded-xl p-6 text-center transition-all ${
                  dragActive
                    ? 'border-indigo-400 bg-indigo-500/10'
                    : capsuleFile
                    ? 'border-emerald-500/40 bg-emerald-500/5'
                    : 'border-white/10 hover:border-white/20 bg-black/20'
                }`}
              >
                <input
                  type="file"
                  id="capsule-upload"
                  accept=".rfcapsule,.zip"
                  onChange={(e) => {
                    if (e.target.files && e.target.files[0]) {
                      setCapsuleFile(e.target.files[0]);
                    }
                  }}
                  className="hidden"
                />

                {capsuleFile ? (
                  <div className="space-y-2">
                    <div className="w-10 h-10 mx-auto rounded-full bg-emerald-500/20 border border-emerald-500/30 flex items-center justify-center text-emerald-400">
                      <CheckCircle2 className="w-5 h-5" />
                    </div>
                    <p className="text-xs font-semibold text-white">{capsuleFile.name}</p>
                    <p className="text-[11px] text-slate-400">
                      {(capsuleFile.size / (1024 * 1024)).toFixed(2)} Mo • Prêt pour injection
                    </p>
                    <label
                      htmlFor="capsule-upload"
                      className="inline-block text-[11px] text-indigo-400 hover:underline cursor-pointer"
                    >
                      Changer de fichier
                    </label>
                  </div>
                ) : (
                  <label htmlFor="capsule-upload" className="cursor-pointer space-y-2 block">
                    <UploadCloud className="w-9 h-9 mx-auto text-indigo-400/80" />
                    <p className="text-xs font-medium text-slate-200">
                      Glissez votre fichier <span className="text-indigo-400 font-mono">.rfcapsule</span> ici
                    </p>
                    <p className="text-[11px] text-slate-500">ou cliquez pour parcourir vos dossiers</p>
                  </label>
                )}
              </div>

              {/* Réécriture automatique d'URL */}
              <div className="space-y-1 pt-1">
                <div className="flex items-center justify-between">
                  <label className="text-xs font-medium text-slate-300 flex items-center gap-1.5">
                    <Globe className="w-3.5 h-3.5 text-cyan-400" />
                    URL cible de destination (Réécriture de domaine)
                  </label>
                  <span className="text-[10px] text-slate-500">Automatique</span>
                </div>
                <input
                  type="text"
                  value={targetSiteUrl}
                  onChange={(e) => setTargetSiteUrl(e.target.value)}
                  placeholder="https://mon-domaine.com"
                  className="w-full px-3.5 py-2.5 rounded-xl bg-black/30 border border-white/10 text-white text-xs font-mono focus:outline-none focus:border-indigo-500"
                />
                <p className="text-[10px] text-slate-500">
                  Comme dans All-in-One WP Migration, tous les liens internes, images et réglages seront réécrits vers cette nouvelle URL.
                </p>
              </div>

              <button
                type="button"
                onClick={handleRestoreCapsule}
                disabled={!capsuleFile || restoring}
                className="w-full mt-4 py-3 rounded-xl bg-gradient-to-r from-teal-600 via-cyan-500 to-indigo-600 hover:from-teal-500 hover:to-indigo-500 text-white text-xs font-bold shadow-lg shadow-cyan-500/20 transition-all flex items-center justify-center gap-2 disabled:opacity-50"
              >
                {restoring ? (
                  <>
                    <RefreshCw className="w-4 h-4 animate-spin" />
                    <span>Décompression, réécriture d'URL et injection BDD...</span>
                  </>
                ) : (
                  <>
                    <span>Déployer et Restaurer le Site</span>
                    <ArrowRight className="w-4 h-4" />
                  </>
                )}
              </button>
            </div>
          )}
        </div>

        {/* Pied de page */}
        <p className="text-center text-[11px] text-slate-500 mt-6 flex items-center justify-center gap-1.5">
          <ShieldCheck className="w-3.5 h-3.5 text-emerald-400" />
          <span>RustForge CMS • 100% Rust Souverain • Zéro dépendance PHP</span>
        </p>
      </div>
    </div>
  );
};
