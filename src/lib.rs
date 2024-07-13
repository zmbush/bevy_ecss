#![doc = include_str!("../README.md")]

mod component;
mod parser;
pub mod property;
mod selector;
mod stylesheet;
mod system;

use std::{error::Error, fmt::Display};

use bevy::{
    app::First,
    asset::AssetEvents,
    ecs::system::SystemState,
    prelude::{
        AssetApp, Button, Component, Entity, IntoSystemConfigs, IntoSystemSetConfigs, Plugin,
        PostUpdate, PreUpdate, Query, SystemSet, With,
    },
    text::Text,
    ui::{BackgroundColor, Interaction, Node, Style, UiImage},
};

use property::StyleSheetState;
use stylesheet::{SCSSLoader, StyleSheetLoader};

use system::{ComponentFilterRegistry, PrepareParams};

pub use component::{Class, StyleSheet};
pub use property::{Property, PropertyToken, PropertyValues};
pub use selector::{Selector, SelectorElement};
pub use stylesheet::{StyleRule, StyleSheetAsset};

/// use `bevy_ecss::prelude::*;` to import common components, and plugins and utility functions.
pub mod prelude {
    pub use super::component::{Class, StyleSheet};
    pub use super::stylesheet::StyleSheetAsset;
    pub use super::EcssPlugin;
    pub use super::RegisterComponentSelector;
    pub use super::RegisterProperty;
}

/// Errors which can happens while parsing `css` into [`Selector`] or [`Property`].
// TODO: Change this to Cow<'static, str>
#[derive(Debug)]
pub enum EcssError {
    /// An unsupported selector was found on a style sheet rule.
    UnsupportedSelector,
    /// An unsupported property was found on a style sheet rule.
    UnsupportedProperty(String),
    /// An invalid property value was found on a style sheet rule.
    InvalidPropertyValue(String),
    /// An invalid selector was found on a style sheet rule.
    InvalidSelector,
    /// An unexpected token was found on a style sheet rule.
    UnexpectedToken(String),
}

impl Error for EcssError {}

impl Display for EcssError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EcssError::UnsupportedSelector => {
                write!(f, "Unsupported selector")
            }
            EcssError::UnsupportedProperty(p) => write!(f, "Unsupported property: {}", p),
            EcssError::InvalidPropertyValue(p) => write!(f, "Invalid property value: {}", p),
            EcssError::InvalidSelector => write!(f, "Invalid selector"),
            EcssError::UnexpectedToken(t) => write!(f, "Unexpected token: {}", t),
        }
    }
}

/// System sets  used by `bevy_ecss` systems
#[derive(SystemSet, Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum EcssSet {
    /// Checks if any entity affected by some style sheet was changed.
    /// Triggers [`StyleSheet::refresh`] if it does.
    ChangeDetection,
    /// Prepares internal state before running apply systems.
    /// This system runs on [`PreUpdate`] schedule.
    Prepare,
    /// All [`Property`] implementation `systems` are run on this system set.
    /// Those stages runs on [`PreUpdate`] schedule after [`EcssSet::Prepare`].
    Apply,
    /// Clears the internal state used by [`Property`] implementation `systems` set.
    /// This system runs on [`PostUpdate`] schedule.
    Cleanup,
}

/// Plugin which add all types, assets, systems and internal resources needed by `bevy_ecss`.
/// You must add this plugin in order to use `bevy_ecss`.
#[derive(Default)]
pub struct EcssPlugin;

impl Plugin for EcssPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_type::<Class>()
            .register_type::<StyleSheet>()
            .init_asset::<StyleSheetAsset>()
            .configure_sets(
                PreUpdate,
                (EcssSet::Prepare, EcssSet::ChangeDetection, EcssSet::Apply).chain(),
            )
            .configure_sets(PostUpdate, EcssSet::Cleanup)
            .init_resource::<StyleSheetState>()
            .init_resource::<ComponentFilterRegistry>()
            .init_asset_loader::<StyleSheetLoader>()
            .add_systems(PreUpdate, system::prepare.in_set(EcssSet::Prepare))
            .add_systems(
                PreUpdate,
                system::watch_tracked_entities.in_set(EcssSet::ChangeDetection),
            )
            .add_systems(PostUpdate, system::clear_state.in_set(EcssSet::Cleanup));

        #[cfg(feature = "sass")]
        app.init_asset_loader::<SCSSLoader>();

        let prepared_state = PrepareParams::new(app.world_mut());
        app.insert_resource(prepared_state);

        register_component_selector(app);
        register_properties(app);

        app.add_systems(First, system::reload_style_sheets.in_set(AssetEvents));
    }
}

fn register_component_selector(app: &mut bevy::prelude::App) {
    app.register_component_selector::<BackgroundColor>("background-color");
    app.register_component_selector::<Text>("text");
    app.register_component_selector::<Button>("button");
    app.register_component_selector::<Node>("node");
    app.register_component_selector::<Style>("style");
    app.register_component_selector::<UiImage>("ui-image");
    app.register_component_selector::<Interaction>("interaction");
}

fn register_properties(app: &mut bevy::prelude::App) {
    use property::impls::*;

    app.register_property::<DisplayProperty>();
    app.register_property::<PositionTypeProperty>();
    app.register_property::<DirectionProperty>();
    app.register_property::<FlexDirectionProperty>();
    app.register_property::<FlexWrapProperty>();
    app.register_property::<AlignItemsProperty>();
    app.register_property::<AlignSelfProperty>();
    app.register_property::<AlignContentProperty>();
    app.register_property::<JustifyContentProperty>();
    app.register_property::<OverflowAxisXProperty>();
    app.register_property::<OverflowAxisYProperty>();

    app.register_property::<LeftProperty>();
    app.register_property::<RightProperty>();
    app.register_property::<TopProperty>();
    app.register_property::<BottomProperty>();
    app.register_property::<WidthProperty>();
    app.register_property::<HeightProperty>();
    app.register_property::<MinWidthProperty>();
    app.register_property::<MinHeightProperty>();
    app.register_property::<MaxWidthProperty>();
    app.register_property::<MaxHeightProperty>();
    app.register_property::<FlexBasisProperty>();
    app.register_property::<FlexGrowProperty>();
    app.register_property::<FlexShrinkProperty>();
    app.register_property::<RowGapProperty>();
    app.register_property::<ColumnGapProperty>();
    app.register_property::<AspectRatioProperty>();

    app.register_property::<GridColumn>();
    app.register_property::<GridRow>();
    app.register_property::<GridTemplateColumns>();
    app.register_property::<GridTemplateRows>();

    app.register_property::<MarginProperty>();
    app.register_property_after::<MarginTopProperty, MarginProperty>();
    app.register_property_after::<MarginBottomProperty, MarginProperty>();
    app.register_property_after::<MarginLeftProperty, MarginProperty>();
    app.register_property_after::<MarginRightProperty, MarginProperty>();

    app.register_property::<PaddingProperty>();
    app.register_property_after::<PaddingTopProperty, PaddingProperty>();
    app.register_property_after::<PaddingBottomProperty, PaddingProperty>();
    app.register_property_after::<PaddingLeftProperty, PaddingProperty>();
    app.register_property_after::<PaddingRightProperty, PaddingProperty>();

    app.register_property::<BorderRadiusProperty>();
    app.register_property::<BorderProperty>();
    app.register_property_after::<BorderTopProperty, BorderProperty>();
    app.register_property_after::<BorderBottomProperty, BorderProperty>();
    app.register_property_after::<BorderLeftProperty, BorderProperty>();
    app.register_property_after::<BorderRightProperty, BorderProperty>();

    app.register_property::<FontColorProperty>();
    app.register_property::<FontProperty>();
    app.register_property::<FontSizeProperty>();
    app.register_property::<TextAlignProperty>();
    app.register_property::<TextContentProperty>();

    app.register_property::<BackgroundColorProperty>();
    app.register_property::<BorderColorProperty>();
    app.register_property::<ImageProperty>();
    app.register_property::<ZIndexProperty>();
}

/// Utility trait which adds the [`register_component_selector`](RegisterComponentSelector::register_component_selector)
/// function on [`App`](bevy::prelude::App) to add a new component selector.
///
/// You can register any component you want and name it as you like.
/// It's advised to use `lower-case` and `kebab-case` to match CSS coding style.
///
/// # Examples
///
/// ```
/// # use bevy::prelude::*;
/// # use bevy_ecss::prelude::*;
/// #
/// # #[derive(Component)]
/// # struct MyFancyComponentSelector;
/// #
/// # fn some_main() {
/// #    let mut app = App::new();
/// #    app.add_plugins(DefaultPlugins).add_plugins(EcssPlugin::default());
/// // You may use it as selector now, like
/// // fancy-pants {
/// //      background-color: pink;
/// // }
/// app.register_component_selector::<MyFancyComponentSelector>("fancy-pants");
/// # }
/// ```

pub trait RegisterComponentSelector {
    fn register_component_selector<T>(&mut self, name: &'static str) -> &mut Self
    where
        T: Component;
}

impl RegisterComponentSelector for bevy::prelude::App {
    fn register_component_selector<T>(&mut self, name: &'static str) -> &mut Self
    where
        T: Component,
    {
        let system_state = SystemState::<Query<Entity, With<T>>>::new(self.world_mut());
        let boxed_state = Box::new(system_state);

        self.world_mut()
            .get_resource_or_insert_with::<ComponentFilterRegistry>(bevy::utils::default)
            .insert(name, boxed_state);

        self
    }
}

/// Utility trait which adds the [`register_property`](RegisterProperty::register_property) function
/// on [`App`](bevy::prelude::App) to add a [`Property`] parser.
///
/// You need to register only custom properties which implements [`Property`] trait.
pub trait RegisterProperty {
    fn register_property<T>(&mut self) -> &mut Self
    where
        T: Property + 'static;

    fn register_property_after<T, After>(&mut self) -> &mut Self
    where
        T: Property + 'static,
        After: Property + 'static;
}

impl RegisterProperty for bevy::prelude::App {
    fn register_property<T>(&mut self) -> &mut Self
    where
        T: Property + 'static,
    {
        self.add_systems(PreUpdate, T::apply_system.in_set(EcssSet::Apply));

        self
    }

    fn register_property_after<T, After>(&mut self) -> &mut Self
    where
        T: Property + 'static,
        After: Property + 'static,
    {
        self.add_systems(
            PreUpdate,
            T::apply_system
                .after(After::apply_system)
                .in_set(EcssSet::Apply),
        );
        self
    }
}
