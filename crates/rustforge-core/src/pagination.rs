use serde::{Deserialize, Serialize};

/// Paramètres de pagination communs (`?page=1&per_page=20`).
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct PageParams {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_per_page")]
    pub per_page: i64,
}

fn default_page() -> i64 {
    1
}
fn default_per_page() -> i64 {
    20
}

impl Default for PageParams {
    fn default() -> Self {
        Self { page: 1, per_page: 20 }
    }
}

impl PageParams {
    pub fn new(page: Option<i64>, per_page: Option<i64>) -> Self {
        Self { page: page.unwrap_or(1), per_page: per_page.unwrap_or(20) }.normalized()
    }

    pub fn normalized(self) -> Self {
        Self { page: self.page.max(1), per_page: self.per_page.clamp(1, 100) }
    }

    pub fn limit(&self) -> i64 {
        self.per_page
    }

    pub fn offset(&self) -> i64 {
        (self.page - 1) * self.per_page
    }
}

/// Page de résultats.
#[derive(Debug, Serialize)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub total_pages: i64,
}

impl<T> Page<T> {
    pub fn new(items: Vec<T>, total: i64, params: PageParams) -> Self {
        let total_pages = if total == 0 { 0 } else { (total + params.per_page - 1) / params.per_page };
        Self { items, total, page: params.page, per_page: params.per_page, total_pages }
    }

    pub fn map<U>(self, f: impl FnMut(T) -> U) -> Page<U> {
        Page {
            items: self.items.into_iter().map(f).collect(),
            total: self.total,
            page: self.page,
            per_page: self.per_page,
            total_pages: self.total_pages,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pagination_math() {
        let p = PageParams { page: 3, per_page: 10 }.normalized();
        assert_eq!(p.offset(), 20);
        let page = Page::new(vec![1, 2], 21, p);
        assert_eq!(page.total_pages, 3);
        assert_eq!(PageParams { page: 0, per_page: 1000 }.normalized().per_page, 100);
    }
}
