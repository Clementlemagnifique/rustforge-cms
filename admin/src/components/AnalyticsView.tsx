import { useEffect, useState } from 'react';
import { api } from '../api';
import type { AnalyticsSettings, AnalyticsSummary } from '../types';
import { analyticsMessages as m } from '../i18n/messages.fr';
import { useStudioStore } from '../stores/useStudioStore';

interface AnalyticsViewProps {
  initialTab?: 'dashboard' | 'settings';
}

export function AnalyticsView({ initialTab = 'dashboard' }: AnalyticsViewProps) {
  const user = useStudioStore((s) => s.user);
  const isAdmin = user?.role === 'admin';

  const [tab, setTab] = useState<'dashboard' | 'settings'>(initialTab);
  const [period, setPeriod] = useState<string>('7d');
  const [summary, setSummary] = useState<AnalyticsSummary | null>(null);
  const [settings, setSettings] = useState<AnalyticsSettings | null>(null);
  const [loading, setLoading] = useState<boolean>(true);
  const [busy, setBusy] = useState<boolean>(false);
  const [error, setError] = useState<string>('');
  const [notice, setNotice] = useState<string>('');
  const [hoveredPoint, setHoveredPoint] = useState<number | null>(null);

  // Formulaire réglages
  const [retentionMonths, setRetentionMonths] = useState<number>(13);
  const [honorDnt, setHonorDnt] = useState<boolean>(true);
  const [excludeStaff, setExcludeStaff] = useState<boolean>(true);
  const [excludedPathsText, setExcludedPathsText] = useState<string>('');

  const [revision, setRevision] = useState<number>(0);

  useEffect(() => {
    let active = true;
    api.analytics.summary(period)
      .then((res) => {
        if (active) {
          setSummary(res);
          setError('');
          setLoading(false);
        }
      })
      .catch(() => {
        if (active) {
          setError('Impossible de charger les statistiques d’audience.');
          setLoading(false);
        }
      });
    return () => {
      active = false;
    };
  }, [period, revision]);

  useEffect(() => {
    let active = true;
    if (tab === 'settings' && isAdmin && !settings) {
      api.analytics.settings()
        .then((cfg) => {
          if (active) {
            setSettings(cfg);
            setRetentionMonths(cfg.retention_months);
            setHonorDnt(cfg.honor_dnt);
            setExcludeStaff(cfg.exclude_authenticated);
            setExcludedPathsText(cfg.excluded_paths.join('\n'));
          }
        })
        .catch(() => {});
    }
    return () => {
      active = false;
    };
  }, [tab, isAdmin, settings]);

  const handleSaveSettings = async (e: React.FormEvent) => {
    e.preventDefault();
    if (busy || !isAdmin) return;
    setBusy(true);
    setNotice('');
    setError('');
    try {
      const paths = excludedPathsText
        .split('\n')
        .map((p) => p.trim())
        .filter((p) => p.length > 0);
      const updated: AnalyticsSettings = {
        enabled: true,
        retention_months: retentionMonths,
        honor_dnt: honorDnt,
        exclude_authenticated: excludeStaff,
        excluded_paths: paths,
      };
      const res = await api.analytics.updateSettings(updated);
      setSettings(res);
      setNotice(m.savedSuccess);
    } catch {
      setError('Erreur lors de l’enregistrement des réglages.');
    } finally {
      setBusy(false);
    }
  };

  const handlePurge = async () => {
    if (!window.confirm(m.purgeConfirm) || busy || !isAdmin) return;
    setBusy(true);
    setNotice('');
    setError('');
    try {
      const res = await api.analytics.purge();
      setNotice(res.message);
      setLoading(true);
      setRevision((r) => r + 1);
    } catch {
      setError('Erreur lors de la purge.');
    } finally {
      setBusy(false);
    }
  };

  const handleClearData = async () => {
    if (!window.confirm(m.clearDataConfirm) || busy || !isAdmin) return;
    setBusy(true);
    setNotice('');
    setError('');
    try {
      const res = await api.analytics.clearData();
      setNotice(res.message);
      setLoading(true);
      setRevision((r) => r + 1);
    } catch {
      setError('Erreur lors de l’effacement des données.');
    } finally {
      setBusy(false);
    }
  };

  const handleExportCsv = () => {
    const url = api.analytics.exportUrl(period);
    window.open(url, '_blank');
  };

  // Calcul dimensions du graphique d'historique
  const points = summary?.timeseries || [];
  const maxViews = points.reduce((acc, p) => Math.max(acc, p.views), 1);
  const chartHeight = 160;

  return (
    <div className="space-y-6 max-w-7xl mx-auto pb-12">
      {/* En-tête principal */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4 border-b border-slate-800 pb-5">
        <div>
          <div className="flex items-center gap-3">
            <h1 className="text-2xl font-bold text-white tracking-tight">{m.title}</h1>
            <span className="px-2.5 py-0.5 rounded-full text-[11px] font-medium bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
              {m.privacyBadge}
            </span>
          </div>
          <p className="text-xs text-slate-400 mt-1">{m.subtitle}</p>
        </div>

        {/* Onglets et actions */}
        <div className="flex flex-wrap items-center gap-3">
          <div className="bg-slate-900 border border-slate-800 rounded-lg p-1 flex">
            <button
              type="button"
              onClick={() => setTab('dashboard')}
              className={`px-3 py-1.5 text-xs font-medium rounded-md transition-colors ${
                tab === 'dashboard' ? 'bg-indigo-600 text-white shadow-sm' : 'text-slate-400 hover:text-slate-200'
              }`}
            >
              {m.dashboardTab}
            </button>
            {isAdmin && (
              <button
                type="button"
                onClick={() => setTab('settings')}
                className={`px-3 py-1.5 text-xs font-medium rounded-md transition-colors ${
                  tab === 'settings' ? 'bg-indigo-600 text-white shadow-sm' : 'text-slate-400 hover:text-slate-200'
                }`}
              >
                {m.settingsTab}
              </button>
            )}
          </div>

          {tab === 'dashboard' && (
            <>
              {/* Sélecteur de période */}
              <select
                value={period}
                onChange={(e) => {
                  setLoading(true);
                  setPeriod(e.target.value);
                }}
                disabled={loading}
                className="bg-slate-900 border border-slate-700/80 rounded-lg px-3 py-1.5 text-xs text-slate-200 focus:outline-none focus:ring-1 focus:ring-indigo-500"
              >
                <option value="24h">{m.period24h}</option>
                <option value="7d">{m.period7d}</option>
                <option value="30d">{m.period30d}</option>
                <option value="all">{m.periodAll}</option>
              </select>

              <button
                type="button"
                onClick={() => {
                  setLoading(true);
                  setRevision((r) => r + 1);
                }}
                disabled={loading}
                className="px-3 py-1.5 bg-slate-800 hover:bg-slate-700 text-slate-300 rounded-lg text-xs font-medium transition-colors border border-slate-700 disabled:opacity-50"
              >
                {m.refresh}
              </button>

              <button
                type="button"
                onClick={handleExportCsv}
                className="px-3 py-1.5 bg-indigo-600/20 hover:bg-indigo-600/30 text-indigo-300 border border-indigo-500/30 rounded-lg text-xs font-medium transition-colors"
              >
                {m.exportCsv}
              </button>
            </>
          )}
        </div>
      </div>

      {/* Messages d'erreur ou de succès */}
      {error && (
        <div role="alert" className="p-3 bg-red-950/40 border border-red-500/30 rounded-lg text-xs text-red-300">
          {error}
        </div>
      )}
      {notice && (
        <div role="status" className="p-3 bg-emerald-950/40 border border-emerald-500/30 rounded-lg text-xs text-emerald-300">
          {notice}
        </div>
      )}

      {/* VUE 1 : TABLEAU DE BORD STATISTIQUES */}
      {tab === 'dashboard' && (
        <div className="space-y-6">
          {/* Cartes KPI */}
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div className="bg-slate-900/80 border border-slate-800 rounded-xl p-5 shadow-sm">
              <span className="text-xs font-medium text-slate-400 uppercase tracking-wider">{m.kpiTotalViews}</span>
              <div className="mt-2 text-3xl font-extrabold text-white">
                {loading ? '…' : (summary?.total_views ?? 0).toLocaleString()}
              </div>
              <p className="mt-1 text-[11px] text-slate-500">Affichages de pages validés</p>
            </div>

            <div className="bg-slate-900/80 border border-slate-800 rounded-xl p-5 shadow-sm">
              <span className="text-xs font-medium text-slate-400 uppercase tracking-wider">{m.kpiUniqueVisitors}</span>
              <div className="mt-2 text-3xl font-extrabold text-indigo-400">
                {loading ? '…' : (summary?.unique_visitors ?? 0).toLocaleString()}
              </div>
              <p className="mt-1 text-[11px] text-slate-500">Empreintes journalières distinctes</p>
            </div>

            <div className="bg-slate-900/80 border border-slate-800 rounded-xl p-5 shadow-sm">
              <span className="text-xs font-medium text-slate-400 uppercase tracking-wider">{m.kpiViewsPerVisitor}</span>
              <div className="mt-2 text-3xl font-extrabold text-emerald-400">
                {loading ? '…' : (summary?.views_per_visitor ?? 0).toFixed(2)}
              </div>
              <p className="mt-1 text-[11px] text-slate-500">Ratio moyen par visiteur</p>
            </div>
          </div>

          {/* Graphique chronologique */}
          <div className="bg-slate-900/80 border border-slate-800 rounded-xl p-5">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-sm font-semibold text-white">{m.chartTitle}</h3>
              <div className="flex items-center gap-4 text-xs">
                <span className="flex items-center gap-1.5 text-slate-400">
                  <span className="w-2.5 h-2.5 rounded-sm bg-indigo-500"></span> Pages vues
                </span>
                <span className="flex items-center gap-1.5 text-slate-400">
                  <span className="w-2.5 h-2.5 rounded-sm bg-emerald-400"></span> Visiteurs uniques
                </span>
              </div>
            </div>

            {loading ? (
              <div className="h-40 flex items-center justify-center text-slate-500 text-xs">Chargement du graphique…</div>
            ) : points.length === 0 ? (
              <div className="h-40 flex items-center justify-center text-slate-500 text-xs">
                Aucune visite enregistrée pour cette période.
              </div>
            ) : (
              <div className="relative">
                {/* Visualisation SVG simple et fluide */}
                <svg className="w-full h-44 overflow-visible" preserveAspectRatio="none" viewBox={`0 0 ${Math.max(points.length * 40, 400)} ${chartHeight}`}>
                  {/* Lignes de repère horizontales */}
                  <line x1="0" y1="20" x2="100%" y2="20" stroke="#1e293b" strokeDasharray="4 4" />
                  <line x1="0" y1="80" x2="100%" y2="80" stroke="#1e293b" strokeDasharray="4 4" />
                  <line x1="0" y1="140" x2="100%" y2="140" stroke="#1e293b" />

                  {points.map((pt, i) => {
                    const totalWidth = Math.max(points.length * 40, 400);
                    const slotWidth = totalWidth / points.length;
                    const barWidth = Math.max(slotWidth * 0.55, 6);
                    const x = i * slotWidth + (slotWidth - barWidth) / 2;

                    const viewHeight = Math.max((pt.views / maxViews) * (chartHeight - 40), 2);
                    const visitorHeight = Math.max((pt.unique_visitors / maxViews) * (chartHeight - 40), 2);
                    const yViews = chartHeight - 20 - viewHeight;
                    const yVisitors = chartHeight - 20 - visitorHeight;

                    const isHovered = hoveredPoint === i;

                    return (
                      <g
                        key={pt.period_key}
                        onMouseEnter={() => setHoveredPoint(i)}
                        onMouseLeave={() => setHoveredPoint(null)}
                        className="cursor-pointer"
                      >
                        {/* Barre pages vues */}
                        <rect
                          x={x}
                          y={yViews}
                          width={barWidth / 2}
                          height={viewHeight}
                          fill={isHovered ? '#818cf8' : '#6366f1'}
                          rx="2"
                        />
                        {/* Barre visiteurs uniques */}
                        <rect
                          x={x + barWidth / 2 + 1}
                          y={yVisitors}
                          width={barWidth / 2}
                          height={visitorHeight}
                          fill={isHovered ? '#6ee7b7' : '#34d399'}
                          rx="2"
                        />
                      </g>
                    );
                  })}
                </svg>

                {/* Info-bulle point survolé */}
                {hoveredPoint !== null && points[hoveredPoint] && (
                  <div className="absolute top-2 right-4 bg-slate-950/90 border border-slate-700/80 rounded-lg p-2.5 text-xs shadow-xl pointer-events-none">
                    <p className="font-semibold text-slate-200">{points[hoveredPoint].period_key}</p>
                    <p className="text-indigo-400 mt-1">Pages vues : {points[hoveredPoint].views}</p>
                    <p className="text-emerald-400">Visiteurs uniques : {points[hoveredPoint].unique_visitors}</p>
                  </div>
                )}
              </div>
            )}
          </div>

          {/* Deux colonnes : Pages & Sources / Appareils */}
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            {/* Colonne gauche : Top Pages */}
            <div className="bg-slate-900/80 border border-slate-800 rounded-xl p-5">
              <h3 className="text-sm font-semibold text-white mb-3">{m.topPagesTitle}</h3>
              {loading ? (
                <p className="text-xs text-slate-500 py-4">Chargement…</p>
              ) : !summary?.top_pages?.length ? (
                <p className="text-xs text-slate-500 py-4">{m.topPagesEmpty}</p>
              ) : (
                <div className="space-y-2.5 overflow-x-auto">
                  <table className="w-full text-xs text-left">
                    <thead>
                      <tr className="text-slate-400 border-b border-slate-800 pb-2">
                        <th className="py-2 font-medium">Page</th>
                        <th className="py-2 text-right font-medium">Vues</th>
                        <th className="py-2 text-right font-medium">Uniques</th>
                      </tr>
                    </thead>
                    <tbody className="divide-y divide-slate-800/60">
                      {summary.top_pages.map((p, idx) => {
                        const pct = summary.total_views > 0 ? (p.views / summary.total_views) * 100 : 0;
                        return (
                          <tr key={p.path} className="hover:bg-slate-800/40 transition-colors">
                            <td className="py-2.5 pr-3 font-mono text-slate-200 max-w-[240px] truncate">
                              <span className="text-slate-500 mr-2 text-[10px]">#{idx + 1}</span>
                              {p.path}
                              <div className="w-full bg-slate-800/80 h-1 rounded-full mt-1.5 overflow-hidden">
                                <div className="bg-indigo-500 h-full rounded-full" style={{ width: `${Math.min(pct, 100)}%` }} />
                              </div>
                            </td>
                            <td className="py-2.5 text-right font-semibold text-white">{p.views.toLocaleString()}</td>
                            <td className="py-2.5 text-right text-slate-400">{p.unique_visitors.toLocaleString()}</td>
                          </tr>
                        );
                      })}
                    </tbody>
                  </table>
                </div>
              )}
            </div>

            {/* Colonne droite : Référents & Appareils */}
            <div className="space-y-6">
              {/* Sources de trafic */}
              <div className="bg-slate-900/80 border border-slate-800 rounded-xl p-5">
                <h3 className="text-sm font-semibold text-white mb-3">{m.topReferrersTitle}</h3>
                {loading ? (
                  <p className="text-xs text-slate-500 py-4">Chargement…</p>
                ) : !summary?.top_referrers?.length ? (
                  <p className="text-xs text-slate-500 py-4">{m.topReferrersEmpty}</p>
                ) : (
                  <div className="space-y-2">
                    {summary.top_referrers.map((r) => (
                      <div key={r.referrer} className="flex items-center justify-between py-1.5 text-xs">
                        <span className="font-mono text-slate-300 truncate max-w-[260px]">{r.referrer}</span>
                        <span className="font-semibold text-white">{r.views.toLocaleString()} vues</span>
                      </div>
                    ))}
                  </div>
                )}
              </div>

              {/* Répartition terminaux */}
              <div className="bg-slate-900/80 border border-slate-800 rounded-xl p-5">
                <h3 className="text-sm font-semibold text-white mb-3">{m.devicesTitle}</h3>
                {loading ? (
                  <p className="text-xs text-slate-500 py-4">Chargement…</p>
                ) : !summary?.devices?.length ? (
                  <p className="text-xs text-slate-500 py-4">{m.devicesEmpty}</p>
                ) : (
                  <div className="space-y-3">
                    {summary.devices.map((d) => (
                      <div key={d.device_type} className="space-y-1">
                        <div className="flex justify-between text-xs">
                          <span className="capitalize text-slate-300 font-medium">
                            {d.device_type === 'desktop' ? 'Ordinateur' : d.device_type === 'mobile' ? 'Mobile' : 'Tablette'}
                          </span>
                          <span className="text-slate-400">
                            {d.views.toLocaleString()} vues ({d.percentage}%)
                          </span>
                        </div>
                        <div className="w-full bg-slate-800 h-1.5 rounded-full overflow-hidden">
                          <div
                            className={`h-full rounded-full ${
                              d.device_type === 'desktop' ? 'bg-blue-500' : d.device_type === 'mobile' ? 'bg-emerald-500' : 'bg-purple-500'
                            }`}
                            style={{ width: `${Math.min(d.percentage, 100)}%` }}
                          />
                        </div>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            </div>
          </div>

          {/* Bandeau d'information CNIL */}
          <div className="bg-slate-900/40 border border-slate-800 rounded-xl p-5 flex items-start gap-4 text-xs text-slate-400">
            <div className="w-8 h-8 rounded-lg bg-emerald-500/10 text-emerald-400 flex items-center justify-center shrink-0 border border-emerald-500/20">
              ✓
            </div>
            <div>
              <h4 className="font-semibold text-slate-200 mb-1">{m.privacyNoticeTitle}</h4>
              <p className="leading-relaxed">{m.privacyNoticeText}</p>
            </div>
          </div>
        </div>
      )}

      {/* VUE 2 : RÉGLAGES DE MESURE (ADMIN SEULEMENT) */}
      {tab === 'settings' && isAdmin && (
        <div className="space-y-6 max-w-3xl">
          <form onSubmit={handleSaveSettings} className="bg-slate-900/80 border border-slate-800 rounded-xl p-6 space-y-6">
            <h2 className="text-base font-semibold text-white border-b border-slate-800 pb-3">{m.settingsTitle}</h2>

            <div className="space-y-4">
              <div>
                <label className="block text-xs font-medium text-slate-300 mb-1">{m.retentionLabel}</label>
                <input
                  type="number"
                  min="1"
                  max="36"
                  value={retentionMonths}
                  onChange={(e) => setRetentionMonths(parseInt(e.target.value, 10) || 13)}
                  disabled={busy}
                  className="w-32 bg-slate-800 border border-slate-700 rounded-lg px-3 py-1.5 text-xs text-white focus:outline-none focus:ring-1 focus:ring-indigo-500"
                />
                <p className="text-[11px] text-slate-500 mt-1">{m.retentionHint}</p>
              </div>

              <div className="pt-2 space-y-3">
                <label className="flex items-start gap-3 cursor-pointer">
                  <input
                    type="checkbox"
                    checked={honorDnt}
                    onChange={(e) => setHonorDnt(e.target.checked)}
                    disabled={busy}
                    className="mt-0.5 rounded border-slate-700 bg-slate-800 text-indigo-600 focus:ring-indigo-500"
                  />
                  <div>
                    <span className="text-xs font-medium text-slate-200">{m.honorDntLabel}</span>
                    <p className="text-[11px] text-slate-500">{m.honorDntHint}</p>
                  </div>
                </label>

                <label className="flex items-start gap-3 cursor-pointer">
                  <input
                    type="checkbox"
                    checked={excludeStaff}
                    onChange={(e) => setExcludeStaff(e.target.checked)}
                    disabled={busy}
                    className="mt-0.5 rounded border-slate-700 bg-slate-800 text-indigo-600 focus:ring-indigo-500"
                  />
                  <div>
                    <span className="text-xs font-medium text-slate-200">{m.excludeStaffLabel}</span>
                    <p className="text-[11px] text-slate-500">{m.excludeStaffHint}</p>
                  </div>
                </label>
              </div>

              <div>
                <label className="block text-xs font-medium text-slate-300 mb-1">{m.excludedPathsLabel}</label>
                <textarea
                  rows={4}
                  value={excludedPathsText}
                  onChange={(e) => setExcludedPathsText(e.target.value)}
                  disabled={busy}
                  placeholder="/admin&#10;/api&#10;/uploads"
                  className="w-full bg-slate-800 border border-slate-700 rounded-lg p-2.5 text-xs font-mono text-slate-200 focus:outline-none focus:ring-1 focus:ring-indigo-500"
                />
                <p className="text-[11px] text-slate-500 mt-1">{m.excludedPathsHint}</p>
              </div>
            </div>

            <div className="pt-2 border-t border-slate-800 flex justify-end">
              <button
                type="submit"
                disabled={busy}
                className="px-4 py-2 bg-indigo-600 hover:bg-indigo-500 text-white rounded-lg text-xs font-medium transition-colors disabled:opacity-50"
              >
                {busy ? m.savingSettings : m.saveSettings}
              </button>
            </div>
          </form>

          {/* Section Purge et Nettoyage */}
          <div className="bg-slate-900/40 border border-slate-800 rounded-xl p-6 space-y-4">
            <h3 className="text-sm font-semibold text-white">{m.purgeSection}</h3>
            <div className="flex flex-wrap gap-4 items-center">
              <button
                type="button"
                onClick={handlePurge}
                disabled={busy}
                className="px-3.5 py-2 bg-slate-800 hover:bg-slate-700 text-slate-200 border border-slate-700 rounded-lg text-xs font-medium transition-colors disabled:opacity-50"
              >
                {m.purgeBtn}
              </button>

              <button
                type="button"
                onClick={handleClearData}
                disabled={busy}
                className="px-3.5 py-2 bg-red-950/40 hover:bg-red-900/50 text-red-300 border border-red-800/50 rounded-lg text-xs font-medium transition-colors disabled:opacity-50"
              >
                {m.clearDataBtn}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
