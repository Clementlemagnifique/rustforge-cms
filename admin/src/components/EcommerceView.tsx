import React, { useState, useEffect, useCallback } from 'react';
import {
  ShoppingBag,
  Package,
  Tag,
  Settings,
  Plus,
  Trash2,
  Edit2,
  CheckCircle,
  AlertCircle,
  RotateCcw,
  XCircle,
  Search,
  FileCheck,
  CreditCard,
  Eye,
} from 'lucide-react';
import { api } from '../api';
import type {
  ProductItem,
  ProductInput,
  OrderItem,
  OrderWithItems,
  DiscountItem,
  DiscountInput,
  CommerceSettings,
} from '../types';

interface EcommerceViewProps {
  initialTab?: 'products' | 'orders' | 'discounts' | 'settings';
}

export function EcommerceView({ initialTab = 'products' }: EcommerceViewProps) {
  const [activeTab, setActiveTab] = useState<'products' | 'orders' | 'discounts' | 'settings'>(initialTab);

  // Notifications
  const [notice, setNotice] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  // Produits
  const [products, setProducts] = useState<ProductItem[]>([]);
  const [productSearch, setProductSearch] = useState('');
  const [productStatusFilter, setProductStatusFilter] = useState<'all' | 'published' | 'draft' | 'archived'>('all');
  const [editingProduct, setEditingProduct] = useState<ProductItem | null>(null);
  const [isProductModalOpen, setIsProductModalOpen] = useState(false);
  const [prodForm, setProdForm] = useState<ProductInput>({
    title: '',
    slug: '',
    excerpt: '',
    description: '',
    price_cents: 2900,
    download_limit: 5,
    file_path: '',
    file_name: '',
    status: 'published',
  });

  // Commandes
  const [orders, setOrders] = useState<OrderItem[]>([]);
  const [orderStatusFilter, setOrderStatusFilter] = useState<string>('all');
  const [selectedOrder, setSelectedOrder] = useState<OrderWithItems | null>(null);
  const [isOrderDetailModalOpen, setIsOrderDetailModalOpen] = useState(false);

  // Codes promo
  const [discounts, setDiscounts] = useState<DiscountItem[]>([]);
  const [isDiscountModalOpen, setIsDiscountModalOpen] = useState(false);
  const [discForm, setDiscForm] = useState<DiscountInput>({
    code: '',
    discount_type: 'percentage',
    amount: 10,
    max_uses: null,
    expires_at: null,
    is_active: true,
  });

  // Réglages
  const [settings, setSettings] = useState<CommerceSettings>({
    currency: 'EUR',
    currency_symbol: '€',
    gateway_mode: 'test',
    stripe_public_key: '',
    stripe_secret_key: '',
    tax_rate_percent: 20.0,
    require_login: false,
    default_download_limit: 5,
  });
  const [savingSettings, setSavingSettings] = useState(false);

  // Chargements de données
  const loadProducts = useCallback(async () => {
    try {
      setLoading(true);
      const data = await api.ecommerce.products.list();
      setProducts(data);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Erreur chargement produits');
    } finally {
      setLoading(false);
    }
  }, []);

  const loadOrders = useCallback(async () => {
    try {
      setLoading(true);
      const data = await api.ecommerce.orders.list({ limit: 100 });
      setOrders(data);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Erreur chargement commandes');
    } finally {
      setLoading(false);
    }
  }, []);

  const loadDiscounts = useCallback(async () => {
    try {
      setLoading(true);
      const data = await api.ecommerce.discounts.list();
      setDiscounts(data);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Erreur chargement réductions');
    } finally {
      setLoading(false);
    }
  }, []);

  const loadSettings = useCallback(async () => {
    try {
      setLoading(true);
      const data = await api.ecommerce.settings.get();
      setSettings(data);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Erreur chargement réglages boutique');
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    let ignore = false;
    async function init() {
      try {
        if (activeTab === 'products') {
          const data = await api.ecommerce.products.list();
          if (!ignore) setProducts(data);
        } else if (activeTab === 'orders') {
          const data = await api.ecommerce.orders.list({ limit: 100 });
          if (!ignore) setOrders(data);
        } else if (activeTab === 'discounts') {
          const data = await api.ecommerce.discounts.list();
          if (!ignore) setDiscounts(data);
        } else if (activeTab === 'settings') {
          const data = await api.ecommerce.settings.get();
          if (!ignore) setSettings(data);
        }
      } catch (e) {
        if (!ignore) setError(e instanceof Error ? e.message : 'Erreur lors du chargement des données');
      }
    }
    void init();
    return () => {
      ignore = true;
    };
  }, [activeTab]);

  // Actions Produits
  function openCreateProduct() {
    setEditingProduct(null);
    setProdForm({
      title: '',
      slug: '',
      excerpt: '',
      description: '',
      price_cents: 2900,
      download_limit: 5,
      file_path: '',
      file_name: '',
      status: 'published',
    });
    setIsProductModalOpen(true);
  }

  function openEditProduct(p: ProductItem) {
    setEditingProduct(p);
    setProdForm({
      title: p.title,
      slug: p.slug,
      excerpt: p.excerpt,
      description: p.description,
      price_cents: p.price_cents,
      download_limit: p.download_limit,
      file_path: p.file_path || '',
      file_name: p.file_name || '',
      status: p.status,
    });
    setIsProductModalOpen(true);
  }

  async function handleSaveProduct(e: React.FormEvent) {
    e.preventDefault();
    setError(null);
    try {
      if (editingProduct) {
        await api.ecommerce.products.update(editingProduct.id, prodForm);
        setNotice(`Produit « ${prodForm.title} » mis à jour.`);
      } else {
        await api.ecommerce.products.create(prodForm);
        setNotice(`Produit « ${prodForm.title} » créé avec succès.`);
      }
      setIsProductModalOpen(false);
      await loadProducts();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Erreur enregistrement produit');
    }
  }

  async function handleDeleteProduct(id: string, title: string) {
    if (!window.confirm(`Supprimer définitivement le produit « ${title} » ?`)) return;
    try {
      await api.ecommerce.products.delete(id);
      setNotice(`Produit « ${title} » supprimé.`);
      await loadProducts();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Impossible de supprimer le produit');
    }
  }

  // Actions Commandes
  async function openOrderDetail(id: string) {
    try {
      const fullOrder = await api.ecommerce.orders.get(id);
      setSelectedOrder(fullOrder);
      setIsOrderDetailModalOpen(true);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Impossible de récupérer la commande');
    }
  }

  async function handleCompleteOrder(id: string) {
    try {
      const updated = await api.ecommerce.orders.complete(id);
      setSelectedOrder(updated);
      setNotice('Commande validée et finalisée avec succès.');
      await loadOrders();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Erreur validation commande');
    }
  }

  async function handleRefundOrder(id: string) {
    if (!window.confirm('Confirmer le remboursement de cette commande ?')) return;
    try {
      const updated = await api.ecommerce.orders.refund(id);
      setSelectedOrder(updated);
      setNotice('Commande marquée comme remboursée.');
      await loadOrders();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Erreur remboursement commande');
    }
  }

  async function handleCancelOrder(id: string) {
    if (!window.confirm('Confirmer l’annulation de cette commande ?')) return;
    try {
      const updated = await api.ecommerce.orders.cancel(id);
      setSelectedOrder(updated);
      setNotice('Commande annulée.');
      await loadOrders();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Erreur annulation commande');
    }
  }

  // Actions Codes Promo
  async function handleSaveDiscount(e: React.FormEvent) {
    e.preventDefault();
    try {
      await api.ecommerce.discounts.create(discForm);
      setNotice(`Code promo « ${discForm.code} » créé avec succès.`);
      setIsDiscountModalOpen(false);
      setDiscForm({
        code: '',
        discount_type: 'percentage',
        amount: 10,
        max_uses: null,
        expires_at: null,
        is_active: true,
      });
      await loadDiscounts();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Erreur création réduction');
    }
  }

  async function handleDeleteDiscount(id: string, code: string) {
    if (!window.confirm(`Supprimer le code promo « ${code} » ?`)) return;
    try {
      await api.ecommerce.discounts.delete(id);
      setNotice(`Code promo « ${code} » supprimé.`);
      await loadDiscounts();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Erreur suppression réduction');
    }
  }

  // Actions Réglages
  async function handleSaveSettings(e: React.FormEvent) {
    e.preventDefault();
    setSavingSettings(true);
    setError(null);
    try {
      const updated = await api.ecommerce.settings.update(settings);
      setSettings(updated);
      setNotice('Réglages de la boutique enregistrés avec succès.');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Erreur enregistrement réglages');
    } finally {
      setSavingSettings(false);
    }
  }

  const filteredProducts = products.filter((p) => {
    const matchSearch =
      p.title.toLowerCase().includes(productSearch.toLowerCase()) ||
      p.slug.toLowerCase().includes(productSearch.toLowerCase());
    const matchStatus = productStatusFilter === 'all' || p.status === productStatusFilter;
    return matchSearch && matchStatus;
  });

  const filteredOrders = orders.filter((o) => {
    if (orderStatusFilter === 'all') return true;
    return o.status === orderStatusFilter;
  });

  function formatCents(cents: number, currency = 'EUR'): string {
    const sym = currency === 'EUR' ? '€' : currency === 'USD' ? '$' : currency;
    return `${(cents / 100).toFixed(2)} ${sym}`;
  }

  return (
    <div className="space-y-6 max-w-6xl mx-auto pb-16">
      {/* En-tête du module */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 pb-4 border-b border-white/10">
        <div>
          <h1 className="text-2xl font-bold tracking-tight text-white flex items-center gap-2">
            <ShoppingBag className="w-6 h-6 text-indigo-400" />
            Boutique & Vente Numérique
          </h1>
          <p className="text-sm text-slate-400 mt-1">
            Catalogue de produits dématérialisés, commandes sécurisées, téléchargements signés HMAC et codes promo.
          </p>
        </div>
        <div className="flex items-center gap-2">
          {activeTab === 'products' && (
            <button
              onClick={openCreateProduct}
              className="inline-flex items-center gap-2 rounded-xl bg-indigo-600 px-4 py-2 text-xs font-semibold text-white shadow-md hover:bg-indigo-500 transition-colors"
            >
              <Plus className="w-4 h-4" />
              Nouveau produit
            </button>
          )}
          {activeTab === 'discounts' && (
            <button
              onClick={() => setIsDiscountModalOpen(true)}
              className="inline-flex items-center gap-2 rounded-xl bg-indigo-600 px-4 py-2 text-xs font-semibold text-white shadow-md hover:bg-indigo-500 transition-colors"
            >
              <Plus className="w-4 h-4" />
              Nouveau code promo
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
          onClick={() => setActiveTab('products')}
          className={`flex items-center gap-2 px-4 py-2.5 text-xs font-medium border-b-2 transition-colors ${
            activeTab === 'products'
              ? 'border-indigo-500 text-white font-semibold'
              : 'border-transparent text-slate-400 hover:text-slate-200'
          }`}
        >
          <Package className="w-4 h-4" />
          Produits ({products.length})
        </button>
        <button
          onClick={() => setActiveTab('orders')}
          className={`flex items-center gap-2 px-4 py-2.5 text-xs font-medium border-b-2 transition-colors ${
            activeTab === 'orders'
              ? 'border-indigo-500 text-white font-semibold'
              : 'border-transparent text-slate-400 hover:text-slate-200'
          }`}
        >
          <ShoppingBag className="w-4 h-4" />
          Commandes ({orders.length})
        </button>
        <button
          onClick={() => setActiveTab('discounts')}
          className={`flex items-center gap-2 px-4 py-2.5 text-xs font-medium border-b-2 transition-colors ${
            activeTab === 'discounts'
              ? 'border-indigo-500 text-white font-semibold'
              : 'border-transparent text-slate-400 hover:text-slate-200'
          }`}
        >
          <Tag className="w-4 h-4" />
          Codes promo ({discounts.length})
        </button>
        <button
          onClick={() => setActiveTab('settings')}
          className={`flex items-center gap-2 px-4 py-2.5 text-xs font-medium border-b-2 transition-colors ${
            activeTab === 'settings'
              ? 'border-indigo-500 text-white font-semibold'
              : 'border-transparent text-slate-400 hover:text-slate-200'
          }`}
        >
          <Settings className="w-4 h-4" />
          Réglages boutique
        </button>
      </div>

      {/* CONTENU ONGLET 1 : PRODUITS */}
      {activeTab === 'products' && (
        <div className="space-y-4">
          <div className="flex flex-col sm:flex-row gap-3 items-center justify-between">
            <div className="relative w-full sm:w-72">
              <Search className="w-4 h-4 text-slate-400 absolute left-3 top-2.5" />
              <input
                type="text"
                value={productSearch}
                onChange={(e) => setProductSearch(e.target.value)}
                placeholder="Rechercher un produit..."
                className="w-full bg-[#121927] border border-white/10 rounded-xl pl-9 pr-3 py-1.5 text-xs text-white placeholder-slate-500 focus:outline-none focus:border-indigo-500"
              />
            </div>
            <div className="flex items-center gap-2 w-full sm:w-auto">
              <select
                value={productStatusFilter}
                onChange={(e) => setProductStatusFilter(e.target.value as any)}
                className="bg-[#121927] border border-white/10 rounded-xl px-3 py-1.5 text-xs text-slate-300 focus:outline-none focus:border-indigo-500"
              >
                <option value="all">Tous les statuts</option>
                <option value="published">Publiés</option>
                <option value="draft">Brouillons</option>
                <option value="archived">Archivés</option>
              </select>
            </div>
          </div>

          {loading && products.length === 0 ? (
            <p className="text-xs text-slate-500 py-8 text-center">Chargement du catalogue…</p>
          ) : filteredProducts.length === 0 ? (
            <div className="p-12 text-center border border-dashed border-white/10 rounded-2xl bg-[#0e1422]">
              <Package className="w-10 h-10 text-slate-600 mx-auto mb-3" />
              <p className="text-sm font-semibold text-slate-300">Aucun produit trouvé</p>
              <p className="text-xs text-slate-500 mt-1">Créez votre premier produit numérique pour démarrer les ventes.</p>
              <button
                onClick={openCreateProduct}
                className="mt-4 inline-flex items-center gap-2 rounded-xl bg-indigo-600 px-4 py-2 text-xs font-semibold text-white shadow-md hover:bg-indigo-500"
              >
                <Plus className="w-4 h-4" />
                Créer un produit
              </button>
            </div>
          ) : (
            <div className="overflow-x-auto border border-white/10 rounded-2xl bg-[#0e1422]">
              <table className="w-full text-left text-xs text-slate-300">
                <thead className="bg-[#141d30] text-slate-400 font-semibold border-b border-white/10">
                  <tr>
                    <th className="px-4 py-3">Produit</th>
                    <th className="px-4 py-3">Prix</th>
                    <th className="px-4 py-3">Ventes</th>
                    <th className="px-4 py-3">Fichier sécurisé</th>
                    <th className="px-4 py-3">Statut</th>
                    <th className="px-4 py-3 text-right">Actions</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-white/5">
                  {filteredProducts.map((p) => (
                    <tr key={p.id} className="hover:bg-white/[0.02] transition-colors">
                      <td className="px-4 py-3.5">
                        <div className="font-semibold text-white">{p.title}</div>
                        <div className="text-[11px] text-slate-500 font-mono mt-0.5">/{p.slug}</div>
                      </td>
                      <td className="px-4 py-3.5 font-bold font-mono text-emerald-400">
                        {formatCents(p.price_cents)}
                      </td>
                      <td className="px-4 py-3.5 font-mono text-slate-300">
                        {p.sales_count} ventes
                      </td>
                      <td className="px-4 py-3.5">
                        {p.file_path ? (
                          <span className="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-md bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 text-[11px]">
                            <FileCheck className="w-3.5 h-3.5" />
                            {p.file_name || 'Protégé'} ({p.download_limit} max)
                          </span>
                        ) : (
                          <span className="text-slate-500 text-[11px]">Aucun fichier</span>
                        )}
                      </td>
                      <td className="px-4 py-3.5">
                        <span
                          className={`inline-block px-2 py-0.5 rounded-full text-[10px] font-semibold tracking-wide uppercase ${
                            p.status === 'published'
                              ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20'
                              : p.status === 'draft'
                              ? 'bg-amber-500/10 text-amber-400 border border-amber-500/20'
                              : 'bg-slate-500/10 text-slate-400 border border-slate-500/20'
                          }`}
                        >
                          {p.status === 'published' ? 'Publié' : p.status === 'draft' ? 'Brouillon' : 'Archivé'}
                        </span>
                      </td>
                      <td className="px-4 py-3.5 text-right space-x-2">
                        <button
                          onClick={() => openEditProduct(p)}
                          className="p-1.5 rounded-lg text-slate-400 hover:text-white hover:bg-white/5"
                          title="Modifier"
                        >
                          <Edit2 className="w-4 h-4" />
                        </button>
                        <button
                          onClick={() => handleDeleteProduct(p.id, p.title)}
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

      {/* CONTENU ONGLET 2 : COMMANDES */}
      {activeTab === 'orders' && (
        <div className="space-y-4">
          <div className="flex items-center justify-between gap-3">
            <div className="flex items-center gap-2">
              <select
                value={orderStatusFilter}
                onChange={(e) => setOrderStatusFilter(e.target.value)}
                className="bg-[#121927] border border-white/10 rounded-xl px-3 py-1.5 text-xs text-slate-300 focus:outline-none focus:border-indigo-500"
              >
                <option value="all">Tous les états</option>
                <option value="pending">En attente (Pending)</option>
                <option value="completed">Finalisées (Completed)</option>
                <option value="refunded">Remboursées (Refunded)</option>
                <option value="failed">Annulées / Échouées (Failed)</option>
              </select>
            </div>
            <button
              onClick={loadOrders}
              className="text-xs text-slate-400 hover:text-white flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-white/5 border border-white/10"
            >
              <RotateCcw className="w-3.5 h-3.5" /> Actualiser
            </button>
          </div>

          {loading && orders.length === 0 ? (
            <p className="text-xs text-slate-500 py-8 text-center">Chargement des commandes…</p>
          ) : filteredOrders.length === 0 ? (
            <div className="p-12 text-center border border-dashed border-white/10 rounded-2xl bg-[#0e1422]">
              <ShoppingBag className="w-10 h-10 text-slate-600 mx-auto mb-3" />
              <p className="text-sm font-semibold text-slate-300">Aucune commande enregistrée</p>
              <p className="text-xs text-slate-500 mt-1">Les ventes passées via le tunnel d'achat apparaîtront ici.</p>
            </div>
          ) : (
            <div className="overflow-x-auto border border-white/10 rounded-2xl bg-[#0e1422]">
              <table className="w-full text-left text-xs text-slate-300">
                <thead className="bg-[#141d30] text-slate-400 font-semibold border-b border-white/10">
                  <tr>
                    <th className="px-4 py-3">Numéro</th>
                    <th className="px-4 py-3">Client</th>
                    <th className="px-4 py-3">Montant</th>
                    <th className="px-4 py-3">Passerelle</th>
                    <th className="px-4 py-3">Statut</th>
                    <th className="px-4 py-3">Date</th>
                    <th className="px-4 py-3 text-right">Détails</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-white/5">
                  {filteredOrders.map((o) => (
                    <tr key={o.id} className="hover:bg-white/[0.02] transition-colors">
                      <td className="px-4 py-3.5 font-mono font-bold text-white">
                        {o.order_number}
                      </td>
                      <td className="px-4 py-3.5">
                        <div className="font-semibold text-white">{o.customer_name || 'Client anonyme'}</div>
                        <div className="text-[11px] text-slate-500 font-mono mt-0.5">{o.customer_email}</div>
                      </td>
                      <td className="px-4 py-3.5 font-mono font-bold text-emerald-400">
                        {formatCents(o.total_cents, o.currency)}
                      </td>
                      <td className="px-4 py-3.5 text-slate-400 uppercase text-[11px]">
                        {o.gateway}
                      </td>
                      <td className="px-4 py-3.5">
                        <span
                          className={`inline-block px-2.5 py-0.5 rounded-full text-[10px] font-semibold tracking-wide uppercase ${
                            o.status === 'completed'
                              ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20'
                              : o.status === 'pending'
                              ? 'bg-amber-500/10 text-amber-400 border border-amber-500/20'
                              : o.status === 'refunded'
                              ? 'bg-purple-500/10 text-purple-400 border border-purple-500/20'
                              : 'bg-red-500/10 text-red-400 border border-red-500/20'
                          }`}
                        >
                          {o.status === 'completed'
                            ? 'Payée'
                            : o.status === 'pending'
                            ? 'En attente'
                            : o.status === 'refunded'
                            ? 'Remboursée'
                            : 'Annulée'}
                        </span>
                      </td>
                      <td className="px-4 py-3.5 text-slate-400 text-[11px]">
                        {new Date(o.created_at).toLocaleString('fr-FR', {
                          day: '2-digit',
                          month: '2-digit',
                          year: 'numeric',
                          hour: '2-digit',
                          minute: '2-digit',
                        })}
                      </td>
                      <td className="px-4 py-3.5 text-right">
                        <button
                          onClick={() => openOrderDetail(o.id)}
                          className="inline-flex items-center gap-1.5 px-3 py-1 rounded-lg bg-indigo-600/20 text-indigo-300 border border-indigo-500/30 hover:bg-indigo-600/30 transition-colors text-xs font-semibold"
                        >
                          <Eye className="w-3.5 h-3.5" /> Gérer
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

      {/* CONTENU ONGLET 3 : CODES PROMO */}
      {activeTab === 'discounts' && (
        <div className="space-y-4">
          {loading && discounts.length === 0 ? (
            <p className="text-xs text-slate-500 py-8 text-center">Chargement des réductions…</p>
          ) : discounts.length === 0 ? (
            <div className="p-12 text-center border border-dashed border-white/10 rounded-2xl bg-[#0e1422]">
              <Tag className="w-10 h-10 text-slate-600 mx-auto mb-3" />
              <p className="text-sm font-semibold text-slate-300">Aucun code promo</p>
              <p className="text-xs text-slate-500 mt-1">Créez des remises en pourcentage ou en montant fixe pour vos clients.</p>
              <button
                onClick={() => setIsDiscountModalOpen(true)}
                className="mt-4 inline-flex items-center gap-2 rounded-xl bg-indigo-600 px-4 py-2 text-xs font-semibold text-white shadow-md hover:bg-indigo-500"
              >
                <Plus className="w-4 h-4" />
                Créer un code promo
              </button>
            </div>
          ) : (
            <div className="overflow-x-auto border border-white/10 rounded-2xl bg-[#0e1422]">
              <table className="w-full text-left text-xs text-slate-300">
                <thead className="bg-[#141d30] text-slate-400 font-semibold border-b border-white/10">
                  <tr>
                    <th className="px-4 py-3">Code</th>
                    <th className="px-4 py-3">Type</th>
                    <th className="px-4 py-3">Montant</th>
                    <th className="px-4 py-3">Utilisations</th>
                    <th className="px-4 py-3">Expiration</th>
                    <th className="px-4 py-3">Statut</th>
                    <th className="px-4 py-3 text-right">Actions</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-white/5">
                  {discounts.map((d) => (
                    <tr key={d.id} className="hover:bg-white/[0.02] transition-colors">
                      <td className="px-4 py-3.5 font-mono font-bold text-white text-sm">
                        {d.code}
                      </td>
                      <td className="px-4 py-3.5 text-slate-400">
                        {d.discount_type === 'percentage' ? 'Pourcentage (%)' : 'Montant fixe (€)'}
                      </td>
                      <td className="px-4 py-3.5 font-bold font-mono text-emerald-400">
                        {d.discount_type === 'percentage' ? `${d.amount} %` : formatCents(d.amount)}
                      </td>
                      <td className="px-4 py-3.5 font-mono">
                        {d.uses_count} / {d.max_uses ?? '∞'}
                      </td>
                      <td className="px-4 py-3.5 text-slate-400 text-[11px]">
                        {d.expires_at ? new Date(d.expires_at).toLocaleDateString('fr-FR') : 'Sans expiration'}
                      </td>
                      <td className="px-4 py-3.5">
                        <span
                          className={`inline-block px-2 py-0.5 rounded-full text-[10px] font-semibold uppercase ${
                            d.is_active
                              ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20'
                              : 'bg-red-500/10 text-red-400 border border-red-500/20'
                          }`}
                        >
                          {d.is_active ? 'Actif' : 'Inactif'}
                        </span>
                      </td>
                      <td className="px-4 py-3.5 text-right">
                        <button
                          onClick={() => handleDeleteDiscount(d.id, d.code)}
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

      {/* CONTENU ONGLET 4 : RÉGLAGES BOUTIQUE */}
      {activeTab === 'settings' && (
        <form onSubmit={handleSaveSettings} className="space-y-6 max-w-2xl bg-[#0e1422] p-6 border border-white/10 rounded-2xl">
          <div className="flex items-center justify-between">
            <div>
              <h3 className="text-base font-semibold text-white">Paramètres généraux de la boutique</h3>
              <p className="text-xs text-slate-400 mt-0.5">Configuration des devises, taxes et passerelles sécurisées.</p>
            </div>
            <button
              type="button"
              onClick={loadSettings}
              className="text-xs text-slate-400 hover:text-white flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-white/5 border border-white/10"
            >
              <RotateCcw className="w-3.5 h-3.5" /> Actualiser
            </button>
          </div>

          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div>
              <label className="block text-xs font-semibold text-slate-300 mb-1">Devise principale</label>
              <select
                value={settings.currency}
                onChange={(e) => {
                  const curr = e.target.value;
                  const sym = curr === 'EUR' ? '€' : curr === 'USD' ? '$' : curr === 'GBP' ? '£' : 'CHF';
                  setSettings({ ...settings, currency: curr, currency_symbol: sym });
                }}
                className="w-full bg-[#121927] border border-white/10 rounded-xl px-3 py-2 text-xs text-white focus:outline-none focus:border-indigo-500"
              >
                <option value="EUR">Euro (EUR - €)</option>
                <option value="USD">Dollar US (USD - $)</option>
                <option value="GBP">Livre Sterling (GBP - £)</option>
                <option value="CHF">Franc Suisse (CHF)</option>
              </select>
            </div>

            <div>
              <label className="block text-xs font-semibold text-slate-300 mb-1">Taux de TVA (%)</label>
              <input
                type="number"
                step="0.1"
                min="0"
                max="100"
                value={settings.tax_rate_percent}
                onChange={(e) => setSettings({ ...settings, tax_rate_percent: parseFloat(e.target.value) || 0 })}
                className="w-full bg-[#121927] border border-white/10 rounded-xl px-3 py-2 text-xs text-white focus:outline-none focus:border-indigo-500 font-mono"
              />
            </div>

            <div>
              <label className="block text-xs font-semibold text-slate-300 mb-1">Mode de paiement / Passerelle</label>
              <select
                value={settings.gateway_mode}
                onChange={(e) => setSettings({ ...settings, gateway_mode: e.target.value })}
                className="w-full bg-[#121927] border border-white/10 rounded-xl px-3 py-2 text-xs text-white focus:outline-none focus:border-indigo-500"
              >
                <option value="test">Simulation locale / Mode test</option>
                <option value="manual">Paiement manuel / Virement bancaire</option>
                <option value="stripe">Passerelle bancaire Stripe (Live / Sandbox)</option>
              </select>
            </div>

            <div>
              <label className="block text-xs font-semibold text-slate-300 mb-1">Limite téléchargements par défaut</label>
              <input
                type="number"
                min="1"
                max="1000"
                value={settings.default_download_limit}
                onChange={(e) => setSettings({ ...settings, default_download_limit: parseInt(e.target.value, 10) || 5 })}
                className="w-full bg-[#121927] border border-white/10 rounded-xl px-3 py-2 text-xs text-white focus:outline-none focus:border-indigo-500 font-mono"
              />
            </div>
          </div>

          {settings.gateway_mode === 'stripe' && (
            <div className="space-y-4 pt-3 border-t border-white/10">
              <h4 className="text-xs font-bold text-indigo-400 uppercase tracking-wider flex items-center gap-1.5">
                <CreditCard className="w-3.5 h-3.5" /> Configuration Stripe
              </h4>
              <div>
                <label className="block text-xs font-semibold text-slate-300 mb-1">Clé publique Stripe (Publishable Key)</label>
                <input
                  type="text"
                  value={settings.stripe_public_key}
                  onChange={(e) => setSettings({ ...settings, stripe_public_key: e.target.value })}
                  placeholder="pk_live_... ou pk_test_..."
                  className="w-full bg-[#121927] border border-white/10 rounded-xl px-3 py-2 text-xs text-white font-mono focus:outline-none focus:border-indigo-500"
                />
              </div>
              <div>
                <label className="block text-xs font-semibold text-slate-300 mb-1">Clé secrète Stripe (Secret Key)</label>
                <input
                  type="password"
                  value={settings.stripe_secret_key}
                  onChange={(e) => setSettings({ ...settings, stripe_secret_key: e.target.value })}
                  placeholder="sk_live_... ou sk_test_..."
                  className="w-full bg-[#121927] border border-white/10 rounded-xl px-3 py-2 text-xs text-white font-mono focus:outline-none focus:border-indigo-500"
                />
              </div>
            </div>
          )}

          <div className="pt-2">
            <button
              type="submit"
              disabled={savingSettings}
              className="rounded-xl bg-indigo-600 px-5 py-2.5 text-xs font-semibold text-white shadow-md hover:bg-indigo-500 transition-colors disabled:opacity-50"
            >
              {savingSettings ? 'Enregistrement…' : 'Enregistrer les réglages'}
            </button>
          </div>
        </form>
      )}

      {/* MODAL CRÉATION / ÉDITION PRODUIT */}
      {isProductModalOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/70 backdrop-blur-sm">
          <div className="w-full max-w-xl bg-[#0e1422] border border-white/10 rounded-2xl shadow-2xl p-6 space-y-4 max-h-[90vh] overflow-y-auto">
            <div className="flex items-center justify-between pb-3 border-b border-white/10">
              <h3 className="text-base font-bold text-white flex items-center gap-2">
                <Package className="w-5 h-5 text-indigo-400" />
                {editingProduct ? 'Modifier le produit' : 'Créer un produit numérique'}
              </h3>
              <button
                onClick={() => setIsProductModalOpen(false)}
                className="text-slate-400 hover:text-white p-1"
              >
                ✕
              </button>
            </div>

            <form onSubmit={handleSaveProduct} className="space-y-4">
              <div>
                <label className="block text-xs font-semibold text-slate-300 mb-1">Titre du produit *</label>
                <input
                  type="text"
                  required
                  value={prodForm.title}
                  onChange={(e) => setProdForm({ ...prodForm, title: e.target.value })}
                  placeholder="Ex: Thème Astra Pro Edition"
                  className="w-full bg-[#121927] border border-white/10 rounded-xl px-3 py-2 text-xs text-white focus:outline-none focus:border-indigo-500"
                />
              </div>

              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="block text-xs font-semibold text-slate-300 mb-1">Identifiant d’URL (Slug)</label>
                  <input
                    type="text"
                    value={prodForm.slug || ''}
                    onChange={(e) => setProdForm({ ...prodForm, slug: e.target.value })}
                    placeholder="laisser vide pour auto"
                    className="w-full bg-[#121927] border border-white/10 rounded-xl px-3 py-2 text-xs text-white font-mono focus:outline-none focus:border-indigo-500"
                  />
                </div>
                <div>
                  <label className="block text-xs font-semibold text-slate-300 mb-1">Prix en euros (€) *</label>
                  <input
                    type="number"
                    step="0.01"
                    min="0"
                    required
                    value={(prodForm.price_cents / 100).toFixed(2)}
                    onChange={(e) =>
                      setProdForm({ ...prodForm, price_cents: Math.round(parseFloat(e.target.value || '0') * 100) })
                    }
                    className="w-full bg-[#121927] border border-white/10 rounded-xl px-3 py-2 text-xs text-white font-mono focus:outline-none focus:border-indigo-500"
                  />
                </div>
              </div>

              <div>
                <label className="block text-xs font-semibold text-slate-300 mb-1">Résumé court</label>
                <input
                  type="text"
                  value={prodForm.excerpt || ''}
                  onChange={(e) => setProdForm({ ...prodForm, excerpt: e.target.value })}
                  placeholder="Brève description affichée dans les listes"
                  className="w-full bg-[#121927] border border-white/10 rounded-xl px-3 py-2 text-xs text-white focus:outline-none focus:border-indigo-500"
                />
              </div>

              <div>
                <label className="block text-xs font-semibold text-slate-300 mb-1">Description complète</label>
                <textarea
                  rows={3}
                  value={prodForm.description || ''}
                  onChange={(e) => setProdForm({ ...prodForm, description: e.target.value })}
                  placeholder="Détails du produit, fonctionnalités, prérequis..."
                  className="w-full bg-[#121927] border border-white/10 rounded-xl px-3 py-2 text-xs text-white focus:outline-none focus:border-indigo-500"
                />
              </div>

              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="block text-xs font-semibold text-slate-300 mb-1">Nom du fichier livré</label>
                  <input
                    type="text"
                    value={prodForm.file_name || ''}
                    onChange={(e) => setProdForm({ ...prodForm, file_name: e.target.value })}
                    placeholder="package-v1.0.zip"
                    className="w-full bg-[#121927] border border-white/10 rounded-xl px-3 py-2 text-xs text-white font-mono focus:outline-none focus:border-indigo-500"
                  />
                </div>
                <div>
                  <label className="block text-xs font-semibold text-slate-300 mb-1">Limite de téléchargements</label>
                  <input
                    type="number"
                    min="1"
                    value={prodForm.download_limit ?? 5}
                    onChange={(e) =>
                      setProdForm({ ...prodForm, download_limit: parseInt(e.target.value, 10) || 5 })
                    }
                    className="w-full bg-[#121927] border border-white/10 rounded-xl px-3 py-2 text-xs text-white font-mono focus:outline-none focus:border-indigo-500"
                  />
                </div>
              </div>

              <div>
                <label className="block text-xs font-semibold text-slate-300 mb-1">Statut</label>
                <select
                  value={prodForm.status || 'published'}
                  onChange={(e) => setProdForm({ ...prodForm, status: e.target.value as any })}
                  className="w-full bg-[#121927] border border-white/10 rounded-xl px-3 py-2 text-xs text-white focus:outline-none focus:border-indigo-500"
                >
                  <option value="published">Publié (En vente immédiatement)</option>
                  <option value="draft">Brouillon (Non visible)</option>
                  <option value="archived">Archivé (Vente fermée)</option>
                </select>
              </div>

              <div className="flex justify-end gap-3 pt-3 border-t border-white/10">
                <button
                  type="button"
                  onClick={() => setIsProductModalOpen(false)}
                  className="px-4 py-2 rounded-xl text-xs font-semibold text-slate-400 hover:text-white"
                >
                  Annuler
                </button>
                <button
                  type="submit"
                  className="px-5 py-2 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-xs font-semibold text-white shadow-md"
                >
                  {editingProduct ? 'Enregistrer les modifications' : 'Créer le produit'}
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* MODAL CRÉATION CODE PROMO */}
      {isDiscountModalOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/70 backdrop-blur-sm">
          <div className="w-full max-w-md bg-[#0e1422] border border-white/10 rounded-2xl shadow-2xl p-6 space-y-4">
            <div className="flex items-center justify-between pb-3 border-b border-white/10">
              <h3 className="text-base font-bold text-white flex items-center gap-2">
                <Tag className="w-5 h-5 text-indigo-400" />
                Nouveau code promo
              </h3>
              <button onClick={() => setIsDiscountModalOpen(false)} className="text-slate-400 hover:text-white p-1">
                ✕
              </button>
            </div>

            <form onSubmit={handleSaveDiscount} className="space-y-4">
              <div>
                <label className="block text-xs font-semibold text-slate-300 mb-1">Code promo *</label>
                <input
                  type="text"
                  required
                  value={discForm.code}
                  onChange={(e) => setDiscForm({ ...discForm, code: e.target.value.toUpperCase() })}
                  placeholder="EXEMPLE20"
                  className="w-full bg-[#121927] border border-white/10 rounded-xl px-3 py-2 text-xs text-white font-mono uppercase focus:outline-none focus:border-indigo-500"
                />
              </div>

              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="block text-xs font-semibold text-slate-300 mb-1">Type de réduction</label>
                  <select
                    value={discForm.discount_type || 'percentage'}
                    onChange={(e) => setDiscForm({ ...discForm, discount_type: e.target.value as any })}
                    className="w-full bg-[#121927] border border-white/10 rounded-xl px-3 py-2 text-xs text-white focus:outline-none focus:border-indigo-500"
                  >
                    <option value="percentage">Pourcentage (%)</option>
                    <option value="fixed_amount">Montant fixe (€)</option>
                  </select>
                </div>
                <div>
                  <label className="block text-xs font-semibold text-slate-300 mb-1">
                    {discForm.discount_type === 'percentage' ? 'Pourcentage (%)' : 'Montant (€)'}
                  </label>
                  <input
                    type="number"
                    min="1"
                    max={discForm.discount_type === 'percentage' ? 100 : 10000}
                    required
                    value={discForm.discount_type === 'percentage' ? discForm.amount : (discForm.amount / 100).toFixed(2)}
                    onChange={(e) => {
                      const val = parseFloat(e.target.value) || 0;
                      setDiscForm({
                        ...discForm,
                        amount: discForm.discount_type === 'percentage' ? Math.round(val) : Math.round(val * 100),
                      });
                    }}
                    className="w-full bg-[#121927] border border-white/10 rounded-xl px-3 py-2 text-xs text-white font-mono focus:outline-none focus:border-indigo-500"
                  />
                </div>
              </div>

              <div>
                <label className="block text-xs font-semibold text-slate-300 mb-1">Nombre max d'utilisations (optionnel)</label>
                <input
                  type="number"
                  min="1"
                  value={discForm.max_uses ?? ''}
                  onChange={(e) =>
                    setDiscForm({
                      ...discForm,
                      max_uses: e.target.value ? parseInt(e.target.value, 10) : null,
                    })
                  }
                  placeholder="Illimité si vide"
                  className="w-full bg-[#121927] border border-white/10 rounded-xl px-3 py-2 text-xs text-white font-mono focus:outline-none focus:border-indigo-500"
                />
              </div>

              <div className="flex justify-end gap-3 pt-3 border-t border-white/10">
                <button
                  type="button"
                  onClick={() => setIsDiscountModalOpen(false)}
                  className="px-4 py-2 rounded-xl text-xs font-semibold text-slate-400 hover:text-white"
                >
                  Annuler
                </button>
                <button
                  type="submit"
                  className="px-5 py-2 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-xs font-semibold text-white shadow-md"
                >
                  Créer le code promo
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* MODAL DÉTAILS ET GESTION DE COMMANDE */}
      {isOrderDetailModalOpen && selectedOrder && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/70 backdrop-blur-sm">
          <div className="w-full max-w-xl bg-[#0e1422] border border-white/10 rounded-2xl shadow-2xl p-6 space-y-4 max-h-[90vh] overflow-y-auto">
            <div className="flex items-center justify-between pb-3 border-b border-white/10">
              <div>
                <h3 className="text-base font-bold text-white flex items-center gap-2">
                  <ShoppingBag className="w-5 h-5 text-indigo-400" />
                  Commande {selectedOrder.order.order_number}
                </h3>
                <p className="text-xs text-slate-400 mt-0.5">
                  Client : {selectedOrder.order.customer_name || 'Sans nom'} ({selectedOrder.order.customer_email})
                </p>
              </div>
              <button
                onClick={() => setIsOrderDetailModalOpen(false)}
                className="text-slate-400 hover:text-white p-1"
              >
                ✕
              </button>
            </div>

            {/* Articles de la commande */}
            <div className="space-y-2">
              <h4 className="text-xs font-bold text-slate-400 uppercase tracking-wider">Articles commandés</h4>
              <div className="divide-y divide-white/5 border border-white/10 rounded-xl bg-[#121927]">
                {selectedOrder.items.map((it) => (
                  <div key={it.id} className="p-3 flex items-center justify-between text-xs">
                    <div>
                      <p className="font-semibold text-white">{it.title}</p>
                      <p className="text-slate-500 text-[11px] mt-0.5">Téléchargements consommés : {it.download_count}</p>
                    </div>
                    <span className="font-mono font-bold text-emerald-400">
                      {formatCents(it.price_cents, selectedOrder.order.currency)}
                    </span>
                  </div>
                ))}
              </div>
            </div>

            {/* Récapitulatif financier */}
            <div className="space-y-1.5 p-3 rounded-xl bg-white/[0.02] border border-white/5 text-xs">
              <div className="flex justify-between text-slate-400">
                <span>Sous-total :</span>
                <span className="font-mono">{formatCents(selectedOrder.order.subtotal_cents, selectedOrder.order.currency)}</span>
              </div>
              {selectedOrder.order.discount_cents > 0 && (
                <div className="flex justify-between text-purple-400">
                  <span>Remise appliquée :</span>
                  <span className="font-mono">- {formatCents(selectedOrder.order.discount_cents, selectedOrder.order.currency)}</span>
                </div>
              )}
              <div className="flex justify-between text-sm font-bold text-white pt-2 border-t border-white/10">
                <span>Total réglé :</span>
                <span className="font-mono text-emerald-400">{formatCents(selectedOrder.order.total_cents, selectedOrder.order.currency)}</span>
              </div>
            </div>

            {/* Actions de transition d'état */}
            <div className="space-y-2 pt-2 border-t border-white/10">
              <h4 className="text-xs font-bold text-slate-400 uppercase tracking-wider">État et actions</h4>
              <div className="flex flex-wrap items-center gap-2">
                {selectedOrder.order.status === 'pending' && (
                  <>
                    <button
                      onClick={() => handleCompleteOrder(selectedOrder.order.id)}
                      className="px-4 py-2 rounded-xl bg-emerald-600 hover:bg-emerald-500 text-xs font-semibold text-white shadow-md flex items-center gap-1.5"
                    >
                      <CheckCircle className="w-4 h-4" /> Valider le paiement
                    </button>
                    <button
                      onClick={() => handleCancelOrder(selectedOrder.order.id)}
                      className="px-4 py-2 rounded-xl bg-red-600/20 text-red-300 border border-red-500/30 hover:bg-red-600/30 text-xs font-semibold flex items-center gap-1.5"
                    >
                      <XCircle className="w-4 h-4" /> Annuler la commande
                    </button>
                  </>
                )}
                {selectedOrder.order.status === 'completed' && (
                  <button
                    onClick={() => handleRefundOrder(selectedOrder.order.id)}
                    className="px-4 py-2 rounded-xl bg-purple-600/20 text-purple-300 border border-purple-500/30 hover:bg-purple-600/30 text-xs font-semibold flex items-center gap-1.5"
                  >
                    <RotateCcw className="w-4 h-4" /> Rembourser la commande
                  </button>
                )}
                <span className="text-[11px] text-slate-500 ml-auto font-mono">
                  Statut : {selectedOrder.order.status}
                </span>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
