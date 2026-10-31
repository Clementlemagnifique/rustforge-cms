import type { NativeDocument } from './document';
export interface DocumentMetrics { words: number; characters: number; reading_minutes: number }
export interface KernelData { body: string; html: string; metrics: DocumentMetrics }
interface Kernel { default: () => Promise<unknown>; inspect_document: (body: string, blocks: string) => string }
class WasmUnavailableError extends Error {}
let loading: Promise<Kernel> | null = null;
async function kernel(): Promise<Kernel> {
  if (!loading) {
    const url = new URL('wasm/rustforge_studio.js', new URL('/admin/', window.location.origin)).href;
    loading = import(/* @vite-ignore */ url).then(async module => { await module.default(); return module as Kernel; })
      .catch(() => { loading = null; throw new WasmUnavailableError('Moteur WASM indisponible ; validation serveur active'); });
  }
  return loading;
}
export async function inspectDocument(body: string, blocks: NativeDocument | null): Promise<KernelData> {
  const module = await kernel();
  const result = JSON.parse(module.inspect_document(body, JSON.stringify(blocks))) as {ok: boolean; data?: KernelData; error?: string};
  if (!result.ok || !result.data) throw new Error(result.error || 'Document refusé par le moteur Rust');
  return result.data;
}

/** La validation serveur reste autoritaire pendant la migration progressive. */
export async function validateNativeBeforeSave(body: string, blocks: NativeDocument): Promise<void> {
  try { await inspectDocument(body, blocks); }
  catch (error) { if (!(error instanceof WasmUnavailableError)) throw error; }
}
