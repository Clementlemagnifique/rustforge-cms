import type { NativeBlock, NativeDocument } from './document';
import { DOCUMENT_SCHEMA, readDocument } from './document';

export interface BlockPattern {
  id: string;
  title: string;
  description: string;
  category: 'hero' | 'layout' | 'content' | 'marketing';
  blocks: Omit<NativeBlock, 'rf_id'>[];
}

export const BLOCK_PATTERNS: BlockPattern[] = [
  {
    id: 'hero_cta',
    title: 'Bannière Héro avec Appel à l’Action',
    description: 'Titre principal H1, paragraphe percutant et bouton de conversion.',
    category: 'hero',
    blocks: [
      {
        rf_type: 'heading',
        rf_text: 'Construisez l’avenir avec une souveraineté totale',
        rf_level: 1,
        rf_ordered: false,
        rf_items: [],
      },
      {
        rf_type: 'paragraph',
        rf_text: 'Une plateforme moderne, ultra-rapide et sécurisée, sans runtime PHP ni extensions arbitraires.',
        rf_level: 2,
        rf_ordered: false,
        rf_items: [],
      },
      {
        rf_type: 'button',
        rf_text: 'Commencer dès aujourd’hui',
        rf_url: '/contact',
        rf_level: 2,
        rf_ordered: false,
        rf_items: [],
      },
    ],
  },
  {
    id: 'columns_2_features',
    title: 'Grille 2 Colonnes Avantages',
    description: 'Deux colonnes équilibrées pour comparer ou valoriser des points forts.',
    category: 'layout',
    blocks: [
      {
        rf_type: 'columns',
        rf_text: '',
        rf_level: 2,
        rf_ordered: false,
        rf_items: [
          '### Performance Rust\nTemps de réponse inférieurs à 2ms et empreinte mémoire minimale.',
          '### Sécurité Intransigeante\nAssainissement Ammonia strict, protection XSS et conformité RGPD.',
        ],
      },
    ],
  },
  {
    id: 'media_text_feature',
    title: 'Média & Texte Valorisé',
    description: 'Image illustrative à gauche avec description détaillée à droite.',
    category: 'marketing',
    blocks: [
      {
        rf_type: 'media_text',
        rf_text: 'Découvrez notre environnement de création conçu pour les équipes exigeantes.\n\nContrôle total sur chaque bloc et zéro dépendance superflue.',
        rf_url: '/uploads/feature-preview.webp',
        rf_alt: 'Aperçu de la fonctionnalité',
        rf_style: 'media-left',
        rf_level: 2,
        rf_ordered: false,
        rf_items: [],
      },
    ],
  },
  {
    id: 'faq_accordion',
    title: 'Section FAQ (Questions / Réponses)',
    description: 'Titre de section et deux accordéons repliables natifs.',
    category: 'content',
    blocks: [
      {
        rf_type: 'heading',
        rf_text: 'Foire Aux Questions',
        rf_level: 2,
        rf_ordered: false,
        rf_items: [],
      },
      {
        rf_type: 'details',
        rf_caption: 'Comment fonctionne la souveraineté des données ?',
        rf_text: 'Toutes les données, révisions et médias restent hébergés sur votre infrastructure PostgreSQL sans télémétrie.',
        rf_level: 2,
        rf_ordered: false,
        rf_items: [],
      },
      {
        rf_type: 'details',
        rf_caption: 'Les menus sont-ils compatibles sans JavaScript ?',
        rf_text: 'Oui, l’affichage responsive s’appuie sur du pur CSS accessible au clavier avec support des touches Entrée et Échap.',
        rf_level: 2,
        rf_ordered: false,
        rf_items: [],
      },
    ],
  },
  {
    id: 'callout_tip',
    title: 'Encadré d’Avertissement ou d’Astuce',
    description: 'Mise en avant stylisée pour remarques importantes.',
    category: 'content',
    blocks: [
      {
        rf_type: 'callout',
        rf_text: 'Pensez à définir des descriptions et textes alternatifs pour optimiser l’accessibilité et le référencement.',
        rf_style: 'tip',
        rf_level: 2,
        rf_ordered: false,
        rf_items: [],
      },
    ],
  },
  {
    id: 'quote_testimonial',
    title: 'Témoignage Client',
    description: 'Citation stylisée suivie de la signature de l’auteur.',
    category: 'marketing',
    blocks: [
      {
        rf_type: 'quote',
        rf_text: 'RustForge a transformé notre gestion de contenu avec une fiabilité et une vitesse incomparables.',
        rf_level: 2,
        rf_ordered: false,
        rf_items: [],
      },
      {
        rf_type: 'paragraph',
        rf_text: '— Direction Technique, Entreprise Partenaire',
        rf_level: 2,
        rf_ordered: false,
        rf_items: [],
      },
    ],
  },
];

export function instantiatePattern(pattern: BlockPattern): NativeBlock[] {
  return pattern.blocks.map(b => ({
    ...b,
    rf_id: crypto.randomUUID(),
    rf_items: [...b.rf_items],
  }));
}

export function insertPatternAfter(
  doc: NativeDocument,
  afterId: string | undefined,
  pattern: BlockPattern,
): NativeDocument {
  const newBlocks = instantiatePattern(pattern);
  if (doc.rf_blocks.length + newBlocks.length > 256) return doc;

  if (!afterId) {
    return {
      rf_schema: DOCUMENT_SCHEMA,
      rf_blocks: [...doc.rf_blocks, ...newBlocks],
    };
  }

  const index = doc.rf_blocks.findIndex(b => b.rf_id === afterId);
  if (index < 0) {
    return {
      rf_schema: DOCUMENT_SCHEMA,
      rf_blocks: [...doc.rf_blocks, ...newBlocks],
    };
  }

  const updated = [...doc.rf_blocks];
  updated.splice(index + 1, 0, ...newBlocks);
  return {
    rf_schema: DOCUMENT_SCHEMA,
    rf_blocks: updated,
  };
}

export interface SyncedPattern {
  id: string;
  slug: string;
  title: string;
  description: string;
  category: string;
  document: unknown;
  is_synced: boolean;
  created_at: string;
  updated_at: string;
}

export function createPatternBlock(pattern: SyncedPattern): NativeBlock {
  const doc = readDocument(pattern.document);
  const items = doc ? doc.rf_blocks.map(b => b.rf_text || b.rf_caption || b.rf_url || '') : [];
  return {
    rf_id: crypto.randomUUID(),
    rf_type: 'pattern',
    rf_ref: pattern.slug,
    rf_caption: pattern.title,
    rf_text: '',
    rf_level: 2,
    rf_ordered: false,
    rf_items: items,
  };
}

export function insertSyncedPatternAfter(
  doc: NativeDocument,
  afterId: string | undefined,
  pattern: SyncedPattern,
): NativeDocument {
  const block = createPatternBlock(pattern);
  if (doc.rf_blocks.length >= 256) return doc;
  if (!afterId) {
    return {
      rf_schema: DOCUMENT_SCHEMA,
      rf_blocks: [...doc.rf_blocks, block],
    };
  }
  const index = doc.rf_blocks.findIndex(b => b.rf_id === afterId);
  if (index < 0) {
    return {
      rf_schema: DOCUMENT_SCHEMA,
      rf_blocks: [...doc.rf_blocks, block],
    };
  }
  const updated = [...doc.rf_blocks];
  updated.splice(index + 1, 0, block);
  return {
    rf_schema: DOCUMENT_SCHEMA,
    rf_blocks: updated,
  };
}

export function detachPatternBlock(
  doc: NativeDocument,
  blockId: string,
  syncedPattern?: SyncedPattern,
): NativeDocument {
  const index = doc.rf_blocks.findIndex(b => b.rf_id === blockId);
  if (index < 0) return doc;
  const block = doc.rf_blocks[index];
  let replacements: NativeBlock[] = [];
  if (syncedPattern) {
    const patternDoc = readDocument(syncedPattern.document);
    if (patternDoc && patternDoc.rf_blocks.length > 0) {
      replacements = patternDoc.rf_blocks.map(b => ({
        ...b,
        rf_id: crypto.randomUUID(),
        rf_items: [...b.rf_items],
      }));
    }
  }
  if (replacements.length === 0) {
    if (block.rf_items.length > 0) {
      replacements = block.rf_items.map(item => ({
        rf_id: crypto.randomUUID(),
        rf_type: 'paragraph' as const,
        rf_text: item,
        rf_level: 2,
        rf_ordered: false,
        rf_items: [],
      }));
    } else {
      replacements = [{
        rf_id: crypto.randomUUID(),
        rf_type: 'paragraph' as const,
        rf_text: block.rf_text || block.rf_caption || '',
        rf_level: 2,
        rf_ordered: false,
        rf_items: [],
      }];
    }
  }
  if (doc.rf_blocks.length - 1 + replacements.length > 256) return doc;
  const updated = [...doc.rf_blocks];
  updated.splice(index, 1, ...replacements);
  return {
    rf_schema: DOCUMENT_SCHEMA,
    rf_blocks: updated,
  };
}
