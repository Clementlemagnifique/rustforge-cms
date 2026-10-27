import React, { useState, useEffect } from 'react';
import { Upload, Trash2, File } from 'lucide-react';
import { api } from '../api';
import type { Media } from '../types';

export const MediaView: React.FC = () => {
  const [mediaList, setMediaList] = useState<Media[]>([]);
  const [loading, setLoading] = useState(true);
  const [uploading, setUploading] = useState(false);

  const fetchMedia = async () => {
    setLoading(true);
    try {
      const res = await api.media.list();
      setMediaList(res.items);
    } catch (err) {
      console.error(err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchMedia();
  }, []);

  const handleFileUpload = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    setUploading(true);
    try {
      await api.media.upload(file);
      fetchMedia();
    } catch (err: any) {
      alert(`Erreur d'upload : ${err.message}`);
    } finally {
      setUploading(false);
    }
  };

  const handleDelete = async (id: string) => {
    if (!confirm('Supprimer définitivement ce média ?')) return;
    try {
      await api.media.delete(id);
      fetchMedia();
    } catch (err: any) {
      alert(err.message);
    }
  };

  return (
    <div className="space-y-6">
      {/* Zone de drop/upload */}
      <div className="p-8 border-2 border-dashed border-white/10 hover:border-indigo-500/40 rounded-2xl bg-[#0f1422] text-center transition-all">
        <div className="w-12 h-12 rounded-2xl bg-indigo-500/10 border border-indigo-500/20 text-indigo-400 flex items-center justify-center mx-auto mb-3">
          <Upload className="w-5 h-5" />
        </div>
        <h4 className="text-sm font-bold text-white mb-1">
          {uploading ? 'Téléversement en cours...' : 'Déposez vos images et documents ici'}
        </h4>
        <p className="text-xs text-slate-400 mb-4">
          Supporte JPEG, PNG, WebP, AVIF, PDF, MP4 jusqu'à 50 Mo
        </p>
        <label className="inline-flex items-center gap-2 px-4 py-2 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-semibold cursor-pointer shadow-md shadow-indigo-600/20 transition-all hover:scale-105 active:scale-95">
          <span>Parcourir les fichiers</span>
          <input
            type="file"
            onChange={handleFileUpload}
            disabled={uploading}
            className="hidden"
          />
        </label>
      </div>

      {/* Grille de médias */}
      <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-6 gap-4">
        {loading ? (
          <div className="col-span-full py-12 text-center text-xs text-slate-500">
            Chargement de la médiathèque...
          </div>
        ) : mediaList.length === 0 ? (
          <div className="col-span-full py-12 text-center text-xs text-slate-500">
            Aucun fichier dans la médiathèque.
          </div>
        ) : (
          mediaList.map((m) => (
            <div
              key={m.id}
              className="group relative rounded-xl bg-[#0f1422] border border-white/5 overflow-hidden flex flex-col hover:border-indigo-500/30 transition-all"
            >
              <div className="aspect-square bg-black/40 flex items-center justify-center overflow-hidden">
                {m.mime_type.startsWith('image/') ? (
                  <img
                    src={m.url}
                    alt={m.alt || m.original_name}
                    className="w-full h-full object-cover group-hover:scale-105 transition-transform"
                  />
                ) : (
                  <File className="w-8 h-8 text-slate-500" />
                )}
              </div>

              <div className="p-2.5 flex items-center justify-between text-[11px]">
                <p className="font-medium text-slate-300 truncate max-w-[120px]" title={m.original_name}>
                  {m.original_name}
                </p>
                <button
                  onClick={() => handleDelete(m.id)}
                  title="Supprimer"
                  className="opacity-0 group-hover:opacity-100 p-1 text-slate-400 hover:text-red-400 transition-opacity"
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
};
