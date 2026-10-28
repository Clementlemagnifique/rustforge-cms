import { useState } from 'react';
import { api } from '../api';
import type { MigrationInspection, MigrationImportResult } from '../types';

export function MigrationView({ onNavigate, onImported }: { onNavigate: (view: string) => void; onImported?: () => void }) {
  const [file, setFile] = useState<File | null>(null);
  const [source, setSource] = useState('');
  const [inspection, setInspection] = useState<MigrationInspection | null>(null);
  const [result, setResult] = useState<MigrationImportResult | null>(null);
  const [acknowledge, setAcknowledge] = useState(false);
  const [contentOnly, setContentOnly] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  const reset = () => { setInspection(null); setResult(null); setAcknowledge(false); setContentOnly(false); setError(''); };
  const inspect = async () => {
    if (!file) return;
    setBusy(true); setError('');
    try { setInspection(await api.migration.inspect(file, source)); }
    catch (e) { setError(e instanceof Error ? e.message : 'Analyse impossible'); }
    finally { setBusy(false); }
  };
  const importContent = async () => {
    if (!file || !inspection) return;
    setBusy(true); setError('');
    try { setResult(await api.migration.import(file, source, inspection.seal, contentOnly)); onImported?.(); }
    catch (e) { setError(e instanceof Error ? e.message : 'Import impossible'); }
    finally { setBusy(false); }
  };
  return <div className="max-w-4xl mx-auto space-y-6">
    <div><h1 className="text-xl font-bold">Importer depuis WordPress</h1><p className="mt-2 text-sm text-slate-400">Vos contenus d’abord. Vérifiez les éléments détectés avant d’autoriser leur import.</p></div>
    <ol className="flex flex-wrap gap-6 text-sm" aria-label="Étapes de l’import"><li>1. Choisir l’export</li><li className={inspection ? 'text-indigo-300' : ''}>2. Vérifier</li><li className={result ? 'text-emerald-300' : ''}>3. Importer et contrôler</li></ol>
    {error && <p role="alert" className="p-4 border border-red-400/30 rounded-xl text-red-300">{error}</p>}
    <section className="p-6 bg-[#0f1422] rounded-2xl border border-white/10 space-y-4">
      <h2 className="font-semibold">1. Votre export WordPress</h2>
      <p className="text-sm text-slate-400">Dans WordPress : Outils → Exporter → Tout le contenu. Sélectionnez ensuite le fichier XML obtenu (16 Mio maximum). Les exports JSON RustForge CMS sont aussi acceptés.</p>
      <label className="block text-sm space-y-2"><span>Fichier XML ou JSON</span><input className="block w-full" type="file" accept=".xml,.json" disabled={busy} onChange={e => { reset(); setFile(e.target.files?.[0] ?? null); }} /></label>
      <label className="block text-sm space-y-2"><span>Adresse du site source, si elle manque dans le fichier</span><input className="w-full rounded-lg bg-black/30 border border-white/10 p-3" type="url" placeholder="https://mon-site.fr" value={source} disabled={busy} onChange={e => { reset(); setSource(e.target.value); }} /></label>
      <button className="rounded-lg bg-indigo-600 px-4 py-2 disabled:opacity-40" disabled={!file || busy || !!result} onClick={inspect}>{busy && !inspection ? 'Analyse…' : 'Analyser sans importer'}</button>
    </section>
    {inspection && !result && <section className="p-6 bg-[#0f1422] rounded-2xl border border-white/10 space-y-4">
      <h2 className="font-semibold">2. Ce qui a été détecté</h2><p className="text-sm break-all">Source : {inspection.source_url}</p>
      <div className="grid grid-cols-3 gap-3 text-center">{[[inspection.entries, 'contenus'], [inspection.terms, 'termes'], [inspection.media_references, 'médias référencés']].map(([n, label]) => <div className="rounded-lg bg-white/5 p-3" key={label}><strong className="block text-xl">{n}</strong><span className="text-xs text-slate-400">{label}</span></div>)}</div>
      <p className="text-sm text-slate-400">{Object.entries(inspection.by_type).map(([type, count]) => `${type} : ${count}`).join(' · ')}</p>
      {inspection.sample.length > 0 && <ul className="text-sm space-y-2">{inspection.sample.map((entry, i) => <li key={i}>{entry.title || 'Sans titre'} <span className="text-slate-500">/{entry.slug}</span></li>)}</ul>}
      <div className="rounded-xl p-4 border border-amber-500/30 bg-amber-500/5"><h3 className="font-semibold text-amber-200">À vérifier avant de continuer</h3><ul className="list-disc pl-5 mt-2 space-y-2 text-sm text-amber-100/80">{inspection.warnings.map(w => <li key={w}>{w}</li>)}</ul></div>
      {inspection.media_references > 0 && <label className="flex gap-3 text-sm"><input type="checkbox" checked={contentOnly} onChange={e => setContentOnly(e.target.checked)} />Importer les contenus seulement ; je transférerai et vérifierai les fichiers médias séparément.</label>}
      <label className="flex gap-3 text-sm"><input type="checkbox" checked={acknowledge} onChange={e => setAcknowledge(e.target.checked)} />J’ai lu ces points et j’autorise l’import dans ce site de test.</label>
      <button className="rounded-lg bg-indigo-600 px-4 py-2 disabled:opacity-40" disabled={busy || !acknowledge || (inspection.media_references > 0 && !contentOnly) || inspection.blocking_conflicts > 0 || inspection.entries === 0} onClick={importContent}>{busy ? 'Import en cours…' : 'Confirmer l’import des contenus'}</button>
    </section>}
    {result && <section className="rounded-2xl border border-emerald-500/30 p-6 space-y-4" role="status">
      <h2 className="text-lg font-semibold text-emerald-300">Import éditorial terminé</h2><p>{result.report.entries_imported} contenus et {result.report.terms_imported} termes enregistrés.</p>
      <p className="text-sm text-slate-400">Contrôlez le résultat avant toute mise en ligne. Cet import ne remplace pas encore une migration complète du site.</p>
      <ul className="list-disc pl-5 space-y-2 text-sm">{[...new Set([...result.checks_remaining, ...result.report.warnings])].map(w => <li key={w}>{w}</li>)}</ul>
      <div className="flex gap-3 flex-wrap"><button onClick={() => onNavigate('entries:post')} className="rounded-lg bg-indigo-600 px-4 py-2">Vérifier les articles</button><button onClick={() => onNavigate('entries:page')} className="rounded-lg bg-white/10 px-4 py-2">Vérifier les pages</button><a href="/fr" target="_blank" rel="noreferrer" className="rounded-lg bg-white/10 px-4 py-2">Voir le site</a></div>
    </section>}
  </div>;
}
