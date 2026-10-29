import React, { useState, useEffect, useCallback } from 'react';
import {
  GraduationCap,
  BookOpen,
  Users,
  Settings,
  Plus,
  Trash2,
  Edit2,
  CheckCircle,
  AlertCircle,
  RotateCcw,
  Search,
  Video,
  Clock,
  Eye,
  Layers,
  FileText,
  UserCheck,
} from 'lucide-react';
import { api } from '../api';
import type {
  CourseItem,
  CourseInput,
  CourseModuleItem,
  LessonItem,
  LessonInput,
  EnrollmentDetails,
  ProductItem,
} from '../types';

interface LmsViewProps {
  initialTab?: 'courses' | 'curriculum' | 'learners' | 'settings';
}

export function LmsView({ initialTab = 'courses' }: LmsViewProps) {
  const [activeTab, setActiveTab] = useState<'courses' | 'curriculum' | 'learners' | 'settings'>(initialTab);

  // Notifications
  const [notice, setNotice] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  // Cours
  const [courses, setCourses] = useState<CourseItem[]>([]);
  const [courseSearch, setCourseSearch] = useState('');
  const [editingCourse, setEditingCourse] = useState<CourseItem | null>(null);
  const [isCourseModalOpen, setIsCourseModalOpen] = useState(false);
  const [courseForm, setCourseForm] = useState<CourseInput>({
    title: '',
    slug: '',
    description: '',
    content: '',
    price_cents: 0,
    is_published: true,
  });

  // Curriculum & Leçons
  const [selectedCourseId, setSelectedCourseId] = useState<string>('');
  const [modules, setModules] = useState<CourseModuleItem[]>([]);
  const [lessons, setLessons] = useState<LessonItem[]>([]);
  const [isModuleModalOpen, setIsModuleModalOpen] = useState(false);
  const [moduleTitle, setModuleTitle] = useState('');
  const [moduleDesc, setModuleDesc] = useState('');
  const [isLessonModalOpen, setIsLessonModalOpen] = useState(false);
  const [editingLesson, setEditingLesson] = useState<LessonItem | null>(null);
  const [lessonForm, setLessonForm] = useState<LessonInput>({
    module_id: null,
    title: '',
    slug: '',
    content: '',
    video_url: '',
    duration_min: 15,
    is_preview: false,
    sort_order: 0,
  });

  // Apprenants & Inscriptions
  const [enrollments, setEnrollments] = useState<EnrollmentDetails[]>([]);
  const [isEnrollModalOpen, setIsEnrollModalOpen] = useState(false);
  const [enrollUserId, setEnrollUserId] = useState('');
  const [enrollCourseId, setEnrollCourseId] = useState('');

  // Produits liés
  const [products, setProducts] = useState<ProductItem[]>([]);

  // Chargement des données
  const loadCourses = useCallback(async () => {
    try {
      setLoading(true);
      const data = await api.lms.courses.list();
      setCourses(data);
      if (data.length > 0 && !selectedCourseId) {
        setSelectedCourseId(data[0].id);
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Erreur chargement cours');
    } finally {
      setLoading(false);
    }
  }, [selectedCourseId]);

  const loadCurriculum = useCallback(async (courseId: string) => {
    if (!courseId) return;
    try {
      setLoading(true);
      const [m, l] = await Promise.all([
        api.lms.modules.list(courseId),
        api.lms.lessons.list(courseId).catch(() => []),
      ]);
      setModules(m);
      setLessons(l);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Erreur chargement programme');
    } finally {
      setLoading(false);
    }
  }, []);

  const loadEnrollments = useCallback(async () => {
    try {
      setLoading(true);
      const data = await api.lms.enrollments.listAll();
      setEnrollments(data);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Erreur chargement apprenants');
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    let ignore = false;
    async function init() {
      try {
        if (activeTab === 'courses') {
          const data = await api.lms.courses.list();
          if (!ignore) {
            setCourses(data);
            if (data.length > 0 && !selectedCourseId) setSelectedCourseId(data[0].id);
          }
          const prods = await api.ecommerce.products.list().catch(() => []);
          if (!ignore) setProducts(prods);
        } else if (activeTab === 'curriculum') {
          if (courses.length === 0) {
            const data = await api.lms.courses.list();
            if (!ignore) {
              setCourses(data);
              const cid = selectedCourseId || (data.length > 0 ? data[0].id : '');
              setSelectedCourseId(cid);
              if (cid) await loadCurriculum(cid);
            }
          } else if (selectedCourseId) {
            await loadCurriculum(selectedCourseId);
          }
        } else if (activeTab === 'learners') {
          const [data, courseList] = await Promise.all([
            api.lms.enrollments.listAll().catch(() => []),
            api.lms.courses.list().catch(() => []),
          ]);
          if (!ignore) {
            setEnrollments(data);
            setCourses(courseList);
            if (courseList.length > 0 && !enrollCourseId) setEnrollCourseId(courseList[0].id);
          }
        }
      } catch (e) {
        if (!ignore) setError(e instanceof Error ? e.message : 'Erreur initialisation');
      }
    }
    void init();
    return () => {
      ignore = true;
    };
  }, [activeTab, selectedCourseId, loadCurriculum, courses.length, enrollCourseId]);

  // Actions Cours
  function openCreateCourse() {
    setEditingCourse(null);
    setCourseForm({
      title: '',
      slug: '',
      description: '',
      content: '',
      price_cents: 0,
      is_published: true,
      product_id: null,
    });
    setIsCourseModalOpen(true);
  }

  function openEditCourse(c: CourseItem) {
    setEditingCourse(c);
    setCourseForm({
      title: c.title,
      slug: c.slug,
      description: c.description,
      content: c.content,
      price_cents: c.price_cents,
      is_published: c.is_published,
      product_id: c.product_id || null,
    });
    setIsCourseModalOpen(true);
  }

  async function handleSaveCourse(e: React.FormEvent) {
    e.preventDefault();
    setError(null);
    try {
      if (editingCourse) {
        await api.lms.courses.update(editingCourse.id, courseForm);
        setNotice(`Formation « ${courseForm.title} » mise à jour.`);
      } else {
        await api.lms.courses.create(courseForm);
        setNotice(`Formation « ${courseForm.title} » créée avec succès.`);
      }
      setIsCourseModalOpen(false);
      await loadCourses();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Erreur enregistrement formation');
    }
  }

  async function handleDeleteCourse(id: string, title: string) {
    if (!window.confirm(`Supprimer définitivement la formation « ${title} » et ses leçons ?`)) return;
    try {
      await api.lms.courses.delete(id);
      setNotice(`Formation « ${title} » supprimée.`);
      await loadCourses();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Erreur suppression formation');
    }
  }

  // Actions Curriculum / Modules
  async function handleCreateModule(e: React.FormEvent) {
    e.preventDefault();
    if (!selectedCourseId) return;
    try {
      await api.lms.modules.create(selectedCourseId, {
        title: moduleTitle,
        description: moduleDesc,
        sort_order: modules.length + 1,
      });
      setNotice(`Module « ${moduleTitle} » ajouté.`);
      setIsModuleModalOpen(false);
      setModuleTitle('');
      setModuleDesc('');
      await loadCurriculum(selectedCourseId);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Erreur création module');
    }
  }

  async function handleDeleteModule(modId: string, title: string) {
    if (!window.confirm(`Supprimer la section « ${title} » ?`)) return;
    try {
      await api.lms.modules.delete(selectedCourseId, modId);
      setNotice(`Section « ${title} » supprimée.`);
      await loadCurriculum(selectedCourseId);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Erreur suppression section');
    }
  }

  // Actions Leçons
  function openCreateLesson(moduleId?: string) {
    setEditingLesson(null);
    setLessonForm({
      module_id: moduleId || (modules.length > 0 ? modules[0].id : null),
      title: '',
      slug: '',
      content: '',
      video_url: '',
      duration_min: 15,
      is_preview: false,
      sort_order: lessons.length + 1,
    });
    setIsLessonModalOpen(true);
  }

  function openEditLesson(l: LessonItem) {
    setEditingLesson(l);
    setLessonForm({
      module_id: l.module_id,
      title: l.title,
      slug: l.slug,
      content: l.content,
      video_url: l.video_url || '',
      duration_min: l.duration_min,
      is_preview: l.is_preview,
      sort_order: l.sort_order,
    });
    setIsLessonModalOpen(true);
  }

  async function handleSaveLesson(e: React.FormEvent) {
    e.preventDefault();
    if (!selectedCourseId) return;
    try {
      if (editingLesson) {
        await api.lms.lessons.update(selectedCourseId, editingLesson.id, lessonForm);
        setNotice(`Leçon « ${lessonForm.title} » mise à jour.`);
      } else {
        await api.lms.lessons.create(selectedCourseId, lessonForm);
        setNotice(`Leçon « ${lessonForm.title} » créée.`);
      }
      setIsLessonModalOpen(false);
      await loadCurriculum(selectedCourseId);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Erreur enregistrement leçon');
    }
  }

  async function handleDeleteLesson(lessonId: string, title: string) {
    if (!window.confirm(`Supprimer la leçon « ${title} » ?`)) return;
    try {
      await api.lms.lessons.delete(selectedCourseId, lessonId);
      setNotice(`Leçon « ${title} » supprimée.`);
      await loadCurriculum(selectedCourseId);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Erreur suppression leçon');
    }
  }

  // Actions Inscriptions
  async function handleEnrollUser(e: React.FormEvent) {
    e.preventDefault();
    if (!enrollCourseId || !enrollUserId) return;
    try {
      await api.lms.enrollments.enroll(enrollCourseId, enrollUserId);
      setNotice('Apprenant inscrit avec succès à la formation.');
      setIsEnrollModalOpen(false);
      setEnrollUserId('');
      await loadEnrollments();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Erreur inscription apprenant');
    }
  }

  const filteredCourses = courses.filter((c) =>
    c.title.toLowerCase().includes(courseSearch.toLowerCase()) ||
    c.slug.toLowerCase().includes(courseSearch.toLowerCase())
  );

  return (
    <div className="space-y-6 max-w-6xl mx-auto pb-16">
      {/* En-tête du module LMS */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 pb-4 border-b border-white/10">
        <div>
          <h1 className="text-2xl font-bold tracking-tight text-white flex items-center gap-2">
            <GraduationCap className="w-6 h-6 text-emerald-400" />
            Formations & E-Learning (LMS)
          </h1>
          <p className="text-sm text-slate-400 mt-1">
            Gestion pédagogique souveraine : cours, chapitres, leçons vidéo, suivi des apprenants et passerelle d'accès automatique.
          </p>
        </div>
        <div className="flex items-center gap-2">
          {activeTab === 'courses' && (
            <button
              onClick={openCreateCourse}
              className="inline-flex items-center gap-2 rounded-xl bg-emerald-600 px-4 py-2 text-xs font-semibold text-white shadow-md hover:bg-emerald-500 transition-colors"
            >
              <Plus className="w-4 h-4" />
              Nouveau cours
            </button>
          )}
          {activeTab === 'curriculum' && selectedCourseId && (
            <>
              <button
                onClick={() => setIsModuleModalOpen(true)}
                className="inline-flex items-center gap-1.5 rounded-xl bg-white/10 px-3.5 py-2 text-xs font-semibold text-slate-200 hover:bg-white/15 transition-colors border border-white/10"
              >
                <Layers className="w-4 h-4" />
                Ajouter une section
              </button>
              <button
                onClick={() => openCreateLesson()}
                className="inline-flex items-center gap-1.5 rounded-xl bg-emerald-600 px-4 py-2 text-xs font-semibold text-white shadow-md hover:bg-emerald-500 transition-colors"
              >
                <Plus className="w-4 h-4" />
                Nouvelle leçon
              </button>
            </>
          )}
          {activeTab === 'learners' && (
            <button
              onClick={() => setIsEnrollModalOpen(true)}
              className="inline-flex items-center gap-2 rounded-xl bg-emerald-600 px-4 py-2 text-xs font-semibold text-white shadow-md hover:bg-emerald-500 transition-colors"
            >
              <UserCheck className="w-4 h-4" />
              Inscrire un apprenant
            </button>
          )}
        </div>
      </div>

      {/* Notifications */}
      {notice && (
        <div role="status" className="p-3.5 rounded-xl bg-emerald-500/10 border border-emerald-500/20 text-emerald-300 text-xs flex items-center justify-between">
          <span className="flex items-center gap-2">
            <CheckCircle className="w-4 h-4 shrink-0" />
            {notice}
          </span>
          <button onClick={() => setNotice(null)} className="text-emerald-400 hover:text-white">✕</button>
        </div>
      )}
      {error && (
        <div role="alert" className="p-3.5 rounded-xl bg-red-500/10 border border-red-500/20 text-red-300 text-xs flex items-center justify-between">
          <span className="flex items-center gap-2">
            <AlertCircle className="w-4 h-4 shrink-0" />
            {error}
          </span>
          <button onClick={() => setError(null)} className="text-red-400 hover:text-white">✕</button>
        </div>
      )}

      {/* Onglets de navigation */}
      <div className="flex border-b border-white/10 gap-2">
        <button
          onClick={() => setActiveTab('courses')}
          className={`flex items-center gap-2 px-4 py-2.5 text-xs font-medium border-b-2 transition-colors ${
            activeTab === 'courses'
              ? 'border-emerald-500 text-white font-semibold'
              : 'border-transparent text-slate-400 hover:text-slate-200'
          }`}
        >
          <BookOpen className="w-4 h-4" />
          Cours & Formations ({courses.length})
        </button>
        <button
          onClick={() => setActiveTab('curriculum')}
          className={`flex items-center gap-2 px-4 py-2.5 text-xs font-medium border-b-2 transition-colors ${
            activeTab === 'curriculum'
              ? 'border-emerald-500 text-white font-semibold'
              : 'border-transparent text-slate-400 hover:text-slate-200'
          }`}
        >
          <Layers className="w-4 h-4" />
          Programme & Leçons
        </button>
        <button
          onClick={() => setActiveTab('learners')}
          className={`flex items-center gap-2 px-4 py-2.5 text-xs font-medium border-b-2 transition-colors ${
            activeTab === 'learners'
              ? 'border-emerald-500 text-white font-semibold'
              : 'border-transparent text-slate-400 hover:text-slate-200'
          }`}
        >
          <Users className="w-4 h-4" />
          Apprenants & Progression ({enrollments.length})
        </button>
        <button
          onClick={() => setActiveTab('settings')}
          className={`flex items-center gap-2 px-4 py-2.5 text-xs font-medium border-b-2 transition-colors ${
            activeTab === 'settings'
              ? 'border-emerald-500 text-white font-semibold'
              : 'border-transparent text-slate-400 hover:text-slate-200'
          }`}
        >
          <Settings className="w-4 h-4" />
          Réglages de formation
        </button>
      </div>

      {/* CONTENU ONGLET 1 : COURS */}
      {activeTab === 'courses' && (
        <div className="space-y-4">
          <div className="flex items-center justify-between gap-3">
            <div className="relative w-full sm:w-72">
              <Search className="w-4 h-4 text-slate-400 absolute left-3 top-2.5" />
              <input
                type="text"
                value={courseSearch}
                onChange={(e) => setCourseSearch(e.target.value)}
                placeholder="Rechercher une formation..."
                className="w-full bg-[#121927] border border-white/10 rounded-xl pl-9 pr-3 py-1.5 text-xs text-white placeholder-slate-500 focus:outline-none focus:border-emerald-500"
              />
            </div>
            <button
              onClick={loadCourses}
              className="text-xs text-slate-400 hover:text-white flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-white/5 border border-white/10"
            >
              <RotateCcw className="w-3.5 h-3.5" /> Actualiser
            </button>
          </div>

          {loading && courses.length === 0 ? (
            <p className="text-xs text-slate-500 py-8 text-center">Chargement des cours…</p>
          ) : filteredCourses.length === 0 ? (
            <div className="p-12 text-center border border-dashed border-white/10 rounded-2xl bg-[#0e1422]">
              <GraduationCap className="w-10 h-10 text-slate-600 mx-auto mb-3" />
              <p className="text-sm font-semibold text-slate-300">Aucune formation créée</p>
              <p className="text-xs text-slate-500 mt-1">Créez votre première formation en ligne structurée.</p>
              <button
                onClick={openCreateCourse}
                className="mt-4 inline-flex items-center gap-2 rounded-xl bg-emerald-600 px-4 py-2 text-xs font-semibold text-white shadow-md hover:bg-emerald-500"
              >
                <Plus className="w-4 h-4" />
                Créer un cours
              </button>
            </div>
          ) : (
            <div className="overflow-x-auto border border-white/10 rounded-2xl bg-[#0e1422]">
              <table className="w-full text-left text-xs text-slate-300">
                <thead className="bg-[#141d30] text-slate-400 font-semibold border-b border-white/10">
                  <tr>
                    <th className="px-4 py-3">Formation</th>
                    <th className="px-4 py-3">Tarification</th>
                    <th className="px-4 py-3">Produit lié</th>
                    <th className="px-4 py-3">Statut</th>
                    <th className="px-4 py-3">Créée le</th>
                    <th className="px-4 py-3 text-right">Actions</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-white/5">
                  {filteredCourses.map((c) => (
                    <tr key={c.id} className="hover:bg-white/[0.02] transition-colors">
                      <td className="px-4 py-3.5">
                        <div className="font-semibold text-white">{c.title}</div>
                        <div className="text-[11px] text-slate-500 font-mono mt-0.5">/{c.slug}</div>
                      </td>
                      <td className="px-4 py-3.5 font-mono">
                        {c.price_cents > 0 ? (
                          <span className="font-bold text-emerald-400">{(c.price_cents / 100).toFixed(2)} €</span>
                        ) : (
                          <span className="text-slate-400">Gratuit / Inclus</span>
                        )}
                      </td>
                      <td className="px-4 py-3.5 text-slate-400 text-[11px]">
                        {c.product_id ? (
                          <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded bg-indigo-500/10 text-indigo-300 border border-indigo-500/20">
                            Produit lié
                          </span>
                        ) : (
                          <span className="text-slate-500">Aucun</span>
                        )}
                      </td>
                      <td className="px-4 py-3.5">
                        <span
                          className={`inline-block px-2 py-0.5 rounded-full text-[10px] font-semibold uppercase ${
                            c.is_published
                              ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20'
                              : 'bg-amber-500/10 text-amber-400 border border-amber-500/20'
                          }`}
                        >
                          {c.is_published ? 'Publié' : 'Brouillon'}
                        </span>
                      </td>
                      <td className="px-4 py-3.5 text-slate-400 text-[11px]">
                        {new Date(c.created_at).toLocaleDateString('fr-FR')}
                      </td>
                      <td className="px-4 py-3.5 text-right space-x-2">
                        <button
                          onClick={() => {
                            setSelectedCourseId(c.id);
                            setActiveTab('curriculum');
                          }}
                          className="inline-flex items-center gap-1 px-2.5 py-1 rounded-lg bg-emerald-600/20 text-emerald-300 border border-emerald-500/30 hover:bg-emerald-600/30 text-xs font-semibold"
                          title="Gérer les chapitres et leçons"
                        >
                          <Layers className="w-3.5 h-3.5" /> Programme
                        </button>
                        <button
                          onClick={() => openEditCourse(c)}
                          className="p-1.5 rounded-lg text-slate-400 hover:text-white hover:bg-white/5"
                          title="Modifier"
                        >
                          <Edit2 className="w-4 h-4" />
                        </button>
                        <button
                          onClick={() => handleDeleteCourse(c.id, c.title)}
                          className="p-1.5 rounded-lg text-red-400 hover:text-red-300 hover:bg-red-500/10"
                          title="Supprimer"
                        >
                          <Trash2 className="w-4 h-4" />
                        </button>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </div>
      )}

      {/* CONTENU ONGLET 2 : PROGRAMME & LEÇONS */}
      {activeTab === 'curriculum' && (
        <div className="space-y-6">
          {/* Sélecteur de cours actif */}
          <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 p-4 bg-[#0e1422] border border-white/10 rounded-2xl">
            <div className="flex items-center gap-3">
              <span className="text-xs font-semibold text-slate-400">Formation active :</span>
              <select
                value={selectedCourseId}
                onChange={(e) => {
                  setSelectedCourseId(e.target.value);
                  void loadCurriculum(e.target.value);
                }}
                className="bg-[#121927] border border-white/10 rounded-xl px-3 py-1.5 text-xs text-white focus:outline-none focus:border-emerald-500"
              >
                {courses.map((c) => (
                  <option key={c.id} value={c.id}>
                    {c.title}
                  </option>
                ))}
              </select>
            </div>
            <div className="text-xs text-slate-500">
              {modules.length} section(s) • {lessons.length} leçon(s)
            </div>
          </div>

          {modules.length === 0 && lessons.length === 0 ? (
            <div className="p-12 text-center border border-dashed border-white/10 rounded-2xl bg-[#0e1422]">
              <Layers className="w-10 h-10 text-slate-600 mx-auto mb-3" />
              <p className="text-sm font-semibold text-slate-300">Programme vide</p>
              <p className="text-xs text-slate-500 mt-1">Ajoutez des sections (modules) et des leçons vidéo ou texte.</p>
              <div className="mt-4 flex justify-center gap-3">
                <button
                  onClick={() => setIsModuleModalOpen(true)}
                  className="inline-flex items-center gap-2 rounded-xl bg-white/10 px-4 py-2 text-xs font-semibold text-white border border-white/10 hover:bg-white/15"
                >
                  <Plus className="w-4 h-4" /> Ajouter une section
                </button>
                <button
                  onClick={() => openCreateLesson()}
                  className="inline-flex items-center gap-2 rounded-xl bg-emerald-600 px-4 py-2 text-xs font-semibold text-white shadow-md hover:bg-emerald-500"
                >
                  <Plus className="w-4 h-4" /> Créer une leçon
                </button>
              </div>
            </div>
          ) : (
            <div className="space-y-4">
              {/* Sections (Modules) */}
              {modules.map((mod, modIdx) => {
                const modLessons = lessons.filter((l) => l.module_id === mod.id);
                return (
                  <div key={mod.id} className="border border-white/10 rounded-2xl bg-[#0e1422] overflow-hidden">
                    <div className="p-4 bg-[#141d30] border-b border-white/10 flex items-center justify-between">
                      <div className="flex items-center gap-2.5">
                        <span className="font-mono text-xs px-2 py-0.5 rounded bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 font-bold">
                          Section {modIdx + 1}
                        </span>
                        <h3 className="font-semibold text-white text-sm">{mod.title}</h3>
                        {mod.description && <span className="text-xs text-slate-400">({mod.description})</span>}
                      </div>
                      <div className="flex items-center gap-2">
                        <button
                          onClick={() => openCreateLesson(mod.id)}
                          className="px-2.5 py-1 rounded-lg bg-emerald-600/20 text-emerald-300 border border-emerald-500/30 hover:bg-emerald-600/30 text-xs font-semibold flex items-center gap-1"
                        >
                          <Plus className="w-3.5 h-3.5" /> Leçon
                        </button>
                        <button
                          onClick={() => handleDeleteModule(mod.id, mod.title)}
                          className="p-1 rounded-lg text-slate-500 hover:text-red-400"
                          title="Supprimer la section"
                        >
                          <Trash2 className="w-3.5 h-3.5" />
                        </button>
                      </div>
                    </div>

                    <div className="divide-y divide-white/5">
                      {modLessons.length === 0 ? (
                        <p className="p-4 text-xs text-slate-500 italic">Aucune leçon dans cette section.</p>
                      ) : (
                        modLessons.map((les, lesIdx) => (
                          <div key={les.id} className="p-3.5 flex items-center justify-between hover:bg-white/[0.02]">
                            <div className="flex items-center gap-3">
                              <span className="text-xs font-mono text-slate-500">{lesIdx + 1}.</span>
                              <div>
                                <div className="flex items-center gap-2">
                                  <span className="font-semibold text-white text-xs">{les.title}</span>
                                  {les.is_preview && (
                                    <span className="px-2 py-0.2 rounded text-[10px] font-semibold bg-indigo-500/10 text-indigo-300 border border-indigo-500/20">
                                      Aperçu gratuit
                                    </span>
                                  )}
                                  {les.video_url && (
                                    <span className="inline-flex items-center gap-1 text-[11px] text-slate-400">
                                      <Video className="w-3 h-3 text-emerald-400" /> Vidéo
                                    </span>
                                  )}
                                </div>
                                <span className="text-[11px] text-slate-500 flex items-center gap-1 mt-0.5">
                                  <Clock className="w-3 h-3" /> {les.duration_min} min • /{les.slug}
                                </span>
                              </div>
                            </div>
                            <div className="flex items-center gap-2">
                              <button
                                onClick={() => openEditLesson(les)}
                                className="p-1.5 rounded-lg text-slate-400 hover:text-white hover:bg-white/5"
                                title="Modifier"
                              >
                                <Edit2 className="w-3.5 h-3.5" />
                              </button>
                              <button
                                onClick={() => handleDeleteLesson(les.id, les.title)}
                                className="p-1.5 rounded-lg text-red-400 hover:text-red-300 hover:bg-red-500/10"
                                title="Supprimer"
                              >
                                <Trash2 className="w-3.5 h-3.5" />
                              </button>
                            </div>
                          </div>
                        ))
                      )}
                    </div>
                  </div>
                );
              })}

              {/* Leçons sans module assigné */}
              {lessons.filter((l) => !l.module_id).length > 0 && (
                <div className="border border-white/10 rounded-2xl bg-[#0e1422] overflow-hidden">
                  <div className="p-3.5 bg-[#141d30] border-b border-white/10 font-semibold text-xs text-slate-400">
                    Leçons hors section
                  </div>
                  <div className="divide-y divide-white/5">
                    {lessons
                      .filter((l) => !l.module_id)
                      .map((les) => (
                        <div key={les.id} className="p-3.5 flex items-center justify-between hover:bg-white/[0.02]">
                          <div>
                            <span className="font-semibold text-white text-xs">{les.title}</span>
                            <span className="text-[11px] text-slate-500 block">{les.duration_min} min</span>
                          </div>
                          <div className="flex items-center gap-2">
                            <button
                              onClick={() => openEditLesson(les)}
                              className="p-1.5 rounded-lg text-slate-400 hover:text-white hover:bg-white/5"
                            >
                              <Edit2 className="w-3.5 h-3.5" />
                            </button>
                            <button
                              onClick={() => handleDeleteLesson(les.id, les.title)}
                              className="p-1.5 rounded-lg text-red-400 hover:text-red-300 hover:bg-red-500/10"
                            >
                              <Trash2 className="w-3.5 h-3.5" />
                            </button>
                          </div>
                        </div>
                      ))}
                  </div>
                </div>
              )}
            </div>
          )}
        </div>
      )}

      {/* CONTENU ONGLET 3 : APPRENANTS & PROGRESSION */}
      {activeTab === 'learners' && (
        <div className="space-y-4">
          <div className="flex items-center justify-between gap-3">
            <h3 className="text-sm font-semibold text-white">Suivi des étudiants inscrits</h3>
            <button
              onClick={loadEnrollments}
              className="text-xs text-slate-400 hover:text-white flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-white/5 border border-white/10"
            >
              <RotateCcw className="w-3.5 h-3.5" /> Actualiser
            </button>
          </div>

          {loading && enrollments.length === 0 ? (
            <p className="text-xs text-slate-500 py-8 text-center">Chargement des apprenants…</p>
          ) : enrollments.length === 0 ? (
            <div className="p-12 text-center border border-dashed border-white/10 rounded-2xl bg-[#0e1422]">
              <Users className="w-10 h-10 text-slate-600 mx-auto mb-3" />
              <p className="text-sm font-semibold text-slate-300">Aucun apprenant pour l'instant</p>
              <p className="text-xs text-slate-500 mt-1">Les inscriptions manuelles ou automatiques après achat de commande s'afficheront ici.</p>
              <button
                onClick={() => setIsEnrollModalOpen(true)}
                className="mt-4 inline-flex items-center gap-2 rounded-xl bg-emerald-600 px-4 py-2 text-xs font-semibold text-white shadow-md hover:bg-emerald-500"
              >
                <UserCheck className="w-4 h-4" /> Inscrire un utilisateur
              </button>
            </div>
          ) : (
            <div className="overflow-x-auto border border-white/10 rounded-2xl bg-[#0e1422]">
              <table className="w-full text-left text-xs text-slate-300">
                <thead className="bg-[#141d30] text-slate-400 font-semibold border-b border-white/10">
                  <tr>
                    <th className="px-4 py-3">Apprenant</th>
                    <th className="px-4 py-3">Formation</th>
                    <th className="px-4 py-3">Progression</th>
                    <th className="px-4 py-3">Statut</th>
                    <th className="px-4 py-3">Inscrit le</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-white/5">
                  {enrollments.map((enr) => {
                    const c = courses.find((item) => item.id === enr.course_id);
                    return (
                      <tr key={enr.id} className="hover:bg-white/[0.02] transition-colors">
                        <td className="px-4 py-3.5">
                          <div className="font-semibold text-white">{enr.display_name || enr.username}</div>
                          <div className="text-[11px] text-slate-500 font-mono mt-0.5">{enr.email}</div>
                        </td>
                        <td className="px-4 py-3.5 font-medium text-slate-200">
                          {c?.title || enr.course_id.slice(0, 8)}
                        </td>
                        <td className="px-4 py-3.5">
                          <div className="flex items-center gap-2.5">
                            <div className="w-24 h-2 bg-slate-800 rounded-full overflow-hidden border border-white/10">
                              <div
                                className="h-full bg-emerald-500 rounded-full transition-all duration-300"
                                style={{ width: `${enr.progress_pct}%` }}
                              />
                            </div>
                            <span className="font-mono text-[11px] font-bold text-emerald-400">
                              {enr.progress_pct}%
                            </span>
                          </div>
                        </td>
                        <td className="px-4 py-3.5">
                          {enr.progress_pct >= 100 ? (
                            <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-semibold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
                              <CheckCircle className="w-3 h-3" /> Terminé
                            </span>
                          ) : (
                            <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-semibold bg-amber-500/10 text-amber-400 border border-amber-500/20">
                              En cours
                            </span>
                          )}
                        </td>
                        <td className="px-4 py-3.5 text-slate-400 text-[11px]">
                          {new Date(enr.created_at).toLocaleDateString('fr-FR')}
                        </td>
                      </tr>
                    );
                  })}
                </tbody>
              </table>
            </div>
          )}
        </div>
      )}

      {/* CONTENU ONGLET 4 : RÉGLAGES FORMATION */}
      {activeTab === 'settings' && (
        <div className="space-y-6 max-w-2xl bg-[#0e1422] p-6 border border-white/10 rounded-2xl">
          <div>
            <h3 className="text-base font-semibold text-white">Politique de formation et passerelle</h3>
            <p className="text-xs text-slate-400 mt-0.5">Règles d'accès pédagogique et couplage transactionnel.</p>
          </div>

          <div className="space-y-4 text-xs text-slate-300 leading-relaxed">
            <div className="p-4 rounded-xl bg-white/[0.02] border border-white/10 space-y-2">
              <h4 className="font-bold text-white flex items-center gap-2">
                <CheckCircle className="w-4 h-4 text-emerald-400" />
                Liaison bidirectionnelle E-Commerce ➔ LMS active
              </h4>
              <p className="text-slate-400">
                Lorsqu'un produit numérique de la boutique est associé à un cours, tout acheteur validé (paiement Stripe, virement ou test) est automatiquement inscrit à la formation et un compte apprenant lui est créé ou lié sans friction.
              </p>
            </div>

            <div className="p-4 rounded-xl bg-white/[0.02] border border-white/10 space-y-2">
              <h4 className="font-bold text-white flex items-center gap-2">
                <Eye className="w-4 h-4 text-indigo-400" />
                Mode Aperçu gratuit pour le marketing
              </h4>
              <p className="text-slate-400">
                Toutes les leçons marquées avec l'option « Aperçu gratuit » sont consultables librement par les visiteurs sans inscription, permettant de découvrir le style pédagogique et la qualité vidéo avant d'acheter le cours complet.
              </p>
            </div>

            <div className="p-4 rounded-xl bg-white/[0.02] border border-white/10 space-y-2">
              <h4 className="font-bold text-white flex items-center gap-2">
                <Clock className="w-4 h-4 text-amber-400" />
                Suivi de complétion souverain
              </h4>
              <p className="text-slate-400">
                La progression des apprenants est calculée de manière atomique au niveau du serveur Rust et enregistrée avec horodatage de complétion pour la délivrance des attestations et certificats.
              </p>
            </div>
          </div>
        </div>
      )}

      {/* MODAL CRÉATION / ÉDITION COURS */}
      {isCourseModalOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/70 backdrop-blur-sm">
          <div className="w-full max-w-xl bg-[#0e1422] border border-white/10 rounded-2xl shadow-2xl p-6 space-y-4 max-h-[90vh] overflow-y-auto">
            <div className="flex items-center justify-between pb-3 border-b border-white/10">
              <h3 className="text-base font-bold text-white flex items-center gap-2">
                <GraduationCap className="w-5 h-5 text-emerald-400" />
                {editingCourse ? 'Modifier le cours' : 'Créer une formation'}
              </h3>
              <button onClick={() => setIsCourseModalOpen(false)} className="text-slate-400 hover:text-white p-1">
                ✕
              </button>
            </div>

            <form onSubmit={handleSaveCourse} className="space-y-4">
              <div>
                <label className="block text-xs font-semibold text-slate-300 mb-1">Titre de la formation *</label>
                <input
                  type="text"
                  required
                  value={courseForm.title}
                  onChange={(e) => setCourseForm({ ...courseForm, title: e.target.value })}
                  placeholder="Ex: Rust & Axum : Du débutant à la production"
                  className="w-full bg-[#121927] border border-white/10 rounded-xl px-3 py-2 text-xs text-white focus:outline-none focus:border-emerald-500"
                />
              </div>

              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="block text-xs font-semibold text-slate-300 mb-1">Identifiant d’URL (Slug)</label>
                  <input
                    type="text"
                    value={courseForm.slug || ''}
                    onChange={(e) => setCourseForm({ ...courseForm, slug: e.target.value })}
                    placeholder="laisser vide pour auto"
                    className="w-full bg-[#121927] border border-white/10 rounded-xl px-3 py-2 text-xs text-white font-mono focus:outline-none focus:border-emerald-500"
                  />
                </div>
                <div>
                  <label className="block text-xs font-semibold text-slate-300 mb-1">Prix en euros (€) (0 = gratuit)</label>
                  <input
                    type="number"
                    step="0.01"
                    min="0"
                    value={((courseForm.price_cents || 0) / 100).toFixed(2)}
                    onChange={(e) =>
                      setCourseForm({
                        ...courseForm,
                        price_cents: Math.round(parseFloat(e.target.value || '0') * 100),
                      })
                    }
                    className="w-full bg-[#121927] border border-white/10 rounded-xl px-3 py-2 text-xs text-white font-mono focus:outline-none focus:border-emerald-500"
                  />
                </div>
              </div>

              <div>
                <label className="block text-xs font-semibold text-slate-300 mb-1">Description / Objectifs</label>
                <textarea
                  rows={2}
                  value={courseForm.description || ''}
                  onChange={(e) => setCourseForm({ ...courseForm, description: e.target.value })}
                  placeholder="Ce que les étudiants vont apprendre dans cette formation..."
                  className="w-full bg-[#121927] border border-white/10 rounded-xl px-3 py-2 text-xs text-white focus:outline-none focus:border-emerald-500"
                />
              </div>

              <div>
                <label className="block text-xs font-semibold text-slate-300 mb-1">Associer à un produit E-Commerce (optionnel)</label>
                <select
                  value={courseForm.product_id || ''}
                  onChange={(e) => setCourseForm({ ...courseForm, product_id: e.target.value || null })}
                  className="w-full bg-[#121927] border border-white/10 rounded-xl px-3 py-2 text-xs text-white focus:outline-none focus:border-emerald-500"
                >
                  <option value="">Aucun produit (inscription manuelle ou gratuite)</option>
                  {products.map((p) => (
                    <option key={p.id} value={p.id}>
                      {p.title} ({(p.price_cents / 100).toFixed(2)} €)
                    </option>
                  ))}
                </select>
              </div>

              <div>
                <label className="block text-xs font-semibold text-slate-300 mb-1">Statut de publication</label>
                <select
                  value={courseForm.is_published ? 'published' : 'draft'}
                  onChange={(e) => setCourseForm({ ...courseForm, is_published: e.target.value === 'published' })}
                  className="w-full bg-[#121927] border border-white/10 rounded-xl px-3 py-2 text-xs text-white focus:outline-none focus:border-emerald-500"
                >
                  <option value="published">Publié (Accessible aux étudiants inscrits)</option>
                  <option value="draft">Brouillon (En cours d'écriture)</option>
                </select>
              </div>

              <div className="flex justify-end gap-3 pt-3 border-t border-white/10">
                <button
                  type="button"
                  onClick={() => setIsCourseModalOpen(false)}
                  className="px-4 py-2 rounded-xl text-xs font-semibold text-slate-400 hover:text-white"
                >
                  Annuler
                </button>
                <button
                  type="submit"
                  className="px-5 py-2 rounded-xl bg-emerald-600 hover:bg-emerald-500 text-xs font-semibold text-white shadow-md"
                >
                  {editingCourse ? 'Enregistrer les modifications' : 'Créer la formation'}
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* MODAL CRÉATION MODULE / SECTION */}
      {isModuleModalOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/70 backdrop-blur-sm">
          <div className="w-full max-w-md bg-[#0e1422] border border-white/10 rounded-2xl shadow-2xl p-6 space-y-4">
            <div className="flex items-center justify-between pb-3 border-b border-white/10">
              <h3 className="text-base font-bold text-white flex items-center gap-2">
                <Layers className="w-5 h-5 text-emerald-400" />
                Ajouter une section
              </h3>
              <button onClick={() => setIsModuleModalOpen(false)} className="text-slate-400 hover:text-white p-1">
                ✕
              </button>
            </div>

            <form onSubmit={handleCreateModule} className="space-y-4">
              <div>
                <label className="block text-xs font-semibold text-slate-300 mb-1">Titre de la section *</label>
                <input
                  type="text"
                  required
                  value={moduleTitle}
                  onChange={(e) => setModuleTitle(e.target.value)}
                  placeholder="Ex: Module 1 : Les bases du langage"
                  className="w-full bg-[#121927] border border-white/10 rounded-xl px-3 py-2 text-xs text-white focus:outline-none focus:border-emerald-500"
                />
              </div>

              <div>
                <label className="block text-xs font-semibold text-slate-300 mb-1">Description courte (optionnel)</label>
                <input
                  type="text"
                  value={moduleDesc}
                  onChange={(e) => setModuleDesc(e.target.value)}
                  placeholder="Concepts fondamentaux abordés"
                  className="w-full bg-[#121927] border border-white/10 rounded-xl px-3 py-2 text-xs text-white focus:outline-none focus:border-emerald-500"
                />
              </div>

              <div className="flex justify-end gap-3 pt-3 border-t border-white/10">
                <button
                  type="button"
                  onClick={() => setIsModuleModalOpen(false)}
                  className="px-4 py-2 rounded-xl text-xs font-semibold text-slate-400 hover:text-white"
                >
                  Annuler
                </button>
                <button
                  type="submit"
                  className="px-5 py-2 rounded-xl bg-emerald-600 hover:bg-emerald-500 text-xs font-semibold text-white shadow-md"
                >
                  Ajouter la section
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* MODAL CRÉATION / ÉDITION LEÇON */}
      {isLessonModalOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/70 backdrop-blur-sm">
          <div className="w-full max-w-xl bg-[#0e1422] border border-white/10 rounded-2xl shadow-2xl p-6 space-y-4 max-h-[90vh] overflow-y-auto">
            <div className="flex items-center justify-between pb-3 border-b border-white/10">
              <h3 className="text-base font-bold text-white flex items-center gap-2">
                <FileText className="w-5 h-5 text-emerald-400" />
                {editingLesson ? 'Modifier la leçon' : 'Créer une leçon'}
              </h3>
              <button onClick={() => setIsLessonModalOpen(false)} className="text-slate-400 hover:text-white p-1">
                ✕
              </button>
            </div>

            <form onSubmit={handleSaveLesson} className="space-y-4">
              <div>
                <label className="block text-xs font-semibold text-slate-300 mb-1">Section parente</label>
                <select
                  value={lessonForm.module_id || ''}
                  onChange={(e) => setLessonForm({ ...lessonForm, module_id: e.target.value || null })}
                  className="w-full bg-[#121927] border border-white/10 rounded-xl px-3 py-2 text-xs text-white focus:outline-none focus:border-emerald-500"
                >
                  <option value="">Hors section</option>
                  {modules.map((m) => (
                    <option key={m.id} value={m.id}>
                      {m.title}
                    </option>
                  ))}
                </select>
              </div>

              <div>
                <label className="block text-xs font-semibold text-slate-300 mb-1">Titre de la leçon *</label>
                <input
                  type="text"
                  required
                  value={lessonForm.title}
                  onChange={(e) => setLessonForm({ ...lessonForm, title: e.target.value })}
                  placeholder="Ex: Configuration de l'environnement et premier binaire"
                  className="w-full bg-[#121927] border border-white/10 rounded-xl px-3 py-2 text-xs text-white focus:outline-none focus:border-emerald-500"
                />
              </div>

              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="block text-xs font-semibold text-slate-300 mb-1">Identifiant URL (Slug)</label>
                  <input
                    type="text"
                    value={lessonForm.slug || ''}
                    onChange={(e) => setLessonForm({ ...lessonForm, slug: e.target.value })}
                    placeholder="laisser vide pour auto"
                    className="w-full bg-[#121927] border border-white/10 rounded-xl px-3 py-2 text-xs text-white font-mono focus:outline-none focus:border-emerald-500"
                  />
                </div>
                <div>
                  <label className="block text-xs font-semibold text-slate-300 mb-1">Durée estimée (minutes)</label>
                  <input
                    type="number"
                    min="1"
                    value={lessonForm.duration_min ?? 15}
                    onChange={(e) =>
                      setLessonForm({ ...lessonForm, duration_min: parseInt(e.target.value, 10) || 15 })
                    }
                    className="w-full bg-[#121927] border border-white/10 rounded-xl px-3 py-2 text-xs text-white font-mono focus:outline-none focus:border-emerald-500"
                  />
                </div>
              </div>

              <div>
                <label className="block text-xs font-semibold text-slate-300 mb-1">URL de la vidéo (optionnel)</label>
                <input
                  type="text"
                  value={lessonForm.video_url || ''}
                  onChange={(e) => setLessonForm({ ...lessonForm, video_url: e.target.value })}
                  placeholder="https://... ou identifiant vidéo sécurisé"
                  className="w-full bg-[#121927] border border-white/10 rounded-xl px-3 py-2 text-xs text-white font-mono focus:outline-none focus:border-emerald-500"
                />
              </div>

              <div>
                <label className="block text-xs font-semibold text-slate-300 mb-1">Contenu texte de la leçon (Markdown)</label>
                <textarea
                  rows={4}
                  value={lessonForm.content || ''}
                  onChange={(e) => setLessonForm({ ...lessonForm, content: e.target.value })}
                  placeholder="Instructions, code d'exemple, explications pédagogiques..."
                  className="w-full bg-[#121927] border border-white/10 rounded-xl px-3 py-2 text-xs text-white focus:outline-none focus:border-emerald-500"
                />
              </div>

              <div className="flex items-center gap-2">
                <input
                  type="checkbox"
                  id="chk-preview"
                  checked={lessonForm.is_preview}
                  onChange={(e) => setLessonForm({ ...lessonForm, is_preview: e.target.checked })}
                  className="rounded border-white/10 bg-slate-800 text-emerald-600 focus:ring-emerald-500"
                />
                <label htmlFor="chk-preview" className="text-xs text-slate-300 cursor-pointer">
                  Aperçu gratuit (accessible aux visiteurs sans inscription)
                </label>
              </div>

              <div className="flex justify-end gap-3 pt-3 border-t border-white/10">
                <button
                  type="button"
                  onClick={() => setIsLessonModalOpen(false)}
                  className="px-4 py-2 rounded-xl text-xs font-semibold text-slate-400 hover:text-white"
                >
                  Annuler
                </button>
                <button
                  type="submit"
                  className="px-5 py-2 rounded-xl bg-emerald-600 hover:bg-emerald-500 text-xs font-semibold text-white shadow-md"
                >
                  {editingLesson ? 'Enregistrer la leçon' : 'Créer la leçon'}
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* MODAL INSCRIPTION MANUELLE D'UN APPRENANT */}
      {isEnrollModalOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/70 backdrop-blur-sm">
          <div className="w-full max-w-md bg-[#0e1422] border border-white/10 rounded-2xl shadow-2xl p-6 space-y-4">
            <div className="flex items-center justify-between pb-3 border-b border-white/10">
              <h3 className="text-base font-bold text-white flex items-center gap-2">
                <UserCheck className="w-5 h-5 text-emerald-400" />
                Inscrire un apprenant
              </h3>
              <button onClick={() => setIsEnrollModalOpen(false)} className="text-slate-400 hover:text-white p-1">
                ✕
              </button>
            </div>

            <form onSubmit={handleEnrollUser} className="space-y-4">
              <div>
                <label className="block text-xs font-semibold text-slate-300 mb-1">Formation</label>
                <select
                  value={enrollCourseId}
                  onChange={(e) => setEnrollCourseId(e.target.value)}
                  className="w-full bg-[#121927] border border-white/10 rounded-xl px-3 py-2 text-xs text-white focus:outline-none focus:border-emerald-500"
                >
                  {courses.map((c) => (
                    <option key={c.id} value={c.id}>
                      {c.title}
                    </option>
                  ))}
                </select>
              </div>

              <div>
                <label className="block text-xs font-semibold text-slate-300 mb-1">Identifiant de l'utilisateur (UUID) *</label>
                <input
                  type="text"
                  required
                  value={enrollUserId}
                  onChange={(e) => setEnrollUserId(e.target.value)}
                  placeholder="Ex: 12345678-1234-1234-1234-123456789abc"
                  className="w-full bg-[#121927] border border-white/10 rounded-xl px-3 py-2 text-xs text-white font-mono focus:outline-none focus:border-emerald-500"
                />
              </div>

              <div className="flex justify-end gap-3 pt-3 border-t border-white/10">
                <button
                  type="button"
                  onClick={() => setIsEnrollModalOpen(false)}
                  className="px-4 py-2 rounded-xl text-xs font-semibold text-slate-400 hover:text-white"
                >
                  Annuler
                </button>
                <button
                  type="submit"
                  className="px-5 py-2 rounded-xl bg-emerald-600 hover:bg-emerald-500 text-xs font-semibold text-white shadow-md"
                >
                  Confirmer l'inscription
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
}
