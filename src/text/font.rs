use std::{collections, sync};

use super::span;

#[derive(Default, Clone)]
pub struct FontSet {
    fonts: sync::Arc<sync::Mutex<collections::HashMap<String, sync::Arc<fontdue::Font>>>>,
}

impl FontSet {
    pub fn new() -> Self {
        Self {
            fonts: Default::default(),
        }
    }

    pub fn add_font(&mut self, name: &str, data: &[u8]) {
        let Ok(font) = fontdue::Font::from_bytes(data, fontdue::FontSettings::default()) else {
            return;
        };

        let Ok(mut fonts) = self.fonts.lock() else {
            return;
        };
        fonts.insert(name.to_owned(), sync::Arc::new(font));
    }

    pub fn new_span(&self, name: &str) -> Option<span::Span> {
        let Ok(fonts) = self.fonts.lock() else {
            return None;
        };
        let font = fonts.get(name)?;

        Some(span::Span::new(font.clone()))
    }
}
