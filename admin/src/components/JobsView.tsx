import { useEffect, useRef, useState } from 'react';
import { api } from '../api';
import type { JobCapture, JobsPage, JobSummary } from '../types';
import { jobsMessages as m } from '../i18n/messages.fr';

type Action = 'cancel' | 'retry' | 'resume-restored';
const label = (state: string) => m.states[state as keyof typeof m.states] || state;
const date = (value: string) => new Date(value).toLocaleString();

export function JobsView() {
  const [filter, setFilter] = useState('');
  const [page, setPage] = useState(1);
  const [revision, setRevision] = useState(0);
  const key = `${filter}:${page}:${revision}`;
  const [result, setResult] = useState<{key: string; data: JobsPage | null; error: string} | null>(null);
  const loading = result?.key !== key;
  const data = !loading ? result?.data : null;
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  const [notice, setNotice] = useState('');
  const [pending, setPending] = useState<{ job: JobSummary; action: Action } | null>(null);
  const [capture, setCapture] = useState<JobCapture | null>(null);
  const mounted = useRef(false);
  useEffect(() => { mounted.current = true; return () => { mounted.current = false; }; }, []);
  useEffect(() => {
    let active = true;
    api.jobs.list(filter, page).then(value => { if (active) setResult({key, data: value, error: ''}); })
      .catch(() => { if (active) setResult({key, data: null, error: m.failed}); });
    return () => { active = false; };
  }, [filter, page, revision, key]);
  function clearPrivateView() { setPending(null); setCapture(null); setError(''); }
  async function run(operation: () => Promise<void>) {
    if (busy) return;
    setBusy(true); setError(''); setNotice('');
    try { await operation(); }
    catch { if (mounted.current) setError(m.failed); }
    finally { if (mounted.current) setBusy(false); }
  }
  const button = 'px-3 py-2 rounded border border-slate-600 text-sm disabled:opacity-40';
  return <section className="space-y-5 text-slate-200" aria-busy={loading || busy}>
    <h2 className="text-xl font-semibold">{m.title}</h2>
    <p className="text-sm text-slate-400">{m.note}</p>
    <div className="flex flex-wrap gap-3 items-center">
      <label>{m.filter} <select className="bg-slate-800 rounded p-2" value={filter} disabled={busy} onChange={e => { clearPrivateView(); setFilter(e.target.value); setPage(1); }}>
        <option value="">{m.all}</option>{Object.entries(m.states).map(([key, text]) => <option key={key} value={key}>{text}</option>)}
      </select></label>
      <button className={button} disabled={loading || busy} onClick={() => { clearPrivateView(); setRevision(r => r + 1); }}>{m.refresh}</button>
      <button className={button} disabled={loading || busy} onClick={() => void run(async () => {
        const result = await api.jobs.captureBatch();
        if (mounted.current) { clearPrivateView(); setNotice(m.handled + result.handled); setPage(1); setRevision(r => r + 1); }
      })}>{m.captureBatch}</button>
    </div>
    {(error || (!loading && result?.error)) && <p role="alert" className="text-red-400">{error || result?.error}</p>}
    {notice && <p role="status">{notice}</p>}
    {loading && <p role="status">{m.loading}</p>}
    {data && <>
      <dl className="flex flex-wrap gap-4 text-sm">{Object.entries(m.states).map(([state, text]) => <div key={state}><dt>{text}</dt><dd className="font-bold">{data.counts[state] || 0}</dd></div>)}</dl>
      {data.items.length === 0 ? <p>{m.empty}</p> : <div className="overflow-x-auto"><table className="w-full text-left text-sm">
        <thead><tr>{[m.type, m.state, m.attempts, m.available, m.error, m.actions].map(title => <th key={title} scope="col" className="p-3">{title}</th>)}</tr></thead>
        <tbody>{data.items.map(job => <tr key={job.id} className="border-t border-slate-700">
          <td className="p-3">{job.kind}<small className="block text-slate-400">{job.id}</small></td>
          <td className="p-3">{label(job.state)}</td><td className="p-3">{job.attempts}/{job.max_attempts}</td>
          <td className="p-3">{date(job.available_at)}</td><td className="p-3">{job.error_code || '—'}</td>
          <td className="p-3"><div className="flex flex-wrap gap-2">
            {['queued', 'retry', 'running', 'paused_restore'].includes(job.state) && <button disabled={busy} className={button} onClick={() => setPending({job, action: 'cancel'})}>{m.cancel}</button>}
            {job.state === 'failed' && <button disabled={busy} className={button} onClick={() => setPending({job, action: 'retry'})}>{m.retry}</button>}
            {job.state === 'paused_restore' && <button disabled={busy} className={button} onClick={() => setPending({job, action: 'resume-restored'})}>{m.resume}</button>}
            {job.capture_available && <button disabled={busy} className={button} onClick={() => void run(async () => { const result = await api.jobs.capture(job.id); if (mounted.current) setCapture(result); })}>{m.preview}</button>}
          </div></td>
        </tr>)}</tbody>
      </table></div>}
      <nav aria-label={m.title} className="flex gap-4 items-center">
        <button className={button} disabled={busy || page <= 1} onClick={() => { clearPrivateView(); setPage(p => p - 1); }}>{m.previous}</button>
        <span>{data.page}/{data.pages} · {data.total}</span>
        <button className={button} disabled={busy || page >= data.pages} onClick={() => { clearPrivateView(); setPage(p => p + 1); }}>{m.next}</button>
      </nav>
    </>}
    {pending && <div className="p-4 border border-amber-700 rounded space-y-3" role="group" aria-label={m.confirm}>
      <p>{m.confirm}</p><p className="break-all">{pending.job.id} · {pending.action === 'cancel' ? m.cancel : pending.action === 'retry' ? m.retry : m.resume}</p>
      <button className={button} disabled={busy} onClick={() => void run(async () => {
        await api.jobs.action(pending.job, pending.action);
        if (mounted.current) { setPage(1); setPending(null); setRevision(r => r + 1); }
      })}>{m.yes}</button>{' '}<button className={button} disabled={busy} onClick={() => setPending(null)}>{m.no}</button>
    </div>}
    {capture && <article className="p-4 border border-slate-600 rounded space-y-3" aria-label={m.preview}>
      <button className={button} onClick={() => setCapture(null)}>{m.close}</button>
      <p>{m.recipient} : {capture.recipient}</p><h3 className="font-semibold">{capture.subject}</h3>
      <pre className="whitespace-pre-wrap break-words font-sans">{capture.text}</pre>
    </article>}
  </section>;
}
