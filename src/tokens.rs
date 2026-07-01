pub mod state {
    pub const HOVER_STATE_LAYER_OPACITY: f32 = 0.08;
    pub const FOCUS_STATE_LAYER_OPACITY: f32 = 0.12;
    pub const PRESSED_STATE_LAYER_OPACITY: f32 = 0.12;
    pub const DRAGGED_STATE_LAYER_OPACITY: f32 = 0.16;

    pub const DISABLED_CONTAINER_OPACITY: f32 = 0.12;
    pub const DISABLED_LABEL_TEXT_OPACITY: f32 = 0.38;
}

pub mod motion {
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct CubicBezier {
        pub x1: f32,
        pub y1: f32,
        pub x2: f32,
        pub y2: f32,
    }

    impl CubicBezier {
        pub const fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
            Self { x1, y1, x2, y2 }
        }
    }

    pub const DURATION_SHORT1_MS: u16 = 50;
    pub const DURATION_SHORT2_MS: u16 = 100;
    pub const DURATION_SHORT3_MS: u16 = 150;
    pub const DURATION_SHORT4_MS: u16 = 200;
    pub const DURATION_MEDIUM1_MS: u16 = 250;
    pub const DURATION_MEDIUM2_MS: u16 = 300;
    pub const DURATION_MEDIUM3_MS: u16 = 350;
    pub const DURATION_MEDIUM4_MS: u16 = 400;
    pub const DURATION_LONG1_MS: u16 = 450;
    pub const DURATION_LONG2_MS: u16 = 500;
    pub const DURATION_LONG3_MS: u16 = 550;
    pub const DURATION_LONG4_MS: u16 = 600;
    pub const DURATION_EXTRA_LONG1_MS: u16 = 700;
    pub const DURATION_EXTRA_LONG2_MS: u16 = 800;
    pub const DURATION_EXTRA_LONG3_MS: u16 = 900;
    pub const DURATION_EXTRA_LONG4_MS: u16 = 1000;

    pub const EASING_EMPHASIZED: CubicBezier = CubicBezier::new(0.2, 0.0, 0.0, 1.0);
    pub const EASING_EMPHASIZED_ACCELERATE: CubicBezier = CubicBezier::new(0.3, 0.0, 0.8, 0.15);
    pub const EASING_EMPHASIZED_DECELERATE: CubicBezier = CubicBezier::new(0.05, 0.7, 0.1, 1.0);
    pub const EASING_STANDARD: CubicBezier = CubicBezier::new(0.2, 0.0, 0.0, 1.0);
    pub const EASING_STANDARD_ACCELERATE: CubicBezier = CubicBezier::new(0.3, 0.0, 1.0, 1.0);
    pub const EASING_STANDARD_DECELERATE: CubicBezier = CubicBezier::new(0.0, 0.0, 0.0, 1.0);
    pub const EASING_LINEAR: CubicBezier = CubicBezier::new(0.0, 0.0, 1.0, 1.0);
    pub const EASING_LEGACY: CubicBezier = CubicBezier::new(0.4, 0.0, 0.2, 1.0);
    pub const EASING_LEGACY_ACCELERATE: CubicBezier = CubicBezier::new(0.4, 0.0, 1.0, 1.0);
    pub const EASING_LEGACY_DECELERATE: CubicBezier = CubicBezier::new(0.0, 0.0, 0.2, 1.0);
}

pub mod shape {
    pub const CORNER_NONE: f32 = 0.0;
    pub const CORNER_EXTRA_SMALL: f32 = 4.0;
    pub const CORNER_SMALL: f32 = 8.0;
    pub const CORNER_MEDIUM: f32 = 12.0;
    pub const CORNER_LARGE: f32 = 16.0;
    pub const CORNER_EXTRA_LARGE: f32 = 28.0;
    pub const CORNER_FULL: f32 = 9999.0;
}

pub mod typography {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum TypefaceRole {
        Brand,
        Plain,
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct TypeScale {
        pub role: TypefaceRole,
        pub size: f32,
        pub line_height: f32,
        pub tracking: f32,
        pub weight: u16,
    }

    const fn scale(
        role: TypefaceRole,
        size: f32,
        line_height: f32,
        tracking: f32,
        weight: u16,
    ) -> TypeScale {
        TypeScale {
            role,
            size,
            line_height,
            tracking,
            weight,
        }
    }

    pub const WEIGHT_REGULAR: u16 = 400;
    pub const WEIGHT_MEDIUM: u16 = 500;
    pub const WEIGHT_BOLD: u16 = 700;

    pub const DISPLAY_LARGE: TypeScale =
        scale(TypefaceRole::Brand, 57.0, 64.0, -0.25, WEIGHT_REGULAR);
    pub const DISPLAY_MEDIUM: TypeScale =
        scale(TypefaceRole::Brand, 45.0, 52.0, 0.0, WEIGHT_REGULAR);
    pub const DISPLAY_SMALL: TypeScale =
        scale(TypefaceRole::Brand, 36.0, 44.0, 0.0, WEIGHT_REGULAR);
    pub const HEADLINE_LARGE: TypeScale =
        scale(TypefaceRole::Brand, 32.0, 40.0, 0.0, WEIGHT_REGULAR);
    pub const HEADLINE_MEDIUM: TypeScale =
        scale(TypefaceRole::Brand, 28.0, 36.0, 0.0, WEIGHT_REGULAR);
    pub const HEADLINE_SMALL: TypeScale =
        scale(TypefaceRole::Brand, 24.0, 32.0, 0.0, WEIGHT_REGULAR);
    pub const TITLE_LARGE: TypeScale = scale(TypefaceRole::Brand, 22.0, 28.0, 0.0, WEIGHT_REGULAR);
    pub const TITLE_MEDIUM: TypeScale = scale(TypefaceRole::Plain, 16.0, 24.0, 0.15, WEIGHT_MEDIUM);
    pub const TITLE_SMALL: TypeScale = scale(TypefaceRole::Plain, 14.0, 20.0, 0.1, WEIGHT_MEDIUM);
    pub const LABEL_LARGE: TypeScale = scale(TypefaceRole::Plain, 14.0, 20.0, 0.1, WEIGHT_MEDIUM);
    pub const LABEL_MEDIUM: TypeScale = scale(TypefaceRole::Plain, 12.0, 16.0, 0.5, WEIGHT_MEDIUM);
    pub const LABEL_SMALL: TypeScale = scale(TypefaceRole::Plain, 11.0, 16.0, 0.5, WEIGHT_MEDIUM);
    pub const BODY_LARGE: TypeScale = scale(TypefaceRole::Plain, 16.0, 24.0, 0.5, WEIGHT_REGULAR);
    pub const BODY_MEDIUM: TypeScale = scale(TypefaceRole::Plain, 14.0, 20.0, 0.25, WEIGHT_REGULAR);
    pub const BODY_SMALL: TypeScale = scale(TypefaceRole::Plain, 12.0, 16.0, 0.4, WEIGHT_REGULAR);
}

pub mod elevation {
    pub const LEVEL0: f32 = 0.0;
    pub const LEVEL1: f32 = 1.0;
    pub const LEVEL2: f32 = 3.0;
    pub const LEVEL3: f32 = 6.0;
    pub const LEVEL4: f32 = 8.0;
    pub const LEVEL5: f32 = 12.0;

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct ShadowLayer {
        pub y: f32,
        pub blur: f32,
        pub spread: f32,
        pub opacity: f32,
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Shadow {
        pub key: ShadowLayer,
        pub ambient: ShadowLayer,
    }

    const fn layer(y: f32, blur: f32, spread: f32, opacity: f32) -> ShadowLayer {
        ShadowLayer {
            y,
            blur,
            spread,
            opacity,
        }
    }

    pub const fn level(level: u8) -> f32 {
        match level {
            0 => LEVEL0,
            1 => LEVEL1,
            2 => LEVEL2,
            3 => LEVEL3,
            4 => LEVEL4,
            _ => LEVEL5,
        }
    }

    pub const fn shadow(level: u8) -> Shadow {
        match level {
            0 => Shadow {
                key: layer(0.0, 0.0, 0.0, 0.3),
                ambient: layer(0.0, 0.0, 0.0, 0.15),
            },
            1 => Shadow {
                key: layer(1.0, 2.0, 0.0, 0.3),
                ambient: layer(1.0, 3.0, 1.0, 0.15),
            },
            2 => Shadow {
                key: layer(1.0, 2.0, 0.0, 0.3),
                ambient: layer(2.0, 6.0, 2.0, 0.15),
            },
            3 => Shadow {
                key: layer(1.0, 3.0, 0.0, 0.3),
                ambient: layer(4.0, 8.0, 3.0, 0.15),
            },
            4 => Shadow {
                key: layer(2.0, 3.0, 0.0, 0.3),
                ambient: layer(6.0, 10.0, 4.0, 0.15),
            },
            _ => Shadow {
                key: layer(4.0, 4.0, 0.0, 0.3),
                ambient: layer(8.0, 12.0, 6.0, 0.15),
            },
        }
    }
}

pub mod component {
    pub mod button {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct ElevationLevels {
            pub active: u8,
            pub hovered: u8,
            pub pressed: u8,
            pub disabled: u8,
        }

        pub const CONTAINER_HEIGHT: f32 = 40.0;
        pub const CONTAINER_SHAPE: f32 = super::super::shape::CORNER_FULL;
        pub const ICON_SIZE: f32 = 18.0;
        pub const LEADING_SPACE: f32 = 24.0;
        pub const TRAILING_SPACE: f32 = 24.0;
        pub const LABEL_TEXT_SIZE: f32 = super::super::typography::LABEL_LARGE.size;
        pub const LABEL_TEXT_LINE_HEIGHT: f32 = super::super::typography::LABEL_LARGE.line_height;
        pub const LABEL_TEXT_WEIGHT: u16 = super::super::typography::LABEL_LARGE.weight;
        pub const WITH_LEADING_ICON_LEADING_SPACE: f32 = 16.0;
        pub const WITH_LEADING_ICON_TRAILING_SPACE: f32 = 24.0;
        pub const WITH_TRAILING_ICON_LEADING_SPACE: f32 = 24.0;
        pub const WITH_TRAILING_ICON_TRAILING_SPACE: f32 = 16.0;
        pub const OUTLINED_OUTLINE_WIDTH: f32 = 1.0;

        pub const FILLED_ELEVATION: ElevationLevels = ElevationLevels {
            active: 0,
            hovered: 1,
            pressed: 0,
            disabled: 0,
        };
        pub const ELEVATED_ELEVATION: ElevationLevels = ElevationLevels {
            active: 1,
            hovered: 2,
            pressed: 1,
            disabled: 0,
        };
        pub const FILLED_TONAL_ELEVATION: ElevationLevels = ElevationLevels {
            active: 0,
            hovered: 1,
            pressed: 0,
            disabled: 0,
        };
        pub const FLAT_ELEVATION: ElevationLevels = ElevationLevels {
            active: 0,
            hovered: 0,
            pressed: 0,
            disabled: 0,
        };
    }

    pub mod checkbox {
        pub const CONTAINER_SHAPE: f32 = 2.0;
        pub const CONTAINER_SIZE: f32 = 18.0;
        pub const ICON_SIZE: f32 = 18.0;
        pub const CHECKMARK_STROKE_WIDTH: f32 = 2.0;
        pub const CHECKMARK_BOTTOM_LEFT_X: f32 = 7.0;
        pub const CHECKMARK_BOTTOM_LEFT_Y: f32 = -14.0;
        pub const CHECKMARK_SHORT_MARK_SIZE: f32 = 5.656_854;
        pub const CHECKMARK_LONG_MARK_SIZE: f32 = 11.313_708;
        pub const STATE_LAYER_SIZE: f32 = 40.0;
        pub const LABEL_TEXT_SIZE: f32 = super::super::typography::BODY_LARGE.size;
        pub const LABEL_TEXT_LINE_HEIGHT: f32 = super::super::typography::BODY_LARGE.line_height;
        pub const LABEL_TEXT_WEIGHT: u16 = super::super::typography::BODY_LARGE.weight;
        pub const SELECT_TRANSITION_DURATION_MS: u16 = super::super::motion::DURATION_MEDIUM3_MS;
        pub const UNSELECT_TRANSITION_DURATION_MS: u16 = super::super::motion::DURATION_SHORT3_MS;
        pub const OPACITY_TRANSITION_DURATION_MS: u16 = super::super::motion::DURATION_SHORT1_MS;
        pub const SELECT_TRANSITION_EASING: super::super::motion::CubicBezier =
            super::super::motion::EASING_EMPHASIZED_DECELERATE;
        pub const UNSELECT_TRANSITION_EASING: super::super::motion::CubicBezier =
            super::super::motion::EASING_EMPHASIZED_ACCELERATE;
        pub const SELECTED_OUTLINE_WIDTH: f32 = 0.0;
        pub const UNSELECTED_OUTLINE_WIDTH: f32 = 2.0;
        pub const SELECTED_DISABLED_CONTAINER_OPACITY: f32 = 0.38;
        pub const UNSELECTED_DISABLED_CONTAINER_OPACITY: f32 = 0.38;
        pub const UNSELECTED_DISABLED_OUTLINE_WIDTH: f32 = 2.0;
    }

    pub mod switch {
        pub const TRACK_WIDTH: f32 = 52.0;
        pub const TRACK_HEIGHT: f32 = 32.0;
        pub const TRACK_SHAPE: f32 = super::super::shape::CORNER_FULL;
        pub const TRACK_OUTLINE_WIDTH: f32 = 2.0;
        pub const HANDLE_SHAPE: f32 = super::super::shape::CORNER_FULL;
        pub const SELECTED_HANDLE_SIZE: f32 = 24.0;
        pub const UNSELECTED_HANDLE_SIZE: f32 = 16.0;
        pub const WITH_ICON_HANDLE_SIZE: f32 = 24.0;
        pub const PRESSED_HANDLE_SIZE: f32 = 28.0;
        pub const SELECTED_ICON_SIZE: f32 = 16.0;
        pub const UNSELECTED_ICON_SIZE: f32 = 16.0;
        pub const STATE_LAYER_SIZE: f32 = 40.0;
        pub const LABEL_TEXT_SIZE: f32 = super::super::typography::BODY_LARGE.size;
        pub const LABEL_TEXT_LINE_HEIGHT: f32 = super::super::typography::BODY_LARGE.line_height;
        pub const LABEL_TEXT_WEIGHT: u16 = super::super::typography::BODY_LARGE.weight;
        pub const TRACK_COLOR_TRANSITION_DURATION_MS: u16 = 67;
        pub const HANDLE_COLOR_TRANSITION_DURATION_MS: u16 = 67;
        pub const HANDLE_SIZE_TRANSITION_DURATION_MS: u16 =
            super::super::motion::DURATION_MEDIUM1_MS;
        pub const PRESSED_HANDLE_SIZE_TRANSITION_DURATION_MS: u16 =
            super::super::motion::DURATION_SHORT2_MS;
        pub const HANDLE_POSITION_TRANSITION_DURATION_MS: u16 =
            super::super::motion::DURATION_MEDIUM2_MS;
        pub const HANDLE_POSITION_TRANSITION_EASING: super::super::motion::CubicBezier =
            super::super::motion::CubicBezier::new(0.175, 0.885, 0.32, 1.275);
        pub const ICON_FILL_TRANSITION_DURATION_MS: u16 = 67;
        pub const ICON_OPACITY_TRANSITION_DURATION_MS: u16 = 33;
        pub const ICON_TRANSFORM_TRANSITION_DURATION_MS: u16 = 167;
        pub const DISABLED_TRACK_OPACITY: f32 = 0.12;
        pub const DISABLED_SELECTED_HANDLE_OPACITY: f32 = 1.0;
        pub const DISABLED_UNSELECTED_HANDLE_OPACITY: f32 = 0.38;
        pub const DISABLED_SELECTED_ICON_OPACITY: f32 = 0.38;
        pub const DISABLED_UNSELECTED_ICON_OPACITY: f32 = 0.38;
    }

    pub mod slider {
        pub const ACTIVE_TRACK_HEIGHT: f32 = 4.0;
        pub const INACTIVE_TRACK_HEIGHT: f32 = 4.0;
        pub const TRACK_SHAPE: f32 = super::super::shape::CORNER_FULL;
        pub const HANDLE_WIDTH: f32 = 20.0;
        pub const HANDLE_HEIGHT: f32 = 20.0;
        pub const HANDLE_RADIUS: f32 = HANDLE_WIDTH / 2.0;
        pub const HANDLE_ELEVATION: u8 = 1;
        pub const STATE_LAYER_SIZE: f32 = 40.0;
        pub const LABEL_CONTAINER_HEIGHT: f32 = 28.0;
        pub const TICK_MARK_SIZE: f32 = 2.0;
        pub const WITH_OVERLAP_HANDLE_OUTLINE_WIDTH: f32 = 1.0;
        pub const DISABLED_ACTIVE_TRACK_OPACITY: f32 = 0.38;
        pub const DISABLED_INACTIVE_TRACK_OPACITY: f32 = 0.12;
        pub const DISABLED_HANDLE_OPACITY: f32 = 0.38;
    }

    pub mod linear_progress {
        pub const ACTIVE_INDICATOR_HEIGHT: f32 = 4.0;
        pub const ACTIVE_INDICATOR_SHAPE: f32 = super::super::shape::CORNER_NONE;
        pub const TRACK_HEIGHT: f32 = 4.0;
        pub const TRACK_SHAPE: f32 = super::super::shape::CORNER_NONE;
    }

    pub mod radio {
        pub const ICON_SIZE: f32 = 20.0;
        pub const TARGET_SIZE: f32 = 48.0;
        pub const OUTER_RING_WIDTH: f32 = 2.0;
        pub const INNER_DOT_SIZE: f32 = 10.0;
        pub const STATE_LAYER_SIZE: f32 = 40.0;
        pub const LABEL_TEXT_SIZE: f32 = super::super::typography::BODY_LARGE.size;
        pub const LABEL_TEXT_LINE_HEIGHT: f32 = super::super::typography::BODY_LARGE.line_height;
        pub const LABEL_TEXT_WEIGHT: u16 = super::super::typography::BODY_LARGE.weight;
        pub const SELECT_TRANSITION_DURATION_MS: u16 = super::super::motion::DURATION_MEDIUM2_MS;
        pub const ICON_COLOR_TRANSITION_DURATION_MS: u16 = super::super::motion::DURATION_SHORT1_MS;
        pub const SELECT_TRANSITION_EASING: super::super::motion::CubicBezier =
            super::super::motion::EASING_EMPHASIZED_DECELERATE;
        pub const DISABLED_SELECTED_ICON_OPACITY: f32 = 0.38;
        pub const DISABLED_UNSELECTED_ICON_OPACITY: f32 = 0.38;
    }

    pub mod text_field {
        pub const CONTAINER_HEIGHT: f32 = 56.0;
        pub const CONTAINER_SHAPE: f32 = super::super::shape::CORNER_EXTRA_SMALL;
        pub const LEADING_SPACE: f32 = 16.0;
        pub const TRAILING_SPACE: f32 = 16.0;
        pub const TOP_SPACE: f32 = 16.0;
        pub const BOTTOM_SPACE: f32 = 16.0;
        pub const INPUT_TEXT_SIZE: f32 = super::super::typography::BODY_LARGE.size;
        pub const INPUT_TEXT_LINE_HEIGHT: f32 = super::super::typography::BODY_LARGE.line_height;
        pub const INPUT_TEXT_WEIGHT: u16 = super::super::typography::BODY_LARGE.weight;
        pub const LABEL_TEXT_SIZE: f32 = super::super::typography::BODY_LARGE.size;
        pub const LABEL_TEXT_LINE_HEIGHT: f32 = super::super::typography::BODY_LARGE.line_height;
        pub const LABEL_TEXT_WEIGHT: u16 = super::super::typography::BODY_LARGE.weight;
        pub const LABEL_TEXT_PADDING_BOTTOM: f32 = 8.0;
        pub const LABEL_TEXT_POPULATED_SIZE: f32 = super::super::typography::BODY_SMALL.size;
        pub const LABEL_TEXT_POPULATED_LINE_HEIGHT: f32 =
            super::super::typography::BODY_SMALL.line_height;
        pub const LABEL_TEXT_POPULATED_WEIGHT: u16 = super::super::typography::BODY_SMALL.weight;
        pub const OUTLINE_LABEL_PADDING: f32 = 4.0;
        pub const OUTLINE_WIDTH: f32 = 1.0;
        pub const HOVER_OUTLINE_WIDTH: f32 = 1.0;
        pub const FOCUS_OUTLINE_WIDTH: f32 = 3.0;
        pub const ACTIVE_INDICATOR_HEIGHT: f32 = 1.0;
        pub const FOCUS_ACTIVE_INDICATOR_HEIGHT: f32 = 2.0;
        pub const LABEL_TRANSITION_DURATION_MS: u16 = super::super::motion::DURATION_SHORT3_MS;
        pub const LABEL_TRANSITION_EASING: super::super::motion::CubicBezier =
            super::super::motion::EASING_STANDARD;
        pub const DISABLED_CONTAINER_OPACITY: f32 = 0.04;
        pub const DISABLED_INPUT_TEXT_OPACITY: f32 = 0.38;
        pub const DISABLED_LABEL_TEXT_OPACITY: f32 = 0.38;
        pub const DISABLED_LEADING_ICON_OPACITY: f32 = 0.38;
        pub const DISABLED_OUTLINE_OPACITY: f32 = 0.12;
        pub const DISABLED_SUPPORTING_TEXT_OPACITY: f32 = 0.38;
        pub const DISABLED_TRAILING_ICON_OPACITY: f32 = 0.38;
    }

    pub mod divider {
        pub const THICKNESS: f32 = 1.0;
        pub const LIST_ITEM_LEADING_SPACE: u16 = 16;
        pub const LIST_ITEM_TRAILING_SPACE: u16 = 16;
    }

    pub mod list {
        pub const CONTAINER_SHAPE: f32 = super::super::shape::CORNER_NONE;
        pub const ONE_LINE_CONTAINER_HEIGHT: f32 = 56.0;
        pub const TWO_LINE_CONTAINER_HEIGHT: f32 = 72.0;
        pub const THREE_LINE_CONTAINER_HEIGHT: f32 = 88.0;
        pub const LEADING_SPACE: f32 = 16.0;
        pub const TRAILING_SPACE: f32 = 16.0;
        pub const LEADING_ICON_SIZE: f32 = 24.0;
        pub const LEADING_AVATAR_SIZE: f32 = 40.0;
        pub const DISABLED_LABEL_TEXT_OPACITY: f32 = 0.30;
        pub const DISABLED_ICON_OPACITY: f32 = 0.38;
    }

    pub mod menu {
        pub const CONTAINER_SHAPE: f32 = super::super::shape::CORNER_EXTRA_SMALL;
        pub const CONTAINER_ELEVATION_LEVEL: u8 = 2;
        pub const TOP_SPACE: f32 = 8.0;
        pub const BOTTOM_SPACE: f32 = 8.0;
        pub const LIST_ITEM_CONTAINER_HEIGHT: f32 = 48.0;
        pub const LIST_ITEM_ICON_SIZE: f32 = 24.0;
    }

    pub mod select {
        pub const MENU_CONTAINER_SHAPE: f32 = super::super::shape::CORNER_EXTRA_SMALL;
        pub const MENU_CONTAINER_ELEVATION_LEVEL: u8 = 2;
        pub const MENU_LIST_ITEM_CONTAINER_HEIGHT: f32 = 48.0;
        pub const MENU_LIST_ITEM_LEADING_ICON_SIZE: f32 = 24.0;
        pub const MENU_LIST_ITEM_TRAILING_ICON_SIZE: f32 = 24.0;
        pub const TEXT_FIELD_CONTAINER_SHAPE: f32 = super::super::shape::CORNER_EXTRA_SMALL;
        pub const TEXT_FIELD_OUTLINE_WIDTH: f32 = 1.0;
        pub const TEXT_FIELD_HOVER_OUTLINE_WIDTH: f32 = 1.0;
        pub const TEXT_FIELD_FOCUS_OUTLINE_WIDTH: f32 = 2.0;
        pub const TEXT_FIELD_DISABLED_OUTLINE_OPACITY: f32 = 0.12;
        pub const TEXT_FIELD_DISABLED_OUTLINE_WIDTH: f32 = 1.0;
        pub const LEADING_ICON_SIZE: f32 = 24.0;
        pub const TRAILING_ICON_SIZE: f32 = 24.0;
    }

    pub mod dialog {
        pub const CONTAINER_SHAPE: f32 = super::super::shape::CORNER_EXTRA_LARGE;
        pub const CONTAINER_ELEVATION_LEVEL: u8 = 3;
        pub const ICON_SIZE: f32 = 24.0;
    }

    pub mod data_table {
        pub const CONTAINER_SHAPE: f32 = super::super::shape::CORNER_EXTRA_SMALL;
        pub const OUTLINE_WIDTH: f32 = 1.0;
        pub const HEADER_CONTAINER_HEIGHT: f32 = 56.0;
        pub const FOOTER_CONTAINER_HEIGHT: f32 = 52.0;
        pub const ROW_ITEM_CONTAINER_HEIGHT: f32 = 52.0;
        pub const ROW_ITEM_OUTLINE_WIDTH: f32 = 1.0;
        pub const ROW_ITEM_DISABLED_LABEL_TEXT_OPACITY: f32 = 0.38;
    }

    pub mod card {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct ElevationLevels {
            pub active: u8,
            pub hovered: u8,
            pub pressed: u8,
            pub dragged: u8,
            pub disabled: u8,
        }

        pub const CONTAINER_SHAPE: f32 = super::super::shape::CORNER_MEDIUM;
        pub const ICON_SIZE: f32 = 24.0;
        pub const DISABLED_CONTAINER_OPACITY: f32 = 0.38;
        pub const OUTLINED_OUTLINE_WIDTH: f32 = 1.0;
        pub const OUTLINED_DISABLED_OUTLINE_OPACITY: f32 = 0.12;

        pub const ELEVATED_ELEVATION: ElevationLevels = ElevationLevels {
            active: 1,
            hovered: 2,
            pressed: 1,
            dragged: 4,
            disabled: 1,
        };
        pub const FILLED_ELEVATION: ElevationLevels = ElevationLevels {
            active: 0,
            hovered: 1,
            pressed: 0,
            dragged: 3,
            disabled: 0,
        };
        pub const OUTLINED_ELEVATION: ElevationLevels = ElevationLevels {
            active: 0,
            hovered: 1,
            pressed: 0,
            dragged: 3,
            disabled: 0,
        };
    }

    pub mod fab {
        pub const CONTAINER_WIDTH: f32 = 56.0;
        pub const CONTAINER_HEIGHT: f32 = 56.0;
        pub const CONTAINER_SHAPE: f32 = super::super::shape::CORNER_LARGE;
        pub const ICON_SIZE: f32 = 24.0;

        pub const ELEVATION: super::button::ElevationLevels = super::button::ElevationLevels {
            active: 3,
            hovered: 4,
            pressed: 3,
            disabled: 0,
        };
        pub const LOWERED_ELEVATION: super::button::ElevationLevels =
            super::button::ElevationLevels {
                active: 1,
                hovered: 2,
                pressed: 1,
                disabled: 0,
            };
    }

    pub mod icon_button {
        pub const CONTAINER_WIDTH: f32 = 40.0;
        pub const CONTAINER_HEIGHT: f32 = 40.0;
        pub const CONTAINER_SHAPE: f32 = super::super::shape::CORNER_FULL;
        pub const ICON_SIZE: f32 = 24.0;
        pub const STATE_LAYER_WIDTH: f32 = 40.0;
        pub const STATE_LAYER_HEIGHT: f32 = 40.0;
        pub const STATE_LAYER_SHAPE: f32 = super::super::shape::CORNER_FULL;
        pub const DISABLED_CONTAINER_OPACITY: f32 = 0.12;
        pub const DISABLED_ICON_OPACITY: f32 = 0.38;
        pub const OUTLINED_OUTLINE_WIDTH: f32 = 1.0;
        pub const OUTLINED_DISABLED_OUTLINE_OPACITY: f32 = 0.12;
    }

    pub mod chip {
        pub const CONTAINER_HEIGHT: f32 = 32.0;
        pub const CONTAINER_SHAPE: f32 = super::super::shape::CORNER_SMALL;
        pub const OUTLINE_WIDTH: f32 = 1.0;
        pub const SELECTED_OUTLINE_WIDTH: f32 = 0.0;
        pub const ICON_SIZE: f32 = 18.0;
        pub const LEADING_SPACE: f32 = 16.0;
        pub const TRAILING_SPACE: f32 = 16.0;
        pub const ICON_LABEL_SPACE: f32 = 8.0;
        pub const WITH_LEADING_ICON_LEADING_SPACE: f32 = 8.0;
        pub const WITH_TRAILING_ICON_TRAILING_SPACE: f32 = 8.0;
        pub const AVATAR_SIZE: f32 = 24.0;
        pub const LABEL_TEXT_SIZE: f32 = super::super::typography::LABEL_LARGE.size;
        pub const LABEL_TEXT_LINE_HEIGHT: f32 = super::super::typography::LABEL_LARGE.line_height;
        pub const LABEL_TEXT_WEIGHT: u16 = super::super::typography::LABEL_LARGE.weight;
        pub const DISABLED_LABEL_TEXT_OPACITY: f32 = 0.38;
        pub const DISABLED_ICON_OPACITY: f32 = 0.38;
        pub const DISABLED_CONTAINER_OPACITY: f32 = 0.12;
        pub const DISABLED_OUTLINE_OPACITY: f32 = 0.12;

        pub const FLAT_ELEVATION: super::button::ElevationLevels = super::button::ElevationLevels {
            active: 0,
            hovered: 0,
            pressed: 0,
            disabled: 0,
        };
        pub const SELECTED_FLAT_ELEVATION: super::button::ElevationLevels =
            super::button::ElevationLevels {
                active: 0,
                hovered: 1,
                pressed: 0,
                disabled: 0,
            };
        pub const ELEVATED_ELEVATION: super::button::ElevationLevels =
            super::button::ElevationLevels {
                active: 1,
                hovered: 2,
                pressed: 1,
                disabled: 0,
            };
    }

    pub mod tooltip {
        pub const PLAIN_CONTAINER_SHAPE: f32 = super::super::shape::CORNER_EXTRA_SMALL;
        pub const PLAIN_SUPPORTING_TEXT: super::super::typography::TypeScale =
            super::super::typography::BODY_SMALL;

        pub const RICH_CONTAINER_SHAPE: f32 = super::super::shape::CORNER_MEDIUM;
        pub const RICH_CONTAINER_ELEVATION_LEVEL: u8 = 2;
        pub const RICH_SUBHEAD_TEXT: super::super::typography::TypeScale =
            super::super::typography::TITLE_SMALL;
        pub const RICH_SUPPORTING_TEXT: super::super::typography::TypeScale =
            super::super::typography::BODY_MEDIUM;
        pub const RICH_ACTION_LABEL_TEXT: super::super::typography::TypeScale =
            super::super::typography::LABEL_LARGE;
    }
}

#[cfg(test)]
mod tests {
    use super::{component, elevation, motion, shape, state, typography};

    #[test]
    fn m3_state_tokens_match_google_values() {
        assert_eq!(state::HOVER_STATE_LAYER_OPACITY, 0.08);
        assert_eq!(state::FOCUS_STATE_LAYER_OPACITY, 0.12);
        assert_eq!(state::PRESSED_STATE_LAYER_OPACITY, 0.12);
        assert_eq!(state::DRAGGED_STATE_LAYER_OPACITY, 0.16);
    }

    #[test]
    fn m3_motion_tokens_match_google_values() {
        assert_eq!(motion::DURATION_SHORT4_MS, 200);
        assert_eq!(motion::DURATION_MEDIUM2_MS, 300);
        assert_eq!(motion::DURATION_EXTRA_LONG4_MS, 1000);
        assert_eq!(
            motion::EASING_EMPHASIZED_DECELERATE,
            motion::CubicBezier::new(0.05, 0.7, 0.1, 1.0)
        );
    }

    #[test]
    fn m3_shape_and_elevation_tokens_match_google_values() {
        assert_eq!(shape::CORNER_EXTRA_LARGE, 28.0);
        assert_eq!(shape::CORNER_FULL, 9999.0);
        assert_eq!(elevation::level(2), 3.0);
        assert_eq!(elevation::level(5), 12.0);
    }

    #[test]
    fn m3_component_sizing_tokens_match_google_values() {
        assert_eq!(component::button::CONTAINER_HEIGHT, 40.0);
        assert_eq!(component::button::LABEL_TEXT_SIZE, 14.0);
        assert_eq!(component::button::LABEL_TEXT_LINE_HEIGHT, 20.0);
        assert_eq!(component::button::LABEL_TEXT_WEIGHT, 500);
        assert_eq!(component::button::FILLED_ELEVATION.hovered, 1);
        assert_eq!(component::button::ELEVATED_ELEVATION.active, 1);
        assert_eq!(component::button::ELEVATED_ELEVATION.hovered, 2);
        assert_eq!(component::button::FILLED_TONAL_ELEVATION.pressed, 0);
        assert_eq!(component::checkbox::CONTAINER_SIZE, 18.0);
        assert_eq!(component::checkbox::ICON_SIZE, 18.0);
        assert_eq!(component::checkbox::CHECKMARK_STROKE_WIDTH, 2.0);
        assert_eq!(component::checkbox::CHECKMARK_BOTTOM_LEFT_X, 7.0);
        assert_eq!(component::checkbox::CHECKMARK_BOTTOM_LEFT_Y, -14.0);
        assert_eq!(component::checkbox::CHECKMARK_SHORT_MARK_SIZE, 5.656_854);
        assert_eq!(component::checkbox::CHECKMARK_LONG_MARK_SIZE, 11.313_708);
        assert_eq!(component::checkbox::STATE_LAYER_SIZE, 40.0);
        assert_eq!(component::checkbox::LABEL_TEXT_SIZE, 16.0);
        assert_eq!(component::checkbox::LABEL_TEXT_LINE_HEIGHT, 24.0);
        assert_eq!(component::checkbox::LABEL_TEXT_WEIGHT, 400);
        assert_eq!(component::checkbox::SELECT_TRANSITION_DURATION_MS, 350);
        assert_eq!(component::checkbox::UNSELECT_TRANSITION_DURATION_MS, 150);
        assert_eq!(component::checkbox::OPACITY_TRANSITION_DURATION_MS, 50);
        assert_eq!(
            component::checkbox::SELECT_TRANSITION_EASING,
            motion::EASING_EMPHASIZED_DECELERATE
        );
        assert_eq!(
            component::checkbox::UNSELECT_TRANSITION_EASING,
            motion::EASING_EMPHASIZED_ACCELERATE
        );
        assert_eq!(
            component::checkbox::SELECTED_DISABLED_CONTAINER_OPACITY,
            0.38
        );
        assert_eq!(component::switch::TRACK_WIDTH, 52.0);
        assert_eq!(component::switch::TRACK_HEIGHT, 32.0);
        assert_eq!(component::switch::TRACK_OUTLINE_WIDTH, 2.0);
        assert_eq!(component::switch::WITH_ICON_HANDLE_SIZE, 24.0);
        assert_eq!(component::switch::PRESSED_HANDLE_SIZE, 28.0);
        assert_eq!(component::switch::SELECTED_ICON_SIZE, 16.0);
        assert_eq!(component::switch::UNSELECTED_ICON_SIZE, 16.0);
        assert_eq!(component::switch::LABEL_TEXT_SIZE, 16.0);
        assert_eq!(component::switch::LABEL_TEXT_LINE_HEIGHT, 24.0);
        assert_eq!(component::switch::LABEL_TEXT_WEIGHT, 400);
        assert_eq!(component::switch::TRACK_COLOR_TRANSITION_DURATION_MS, 67);
        assert_eq!(component::switch::HANDLE_COLOR_TRANSITION_DURATION_MS, 67);
        assert_eq!(component::switch::HANDLE_SIZE_TRANSITION_DURATION_MS, 250);
        assert_eq!(
            component::switch::PRESSED_HANDLE_SIZE_TRANSITION_DURATION_MS,
            100
        );
        assert_eq!(
            component::switch::HANDLE_POSITION_TRANSITION_DURATION_MS,
            300
        );
        assert_eq!(
            component::switch::HANDLE_POSITION_TRANSITION_EASING,
            motion::CubicBezier::new(0.175, 0.885, 0.32, 1.275)
        );
        assert_eq!(component::switch::ICON_FILL_TRANSITION_DURATION_MS, 67);
        assert_eq!(component::switch::ICON_OPACITY_TRANSITION_DURATION_MS, 33);
        assert_eq!(
            component::switch::ICON_TRANSFORM_TRANSITION_DURATION_MS,
            167
        );
        assert_eq!(component::switch::DISABLED_TRACK_OPACITY, 0.12);
        assert_eq!(component::switch::DISABLED_SELECTED_HANDLE_OPACITY, 1.0);
        assert_eq!(component::slider::ACTIVE_TRACK_HEIGHT, 4.0);
        assert_eq!(component::slider::INACTIVE_TRACK_HEIGHT, 4.0);
        assert_eq!(component::slider::HANDLE_WIDTH, 20.0);
        assert_eq!(component::slider::HANDLE_HEIGHT, 20.0);
        assert_eq!(component::slider::STATE_LAYER_SIZE, 40.0);
        assert_eq!(component::slider::LABEL_CONTAINER_HEIGHT, 28.0);
        assert_eq!(component::linear_progress::TRACK_HEIGHT, 4.0);
        assert_eq!(component::linear_progress::ACTIVE_INDICATOR_HEIGHT, 4.0);
        assert_eq!(component::radio::ICON_SIZE, 20.0);
        assert_eq!(component::radio::TARGET_SIZE, 48.0);
        assert_eq!(component::radio::STATE_LAYER_SIZE, 40.0);
        assert_eq!(component::radio::OUTER_RING_WIDTH, 2.0);
        assert_eq!(component::radio::INNER_DOT_SIZE, 10.0);
        assert_eq!(component::radio::LABEL_TEXT_SIZE, 16.0);
        assert_eq!(component::radio::LABEL_TEXT_LINE_HEIGHT, 24.0);
        assert_eq!(component::radio::LABEL_TEXT_WEIGHT, 400);
        assert_eq!(component::radio::SELECT_TRANSITION_DURATION_MS, 300);
        assert_eq!(component::radio::ICON_COLOR_TRANSITION_DURATION_MS, 50);
        assert_eq!(
            component::radio::SELECT_TRANSITION_EASING,
            motion::EASING_EMPHASIZED_DECELERATE
        );
        assert_eq!(component::text_field::CONTAINER_HEIGHT, 56.0);
        assert_eq!(component::text_field::LEADING_SPACE, 16.0);
        assert_eq!(component::text_field::TRAILING_SPACE, 16.0);
        assert_eq!(component::text_field::TOP_SPACE, 16.0);
        assert_eq!(component::text_field::BOTTOM_SPACE, 16.0);
        assert_eq!(component::text_field::INPUT_TEXT_SIZE, 16.0);
        assert_eq!(component::text_field::INPUT_TEXT_LINE_HEIGHT, 24.0);
        assert_eq!(component::text_field::INPUT_TEXT_WEIGHT, 400);
        assert_eq!(component::text_field::LABEL_TEXT_SIZE, 16.0);
        assert_eq!(component::text_field::LABEL_TEXT_LINE_HEIGHT, 24.0);
        assert_eq!(component::text_field::LABEL_TEXT_WEIGHT, 400);
        assert_eq!(component::text_field::LABEL_TEXT_PADDING_BOTTOM, 8.0);
        assert_eq!(component::text_field::LABEL_TEXT_POPULATED_SIZE, 12.0);
        assert_eq!(
            component::text_field::LABEL_TEXT_POPULATED_LINE_HEIGHT,
            16.0
        );
        assert_eq!(component::text_field::LABEL_TEXT_POPULATED_WEIGHT, 400);
        assert_eq!(component::text_field::OUTLINE_LABEL_PADDING, 4.0);
        assert_eq!(component::text_field::FOCUS_OUTLINE_WIDTH, 3.0);
        assert_eq!(component::text_field::LABEL_TRANSITION_DURATION_MS, 150);
        assert_eq!(
            component::text_field::LABEL_TRANSITION_EASING,
            motion::EASING_STANDARD
        );
        assert_eq!(component::text_field::DISABLED_LEADING_ICON_OPACITY, 0.38);
        assert_eq!(component::divider::THICKNESS, 1.0);
        assert_eq!(component::divider::LIST_ITEM_LEADING_SPACE, 16);
        assert_eq!(component::list::ONE_LINE_CONTAINER_HEIGHT, 56.0);
        assert_eq!(component::menu::CONTAINER_ELEVATION_LEVEL, 2);
        assert_eq!(component::menu::TOP_SPACE, 8.0);
        assert_eq!(component::select::MENU_CONTAINER_ELEVATION_LEVEL, 2);
        assert_eq!(component::select::MENU_LIST_ITEM_CONTAINER_HEIGHT, 48.0);
        assert_eq!(component::select::TRAILING_ICON_SIZE, 24.0);
        assert_eq!(component::select::TEXT_FIELD_DISABLED_OUTLINE_WIDTH, 1.0);
        assert_eq!(component::dialog::CONTAINER_ELEVATION_LEVEL, 3);
        assert_eq!(component::data_table::CONTAINER_SHAPE, 4.0);
        assert_eq!(component::data_table::OUTLINE_WIDTH, 1.0);
        assert_eq!(component::data_table::HEADER_CONTAINER_HEIGHT, 56.0);
        assert_eq!(component::data_table::ROW_ITEM_CONTAINER_HEIGHT, 52.0);
        assert_eq!(component::card::CONTAINER_SHAPE, 12.0);
        assert_eq!(component::card::ICON_SIZE, 24.0);
        assert_eq!(component::card::ELEVATED_ELEVATION.active, 1);
        assert_eq!(component::card::ELEVATED_ELEVATION.hovered, 2);
        assert_eq!(component::card::FILLED_ELEVATION.dragged, 3);
        assert_eq!(component::card::OUTLINED_OUTLINE_WIDTH, 1.0);
        assert_eq!(component::fab::CONTAINER_WIDTH, 56.0);
        assert_eq!(component::fab::CONTAINER_HEIGHT, 56.0);
        assert_eq!(component::fab::CONTAINER_SHAPE, 16.0);
        assert_eq!(component::fab::ICON_SIZE, 24.0);
        assert_eq!(component::fab::ELEVATION.active, 3);
        assert_eq!(component::fab::ELEVATION.hovered, 4);
        assert_eq!(component::icon_button::CONTAINER_WIDTH, 40.0);
        assert_eq!(component::icon_button::CONTAINER_HEIGHT, 40.0);
        assert_eq!(component::icon_button::CONTAINER_SHAPE, 9999.0);
        assert_eq!(component::icon_button::ICON_SIZE, 24.0);
        assert_eq!(component::icon_button::DISABLED_CONTAINER_OPACITY, 0.12);
        assert_eq!(component::icon_button::OUTLINED_OUTLINE_WIDTH, 1.0);
        assert_eq!(component::chip::CONTAINER_HEIGHT, 32.0);
        assert_eq!(component::chip::CONTAINER_SHAPE, 8.0);
        assert_eq!(component::chip::OUTLINE_WIDTH, 1.0);
        assert_eq!(component::chip::SELECTED_OUTLINE_WIDTH, 0.0);
        assert_eq!(component::chip::ICON_SIZE, 18.0);
        assert_eq!(component::chip::LABEL_TEXT_SIZE, 14.0);
        assert_eq!(component::chip::LABEL_TEXT_LINE_HEIGHT, 20.0);
        assert_eq!(component::chip::LABEL_TEXT_WEIGHT, 500);
        assert_eq!(component::chip::LEADING_SPACE, 16.0);
        assert_eq!(component::chip::TRAILING_SPACE, 16.0);
        assert_eq!(component::chip::ICON_LABEL_SPACE, 8.0);
        assert_eq!(component::chip::WITH_LEADING_ICON_LEADING_SPACE, 8.0);
        assert_eq!(component::chip::WITH_TRAILING_ICON_TRAILING_SPACE, 8.0);
        assert_eq!(component::chip::AVATAR_SIZE, 24.0);
        assert_eq!(component::chip::ELEVATED_ELEVATION.active, 1);
        assert_eq!(component::chip::ELEVATED_ELEVATION.hovered, 2);
        assert_eq!(component::chip::SELECTED_FLAT_ELEVATION.hovered, 1);
        assert_eq!(component::tooltip::PLAIN_CONTAINER_SHAPE, 4.0);
        assert_eq!(component::tooltip::PLAIN_SUPPORTING_TEXT.size, 12.0);
        assert_eq!(component::tooltip::RICH_CONTAINER_SHAPE, 12.0);
        assert_eq!(component::tooltip::RICH_CONTAINER_ELEVATION_LEVEL, 2);
        assert_eq!(component::tooltip::RICH_SUBHEAD_TEXT.size, 14.0);
        assert_eq!(component::tooltip::RICH_SUPPORTING_TEXT.size, 14.0);
        assert_eq!(component::tooltip::RICH_ACTION_LABEL_TEXT.size, 14.0);
    }

    #[test]
    fn m3_typography_tokens_match_google_values() {
        assert_eq!(typography::DISPLAY_LARGE.size, 57.0);
        assert_eq!(typography::DISPLAY_LARGE.line_height, 64.0);
        assert_eq!(typography::DISPLAY_LARGE.tracking, -0.25);
        assert_eq!(typography::LABEL_LARGE.size, 14.0);
        assert_eq!(typography::LABEL_LARGE.weight, 500);
        assert_eq!(typography::BODY_MEDIUM.tracking, 0.25);
    }
}
