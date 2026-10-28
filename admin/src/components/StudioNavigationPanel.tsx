import { useEffect, useRef } from 'react';
import type { ReactNode } from 'react';
import { studioMessages } from '../i18n/messages.fr';

/** Le tiroir mobile garde le clavier dans le menu et rend le focus à son déclencheur. */
export function StudioNavigationPanel({ open, onClose, children }: {
  open: boolean;
  onClose: () => void;
  children: ReactNode;
}) {
  const panel = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    const desktop = window.matchMedia('(min-width: 768px)');
    if (desktop.matches) { onClose(); return; }
    const previous = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    const focusable = () => Array.from(panel.current?.querySelectorAll<HTMLElement>(
      'button:not(:disabled), a[href], input:not(:disabled), select:not(:disabled), textarea:not(:disabled), [tabindex="0"]'
    ) ?? []).filter(element => element.getClientRects().length > 0);
    focusable()[0]?.focus();
    const handleKey = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        event.preventDefault();
        onClose();
      } else if (event.key === 'Tab') {
        const elements = focusable();
        const first = elements[0];
        const last = elements[elements.length - 1];
        if (event.shiftKey && document.activeElement === first) {
          event.preventDefault(); last?.focus();
        } else if (!event.shiftKey && document.activeElement === last) {
          event.preventDefault(); first?.focus();
        }
      }
    };
    const handleResize = () => { if (desktop.matches) onClose(); };
    document.addEventListener('keydown', handleKey);
    desktop.addEventListener('change', handleResize);
    return () => {
      document.removeEventListener('keydown', handleKey);
      desktop.removeEventListener('change', handleResize);
      previous?.focus();
    };
  }, [open, onClose]);

  return <div ref={panel} role={open ? 'dialog' : undefined} aria-modal={open || undefined}
    aria-label={open ? studioMessages.mobileNavigationLabel : undefined}
    className={`${open ? 'fixed inset-0 z-40' : 'hidden'} md:block shrink-0`}>
    {open && <button aria-label={studioMessages.closeMobileMenu} onClick={onClose}
      className="fixed inset-0 bg-black/60 md:hidden" />}
    <div className={open ? 'relative w-fit' : undefined}>{children}</div>
  </div>;
}
