export interface ColumnDefinition {
  id: string; label: string; visible: boolean; width?: string; sortable?: boolean; isCustomField?: boolean;
}
export function validColumns(value: unknown): ColumnDefinition[] | null {
  if (!Array.isArray(value) || value.length === 0 || value.length > 100) return null;
  const ids = new Set<string>();
  for (const col of value) {
    if (!col || typeof col !== 'object' || typeof col.id !== 'string' || !/^[a-zA-Z0-9_:-]{1,100}$/.test(col.id)
      || ids.has(col.id) || typeof col.label !== 'string' || col.label.length > 120 || typeof col.visible !== 'boolean'
      || (col.width !== undefined && (typeof col.width !== 'string' || !/^(auto|[0-9]{1,4}px|[0-9]{1,3}%)$/.test(col.width)))) return null;
    ids.add(col.id);
  }
  return value.map(c => ({id:c.id,label:c.label,visible:c.visible,width:c.width,sortable:c.sortable === true,isCustomField:c.isCustomField === true}));
}
export function csvCell(value: unknown): string {
  let text = String(value ?? '');
  // Les caractères de contrôle initiaux peuvent masquer une formule aux tableurs.
  let start = 0;
  while (start < text.length && (text.charCodeAt(start) <= 32 || /\s/.test(text[start]))) start++;
  if ('=+@-'.includes(text[start] || '\uFFFF') || /^[\t\r\n]/.test(text)) text = "'" + text;
  return '"' + text.replace(/"/g, '""') + '"';
}
