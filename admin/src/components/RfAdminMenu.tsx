/** Shared server registry; preserves the existing admin shell and planned capabilities. */
import { useState } from 'react';
import type { ComponentType } from 'react';
import { Gauge, Pin, Image as ImageIcon, FileText, MessageSquare, Sparkles,
  GraduationCap, Palette, Plug, Users, Wrench, Settings, BarChart3, Globe,
  Images, ShoppingBag, Mail, Calendar, Workflow, ChevronLeft, ChevronRight } from 'lucide-react';
import type { AdminMenuItem, ContentType } from '../types';
import { menuTree, menuTrail } from '../navigation';
import type { MenuBranch } from '../navigation';
const icons: Record<string,ComponentType<{size?:number;className?:string}>> = {
  rf_dashboard:Gauge,rf_posts:Pin,rf_pages:FileText,rf_media:ImageIcon,rf_galleries:Images,
  rf_comments:MessageSquare,rf_ai:Sparkles,rf_lms:GraduationCap,rf_commerce:ShoppingBag,
  rf_forms:FileText,rf_appearance:Palette,rf_modules:Plug,rf_users:Users,rf_tools:Wrench,
  rf_settings:Settings,rf_analytics:BarChart3,rf_translations:Globe,rf_mail:Mail,
  rf_events:Calendar,rf_automations:Workflow,rf_navigation:Workflow,
};
interface Props {
  currentView:string; onNavigate:(route:string)=>void; contentTypes:ContentType[];
  items:AdminMenuItem[]; isCollapsed:boolean; onToggleCollapsed:()=>void;
}
function label(item:AdminMenuItem) {
  return item.planning ? `${item.label} · À préparer` : item.label;
}
function Submenu({nodes,currentView,onNavigate,depth=0}:{nodes:MenuBranch[];currentView:string;onNavigate:(route:string)=>void;depth?:number}) {
  return <ul className="list-none m-0 p-0">{nodes.map(({item,children})=><li key={item.id}>
    {item.route ? <button type="button" onClick={()=>onNavigate(item.route!)} aria-current={currentView===item.route?'page':undefined}
      style={{paddingLeft:depth?24:16}} className={`w-full text-left pr-3 py-2 text-xs block leading-snug hover:bg-[#2271b1] hover:text-white focus-visible:outline-2 focus-visible:outline-white ${currentView===item.route?'text-white font-semibold':'text-[#c3c4c7]'}`}>{label(item)}</button>
      : <span className="block px-4 py-2 text-xs text-slate-400">{label(item)}</span>}
    {children.length>0 && <Submenu nodes={children} currentView={currentView} onNavigate={onNavigate} depth={depth+1}/>}
  </li>)}</ul>;
}
export function RfAdminMenu({currentView,onNavigate,contentTypes,items,isCollapsed,onToggleCollapsed}:Props) {
  const [hoveredId,setHoveredId]=useState<string|null>(null);
  const tree=menuTree(items);
  const trail=menuTrail(items,currentView);
  const customTypes=contentTypes.filter(ct=>!['post','page'].includes(ct.slug));
  return <aside id="rfadminmenu" role="navigation" aria-label="Menu principal d’administration"
    className={`fixed top-8 bottom-0 left-0 z-40 bg-[#1d2327] border-r border-[#2c3338] select-none flex flex-col text-[13px] font-sans ${isCollapsed?'w-14':'w-44 md:w-48'}`}>
    <div className="rf-menu-scroll flex-1 overflow-y-auto overflow-x-hidden py-2 space-y-0.5">{tree.map(({item,children})=>{
      const Icon=icons[item.id]||FileText;
      const active=trail.some(node=>node.id===item.id);
      const primary=item.route||children.find(node=>node.item.route)?.item.route;
      return <div key={item.id} data-separator-before={item.separator_before || undefined} className={`rf-menu-root relative ${active?'rf-current-parent':''}`} onMouseEnter={()=>setHoveredId(item.id)} onMouseLeave={()=>setHoveredId(null)}
        onFocus={()=>setHoveredId(item.id)} onBlur={event=>{if(!event.currentTarget.contains(event.relatedTarget))setHoveredId(null);}}>
        <button type="button" title={label(item)} aria-label={label(item)} aria-current={active?'page':undefined}
          onClick={()=>{if(primary)onNavigate(primary);}}
          className={`w-full flex items-center gap-2.5 px-3 py-2 text-left transition-colors relative group focus-visible:outline-2 focus-visible:outline-white ${active?'bg-[#2271b1] text-white font-semibold':'text-[#c3c4c7] hover:bg-[#131619] hover:text-[#72aee6]'}`}>
          <Icon size={18} className="shrink-0"/>{!isCollapsed&&<span className="min-w-0 leading-tight">{item.label}{item.planning&&<span className="block text-[10px] font-normal text-slate-300">À préparer</span>}</span>}
        </button>
        {children.length>0&&active&&<div className="rf-menu-inline bg-[#131619] py-1"><Submenu nodes={children} currentView={currentView} onNavigate={onNavigate}/></div>}
        {children.length>0&&hoveredId===item.id&&(isCollapsed||!active)&&<div className={`rf-menu-flyout fixed ${isCollapsed?'left-14':'left-44 md:left-48'} w-64 max-h-[70vh] overflow-y-auto bg-[#131619] shadow-2xl py-1 border border-white/10 z-50`} style={{marginTop:'-36px'}}>
          <div className="px-4 py-2 font-semibold text-white border-b border-white/10">{item.label}</div>
          <Submenu nodes={children} currentView={currentView} onNavigate={route=>{setHoveredId(null);onNavigate(route);}}/>
        </div>}
      </div>;
    })}
    {!isCollapsed&&customTypes.length>0&&<div className="mt-3 pt-2 border-t border-[#2c3338] px-3"><span className="text-[11px] text-slate-400">Contenus personnalisés</span>{customTypes.map(ct=><button key={ct.id} type="button" onClick={()=>onNavigate(`entries:${ct.slug}`)} className="block w-full text-left py-2 text-xs text-[#c3c4c7]">{ct.name}</button>)}</div>}
    </div>
    <div className="rf-menu-footer border-t border-[#2c3338] p-1.5"><button type="button" onClick={onToggleCollapsed} aria-expanded={!isCollapsed} title={isCollapsed?'Déployer le menu latéral':'Replier le menu latéral'} className="w-full flex gap-2 items-center p-2 text-slate-300 text-xs">{isCollapsed?<ChevronRight size={16}/>:<><ChevronLeft size={16}/><span>Replier le menu</span></>}</button></div>
  </aside>;
}
