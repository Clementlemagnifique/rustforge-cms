use crate::{messages::text, navigation::MenuNode};
use leptos::prelude::*;
#[component]
pub fn Catalog(#[prop(optional)] module: Option<&'static str>) -> impl IntoView {
    view! {<details class="rf-panel rf-screen-panel rf-layout-options"><summary>{text("blocks.catalog")}</summary><p class="rf-description">{text("blocks.hint")}</p><div class="rf-screen-grid">{super::PLANNED.iter().filter(move |block|module.is_none_or(|module|module==block.module)).map(|block|view!{<article><h3>{text(block.title)}" — "{if block.id=="form_embed"{text("form_block.partial")}else{text("planning")}}</h3><ul>{block.options.iter().map(|key|view!{<li>{text(key)}</li>}).collect_view()}</ul><p class="rf-description">{text("blocks.reference")}" "{block.reference}</p>{if block.id=="form_embed"{view!{<p>{text("form_block.available")}</p>}.into_any()}else{view!{<button class="rf-button" disabled>{text("blocks.insert_later")}</button>}.into_any()}}</article>}).collect_view()}</div></details>}
}
#[component]
pub fn FutureOptions(node: MenuNode) -> impl IntoView {
    view! {<details class="rf-panel rf-screen-panel rf-layout-options"><summary>{text("blocks.future_options")}</summary><p>{text("blocks.future_hint")}</p><ul class="rf-section-links">{node.children.into_iter().map(|child|{let href=child.route.clone().map(|route|format!("#{route}"));view!{<li><div>{if let Some(href)=href{view!{<a href=href>{child.label}</a>}.into_any()}else{view!{<span>{child.label}</span>}.into_any()}}<ul>{child.details.features.into_iter().map(|feature|view!{<li>{feature}</li>}).collect_view()}</ul></div><span class="rf-description">{if child.planning{text("planning")}else{text("blocks.initial_or_transition")}}</span></li>}}).collect_view()}</ul></details>}
}

#[component]
pub fn Adjustments() -> impl IntoView {
    view! {<details class="rf-panel rf-screen-panel rf-layout-options"><summary>{text("adjustment.title")}</summary><p>{text("adjustment.hint")}<a href="https://meowapps.com" target="_blank" rel="noopener noreferrer">"Meow Apps"</a></p><ul>{super::FUTURE_ADJUSTMENTS.iter().map(|(title,options)|view!{<li><h3>{text(title)}" — "{text("planning")}</h3><p>{text(options)}</p></li>}).collect_view()}</ul></details>}
}
