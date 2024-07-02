use std::{
    ffi::OsStr,
    hash::{Hash, Hasher},
};

use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt},
    log::warn,
    prelude::Asset,
    reflect::TypePath,
    utils::{AHasher, HashMap},
};
use grass::InputSyntax;
use smallvec::SmallVec;
use thiserror::Error;

use crate::{parser::StyleSheetParser, property::PropertyValues, selector::Selector};

#[derive(Debug, TypePath, Asset)]
/// A cascading style sheet (`css`) asset file.
///
/// _Note_: This asset only store intermediate data, like rules and properties.
/// The parsing to final ECS component values is done by a internal `exclusive_system` and is
/// cached on [`Local`](bevy::prelude::Local) resources, which isn't possible to get outside the system.
pub struct StyleSheetAsset {
    path: String,
    hash: u64,
    rules: SmallVec<[StyleRule; 8]>,
}

impl StyleSheetAsset {
    /// Parses a string with a valid CSS into a list of [`StyleRule`]s.
    ///
    /// This used by internal asset loader to keep track of where each asset came from.
    /// If you are creating this struct by hand, you can safely supply an  empty string as path.
    pub fn parse(path: &str, content: &str) -> Self {
        let mut hasher = AHasher::default();
        content.hash(&mut hasher);
        let hash = hasher.finish();

        Self {
            path: path.to_string(),
            hash,
            rules: StyleSheetParser::parse(content),
        }
    }

    /// Returns the [`PropertyValues`] on the given [`Selector`] with the given name.
    pub fn get_properties(&self, selector: &Selector, name: &str) -> Option<&PropertyValues> {
        self.rules
            .iter()
            .find(|&rule| &rule.selector == selector)
            .and_then(|rule| rule.properties.get(name))
    }

    /// Iterates over all existing rules
    pub fn iter(&self) -> impl Iterator<Item = &StyleRule> {
        self.rules.iter()
    }

    /// Internal hash computed from content and used for equality and ordering comparison
    pub fn hash(&self) -> u64 {
        self.hash
    }

    /// Asset path, for debug reasons only
    pub fn path(&self) -> &str {
        &self.path
    }
}

/// Represents a single rule inside a style sheet with a [`Selector`] which determines which entities
/// should be applied the [`PropertyValues`].
///
/// Note that this struct holds intermediate data, the final value is parsed by [`Property`](crate::Property) on
/// the first time it's [`system`](crate::Property::apply_system) is invoked.
#[derive(Debug, Clone)]
pub struct StyleRule {
    /// Selector used to match entities to apply properties.
    pub selector: Selector,
    /// Properties values to be applied on selected entities.
    pub properties: HashMap<String, PropertyValues>,
}

#[derive(Default)]
pub(crate) struct StyleSheetLoader;

#[derive(Debug, Error)]
pub enum StyleSheetLoaderError {
    #[error("File not found: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid file format: {0}")]
    UTF8(#[from] std::str::Utf8Error),
    #[cfg(feature = "sass")]
    #[error("Could not compile sass: {0}")]
    SASSError(#[from] Box<grass::Error>),
}

impl AssetLoader for StyleSheetLoader {
    type Asset = StyleSheetAsset;
    type Settings = ();
    type Error = StyleSheetLoaderError;

    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _settings: &'a (),
        load_context: &'a mut bevy::asset::LoadContext<'_>,
    ) -> Result<StyleSheetAsset, StyleSheetLoaderError> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let content = std::str::from_utf8(&bytes)?;
        let stylesheet =
            StyleSheetAsset::parse(load_context.path().to_str().unwrap_or_default(), content);
        Ok(stylesheet)
    }

    fn extensions(&self) -> &[&str] {
        &["css"]
    }
}

#[derive(Default)]
#[cfg(feature = "sass")]
pub(crate) struct SCSSLoader;

#[cfg(feature = "sass")]
impl AssetLoader for SCSSLoader {
    type Asset = StyleSheetAsset;
    type Settings = ();
    type Error = StyleSheetLoaderError;

    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _settings: &'a Self::Settings,
        load_context: &'a mut bevy::asset::LoadContext<'_>,
    ) -> Result<StyleSheetAsset, StyleSheetLoaderError> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let content = std::str::from_utf8(&bytes)?;
        let input_syntax = match load_context.path().extension().and_then(OsStr::to_str) {
            Some("scss") => InputSyntax::Scss,
            Some("sass") => InputSyntax::Sass,
            Some("css") => InputSyntax::Css,
            _ => {
                warn!(
                    "Could not determine sass type for {}",
                    load_context.path().display()
                );
                InputSyntax::Scss
            }
        };
        let css = grass::from_string(
            content,
            &grass::Options::default().input_syntax(input_syntax),
        )?;
        let stylesheet =
            StyleSheetAsset::parse(load_context.path().to_str().unwrap_or_default(), &css);
        Ok(stylesheet)
    }

    fn extensions(&self) -> &[&str] {
        &["scss", "sass"]
    }
}
