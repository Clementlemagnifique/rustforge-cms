import React, { useState, useEffect, useCallback } from 'react';
import {
  FileText,
  Plus,
  Trash2,
  Edit2,
  Inbox,
  ShieldCheck,
  CheckCircle,
  Copy,
  Download,
  Eye,
  Mail,
  Save,
  ArrowLeft,
} from 'lucide-react';
import { api } from '../api';
import type { FormItem, FormSubmissionItem, FormField } from '../types';

export function FormsView() {
  const [activeTab, setActiveTab] = useState<'forms' | 'submissions'>('forms');
  const [forms, setForms] = useState<FormItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  // Form builder state
  const [editingForm, setEditingForm] = useState<FormItem | null>(null);
  const [isCreating, setIsCreating] = useState(false);
  const [builderName, setBuilderName] = useState('');
  const [builderSlug, setBuilderSlug] = useState('');
  const [builderDesc, setBuilderDesc] = useState('');
  const [builderActive, setBuilderActive] = useState(true);
  const [builderSuccessMsg, setBuilderSuccessMsg] = useState('Merci ! Votre message a bien été envoyé.');
  const [builderNotifyEmail, setBuilderNotifyEmail] = useState('');
  const [builderHoneypot, setBuilderHoneypot] = useState(true);
  const [builderFields, setBuilderFields] = useState<FormField[]>([
    { name: 'nom', label: 'Nom complet', type: 'text', required: true, placeholder: 'Ex: Marie Dupont' },
    { name: 'email', label: 'Adresse e-mail', type: 'email', required: true, placeholder: 'marie@example.com' },
    { name: 'message', label: 'Votre message', type: 'textarea', required: true, placeholder: 'Comment pouvons-nous vous aider ?' },
  ]);

  // Submissions state
  const [selectedFormId, setSelectedFormId] = useState<string>('');
  const [submissions, setSubmissions] = useState<FormSubmissionItem[]>([]);
  const [loadingSubs, setLoadingSubs] = useState(false);
  const [inspectingSub, setInspectingSub] = useState<FormSubmissionItem | null>(null);

  const loadForms = useCallback(async () => {
    setError(null);
    try {
      const data = await api.forms.list();
      setForms(data);
      if (data.length > 0 && !selectedFormId) {
        setSelectedFormId(data[0].id);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Erreur lors du chargement des formulaires');
    } finally {
      setLoading(false);
    }
  }, [selectedFormId]);

  useEffect(() => {
    let ignore = false;
    async function fetchForms() {
      try {
        const data = await api.forms.list();
        if (!ignore) {
          setForms(data);
          if (data.length > 0 && !selectedFormId) {
            setSelectedFormId(data[0].id);
          }
          setLoading(false);
        }
      } catch (err) {
        if (!ignore) {
          setError(err instanceof Error ? err.message : 'Erreur lors du chargement des formulaires');
          setLoading(false);
        }
      }
    }
    void fetchForms();
    return () => {
      ignore = true;
    };
  }, [selectedFormId]);

  useEffect(() => {
    if (activeTab !== 'submissions' || !selectedFormId) return;
    let ignore = false;
    async function fetchSubs() {
      try {
        const res = await api.forms.submissions(selectedFormId);
        if (!ignore) {
          setSubmissions(res.items);
          setLoadingSubs(false);
        }
      } catch (err) {
        if (!ignore) {
          setError(err instanceof Error ? err.message : 'Erreur lors du chargement des soumissions');
          setLoadingSubs(false);
        }
      }
    }
    void fetchSubs();
    return () => {
      ignore = true;
    };
  }, [activeTab, selectedFormId]);

  function startCreate() {
    setIsCreating(true);
    setEditingForm(null);
    setBuilderName('Formulaire de contact');
    setBuilderSlug('contact');
    setBuilderDesc('Formulaire de contact général du site.');
    setBuilderActive(true);
    setBuilderSuccessMsg('Merci ! Votre message a bien été envoyé.');
    setBuilderNotifyEmail('');
    setBuilderHoneypot(true);
    setBuilderFields([
      { name: 'nom', label: 'Nom complet', type: 'text', required: true, placeholder: 'Ex: Marie Dupont' },
      { name: 'email', label: 'Adresse e-mail', type: 'email', required: true, placeholder: 'marie@example.com' },
      { name: 'message', label: 'Votre message', type: 'textarea', required: true, placeholder: 'Comment pouvons-nous vous aider ?' },
    ]);
  }

  function startEdit(form: FormItem) {
    setIsCreating(false);
    setEditingForm(form);
    setBuilderName(form.name);
    setBuilderSlug(form.slug);
    setBuilderDesc(form.description || '');
    setBuilderActive(form.is_active);
    setBuilderSuccessMsg((form.settings?.success_message as string) || 'Merci ! Votre message a bien été envoyé.');
    setBuilderNotifyEmail((form.settings?.notify_email as string) || '');
    setBuilderHoneypot(form.settings?.honeypot !== false);

    // Parse fields
    if (Array.isArray(form.fields)) {
      setBuilderFields(form.fields as FormField[]);
    } else if (form.fields && typeof form.fields === 'object' && form.fields.properties) {
      const props = form.fields.properties as Record<string, { type?: string; title?: string }>;
      const req = (form.fields.required as string[]) || [];
      const parsed: FormField[] = Object.entries(props).map(([k, v]) => ({
        name: k,
        label: v.title || k,
        type: v.type === 'integer' || v.type === 'number' ? 'text' : (v.type as FormField['type']) || 'text',
        required: req.includes(k),
      }));
      setBuilderFields(parsed.length > 0 ? parsed : [{ name: 'message', label: 'Message', type: 'textarea', required: true }]);
    } else {
      setBuilderFields([{ name: 'message', label: 'Message', type: 'textarea', required: true }]);
    }
  }

  async function handleSaveForm(e: React.FormEvent) {
    e.preventDefault();
    if (!builderName.trim()) {
      alert('Veuillez renseigner le nom du formulaire.');
      return;
    }

    // Convert fields to valid json schema
    const properties: Record<string, unknown> = {};
    const required: string[] = [];
    for (const f of builderFields) {
      const fieldKey = f.name.trim().toLowerCase().replace(/[^a-z0-9_]/g, '_') || 'champ';
      properties[fieldKey] = {
        type: 'string',
        title: f.label.trim() || fieldKey,
      };
      if (f.required) required.push(fieldKey);
    }

    const schemaObj = {
      type: 'object',
      properties,
      required,
      additionalProperties: false,
    };

    const payload = {
      name: builderName.trim(),
      slug: builderSlug.trim() || undefined,
      description: builderDesc.trim(),
      is_active: builderActive,
      fields: schemaObj,
      settings: {
        ...(editingForm?.settings ?? {}),
        success_message: builderSuccessMsg.trim(),
        notify_email: builderNotifyEmail.trim() || null,
        honeypot: builderHoneypot,
      },
    };

    try {
      if (editingForm) {
        await api.forms.update(editingForm.id, payload);
        setNotice(`Le formulaire « ${builderName} » a été mis à jour.`);
      } else {
        await api.forms.create(payload);
        setNotice(`Le formulaire « ${builderName} » a été créé.`);
      }
      setEditingForm(null);
      setIsCreating(false);
      void loadForms();
      setTimeout(() => setNotice(null), 4000);
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Erreur lors de l’enregistrement');
    }
  }

  async function handleDeleteForm(id: string, name: string) {
    if (!window.confirm(`Confirmez-vous la suppression du formulaire « ${name} » et de toutes ses soumissions ?`)) {
      return;
    }
    try {
      await api.forms.delete(id);
      void loadForms();
      setNotice(`Formulaire « ${name} » supprimé.`);
      setTimeout(() => setNotice(null), 4000);
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Erreur lors de la suppression');
    }
  }

  async function handleMarkRead(sub: FormSubmissionItem) {
    try {
      await api.forms.markRead(sub.id);
      setSubmissions((prev) =>
        prev.map((s) => (s.id === sub.id ? { ...s, is_read: true } : s))
      );
      if (inspectingSub?.id === sub.id) {
        setInspectingSub({ ...inspectingSub, is_read: true });
      }
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Erreur');
    }
  }

  function exportSubmissionsCsv() {
    if (submissions.length === 0) return;
    const allKeys = Array.from(
      new Set(submissions.flatMap((s) => Object.keys(s.data || {})))
    );
    const header = ['ID', 'Date', 'Statut', ...allKeys].join(';');
    const rows = submissions.map((s) => {
      const vals = allKeys.map((k) => JSON.stringify(s.data?.[k] ?? ''));
      return [s.id, s.created_at, s.is_spam ? 'Spam' : 'Valide', ...vals].join(';');
    });
    const csvContent = 'data:text/csv;charset=utf-8,\uFEFF' + [header, ...rows].join('\n');
    const encoded = encodeURI(csvContent);
    const link = document.createElement('a');
    link.setAttribute('href', encoded);
    link.setAttribute('download', `soumissions-${selectedFormId}.csv`);
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
  }

  return (
    <div className="max-w-6xl space-y-6">
      {/* Header */}
      <div className="flex flex-wrap items-center justify-between gap-4">
        <div>
          <h2 className="text-xl font-bold text-white flex items-center gap-2.5">
            <FileText className="w-5 h-5 text-indigo-400" />
            Formulaires & Soumissions Souverains
          </h2>
          <p className="text-xs text-slate-400 mt-1">
            Capture sécurisée des messages et demandes de contact sans script tiers ni captcha intrusif.
          </p>
        </div>

        {!isCreating && !editingForm && (
          <button
            type="button"
            onClick={startCreate}
            className="inline-flex items-center gap-2 rounded-xl bg-indigo-600 hover:bg-indigo-500 px-4 py-2.5 text-xs font-semibold text-white shadow-lg shadow-indigo-600/20"
          >
            <Plus className="w-4 h-4" />
            Nouveau Formulaire
          </button>
        )}
      </div>

      {notice && (
        <div className="rounded-xl border border-emerald-500/30 bg-emerald-500/10 p-3.5 text-xs text-emerald-200 flex items-center gap-2">
          <CheckCircle className="w-4 h-4 text-emerald-400 shrink-0" />
          <span>{notice}</span>
        </div>
      )}

      {error && (
        <div className="rounded-xl border border-red-500/30 bg-red-500/10 p-3.5 text-xs text-red-200">
          {error}
        </div>
      )}

      {/* Editor Modal / Sub-view */}
      {(isCreating || editingForm) && (
        <form onSubmit={handleSaveForm} className="rounded-2xl border border-white/10 bg-[#0f1422] p-6 space-y-6 text-xs shadow-2xl">
          <div className="flex items-center justify-between border-b border-white/10 pb-4">
            <div className="flex items-center gap-3">
              <button
                type="button"
                onClick={() => {
                  setIsCreating(false);
                  setEditingForm(null);
                }}
                className="rounded-lg p-1.5 hover:bg-white/10 text-slate-400 hover:text-white"
              >
                <ArrowLeft className="w-4 h-4" />
              </button>
              <h3 className="text-base font-bold text-white">
                {isCreating ? 'Créer un nouveau formulaire' : `Modifier « ${editingForm?.name} »`}
              </h3>
            </div>
            <div className="flex items-center gap-3">
              <label className="flex items-center gap-2 cursor-pointer">
                <input
                  type="checkbox"
                  checked={builderActive}
                  onChange={(e) => setBuilderActive(e.target.checked)}
                  className="rounded bg-slate-900 border-white/20 text-indigo-600"
                />
                <span className="text-slate-300">Actif pour les soumissions publiques</span>
              </label>
            </div>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <label className="block space-y-1">
              <span className="text-slate-400">Nom du formulaire *</span>
              <input
                type="text"
                required
                value={builderName}
                onChange={(e) => setBuilderName(e.target.value)}
                placeholder="Ex: Formulaire de contact"
                className="w-full rounded-xl bg-slate-950 border border-white/10 px-3 py-2 text-xs text-white"
              />
            </label>

            <label className="block space-y-1">
              <span className="text-slate-400">Identifiant unique (slug)</span>
              <input
                type="text"
                value={builderSlug}
                onChange={(e) => setBuilderSlug(e.target.value)}
                placeholder="Ex: contact"
                className="w-full rounded-xl bg-slate-950 border border-white/10 px-3 py-2 font-mono text-xs text-white"
              />
            </label>
          </div>

          <label className="block space-y-1">
            <span className="text-slate-400">Description interne (optionnelle)</span>
            <input
              type="text"
              value={builderDesc}
              onChange={(e) => setBuilderDesc(e.target.value)}
              placeholder="Ex: Reçu depuis la page contact de présentation"
              className="w-full rounded-xl bg-slate-950 border border-white/10 px-3 py-2 text-xs text-white"
            />
          </label>

          {/* Builder de champs */}
          <div className="space-y-3 pt-3 border-t border-white/5">
            <div className="flex items-center justify-between">
              <span className="font-semibold text-slate-200">Champs du formulaire :</span>
              <button
                type="button"
                onClick={() =>
                  setBuilderFields((prev) => [
                    ...prev,
                    {
                      name: `champ_${prev.length + 1}`,
                      label: `Champ ${prev.length + 1}`,
                      type: 'text',
                      required: false,
                    },
                  ])
                }
                className="inline-flex items-center gap-1.5 rounded-lg border border-indigo-400/30 bg-indigo-500/10 px-2.5 py-1 text-[11px] text-indigo-200 hover:bg-indigo-500/20"
              >
                <Plus className="w-3.5 h-3.5" />
                Ajouter un champ
              </button>
            </div>

            <div className="space-y-2">
              {builderFields.map((f, idx) => (
                <div
                  key={idx}
                  className="grid grid-cols-1 sm:grid-cols-12 gap-2 items-center rounded-xl border border-white/5 bg-black/30 p-2.5"
                >
                  <div className="sm:col-span-4">
                    <input
                      type="text"
                      placeholder="Libellé affiché"
                      value={f.label}
                      onChange={(e) => {
                        const updated = [...builderFields];
                        updated[idx].label = e.target.value;
                        setBuilderFields(updated);
                      }}
                      className="w-full rounded-lg bg-slate-900 border border-white/10 px-2.5 py-1.5 text-xs text-white"
                    />
                  </div>
                  <div className="sm:col-span-3">
                    <input
                      type="text"
                      placeholder="Nom_technique"
                      value={f.name}
                      onChange={(e) => {
                        const updated = [...builderFields];
                        updated[idx].name = e.target.value;
                        setBuilderFields(updated);
                      }}
                      className="w-full rounded-lg bg-slate-900 border border-white/10 px-2.5 py-1.5 font-mono text-[11px] text-slate-300"
                    />
                  </div>
                  <div className="sm:col-span-3">
                    <select
                      value={f.type}
                      onChange={(e) => {
                        const updated = [...builderFields];
                        updated[idx].type = e.target.value as FormField['type'];
                        setBuilderFields(updated);
                      }}
                      className="w-full rounded-lg bg-slate-900 border border-white/10 px-2 py-1.5 text-xs text-white"
                    >
                      <option value="text">Texte court</option>
                      <option value="email">Adresse e-mail</option>
                      <option value="tel">Numéro de téléphone</option>
                      <option value="textarea">Zone de message</option>
                    </select>
                  </div>
                  <div className="sm:col-span-1 flex items-center justify-center">
                    <label className="flex items-center gap-1 cursor-pointer text-[10px] text-slate-400" title="Champ obligatoire">
                      <input
                        type="checkbox"
                        checked={f.required}
                        onChange={(e) => {
                          const updated = [...builderFields];
                          updated[idx].required = e.target.checked;
                          setBuilderFields(updated);
                        }}
                      />
                      Req
                    </label>
                  </div>
                  <div className="sm:col-span-1 flex items-center justify-end">
                    <button
                      type="button"
                      disabled={builderFields.length <= 1}
                      onClick={() => setBuilderFields(builderFields.filter((_, i) => i !== idx))}
                      className="text-red-400 hover:text-red-300 p-1 disabled:opacity-30"
                    >
                      <Trash2 className="w-3.5 h-3.5" />
                    </button>
                  </div>
                </div>
              ))}
            </div>
          </div>

          {/* Options de soumission et sécurité */}
          <div className="space-y-4 pt-3 border-t border-white/5">
            <span className="font-semibold text-slate-200 block">Comportement & Anti-Spam Souverain :</span>

            <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
              <label className="block space-y-1">
                <span className="text-slate-400">Message de succès affiché :</span>
                <input
                  type="text"
                  value={builderSuccessMsg}
                  onChange={(e) => setBuilderSuccessMsg(e.target.value)}
                  className="w-full rounded-xl bg-slate-950 border border-white/10 px-3 py-2 text-xs text-white"
                />
              </label>

              <label className="block space-y-1">
                <span className="text-slate-400">Notification par e-mail (optionnelle) :</span>
                <input
                  type="email"
                  value={builderNotifyEmail}
                  onChange={(e) => setBuilderNotifyEmail(e.target.value)}
                  placeholder="contact@mon-site.fr"
                  className="w-full rounded-xl bg-slate-950 border border-white/10 px-3 py-2 text-xs text-white"
                />
              </label>
            </div>

            <label className="flex items-center gap-3 p-3 rounded-xl bg-emerald-500/5 border border-emerald-500/20 cursor-pointer">
              <input
                type="checkbox"
                checked={builderHoneypot}
                onChange={(e) => setBuilderHoneypot(e.target.checked)}
                className="rounded bg-slate-900 border-white/20 text-emerald-500"
              />
              <div>
                <span className="font-semibold text-emerald-300 block">Protection anti-robot Honeypot invisible</span>
                <span className="text-[11px] text-slate-400">
                  Bloque automatiquement les robots de spam sans gêner vos visiteurs avec des captchas fastidieux.
                </span>
              </div>
            </label>
          </div>

          <div className="flex items-center justify-end gap-3 pt-3 border-t border-white/10">
            <button
              type="button"
              onClick={() => {
                setIsCreating(false);
                setEditingForm(null);
              }}
              className="px-4 py-2 text-xs text-slate-400 hover:text-white"
            >
              Annuler
            </button>
            <button
              type="submit"
              className="inline-flex items-center gap-2 rounded-xl bg-indigo-600 hover:bg-indigo-500 px-5 py-2.5 text-xs font-semibold text-white shadow-lg shadow-indigo-600/20"
            >
              <Save className="w-4 h-4" />
              Enregistrer le formulaire
            </button>
          </div>
        </form>
      )}

      {/* Main Tabs */}
      {!isCreating && !editingForm && (
        <div className="space-y-4">
          <div className="flex items-center gap-2 border-b border-white/10">
            <button
              type="button"
              onClick={() => setActiveTab('forms')}
              className={`flex items-center gap-2 px-4 py-2.5 text-xs font-semibold border-b-2 transition-all ${
                activeTab === 'forms'
                  ? 'border-indigo-500 text-indigo-400 bg-white/[0.02]'
                  : 'border-transparent text-slate-400 hover:text-slate-200'
              }`}
            >
              <FileText className="w-4 h-4" />
              <span>Formulaires déclarés ({forms.length})</span>
            </button>

            <button
              type="button"
              onClick={() => setActiveTab('submissions')}
              className={`flex items-center gap-2 px-4 py-2.5 text-xs font-semibold border-b-2 transition-all ${
                activeTab === 'submissions'
                  ? 'border-indigo-500 text-indigo-400 bg-white/[0.02]'
                  : 'border-transparent text-slate-400 hover:text-slate-200'
              }`}
            >
              <Inbox className="w-4 h-4" />
              <span>Boîte de réception des messages</span>
            </button>
          </div>

          {activeTab === 'forms' && (
            <div className="rounded-2xl border border-white/10 bg-[#0f1422] overflow-hidden">
              {loading ? (
                <p className="p-6 text-xs text-slate-400">Chargement des formulaires…</p>
              ) : forms.length === 0 ? (
                <div className="p-8 text-center space-y-3">
                  <FileText className="w-8 h-8 text-slate-500 mx-auto" />
                  <p className="text-xs text-slate-400">Aucun formulaire pour le moment.</p>
                  <button
                    type="button"
                    onClick={startCreate}
                    className="inline-flex items-center gap-1.5 rounded-lg bg-indigo-600 px-3 py-1.5 text-xs font-medium text-white"
                  >
                    <Plus className="w-3.5 h-3.5" />
                    Créer mon premier formulaire
                  </button>
                </div>
              ) : (
                <table className="w-full text-left text-xs border-collapse">
                  <thead>
                    <tr className="border-b border-white/10 text-slate-400 bg-white/[0.02]">
                      <th className="p-4 font-semibold">Nom & Description</th>
                      <th className="p-4 font-semibold">Identifiant (Slug)</th>
                      <th className="p-4 font-semibold">Statut</th>
                      <th className="p-4 font-semibold">Date de création</th>
                      <th className="p-4 font-semibold text-right">Actions</th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-white/5">
                    {forms.map((f) => (
                      <tr key={f.id} className="hover:bg-white/[0.02] text-slate-300">
                        <td className="p-4 font-medium text-white">
                          <div>{f.name}</div>
                          {f.description && <div className="text-[11px] text-slate-400 font-normal">{f.description}</div>}
                        </td>
                        <td className="p-4 font-mono text-[11px] text-indigo-300">
                          <button
                            type="button"
                            onClick={() => {
                              navigator.clipboard.writeText(f.slug);
                              setNotice(`Slug « ${f.slug} » copié dans le presse-papier.`);
                              setTimeout(() => setNotice(null), 3000);
                            }}
                            className="inline-flex items-center gap-1 px-2 py-0.5 rounded bg-black/40 hover:bg-black/60 border border-white/10"
                            title="Copier le slug du formulaire"
                          >
                            <span>{f.slug}</span>
                            <Copy className="w-3 h-3 text-slate-400" />
                          </button>
                        </td>
                        <td className="p-4">
                          {f.is_active ? (
                            <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded bg-emerald-500/15 text-emerald-300 text-[10px] font-semibold border border-emerald-500/30">
                              <CheckCircle className="w-3 h-3" />
                              Actif
                            </span>
                          ) : (
                            <span className="px-2 py-0.5 rounded bg-slate-500/15 text-slate-400 text-[10px] font-semibold">
                              Inactif
                            </span>
                          )}
                        </td>
                        <td className="p-4 text-slate-400 text-[11px]">
                          {new Date(f.created_at).toLocaleDateString('fr-FR', {
                            day: 'numeric',
                            month: 'short',
                            year: 'numeric',
                          })}
                        </td>
                        <td className="p-4 text-right">
                          <div className="flex items-center justify-end gap-2">
                            <button
                              type="button"
                              onClick={() => {
                                setSelectedFormId(f.id);
                                setActiveTab('submissions');
                              }}
                              className="px-2 py-1 rounded border border-white/10 bg-white/5 hover:bg-white/10 text-slate-200 text-[11px]"
                              title="Voir les soumissions de ce formulaire"
                            >
                              Messages
                            </button>
                            <button
                              type="button"
                              onClick={() => startEdit(f)}
                              className="p-1 rounded text-slate-400 hover:text-white hover:bg-white/10"
                              title="Modifier ce formulaire"
                            >
                              <Edit2 className="w-3.5 h-3.5" />
                            </button>
                            <button
                              type="button"
                              onClick={() => handleDeleteForm(f.id, f.name)}
                              className="p-1 rounded text-red-400 hover:text-red-300 hover:bg-white/10"
                              title="Supprimer"
                            >
                              <Trash2 className="w-3.5 h-3.5" />
                            </button>
                          </div>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              )}
            </div>
          )}

          {activeTab === 'submissions' && (
            <div className="space-y-4">
              <div className="flex flex-wrap items-center justify-between gap-3 bg-[#0f1422] border border-white/10 rounded-2xl p-4 text-xs">
                <div className="flex items-center gap-3">
                  <span className="text-slate-400">Filtrer par formulaire :</span>
                  <select
                    value={selectedFormId}
                    onChange={(e) => setSelectedFormId(e.target.value)}
                    className="rounded-xl bg-slate-950 border border-white/10 px-3 py-1.5 text-xs text-white"
                  >
                    {forms.map((f) => (
                      <option key={f.id} value={f.id}>
                        {f.name} ({f.slug})
                      </option>
                    ))}
                  </select>
                </div>

                <div className="flex items-center gap-2">
                  <button
                    type="button"
                    disabled={submissions.length === 0}
                    onClick={exportSubmissionsCsv}
                    className="inline-flex items-center gap-1.5 rounded-lg border border-white/10 bg-white/5 hover:bg-white/10 px-3 py-1.5 text-slate-200 disabled:opacity-40"
                  >
                    <Download className="w-3.5 h-3.5" />
                    Exporter en CSV
                  </button>
                </div>
              </div>

              {/* Tableau des soumissions */}
              <div className="rounded-2xl border border-white/10 bg-[#0f1422] overflow-hidden">
                {loadingSubs ? (
                  <p className="p-6 text-xs text-slate-400">Chargement des soumissions…</p>
                ) : submissions.length === 0 ? (
                  <div className="p-8 text-center space-y-2">
                    <Inbox className="w-8 h-8 text-slate-500 mx-auto" />
                    <p className="text-xs text-slate-400">Aucune soumission reçue pour ce formulaire pour le moment.</p>
                  </div>
                ) : (
                  <table className="w-full text-left text-xs border-collapse">
                    <thead>
                      <tr className="border-b border-white/10 text-slate-400 bg-white/[0.02]">
                        <th className="p-3.5 font-semibold">Date & Heure</th>
                        <th className="p-3.5 font-semibold">Aperçu du contenu</th>
                        <th className="p-3.5 font-semibold">Sécurité</th>
                        <th className="p-3.5 font-semibold text-right">Détails</th>
                      </tr>
                    </thead>
                    <tbody className="divide-y divide-white/5">
                      {submissions.map((sub) => {
                        const summary = Object.entries(sub.data || {})
                          .map(([k, v]) => `${k}: ${String(v)}`)
                          .join(' • ');
                        return (
                          <tr
                            key={sub.id}
                            className={`hover:bg-white/[0.02] ${!sub.is_read ? 'bg-indigo-950/20 font-medium' : 'text-slate-400'}`}
                          >
                            <td className="p-3.5 text-slate-300 shrink-0">
                              <div className="flex items-center gap-2">
                                {!sub.is_read && <span className="w-2 h-2 rounded-full bg-indigo-400" />}
                                <span>
                                  {new Date(sub.created_at).toLocaleDateString('fr-FR', {
                                    day: 'numeric',
                                    month: 'short',
                                    hour: '2-digit',
                                    minute: '2-digit',
                                  })}
                                </span>
                              </div>
                            </td>
                            <td className="p-3.5 max-w-md truncate text-white">
                              {summary || '(Aucune donnée)'}
                            </td>
                            <td className="p-3.5">
                              {sub.is_spam ? (
                                <span className="px-2 py-0.5 rounded bg-red-500/15 text-red-300 text-[10px] font-semibold border border-red-500/30">
                                  Spam (Honeypot)
                                </span>
                              ) : (
                                <span className="inline-flex items-center gap-1 text-[10px] text-emerald-400">
                                  <ShieldCheck className="w-3.5 h-3.5" />
                                  Vérifié
                                </span>
                              )}
                            </td>
                            <td className="p-3.5 text-right">
                              <button
                                type="button"
                                onClick={() => {
                                  setInspectingSub(sub);
                                  if (!sub.is_read) void handleMarkRead(sub);
                                }}
                                className="inline-flex items-center gap-1 rounded px-2.5 py-1 bg-white/5 hover:bg-white/10 text-slate-200 text-[11px]"
                              >
                                <Eye className="w-3.5 h-3.5" />
                                Consulter
                              </button>
                            </td>
                          </tr>
                        );
                      })}
                    </tbody>
                  </table>
                )}
              </div>
            </div>
          )}
        </div>
      )}

      {/* Inspecting Modal */}
      {inspectingSub && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/70 backdrop-blur-sm animate-fade-in">
          <div className="w-full max-w-lg rounded-2xl border border-white/10 bg-[#0f1422] p-6 space-y-4 shadow-2xl text-xs">
            <div className="flex items-center justify-between border-b border-white/10 pb-3">
              <h3 className="font-bold text-white text-sm flex items-center gap-2">
                <Mail className="w-4 h-4 text-indigo-400" />
                Détail de la soumission
              </h3>
              <button
                type="button"
                onClick={() => setInspectingSub(null)}
                className="text-slate-400 hover:text-white p-1"
              >
                ✕
              </button>
            </div>

            <div className="space-y-1 text-slate-400 text-[11px]">
              <div>Reçu le : {new Date(inspectingSub.created_at).toLocaleString('fr-FR')}</div>
              <div>Empreinte IP : <span className="font-mono">{inspectingSub.ip_hash}</span></div>
            </div>

            <div className="space-y-2 rounded-xl bg-black/40 border border-white/5 p-4">
              {Object.entries(inspectingSub.data || {}).map(([key, val]) => (
                <div key={key} className="space-y-0.5">
                  <span className="font-semibold text-slate-300 uppercase text-[10px] tracking-wider">{key} :</span>
                  <div className="text-white text-xs whitespace-pre-wrap bg-white/5 rounded p-2">
                    {String(val)}
                  </div>
                </div>
              ))}
            </div>

            <div className="flex justify-end pt-2">
              <button
                type="button"
                onClick={() => setInspectingSub(null)}
                className="rounded-xl bg-indigo-600 hover:bg-indigo-500 px-4 py-2 text-xs font-semibold text-white"
              >
                Fermer
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
