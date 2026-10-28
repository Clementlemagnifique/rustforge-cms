import { useState } from 'react';
import { api } from '../api';
import type { AdminNavigation } from '../types';
const labels: Record<string, string> = { forms: 'Formulaires', galleries: 'Galeries', zen_mode: 'Lecture Zen', ecommerce: 'Boutique numérique', lms: 'Formation' };
export function NativeModulesView({ navigation, onChanged }: { navigation: AdminNavigation; onChanged: (data: AdminNavigation) => void }) {
  const [patch, setPatch] = useState<Record<string, boolean>>({});
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState('');
  const [error, setError] = useState('');
  async function save(event: React.FormEvent) {
    event.preventDefault(); setSaving(true); setError(''); setMessage('');
    try { const updated = await api.admin.updateModules(patch); onChanged(updated); setPatch({}); setMessage('Modules enregistrés. Le menu a été actualisé.'); }
    catch (e) { setError(e instanceof Error ? e.message : 'Enregistrement impossible'); }
    finally { setSaving(false); }
  }
  return <form onSubmit={save} className="max-w-3xl space-y-5"><div><h2 className="text-xl font-semibold">Modules natifs</h2><p className="mt-2 text-sm text-slate-400">Activez les capacités utiles. La désactivation conserve les données et ferme les routes du module. Certains écrans métier restent à construire; leur état est indiqué dans le plan des écrans.</p></div>
    <fieldset disabled={saving} className="space-y-3"><legend className="sr-only">Activation des modules disponibles</legend>{navigation.supported_modules.map(key => <label key={key} className="flex gap-3 p-4 border border-white/10 rounded-xl"><input type="checkbox" checked={patch[key] ?? navigation.modules[key] ?? false} onChange={e => setPatch(p => ({ ...p, [key]: e.target.checked }))} /><span>{labels[key] || key}</span></label>)}</fieldset>
    {error && <p role="alert" className="text-red-300">{error}</p>}{message && <p role="status" className="text-emerald-300">{message}</p>}
    <button disabled={saving || Object.keys(patch).length === 0} className="rounded-xl bg-indigo-600 px-5 py-2 disabled:opacity-40">{saving ? 'Enregistrement…' : 'Enregistrer les modules'}</button>
    <p className="text-xs text-slate-500">Les réglages actuels du site sont respectés. Les autres modules du plan seront activables lorsqu’ils seront disponibles.</p>
  </form>;
}
