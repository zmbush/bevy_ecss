use std::any::Any;

use bevy::{
    ecs::query::{QueryData, QueryFilter, QueryItem},
    log::{error, trace},
    prelude::{
        AssetId, AssetServer, Assets, Color, Commands, Deref, DerefMut, Entity, Local, Query, Res,
        Resource,
    },
    ui::{
        GridPlacement, GridTrack, GridTrackRepetition, MaxTrackSizingFunction,
        MinTrackSizingFunction, RepeatedGridTrack, UiRect, Val,
    },
    utils::HashMap,
};

use cssparser::Token;
use smallvec::SmallVec;

use crate::{parser::ParsedToken, selector::Selector, EcssError, SelectorElement, StyleSheetAsset};

mod colors;
pub mod impls;

/// A property value token which was parsed from a CSS rule.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum PropertyToken {
    /// A value which was parsed percent value, like `100%` or `73.23%`.
    Percentage(f32),
    /// A value which was parsed dimension value, like `10px` or `35em.
    ///
    /// Currently there is no distinction between [`length-values`](https://developer.mozilla.org/en-US/docs/Web/CSS/length)
    /// except for `vmin`, `vmax`, `vh` and `vw`.
    Dimension(f32),
    /// A minimum viewport axis value like `10vmin`
    VMin(f32),
    /// A maximum viewport axis value like `10vmax`
    VMax(f32),
    /// A viewport height value like `10vh`
    Vh(f32),
    /// A viewport width value like `10vw`
    Vw(f32),
    /// A Fraction of a grid like `1fr`
    Fr(f32),
    /// A numeric float value, like `31.1` or `43`.
    Number(f32),
    /// A plain identifier, like `none` or `center`.
    Identifier(String),
    /// A identifier prefixed by a hash, like `#001122`.
    Hash(String),
    /// A quoted string, like `"some value"`.
    String(String),
    /// A Function name
    Function(String, Vec<PropertyToken>),
    /// A Literal `/`
    Slash,
}

/// A list of [`PropertyToken`] which was parsed from a single property.
#[derive(Debug, Default, Clone, Deref)]
pub struct PropertyValues(pub(crate) SmallVec<[PropertyToken; 8]>);

impl PropertyValues {
    /// Tries to parses the current values as a single [`String`].
    pub fn string(&self) -> Option<String> {
        self.0.iter().find_map(|token| match token {
            PropertyToken::String(id) => {
                if id.is_empty() {
                    None
                } else {
                    Some(id.clone())
                }
            }
            _ => None,
        })
    }

    /// Tries to parses the current values as a single [`Color`].
    ///
    /// Currently only [named colors](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color)
    /// and [hex-colors](https://developer.mozilla.org/en-US/docs/Web/CSS/hex-color) are supported.
    pub fn color(&self) -> Option<Color> {
        if self.0.len() == 1 {
            match &self.0[0] {
                PropertyToken::Identifier(name) => colors::parse_named_color(name.as_str()),
                PropertyToken::Hash(hash) => colors::parse_hex_color(hash.as_str()),
                _ => None,
            }
        } else {
            // TODO: Implement color function like rgba(255, 255, 255, 255)
            // https://developer.mozilla.org/en-US/docs/Web/CSS/color_value
            None
        }
    }

    /// Tries to parses the current values as a single identifier.
    pub fn identifier(&self) -> Option<&str> {
        self.0.iter().find_map(|token| match token {
            PropertyToken::Identifier(id) => {
                if id.is_empty() {
                    None
                } else {
                    Some(id.as_str())
                }
            }
            _ => None,
        })
    }

    /// Tries to parses the current values as a single [`Val`].
    ///
    /// Only [`Percentage`](PropertyToken::Percentage) and [`Dimension`](PropertyToken::Dimension`) are considered valid values,
    /// where former is converted to [`Val::Percent`] and latter is converted to [`Val::Px`].
    pub fn val(&self) -> Option<Val> {
        self.0.iter().find_map(|token| match token {
            PropertyToken::Percentage(val) => Some(Val::Percent(*val)),
            PropertyToken::Dimension(val) => Some(Val::Px(*val)),
            PropertyToken::VMin(val) => Some(Val::VMin(*val)),
            PropertyToken::VMax(val) => Some(Val::VMax(*val)),
            PropertyToken::Vh(val) => Some(Val::Vh(*val)),
            PropertyToken::Vw(val) => Some(Val::Vw(*val)),
            PropertyToken::Identifier(val) if val == "auto" => Some(Val::Auto),
            _ => None,
        })
    }

    pub fn grid_template(&self) -> Option<Vec<RepeatedGridTrack>> {
        Some(
            self.0
                .iter()
                .filter_map(|token| match token {
                    PropertyToken::Percentage(val) => Some(GridTrack::percent(*val)),
                    PropertyToken::Dimension(val) => Some(GridTrack::px(*val)),
                    PropertyToken::Fr(val) => Some(GridTrack::fr(*val)),
                    PropertyToken::Identifier(val) if val == "auto" => Some(GridTrack::auto()),
                    PropertyToken::Function(fun, args) if fun == "repeat" => {
                        if args.len() != 2 {
                            error!("Expected 2 arguments to repeat");
                            return None;
                        }
                        let repeat = GridTrackRepetition::try_from(&args[0]).ok()?;

                        match &args[1] {
                            PropertyToken::Percentage(val) => {
                                Some(RepeatedGridTrack::percent(repeat, *val))
                            }
                            PropertyToken::Dimension(val) => {
                                Some(RepeatedGridTrack::px(repeat, *val))
                            }
                            PropertyToken::Fr(val) => {
                                if let GridTrackRepetition::Count(repeat) = repeat {
                                    Some(RepeatedGridTrack::fr(repeat, *val))
                                } else {
                                    error!(
                                "fr based repeats must have a count, not auto-fit, or auto-fill"
                            );
                                    None
                                }
                            }
                            PropertyToken::Identifier(val) if val == "auto" => {
                                if let GridTrackRepetition::Count(repeat) = repeat {
                                    Some(RepeatedGridTrack::auto(repeat))
                                } else {
                                    error!(
                                "auto based repeats must have a count, not auto-fit, or auto-fill"
                            );
                                    None
                                }
                            }
                            _ => {
                                error!("Could not determine second argument to repeat");
                                None
                            }
                        }
                    }
                    PropertyToken::Function(fun, args) if fun == "fit-content" => {
                        if args.len() != 1 {
                            error!("Expected 1 arguments to fit-content");
                            return None;
                        }
                        match &args[0] {
                            PropertyToken::Dimension(val) => Some(GridTrack::fit_content_px(*val)),
                            PropertyToken::Percentage(val) => {
                                Some(GridTrack::fit_content_percent(*val))
                            }
                            _ => {
                                error!("fit-content only accepts px or percent");
                                None
                            }
                        }
                    }
                    PropertyToken::Function(fun, args) if fun == "minmax" => {
                        if args.len() != 2 {
                            error!("Expected 2 arguments to minmax");
                            return None;
                        }
                        Some(GridTrack::minmax(
                            MinTrackSizingFunction::try_from(&args[0]).ok()?,
                            MaxTrackSizingFunction::try_from(&args[1]).ok()?,
                        ))
                    }
                    _ => None,
                })
                .collect(),
        )
    }

    pub fn grid_placement(&self) -> Option<GridPlacement> {
        use PropertyToken::*;
        match &self.0[..] {
            [Number(start)] => Some(GridPlacement::start(*start as i16)),
            [Number(start), Slash, Number(end)] => {
                Some(GridPlacement::start_end(*start as i16, *end as i16))
            }

            [Identifier(start), Slash, Number(end)] if start == "auto" => {
                Some(GridPlacement::end(*end as i16))
            }
            [Number(start), Slash, Identifier(end)] if end == "auto" => {
                Some(GridPlacement::start(*start as i16))
            }
            [Identifier(start)] if start == "auto" => Some(GridPlacement::auto()),
            [Identifier(start), Slash, Identifier(end)] if start == "auto" && end == "auto" => {
                Some(GridPlacement::auto())
            }

            [Identifier(id), Number(span)] if id == "span" => {
                Some(GridPlacement::span(*span as u16))
            }
            [Identifier(id), Number(span), Slash, Number(end)] if id == "span" => {
                Some(GridPlacement::end_span(*end as i16, *span as u16))
            }
            [Number(start), Slash, Identifier(id), Number(span)] if id == "span" => {
                Some(GridPlacement::start_span(*start as i16, *span as u16))
            }
            _ => None,
        }
    }

    /// Tries to parses the current values as a single [`f32`].
    ///
    /// Only [`Percentage`](PropertyToken::Percentage), [`Dimension`](PropertyToken::Dimension`) and [`Number`](PropertyToken::Number`)
    /// are considered valid values.
    pub fn f32(&self) -> Option<f32> {
        self.0.iter().find_map(|token| match token {
            PropertyToken::Percentage(val)
            | PropertyToken::Dimension(val)
            | PropertyToken::Number(val) => Some(*val),
            _ => None,
        })
    }

    /// Tries to parses the current values as a single [`Option<f32>`].
    ///
    /// This function is useful for properties where either a numeric value or a `none` value is expected.
    ///
    /// If a [`Option::None`] is returned, it means some invalid value was found.
    ///
    /// If there is a [`Percentage`](PropertyToken::Percentage), [`Dimension`](PropertyToken::Dimension`) or [`Number`](PropertyToken::Number`) token,
    /// a [`Option::Some`] with parsed [`Option<f32>`] is returned.
    /// If there is a identifier with a `none` value, then [`Option::Some`] with [`None`] is returned.
    pub fn option_f32(&self) -> Option<Option<f32>> {
        self.0.iter().find_map(|token| match token {
            PropertyToken::Percentage(val)
            | PropertyToken::Dimension(val)
            | PropertyToken::Number(val) => Some(Some(*val)),
            PropertyToken::Identifier(ident) => match ident.as_str() {
                "none" => Some(None),
                _ => None,
            },
            _ => None,
        })
    }

    /// Tries to parses the current values as a single [`Option<UiRect<Val>>`].
    ///
    /// Optional values are handled by this function, so if only one value is present it is used as `top`, `right`, `bottom` and `left`,
    /// otherwise values are applied in the following order: `top`, `right`, `bottom` and `left`.
    ///
    /// Note that it is not possible to create a [`UiRect`] with only `top` value, since it'll be understood to replicated it on all fields.
    pub fn rect(&self) -> Option<UiRect> {
        if self.0.len() == 1 {
            self.val().map(UiRect::all)
        } else {
            self.0
                .iter()
                .fold((None, 0), |(rect, idx), token| {
                    let val = match token {
                        PropertyToken::Percentage(val) => Val::Percent(*val),
                        PropertyToken::Dimension(val) => Val::Px(*val),
                        PropertyToken::VMin(val) => Val::VMin(*val),
                        PropertyToken::VMax(val) => Val::VMax(*val),
                        PropertyToken::Vh(val) => Val::Vh(*val),
                        PropertyToken::Vw(val) => Val::Vw(*val),
                        PropertyToken::Identifier(val) if val == "auto" => Val::Auto,
                        _ => return (rect, idx),
                    };
                    let mut rect: UiRect = rect.unwrap_or_default();

                    match idx {
                        0 => rect.top = val,
                        1 => rect.right = val,
                        2 => rect.bottom = val,
                        3 => rect.left = val,
                        _ => (),
                    }
                    (Some(rect), idx + 1)
                })
                .0
        }
    }
}

impl TryFrom<&PropertyToken> for GridTrackRepetition {
    type Error = ();

    fn try_from(value: &PropertyToken) -> Result<Self, Self::Error> {
        Ok(match value {
            PropertyToken::Number(value) => GridTrackRepetition::Count(*value as u16),
            PropertyToken::Identifier(val) if val == "auto-fill" => GridTrackRepetition::AutoFill,
            PropertyToken::Identifier(val) if val == "auto-fit" => GridTrackRepetition::AutoFit,
            _ => {
                error!("first argument to repeat must be a u16, auto-fill, or auto-fit");
                return Err(());
            }
        })
    }
}

impl TryFrom<&PropertyToken> for MinTrackSizingFunction {
    type Error = ();
    fn try_from(value: &PropertyToken) -> Result<Self, Self::Error> {
        Ok(match value {
            PropertyToken::Number(value) => MinTrackSizingFunction::Px(*value),
            PropertyToken::Percentage(value) => MinTrackSizingFunction::Percent(*value),
            PropertyToken::VMin(value) => MinTrackSizingFunction::VMin(*value),
            PropertyToken::VMax(value) => MinTrackSizingFunction::VMax(*value),
            PropertyToken::Vh(value) => MinTrackSizingFunction::Vh(*value),
            PropertyToken::Vw(value) => MinTrackSizingFunction::Vw(*value),
            PropertyToken::Identifier(val) if val == "min-content" => {
                MinTrackSizingFunction::MinContent
            }
            PropertyToken::Identifier(val) if val == "max-content" => {
                MinTrackSizingFunction::MaxContent
            }
            PropertyToken::Identifier(val) if val == "auto" => MinTrackSizingFunction::Auto,
            _ => {
                error!("first argument to minmax must be a px, percentage, vmin, vmax, vh, vw, min-content, max-content, or auto");
                return Err(());
            }
        })
    }
}

impl TryFrom<&PropertyToken> for MaxTrackSizingFunction {
    type Error = ();
    fn try_from(value: &PropertyToken) -> Result<Self, Self::Error> {
        Ok(match value {
            PropertyToken::Number(value) => MaxTrackSizingFunction::Px(*value),
            PropertyToken::Percentage(value) => MaxTrackSizingFunction::Percent(*value),
            PropertyToken::VMin(value) => MaxTrackSizingFunction::VMin(*value),
            PropertyToken::VMax(value) => MaxTrackSizingFunction::VMax(*value),
            PropertyToken::Vh(value) => MaxTrackSizingFunction::Vh(*value),
            PropertyToken::Vw(value) => MaxTrackSizingFunction::Vw(*value),
            PropertyToken::Identifier(val) if val == "min-content" => {
                MaxTrackSizingFunction::MinContent
            }
            PropertyToken::Identifier(val) if val == "max-content" => {
                MaxTrackSizingFunction::MaxContent
            }
            PropertyToken::Identifier(val) if val == "auto" => MaxTrackSizingFunction::Auto,
            _ => {
                error!("second argument to minmax must be a px, percentage, vmin, vmax, vh, vw, min-content, max-content, or auto");
                return Err(());
            }
        })
    }
}

impl<'i> TryFrom<ParsedToken<'i>> for PropertyToken {
    type Error = ();

    fn try_from(value: ParsedToken<'i>) -> Result<Self, Self::Error> {
        match value {
            ParsedToken::Single(tok) => tok.try_into(),
            ParsedToken::Function(name, args) => Ok(PropertyToken::Function(
                name.to_string(),
                args.into_iter().filter_map(|t| t.try_into().ok()).collect(),
            )),
        }
    }
}

impl<'i> TryFrom<Token<'i>> for PropertyToken {
    type Error = ();

    fn try_from(token: Token<'i>) -> Result<Self, Self::Error> {
        match token {
            Token::Ident(val) => Ok(Self::Identifier(val.to_string())),
            Token::Hash(val) => Ok(Self::Hash(val.to_string())),
            Token::IDHash(val) => Ok(Self::Hash(val.to_string())),
            Token::QuotedString(val) => Ok(Self::String(val.to_string())),
            Token::Number { value, .. } => Ok(Self::Number(value)),
            Token::Percentage { unit_value, .. } => Ok(Self::Percentage(unit_value * 100.0)),
            Token::Dimension { value, unit, .. } => match unit.as_bytes() {
                b"vmin" => Ok(Self::VMin(value)),
                b"vmax" => Ok(Self::VMax(value)),
                b"vh" => Ok(Self::Vh(value)),
                b"vw" => Ok(Self::Vw(value)),
                b"fr" => Ok(Self::Fr(value)),
                _ => Ok(Self::Dimension(value)),
            },
            Token::Delim('/') => Ok(Self::Slash),
            Token::WhiteSpace(_) => Err(()),
            tt => {
                error!("unmatched TT: {tt:?}");
                Err(())
            }
        }
    }
}

/// Internal cache state. Used by [`CachedProperties`] to avoid parsing properties of the same rule on same sheet.
#[derive(Default, Debug, Clone)]
pub enum CacheState<T> {
    /// No parse was performed yet
    #[default]
    None,
    /// Parse was performed and yielded a valid value.
    Ok(T),
    /// Parse was performed but returned an error.
    Error,
}

/// Internal cache map. Used by [`PropertyMeta`] to keep track of which properties was already parsed.
#[derive(Debug, Default, Deref, DerefMut)]
pub struct CachedProperties<T>(HashMap<Selector, CacheState<T>>);

/// Internal property cache map. Used by [`Property::apply_system`] to keep track of which properties was already parsed.
#[derive(Debug, Default, Deref, DerefMut)]
pub struct PropertyMeta<T: Property>(HashMap<u64, CachedProperties<T::Cache>>);

impl<T: Property> PropertyMeta<T> {
    /// Gets a cached property value or try to parse.
    ///
    /// If there are some error while parsing, a [`CacheState::Error`] is stored to avoid trying to parse again on next try.
    fn get_or_parse(
        &mut self,
        rules: &StyleSheetAsset,
        selector: &Selector,
    ) -> &CacheState<T::Cache> {
        let cached_properties = self.entry(rules.hash()).or_default();

        // Avoid using HashMap::entry since it requires ownership of key
        if cached_properties.contains_key(selector) {
            cached_properties.get(selector).unwrap()
        } else {
            let new_cache = rules
                .get_properties(selector, T::name())
                .map(|values| match T::parse(values) {
                    Ok(cache) => CacheState::Ok(cache),
                    Err(err) => {
                        error!("Failed to parse property {}. Error: {}", T::name(), err);
                        // TODO: Clear cache state when the asset is reloaded, since values may be changed.
                        CacheState::Error
                    }
                })
                .unwrap_or(CacheState::None);

            cached_properties.insert(selector.clone(), new_cache);
            cached_properties.get(selector).unwrap()
        }
    }
}

#[derive(Debug, Clone, Default, Deref, DerefMut)]
pub struct TrackedEntities(HashMap<SelectorElement, SmallVec<[Entity; 8]>>);

/// Maps which entities was selected by a [`Selector`]
#[derive(Debug, Clone, Default, Deref, DerefMut)]
pub struct SelectedEntities(SmallVec<[(Selector, SmallVec<[Entity; 8]>); 8]>);

/// Maps sheets for each [`StyleSheetAsset`].
#[derive(Debug, Clone, Default, Resource, Deref, DerefMut)]
pub struct StyleSheetState(Vec<(AssetId<StyleSheetAsset>, TrackedEntities, SelectedEntities)>);

impl StyleSheetState {
    pub(crate) fn has_any_selected_entities(&self) -> bool {
        self.iter().any(|(_, _, v)| !v.is_empty())
    }

    pub(crate) fn clear_selected_entities(&mut self) {
        self.iter_mut().for_each(|(_, _, v)| v.clear());
    }
}

/// Determines how a property should interact and modify the [ecs world](`bevy::prelude::World`).
///
/// Each implementation of this trait should be registered with [`RegisterProperty`](crate::RegisterProperty) trait, where
/// will be converted into a `system` and run whenever a matched, specified by [`name()`](`Property::name()`) property is found.
///
/// These are the associated types that must by specified by implementors:
/// - [`Cache`](Property::Cache) is a cached value to be applied by this trait.
/// On the first time the `system` runs it'll call [`parse`](`Property::parse`) and cache the value.
/// Subsequential runs will only fetch the cached value.
/// - [`Components`](Property::Components) is which components will be send to [`apply`](`Property::apply`) function whenever a
/// valid cache exists and a matching property was found on any sheet rule. Check [`QueryData`] for more.
/// - [`Filters`](Property::Filters) is used to filter which entities will be applied the property modification.
/// Entities are first filtered by [`selectors`](`Selector`), but it can be useful to also ensure some behavior for safety reasons,
/// like only inserting [`JustifyText`](bevy::prelude::JustifyText) if the entity also has a [`Text`](bevy::prelude::Text) component.
///  Check [`WorldQuery`] for more.
///
/// These are tree functions required to be implemented:
/// - [`name`](Property::name) indicates which property name should matched for.
/// - [`parse`](Property::parse) parses the [`PropertyValues`] into the [`Cache`](Property::Cache) value to be reused across multiple entities.
/// - [`apply`](Property::apply) applies on the given [`Components`](Property::Components) the [`Cache`](Property::Cache) value.
/// Additionally, an [`AssetServer`] and [`Commands`] parameters are provided for more complex use cases.
///
/// Also, there one function which have default implementations:
/// - [`apply_system`](Property::apply_system) is a [`system`](https://docs.rs/bevy_ecs/latest/bevy_ecs/system/index.html) which interacts with
/// [ecs world](`bevy::prelude::World`) and call the [`apply`](Property::apply) function on every matched entity.
pub trait Property: Default + Sized + Send + Sync + 'static {
    /// The cached value type to be applied by property.
    type Cache: Default + Any + Send + Sync;
    /// Which components should be queried when applying the modification. Check [`QueryData`] for more.
    type Components: QueryData;
    /// Filters conditions to be applied when querying entities by this property. Check [`QueryFilter`] for more.
    type Filters: QueryFilter;

    /// Indicates which property name should matched for. Must match the same property name as on `css` file.
    ///
    /// For compliance, use always `lower-case` and `kebab-case` names.
    fn name() -> &'static str;

    /// Parses the [`PropertyValues`] into the [`Cache`](Property::Cache) value to be reused across multiple entities.
    ///
    /// This function is called only once, on the first time a matching property is found while applying style rule.
    /// If an error is returned, it is also cached so no more attempt are made.
    fn parse(values: &PropertyValues) -> Result<Self::Cache, EcssError>;

    /// Applies on the given [`Components`](Property::Components) the [`Cache`](Property::Cache) value.
    /// Additionally, an [`AssetServer`] and [`Commands`] parameters are provided for more complex use cases.
    ///
    /// If mutability is desired while applying the changes, declare [`Components`](Property::Components) as mutable.
    fn apply(
        cache: &Self::Cache,
        components: QueryItem<Self::Components>,
        asset_server: &AssetServer,
        commands: &mut Commands,
    );

    /// The [`system`](https://docs.rs/bevy_ecs/latest/bevy_ecs/system/index.html) which interacts with
    /// [ecs world](`bevy::prelude::World`) and call [`apply`](Property::apply) function on every matched entity.
    ///
    /// The default implementation will cover most use cases, by just implementing [`apply`](Property::apply)
    fn apply_system(
        mut local: Local<PropertyMeta<Self>>,
        assets: Res<Assets<StyleSheetAsset>>,
        apply_sheets: Res<StyleSheetState>,
        mut q_nodes: Query<Self::Components, Self::Filters>,
        asset_server: Res<AssetServer>,
        mut commands: Commands,
    ) {
        for (asset_id, _, selected) in apply_sheets.iter() {
            if let Some(rules) = assets.get(*asset_id) {
                for (selector, entities) in selected.iter() {
                    if let CacheState::Ok(cached) = local.get_or_parse(rules, selector) {
                        trace!(
                            r#"Applying property "{}" from sheet "{}" ({})"#,
                            Self::name(),
                            rules.path(),
                            selector
                        );
                        for entity in entities {
                            if let Ok(components) = q_nodes.get_mut(*entity) {
                                Self::apply(cached, components, &asset_server, &mut commands);
                            }
                        }
                    }
                }
            }
        }
    }
}
