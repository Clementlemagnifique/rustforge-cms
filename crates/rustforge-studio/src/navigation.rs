//! Projection typée du registre serveur déjà filtré par permissions et modules.
use serde::Deserialize;
use std::collections::HashSet;

#[derive(Clone, Debug, Deserialize)]
pub struct MenuItem {
    pub id: String,
    pub parent: Option<String>,
    pub label: String,
    pub route: Option<String>,
    #[serde(default)]
    pub planning: bool,
    #[serde(default)]
    pub separator_before: bool,
    #[serde(flatten)]
    pub details: MenuDetails,
}
#[derive(Clone, Debug, Default, Deserialize)]
pub struct MenuDetails {
    #[serde(default)]
    pub features: Vec<String>,
    #[serde(default)]
    pub tasks: Vec<String>,
}
#[derive(Clone, Debug)]
pub struct MenuNode {
    pub id: String,
    pub label: String,
    pub route: Option<String>,
    pub children: Vec<MenuNode>,
    pub planning: bool,
    pub separator_before: bool,
    pub details: MenuDetails,
}
impl MenuNode {
    pub fn contains_route(&self, route: &str) -> bool {
        self.route.as_deref() == Some(route)
            || crate::editor::Location::parse(route).is_some_and(|location| {
                self.route.as_deref() == Some(format!("entries:{}", location.content_type).as_str())
            })
            || (route
                .strip_prefix("media:")
                .is_some_and(crate::editor::local_id)
                && self.route.as_deref() == Some("media"))
            || (route
                .strip_prefix("user:")
                .is_some_and(crate::editor::local_id)
                && self.route.as_deref() == Some("users"))
            || self
                .children
                .iter()
                .any(|child| child.contains_route(route))
    }
}
/// Accepte uniquement des routes locales, sans URL, paramètre ou code arbitraire.
pub fn route_from_hash(hash: &str) -> Option<String> {
    let input = hash.strip_prefix('#').unwrap_or(hash);
    if input.is_empty() {
        return Some("dashboard".into());
    }
    if input.len() > 768 {
        return None;
    }
    let mut decoded = Vec::new();
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            let pair = std::str::from_utf8(bytes.get(i + 1..i + 3)?).ok()?;
            decoded.push(u8::from_str_radix(pair, 16).ok()?);
            i += 3;
        } else {
            decoded.push(bytes[i]);
            i += 1;
        }
    }
    let route = String::from_utf8(decoded).ok()?;
    if route.len() > 256
        || !route
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_:-".contains(&b))
    {
        return None;
    }
    Some(route)
}
pub fn project(items: Vec<MenuItem>) -> Result<Vec<MenuNode>, &'static str> {
    if items.len() > 1000 {
        return Err("Navigation trop volumineuse");
    }
    let mut ids = HashSet::new();
    for item in &items {
        if item.id.is_empty()
            || item.id.len() > 100
            || !ids.insert(item.id.clone())
            || item.label.len() > 512
        {
            return Err("Identifiant de navigation invalide ou dupliqué");
        }
        if item
            .route
            .as_ref()
            .is_some_and(|r| r.is_empty() || route_from_hash(r).as_ref() != Some(r))
        {
            return Err("Route de navigation invalide");
        }
    }
    for item in &items {
        let mut parent = item.parent.as_deref();
        let mut depth = 1;
        while let Some(id) = parent {
            depth += 1;
            if depth > 3 {
                return Err("Navigation cyclique ou trop profonde");
            }
            parent = items
                .iter()
                .find(|i| i.id == id)
                .ok_or("Parent de navigation absent")?
                .parent
                .as_deref();
        }
    }
    fn children(items: &[MenuItem], parent: Option<&str>) -> Vec<MenuNode> {
        items
            .iter()
            .filter(|i| i.parent.as_deref() == parent)
            .map(|item| MenuNode {
                id: item.id.clone(),
                label: item.label.clone(),
                route: item.route.clone(),
                planning: item.planning,
                separator_before: item.separator_before,
                details: item.details.clone(),
                children: children(items, Some(&item.id)),
            })
            .collect()
    }
    Ok(children(&items, None))
}
#[cfg(test)]
mod tests {
    use super::*;
    fn node(id: &str, parent: Option<&str>, route: Option<&str>) -> MenuItem {
        MenuItem {
            id: id.into(),
            parent: parent.map(str::to_owned),
            route: route.map(str::to_owned),
            label: id.into(),
            planning: false,
            separator_before: false,
            details: MenuDetails::default(),
        }
    }
    #[test]
    fn keeps_groups_and_three_levels_in_server_order() {
        let tree = project(vec![
            node("tools", None, None),
            node("settings", Some("tools"), Some("settings")),
            node("reading", Some("settings"), Some("settings:reading")),
        ])
        .unwrap();
        assert_eq!(tree[0].id, "tools");
        assert_eq!(tree[0].children[0].children[0].id, "reading");
        assert!(tree[0].contains_route("settings:reading"));
    }
    #[test]
    fn preserves_separators_and_planning_details() {
        let mut item = node("forms", None, Some("plan:rf_forms"));
        item.separator_before = true;
        item.planning = true;
        item.details.features = vec!["Soumissions et notifications".into()];
        item.details.tasks = vec!["MOD-FORM-01".into()];
        let tree = project(vec![item]).unwrap();
        assert!(tree[0].separator_before && tree[0].planning);
        assert_eq!(tree[0].details.features[0], "Soumissions et notifications");
        assert_eq!(tree[0].details.tasks[0], "MOD-FORM-01");
    }
    #[test]
    fn refuses_cycles_orphans_and_external_routes() {
        for items in [
            vec![node("a", Some("a"), None)],
            vec![node("a", Some("b"), None)],
            vec![node("a", None, Some("https://evil.test"))],
            vec![node("a", None, None), node("a", None, None)],
        ] {
            assert!(project(items).is_err());
        }
    }
    #[test]
    fn parses_history_hashes_without_accepting_script_or_query() {
        assert_eq!(
            route_from_hash("#settings%3Areading").as_deref(),
            Some("settings:reading")
        );
        assert_eq!(route_from_hash("").as_deref(), Some("dashboard"));
        for hash in [
            "#javascript:alert(1)",
            "#//evil",
            "#entries?token=x",
            "#x%",
            "#%00",
            "#%C3%A9",
        ] {
            assert!(route_from_hash(hash).is_none());
        }
    }
}

#[cfg(test)]
mod registry_contract_tests {
    #[test]
    fn accepts_the_real_server_registry_without_dropping_groups() {
        let items: Vec<super::MenuItem> = serde_json::from_str(include_str!(
            "../../rustforge-api/src/admin_navigation.json"
        ))
        .unwrap();
        let total = items.len();
        let tree = super::project(items).unwrap();
        fn count(nodes: &[super::MenuNode]) -> usize {
            nodes.iter().map(|n| 1 + count(&n.children)).sum()
        }
        assert_eq!(count(&tree), total);
    }
}
