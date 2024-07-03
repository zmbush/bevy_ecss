use bevy::{ecs::query::QueryItem, prelude::*};

use crate::EcssError;

use super::{Property, PropertyValues};

pub use style::*;
pub use text::*;

/// Impls for `bevy_ui` [`Style`] component
mod style {
    use super::*;

    /// Implements a new property for [`Style`] component which expects a single value.
    macro_rules! impl_style_single_value {
        ($name:expr, $struct:ident, $cache:ty, $parse_func:ident, $style_prop:ident$(.$style_field:ident)*) => {
            impl_style_single_value!($name, $struct, $cache, $parse_func, $style_prop$(.$style_field)*, true);
        };

        ($name:expr, $struct:ident, $cache:ty, $parse_func:ident, $style_prop:ident$(.$style_field:ident)*, $do_default:expr) => {
            #[doc = "Applies the `"]
            #[doc = $name]
            #[doc = "` property on [Style::"]
            #[doc = stringify!($style_prop)]
            $(#[doc = concat!("::",stringify!($style_field))])*
            #[doc = "](`Style`) field of all sections on matched [`Style`] components."]
            #[derive(Default)]
            pub struct $struct;

            impl Property for $struct {
                type Cache = $cache;
                type Components = &'static mut Style;
                type Filters = With<Node>;

                fn name() -> &'static str {
                    $name
                }

                fn parse<'a>(values: &PropertyValues) -> Result<Self::Cache, EcssError> {
                    if let Some(val) = values.$parse_func() {
                        Ok(val)
                    } else {
                        Err(EcssError::InvalidPropertyValue(Self::name().to_string()))
                    }
                }

                fn apply<'w>(
                    cache: Option<&Self::Cache>,
                    mut components: QueryItem<Self::Components>,
                    _asset_server: &AssetServer,
                    _commands: &mut Commands,
                ) {
                    if $do_default {
                        components.$style_prop$(.$style_field)? = cache
                            .cloned()
                            .unwrap_or_default();
                    } else {
                        if let Some(cache) = cache {
                            components.$style_prop$(.$style_field)? = cache.clone();
                        }
                    }
                }
            }
        };
    }

    // Val properties
    impl_style_single_value!("left", LeftProperty, Val, val, left);
    impl_style_single_value!("right", RightProperty, Val, val, right);
    impl_style_single_value!("top", TopProperty, Val, val, top);
    impl_style_single_value!("bottom", BottomProperty, Val, val, bottom);

    impl_style_single_value!("width", WidthProperty, Val, val, width);
    impl_style_single_value!("height", HeightProperty, Val, val, height);

    impl_style_single_value!("min-width", MinWidthProperty, Val, val, min_width);
    impl_style_single_value!("min-height", MinHeightProperty, Val, val, min_height);

    impl_style_single_value!("max-width", MaxWidthProperty, Val, val, max_width);
    impl_style_single_value!("max-height", MaxHeightProperty, Val, val, max_height);

    impl_style_single_value!("flex-basis", FlexBasisProperty, Val, val, flex_basis);

    impl_style_single_value!("flex-grow", FlexGrowProperty, f32, f32, flex_grow);
    impl_style_single_value!("flex-shrink", FlexShrinkProperty, f32, f32, flex_shrink);

    impl_style_single_value!("row-gap", RowGapProperty, Val, val, row_gap);
    impl_style_single_value!("column-gap", ColumnGapProperty, Val, val, column_gap);

    impl_style_single_value!(
        "grid-template-columns",
        GridTemplateColumns,
        Vec<RepeatedGridTrack>,
        grid_template,
        grid_template_columns
    );
    impl_style_single_value!(
        "grid-template-rows",
        GridTemplateRows,
        Vec<RepeatedGridTrack>,
        grid_template,
        grid_template_rows
    );
    impl_style_single_value!("grid-row", GridRow, GridPlacement, grid_placement, grid_row);
    impl_style_single_value!(
        "grid-column",
        GridColumn,
        GridPlacement,
        grid_placement,
        grid_column
    );

    impl_style_single_value!(
        "aspect-ratio",
        AspectRatioProperty,
        Option<f32>,
        option_f32,
        aspect_ratio
    );

    /// Implements a new property for [`Style`] component which expects a rect value.
    macro_rules! impl_style_rect {
        ($name:expr, $struct:ident, {$struct_top:ident, $struct_bottom:ident, $struct_left:ident, $struct_right:ident}, $style_prop:ident$(.$style_field:ident)*) => {
            #[doc = "Applies the `"]
            #[doc = $name]
            #[doc = "` property on [Style::"]
            #[doc = stringify!($style_prop)]
            $(#[doc = concat!("::",stringify!($style_field))])*
            #[doc = "](`Style`) field of all sections on matched [`Style`] components."]
            #[derive(Default)]
            pub struct $struct;

            impl Property for $struct {
                type Cache = UiRect;
                type Components = &'static mut Style;
                type Filters = With<Node>;

                fn name() -> &'static str {
                    $name
                }

                fn parse<'a>(values: &PropertyValues) -> Result<Self::Cache, EcssError> {
                    if let Some(val) = values.rect() {
                        Ok(val)
                    } else {
                        Err(EcssError::InvalidPropertyValue(Self::name().to_string()))
                    }
                }

                fn apply<'w>(
                    cache: Option<&Self::Cache>,
                    mut components: QueryItem<Self::Components>,
                    _asset_server: &AssetServer,
                    _commands: &mut Commands,
                ) {
                        components.$style_prop$(.$style_field)? = cache
                            .copied()
                            .unwrap_or_default();
                }
            }

            impl_style_single_value!(concat!($name, "-top"), $struct_top, Val, val, $style_prop.top, false);
            impl_style_single_value!(concat!($name, "-bottom"), $struct_bottom, Val, val, $style_prop.bottom, false);
            impl_style_single_value!(concat!($name, "-left"), $struct_left, Val, val, $style_prop.left, false);
            impl_style_single_value!(concat!($name, "-right"), $struct_right, Val, val, $style_prop.right, false);
        };
    }

    impl_style_rect!("margin", MarginProperty, {
        MarginTopProperty,
        MarginBottomProperty,
        MarginLeftProperty,
        MarginRightProperty
    }, margin);
    impl_style_rect!("padding", PaddingProperty, {
        PaddingTopProperty,
        PaddingBottomProperty,
        PaddingLeftProperty,
        PaddingRightProperty
    }, padding);
    impl_style_rect!("border", BorderProperty, {
        BorderTopProperty,
        BorderBottomProperty,
        BorderLeftProperty,
        BorderRightProperty
    }, border);

    /// Implements a new property for [`Style`] component which expects an enum.
    macro_rules! impl_style_enum {
        ($cache:ty, $name:expr, $struct:ident, $style_prop:ident$(.$style_field:ident)*, $($prop:expr => $variant:expr),+$(,)?) => {
            #[doc = "Applies the `"]
            #[doc = $name]
            #[doc = "` property on [Style::"]
            #[doc = stringify!($style_prop)]
            #[doc = "]("]
            #[doc = concat!("`", stringify!($cache), "`")]
            #[doc = ") field of all sections on matched [`Style`] components."]
            #[derive(Default)]
            pub struct $struct;

            impl Property for $struct {
                type Cache = $cache;
                type Components = &'static mut Style;
                type Filters = With<Node>;

                fn name() -> &'static str {
                    $name
                }

                fn parse<'a>(values: &PropertyValues) -> Result<Self::Cache, EcssError> {
                    if let Some(identifier) = values.identifier() {
                        use $cache::*;
                        // Chain if-let when `cargofmt` supports it
                        // https://github.com/rust-lang/rustfmt/pull/5203
                        match identifier {
                            $($prop => return Ok($variant)),+,
                            _ => (),
                        }
                    }

                    Err(EcssError::InvalidPropertyValue(Self::name().to_string()))
                }

                fn apply<'w>(
                    cache: Option<&Self::Cache>,
                    mut components: QueryItem<Self::Components>,
                    _asset_server: &AssetServer,
                    _commands: &mut Commands,
                ) {
                    components.$style_prop$(.$style_field)? = cache
                        .copied()
                        .unwrap_or_default();
                }
            }
        };
    }

    impl_style_enum!(Display, "display", DisplayProperty, display,
        "flex" => Flex,
        "grid" => Grid,
        "none" => None,
    );

    impl_style_enum!(PositionType, "position-type", PositionTypeProperty, position_type,
        "absolute" => Absolute,
        "relative" => Relative,
    );

    impl_style_enum!(Direction, "direction", DirectionProperty, direction,
        "inherit" => Inherit,
        "left-to-right" => LeftToRight,
        "right-to-left" => RightToLeft,
    );

    impl_style_enum!(FlexDirection, "flex-direction", FlexDirectionProperty, flex_direction,
        "row" => Row,
        "column" => Column,
        "row-reverse" => RowReverse,
        "column-reverse" => ColumnReverse,
    );

    impl_style_enum!(FlexWrap, "flex-wrap", FlexWrapProperty, flex_wrap,
        "no-wrap" => NoWrap,
        "wrap" => Wrap,
        "wrap-reverse" => WrapReverse,
    );

    impl_style_enum!(AlignItems, "align-items", AlignItemsProperty, align_items,
        "flex-start" => FlexStart,
        "flex-end" => FlexEnd,
        "center" => Center,
        "baseline" => Baseline,
        "stretch" => Stretch,
    );

    impl_style_enum!(AlignSelf, "align-self", AlignSelfProperty, align_self,
        "auto" => Auto,
        "flex-start" => FlexStart,
        "flex-end" => FlexEnd,
        "center" => Center,
        "baseline" => Baseline,
        "stretch" => Stretch,
    );

    impl_style_enum!(AlignContent, "align-content", AlignContentProperty, align_content,
        "flex-start" => FlexStart,
        "flex-end" => FlexEnd,
        "center" => Center,
        "stretch" => Stretch,
        "space-between" => SpaceBetween,
        "space-around" => SpaceAround,
    );

    impl_style_enum!(JustifyContent, "justify-content", JustifyContentProperty, justify_content,
        "flex-start" => FlexStart,
        "flex-end" => FlexEnd,
        "center" => Center,
        "space-between" => SpaceBetween,
        "space-around" => SpaceAround,
        "space-evenly" => SpaceEvenly,
    );

    impl_style_enum!(OverflowAxis, "overflow-x", OverflowAxisXProperty, overflow.x,
        "visible" => Visible,
        "hidden" => Clip,
    );

    impl_style_enum!(OverflowAxis, "overflow-y", OverflowAxisYProperty, overflow.y,
        "visible" => Visible,
        "hidden" => Clip,
    );
}

/// Impls for `bevy_text` [`Text`] component
mod text {
    use super::*;

    /// Applies the `color` property on [`TextStyle::color`](`TextStyle`) field of all sections on matched [`Text`] components.
    #[derive(Default)]
    pub struct FontColorProperty;

    impl Property for FontColorProperty {
        type Cache = Color;
        type Components = &'static mut Text;
        type Filters = With<Node>;

        fn name() -> &'static str {
            "color"
        }

        fn parse<'a>(values: &PropertyValues) -> Result<Self::Cache, EcssError> {
            if let Some(color) = values.color() {
                Ok(color)
            } else {
                Err(EcssError::InvalidPropertyValue(Self::name().to_string()))
            }
        }

        fn apply<'w>(
            cache: Option<&Self::Cache>,
            mut components: QueryItem<Self::Components>,
            _asset_server: &AssetServer,
            _commands: &mut Commands,
        ) {
            let color = cache.copied().unwrap_or_default();
            components
                .sections
                .iter_mut()
                .for_each(|section| section.style.color = color);
        }
    }

    /// Applies the `font` property on [`TextStyle::font`](`TextStyle`) property of all sections on matched [`Text`] components.
    #[derive(Default)]
    pub struct FontProperty;

    impl Property for FontProperty {
        type Cache = String;
        type Components = &'static mut Text;
        type Filters = With<Node>;

        fn name() -> &'static str {
            "font"
        }

        fn parse<'a>(values: &PropertyValues) -> Result<Self::Cache, EcssError> {
            if let Some(path) = values.string() {
                Ok(path)
            } else {
                Err(EcssError::InvalidPropertyValue(Self::name().to_string()))
            }
        }

        fn apply<'w>(
            cache: Option<&Self::Cache>,
            mut components: QueryItem<Self::Components>,
            asset_server: &AssetServer,
            _commands: &mut Commands,
        ) {
            if let Some(cache) = cache {
                components
                    .sections
                    .iter_mut()
                    .for_each(|section| section.style.font = asset_server.load(cache));
            }
        }
    }

    /// Applies the `font-size` property on [`TextStyle::font_size`](`TextStyle`) property of all sections on matched [`Text`] components.
    #[derive(Default)]
    pub struct FontSizeProperty;

    impl Property for FontSizeProperty {
        type Cache = f32;
        type Components = &'static mut Text;
        type Filters = With<Node>;

        fn name() -> &'static str {
            "font-size"
        }

        fn parse<'a>(values: &PropertyValues) -> Result<Self::Cache, EcssError> {
            if let Some(size) = values.f32() {
                Ok(size)
            } else {
                Err(EcssError::InvalidPropertyValue(Self::name().to_string()))
            }
        }

        fn apply<'w>(
            cache: Option<&Self::Cache>,
            mut components: QueryItem<Self::Components>,
            _asset_server: &AssetServer,
            _commands: &mut Commands,
        ) {
            let size = cache
                .copied()
                .unwrap_or_else(|| TextStyle::default().font_size);
            components
                .sections
                .iter_mut()
                .for_each(|section| section.style.font_size = size);
        }
    }

    /// Applies the `text-align` property on [`Text::horizontal`](`JustifyText`) components.
    #[derive(Default)]
    pub struct TextAlignProperty;

    impl Property for TextAlignProperty {
        // Using Option since Cache must impl Default, which  doesn't
        type Cache = Option<JustifyText>;
        type Components = &'static mut Text;
        type Filters = With<Node>;

        fn name() -> &'static str {
            "text-align"
        }

        fn parse<'a>(values: &PropertyValues) -> Result<Self::Cache, EcssError> {
            if let Some(ident) = values.identifier() {
                match ident {
                    "left" => return Ok(Some(JustifyText::Left)),
                    "center" => return Ok(Some(JustifyText::Center)),
                    "right" => return Ok(Some(JustifyText::Right)),
                    _ => (),
                }
            }
            Err(EcssError::InvalidPropertyValue(Self::name().to_string()))
        }

        fn apply<'w>(
            cache: Option<&Self::Cache>,
            mut components: QueryItem<Self::Components>,
            _asset_server: &AssetServer,
            _commands: &mut Commands,
        ) {
            components.justify = cache
                .copied()
                .unwrap_or(Some(JustifyText::Left))
                .expect("Should always have a inner value");
        }
    }

    /// Apply a custom `text-content` which updates [`TextSection::value`](`TextSection`) of all sections on matched [`Text`] components
    #[derive(Default)]
    pub struct TextContentProperty;

    impl Property for TextContentProperty {
        type Cache = String;
        type Components = &'static mut Text;
        type Filters = With<Node>;

        fn name() -> &'static str {
            "text-content"
        }

        fn parse<'a>(values: &PropertyValues) -> Result<Self::Cache, EcssError> {
            if let Some(content) = values.string() {
                Ok(content)
            } else {
                Err(EcssError::InvalidPropertyValue(Self::name().to_string()))
            }
        }

        fn apply<'w>(
            cache: Option<&Self::Cache>,
            mut components: QueryItem<Self::Components>,
            _asset_server: &AssetServer,
            _commands: &mut Commands,
        ) {
            if let Some(cache) = cache {
                components
                    .sections
                    .iter_mut()
                    // TODO: Maybe change this so each line break is a new section
                    .for_each(|section| section.value.clone_from(cache));
            }
        }
    }
}

/// Applies the `background-color` property on [`BackgroundColor`] component of matched entities.
#[derive(Default)]
pub struct BackgroundColorProperty;

impl Property for BackgroundColorProperty {
    type Cache = Color;
    type Components = (
        Option<&'static mut BackgroundColor>,
        Option<&'static mut UiImage>,
    );
    type Filters = Or<(With<BackgroundColor>, With<UiImage>)>;

    fn name() -> &'static str {
        "background-color"
    }

    fn parse<'a>(values: &PropertyValues) -> Result<Self::Cache, EcssError> {
        if let Some(color) = values.color() {
            Ok(color)
        } else {
            Err(EcssError::InvalidPropertyValue(Self::name().to_string()))
        }
    }

    fn apply<'w>(
        cache: Option<&Self::Cache>,
        (bg, img): QueryItem<Self::Components>,
        _asset_server: &AssetServer,
        _commands: &mut Commands,
    ) {
        if let Some(mut bg) = bg {
            *bg = cache.copied().map(BackgroundColor).unwrap_or_default();
        }
        if let Some(mut img) = img {
            img.color = cache.copied().unwrap_or_default();
        }
    }
}

/// Applies the `border-color` property on [`BorderColor`] component of matched entities.
#[derive(Default)]
pub struct BorderColorProperty;

impl Property for BorderColorProperty {
    type Cache = Color;
    type Components = Entity;
    type Filters = With<BorderColor>;

    fn name() -> &'static str {
        "border-color"
    }

    fn parse<'a>(values: &PropertyValues) -> Result<Self::Cache, EcssError> {
        if let Some(color) = values.color() {
            Ok(color)
        } else {
            Err(EcssError::InvalidPropertyValue(Self::name().to_string()))
        }
    }

    fn apply<'w>(
        cache: Option<&Self::Cache>,
        components: QueryItem<Self::Components>,
        _asset_server: &AssetServer,
        commands: &mut Commands,
    ) {
        commands
            .entity(components)
            .insert(cache.copied().map(BorderColor).unwrap_or_default());
    }
}

/// Applies the `image-path` property on [`bevy::ui::UiImage`] texture property of all sections on matched [`bevy::ui::UiImage`] components.
#[derive(Default)]
pub struct ImageProperty;

impl Property for ImageProperty {
    type Cache = String;
    type Components = &'static mut UiImage;
    type Filters = With<Node>;

    fn name() -> &'static str {
        "image-path"
    }

    fn parse<'a>(values: &PropertyValues) -> Result<Self::Cache, EcssError> {
        if let Some(path) = values.string() {
            Ok(path)
        } else {
            Err(EcssError::InvalidPropertyValue(Self::name().to_string()))
        }
    }

    fn apply<'w>(
        cache: Option<&Self::Cache>,
        mut components: QueryItem<Self::Components>,
        asset_server: &AssetServer,
        _commands: &mut Commands,
    ) {
        components.texture = match cache {
            Some(cache) => asset_server.load(cache),
            None => bevy::render::texture::TRANSPARENT_IMAGE_HANDLE,
        };
    }
}

/// Applies the `border-radius` property on [`bevy::prelude::BorderRadius`] components.
#[derive(Default)]
pub struct BorderRadiusProperty;

impl Property for BorderRadiusProperty {
    type Cache = BorderRadius;
    type Components = &'static mut BorderRadius;
    type Filters = With<Node>;

    fn name() -> &'static str {
        "border-radius"
    }

    fn parse<'a>(values: &PropertyValues) -> Result<Self::Cache, EcssError> {
        if let Some(border_radius) = values.border_radius() {
            Ok(border_radius)
        } else {
            Err(EcssError::InvalidPropertyValue(Self::name().to_string()))
        }
    }

    fn apply<'w>(
        cache: Option<&Self::Cache>,
        mut components: QueryItem<Self::Components>,
        _asset_server: &AssetServer,
        _commands: &mut Commands,
    ) {
        *components = cache.copied().unwrap_or_default();
    }
}
