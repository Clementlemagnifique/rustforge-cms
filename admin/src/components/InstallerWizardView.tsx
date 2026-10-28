import { useState } from 'react';
import { api } from '../api';
import type { User } from '../types';

function consumeSetupKey() {
  const params = new URLSearchParams(window.location.hash.slice(1));
  const token = params.get('setup') || '';
  if (token) window.history.replaceState(null, '', window.location.pathname + window.location.search);
  return token;
}
export function InstallerWizardView({ onInstalled }: { onInstalled: (token: string, user: User) => void }) {
  const [setupToken, setSetupToken] = useState(consumeSetupKey);
  const [title, setTitle] = useState('Mon site');
  const [siteUrl, setSiteUrl] = useState(window.location.origin);
  const [email, setEmail] = useState('');
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [showPassword, setShowPassword] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  const install = async (event: React.FormEvent) => {
    event.preventDefault(); setError(''); setBusy(true);
    try {
      const result = await api.setup.install({ setup_token: setupToken, site_title: title, site_url: siteUrl,
        admin_email: email, admin_username: username, admin_password: password });
      onInstalled(result.token, result.user);
    } catch (e) { setError(e instanceof Error ? e.message : 'Installation impossible. Vérifiez les informations et réessayez.'); }
    finally { setBusy(false); }
  };
  return <main className="min-h-screen bg-[#0a0d14] text-slate-100 flex items-center justify-center p-6">
    <div className="w-full max-w-xl space-y-6">
      <header><div className="text-indigo-300 font-bold tracking-wide">RustForge CMS · version de test</div><h1 className="text-3xl font-bold mt-3">Bienvenue chez vous.</h1><p className="text-slate-400 mt-3">La base est connectée. Donnez un nom à votre site et créez votre compte pour commencer.</p></header>
      <form onSubmit={install} className="bg-[#101625] border border-white/10 rounded-2xl p-6 space-y-4">
        {error && <p role="alert" className="text-red-300">{error}</p>}
        {!setupToken && <p className="text-sm text-amber-200">Ouvrez le lien d’installation affiché dans le terminal, ou copiez sa clé ci-dessous.</p>}
        <details open={!setupToken}><summary className="text-sm text-slate-400 cursor-pointer">Clé d’installation</summary><label className="block text-sm mt-2">Clé fournie par votre lanceur<input aria-label="Clé d’installation" type="password" autoComplete="off" required value={setupToken} onChange={e => setSetupToken(e.target.value)} className="mt-2 w-full rounded-lg border border-white/10 bg-black/30 p-3" /></label></details>
        <label className="block text-sm">Nom du site<input required maxLength={300} autoComplete="organization" value={title} onChange={e => setTitle(e.target.value)} className="mt-2 w-full rounded-lg border border-white/10 bg-black/30 p-3" /></label>
        <label className="block text-sm">Adresse du site<input type="url" required value={siteUrl} onChange={e => setSiteUrl(e.target.value)} className="mt-2 w-full rounded-lg border border-white/10 bg-black/30 p-3" /></label>
        <label className="block text-sm">Votre adresse email<input type="email" required autoComplete="email" value={email} onChange={e => setEmail(e.target.value)} className="mt-2 w-full rounded-lg border border-white/10 bg-black/30 p-3" /></label>
        <label className="block text-sm">Identifiant<input required minLength={3} maxLength={32} pattern="[a-zA-Z0-9_.-]+" autoComplete="username" value={username} onChange={e => setUsername(e.target.value)} className="mt-2 w-full rounded-lg border border-white/10 bg-black/30 p-3" /><span className="block mt-1 text-xs text-slate-500">3 à 32 caractères : lettres, chiffres, tirets ou points.</span></label>
        <label className="block text-sm">Mot de passe<input type={showPassword ? 'text' : 'password'} required minLength={10} autoComplete="new-password" value={password} onChange={e => setPassword(e.target.value)} className="mt-2 w-full rounded-lg border border-white/10 bg-black/30 p-3" /></label>
        <label className="flex items-center gap-2 text-xs text-slate-400"><input type="checkbox" checked={showPassword} onChange={e => setShowPassword(e.target.checked)} />Afficher le mot de passe · 10 caractères minimum</label>
        <button disabled={busy} type="submit" className="w-full rounded-xl bg-indigo-600 hover:bg-indigo-500 px-4 py-3 font-semibold disabled:opacity-50">{busy ? 'Création de votre site…' : 'Créer mon site'}</button>
      </form>
      <p className="text-xs text-slate-500">Votre compte est personnel. L’assistant IA reste facultatif ; vous pouvez administrer votre site sans fournisseur IA.</p>
    </div>
  </main>;
}
