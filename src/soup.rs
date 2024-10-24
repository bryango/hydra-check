use std::fmt::Debug;

use anyhow::anyhow;
use scraper::{selectable::Selectable, ElementRef, Selector};

/// A simple wrapper Trait that provides the `find` and `find_all` methods
/// to [`scraper`]'s [`Selectable`] elements, inspired by the interface of
/// Python's BeautifulSoup.
pub trait SoupFind<'a> {
    /// Finds all child elements matching the CSS selectors
    /// and collect them into a [`Vec`]
    fn find_all(self, selectors: &str) -> Vec<ElementRef<'a>>;

    /// Finds the first element that matches the CSS selectors,
    /// returning [`None`] if not found.
    fn find(self, selectors: &str) -> anyhow::Result<ElementRef<'a>>;
}

impl<'a, T: Selectable<'a> + Debug> SoupFind<'a> for T {
    fn find_all(self, selectors: &str) -> Vec<ElementRef<'a>> {
        let selector = Selector::parse(selectors).expect("the selector should be valid");
        self.select(&selector).collect()
    }

    fn find(self, selectors: &str) -> anyhow::Result<ElementRef<'a>> {
        let selector = Selector::parse(selectors).expect("the selector should be valid");
        let err = anyhow!("could not select '{:?}' in '{:?}'", selector, self);
        let element = self.select(&selector).next().ok_or(err)?;
        Ok(element)
    }
}
