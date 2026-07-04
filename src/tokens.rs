pub mod state {
    pub const HOVER_STATE_LAYER_OPACITY: f32 = 0.08;
    pub const FOCUS_STATE_LAYER_OPACITY: f32 = 0.10;
    pub const PRESSED_STATE_LAYER_OPACITY: f32 = 0.10;
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

        pub fn transform(self, progress: f32) -> f32 {
            if progress <= 0.0 {
                return 0.0;
            }

            if progress >= 1.0 {
                return 1.0;
            }

            let target_x = progress.clamp(0.0, 1.0);
            let mut start = 0.0;
            let mut end = 1.0;

            for _ in 0..20 {
                let midpoint = (start + end) / 2.0;

                if bezier_axis(midpoint, self.x1, self.x2) < target_x {
                    start = midpoint;
                } else {
                    end = midpoint;
                }
            }

            bezier_axis((start + end) / 2.0, self.y1, self.y2).clamp(0.0, 1.0)
        }
    }

    fn bezier_axis(t: f32, p1: f32, p2: f32) -> f32 {
        let inverse = 1.0 - t;

        3.0 * inverse * inverse * t * p1 + 3.0 * inverse * t * t * p2 + t * t * t
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

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Spring {
        pub damping_ratio: f32,
        pub stiffness: f32,
    }

    pub const SPRING_DEFAULT_DISPLACEMENT_THRESHOLD: f32 = 0.01;

    pub const EXPRESSIVE_DEFAULT_SPATIAL: Spring = Spring {
        damping_ratio: 0.8,
        stiffness: 380.0,
    };
    pub const EXPRESSIVE_DEFAULT_EFFECTS: Spring = Spring {
        damping_ratio: 1.0,
        stiffness: 1600.0,
    };
    pub const EXPRESSIVE_FAST_SPATIAL: Spring = Spring {
        damping_ratio: 0.6,
        stiffness: 800.0,
    };
    pub const EXPRESSIVE_FAST_EFFECTS: Spring = Spring {
        damping_ratio: 1.0,
        stiffness: 3800.0,
    };
    pub const EXPRESSIVE_SLOW_SPATIAL: Spring = Spring {
        damping_ratio: 0.8,
        stiffness: 200.0,
    };
    pub const EXPRESSIVE_SLOW_EFFECTS: Spring = Spring {
        damping_ratio: 1.0,
        stiffness: 800.0,
    };
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
    pub mod badge {
        pub const SMALL_SIZE: f32 = 6.0;
        pub const LARGE_CONTAINER_HEIGHT: f32 = 16.0;
        pub const LARGE_CONTAINER_MIN_WIDTH: f32 = 16.0;
        pub const LARGE_CONTAINER_MAX_WIDTH: f32 = 34.0;
        pub const LARGE_CONTAINER_SHAPE: f32 = 8.0;
        pub const LARGE_HORIZONTAL_SPACE: f32 = 4.0;
        pub const ICON_ONLY_OFFSET: f32 = 6.0;
        pub const WITH_CONTENT_HORIZONTAL_OFFSET: f32 = 12.0;
        pub const WITH_CONTENT_VERTICAL_OFFSET: f32 = 14.0;
        pub const LABEL_TEXT: super::super::typography::TypeScale =
            super::super::typography::LABEL_SMALL;
    }

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
        pub const HANDLE_ELEVATION: f32 = 2.0;
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
        pub const ACTIVE_WAVE_AMPLITUDE: f32 = 3.0;
        pub const ACTIVE_WAVE_WAVELENGTH: f32 = 40.0;
        pub const INDETERMINATE_ACTIVE_WAVE_WAVELENGTH: f32 = 20.0;
        pub const STOP_SIZE: f32 = 4.0;
        pub const STOP_TRAILING_SPACE: f32 = 0.0;
        pub const TRACK_ACTIVE_SPACE: f32 = 4.0;
        pub const TRACK_HEIGHT: f32 = 4.0;
        pub const TRACK_SHAPE: f32 = super::super::shape::CORNER_NONE;
        pub const TRACK_THICKNESS: f32 = 4.0;
        pub const WAVE_HEIGHT: f32 = 10.0;
        pub const DETERMINATE_TRANSITION_DURATION_MS: u16 =
            super::super::motion::DURATION_MEDIUM1_MS;
        pub const INDETERMINATE_DURATION_MS: u16 = 1750;
        pub const FIRST_LINE_HEAD_DURATION_MS: u16 = 1000;
        pub const FIRST_LINE_TAIL_DURATION_MS: u16 = 1000;
        pub const SECOND_LINE_HEAD_DURATION_MS: u16 = 850;
        pub const SECOND_LINE_TAIL_DURATION_MS: u16 = 850;
        pub const FIRST_LINE_HEAD_DELAY_MS: u16 = 0;
        pub const FIRST_LINE_TAIL_DELAY_MS: u16 = 250;
        pub const SECOND_LINE_HEAD_DELAY_MS: u16 = 650;
        pub const SECOND_LINE_TAIL_DELAY_MS: u16 = 900;
        pub const DETERMINATE_EASING: super::super::motion::CubicBezier =
            super::super::motion::CubicBezier::new(0.4, 0.0, 0.6, 1.0);
    }

    pub mod loading_indicator {
        pub const CONTAINER_WIDTH: f32 = 48.0;
        pub const CONTAINER_HEIGHT: f32 = 48.0;
        pub const CONTAINER_SHAPE: f32 = super::super::shape::CORNER_FULL;
        pub const ACTIVE_SIZE: f32 = 38.0;
        pub const MORPH_INTERVAL_MS: u16 = 650;
        pub const GLOBAL_ROTATION_DURATION_MS: u16 = 4666;
        pub const INDETERMINATE_SHAPE_COUNT: usize = 7;
        pub const DETERMINATE_SHAPE_COUNT: usize = 2;
        pub const MORPH_SPRING_DAMPING_RATIO: f32 = 0.6;
        pub const MORPH_SPRING_STIFFNESS: f32 = 200.0;
        pub const ACTIVE_INDICATOR_SCALE: f32 = ACTIVE_SIZE / CONTAINER_WIDTH;
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
        pub const TOP_SPACE: f32 = 12.0;
        pub const BOTTOM_SPACE: f32 = 12.0;
        pub const LEADING_ICON_SIZE: f32 = 24.0;
        pub const LEADING_AVATAR_SIZE: f32 = 40.0;
        pub const LABEL_TEXT: super::super::typography::TypeScale =
            super::super::typography::BODY_LARGE;
        pub const SUPPORTING_TEXT: super::super::typography::TypeScale =
            super::super::typography::BODY_MEDIUM;
        pub const TRAILING_SUPPORTING_TEXT: super::super::typography::TypeScale =
            super::super::typography::LABEL_SMALL;
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

    pub mod navigation_bar {
        pub const CONTAINER_HEIGHT: f32 = 80.0;
        pub const CONTAINER_ELEVATION_LEVEL: u8 = 2;
        pub const ACTIVE_INDICATOR_WIDTH: f32 = 64.0;
        pub const ACTIVE_INDICATOR_HEIGHT: f32 = 32.0;
        pub const ACTIVE_INDICATOR_SHAPE: f32 = super::super::shape::CORNER_FULL;
        pub const ICON_SIZE: f32 = 24.0;
        pub const LABEL_TEXT: super::super::typography::TypeScale =
            super::super::typography::LABEL_MEDIUM;
        pub const ITEM_HORIZONTAL_PADDING: f32 = 8.0;
        pub const INDICATOR_TO_LABEL_PADDING: f32 = 4.0;
        pub const INDICATOR_VERTICAL_OFFSET: f32 = 12.0;
        pub const ITEM_ANIMATION_DURATION_MS: u16 = super::super::motion::DURATION_SHORT2_MS;
    }

    pub mod navigation_rail {
        pub const CONTAINER_WIDTH: f32 = 96.0;
        pub const EXPANDED_CONTAINER_WIDTH: f32 = 220.0;
        pub const CONTAINER_ELEVATION_LEVEL: u8 = 3;
        pub const ACTIVE_INDICATOR_WIDTH: f32 = 56.0;
        pub const ACTIVE_INDICATOR_HEIGHT: f32 = 32.0;
        pub const EXPANDED_ACTIVE_INDICATOR_HEIGHT: f32 = 56.0;
        pub const EXPANDED_ACTIVE_INDICATOR_MARGIN_HORIZONTAL: f32 = 20.0;
        pub const EXPANDED_ACTIVE_INDICATOR_PADDING_START: f32 = 16.0;
        pub const EXPANDED_ACTIVE_INDICATOR_PADDING_END: f32 = 16.0;
        pub const ACTIVE_INDICATOR_SHAPE: f32 = super::super::shape::CORNER_FULL;
        pub const ICON_SIZE: f32 = 24.0;
        pub const ICON_LABEL_HORIZONTAL_SPACE: f32 = 8.0;
        pub const LABEL_TEXT: super::super::typography::TypeScale =
            super::super::typography::LABEL_MEDIUM;
        pub const NO_LABEL_ACTIVE_INDICATOR_HEIGHT: f32 = 56.0;
        pub const ITEM_WIDTH: f32 = CONTAINER_WIDTH;
        pub const ITEM_HEIGHT: f32 = 64.0;
        pub const VERTICAL_PADDING: f32 = 4.0;
        pub const CONTENT_TOP_MARGIN: f32 = 44.0;
        pub const ITEM_TOP_PADDING: f32 = 6.0;
        pub const ITEM_VERTICAL_PADDING: f32 = 4.0;
        pub const HEADER_PADDING: f32 = 40.0;
        pub const ITEM_ANIMATION_DURATION_MS: u16 = super::super::motion::DURATION_SHORT3_MS;
    }

    pub mod navigation_drawer {
        pub const CONTAINER_WIDTH: f32 = 360.0;
        pub const MINIMUM_CONTAINER_WIDTH: f32 = 240.0;
        pub const ACTIVE_INDICATOR_WIDTH: f32 = 336.0;
        pub const ACTIVE_INDICATOR_HEIGHT: f32 = 56.0;
        pub const ACTIVE_INDICATOR_SHAPE: f32 = super::super::shape::CORNER_FULL;
        pub const ICON_SIZE: f32 = 24.0;
        pub const LABEL_TEXT: super::super::typography::TypeScale =
            super::super::typography::LABEL_LARGE;
        pub const HEADLINE_TEXT: super::super::typography::TypeScale =
            super::super::typography::TITLE_SMALL;
        pub const ITEM_HORIZONTAL_PADDING: f32 = 12.0;
        pub const ITEM_CONTENT_LEADING_SPACE: f32 = 16.0;
        pub const ITEM_CONTENT_TRAILING_SPACE: f32 = 24.0;
        pub const ICON_LABEL_SPACE: f32 = 12.0;
        pub const LABEL_BADGE_SPACE: f32 = 12.0;
        pub const MODAL_CONTAINER_ELEVATION_LEVEL: u8 = 1;
        pub const STANDARD_CONTAINER_ELEVATION_LEVEL: u8 = 0;
    }

    pub mod adaptive_navigation {
        pub const WIDTH_COMPACT_MAX: f32 = 600.0;
        pub const WIDTH_MEDIUM_MAX: f32 = 840.0;
        pub const HEIGHT_COMPACT_MAX: f32 = 480.0;
        pub const HEIGHT_MEDIUM_MAX: f32 = 900.0;
    }

    pub mod dialog {
        pub const CONTAINER_SHAPE: f32 = super::super::shape::CORNER_EXTRA_LARGE;
        pub const CONTAINER_ELEVATION_LEVEL: u8 = 3;
        pub const CONTAINER_MIN_WIDTH: f32 = 280.0;
        pub const CONTAINER_MAX_WIDTH: f32 = 560.0;
        pub const CONTAINER_PADDING: f32 = 24.0;
        pub const ICON_SIZE: f32 = 24.0;
        pub const ICON_BOTTOM_PADDING: f32 = 16.0;
        pub const TITLE_BOTTOM_PADDING: f32 = 16.0;
        pub const SUPPORTING_TEXT_BOTTOM_PADDING: f32 = 24.0;
        pub const ACTIONS_HORIZONTAL_SPACING: f32 = 8.0;
        pub const ACTIONS_VERTICAL_SPACING: f32 = 8.0;
        pub const SCRIM_OPACITY: f32 = 0.32;

        // Android framework @style/Animation.Dialog uses dialog_enter/dialog_exit.
        pub const ENTER_SCALE_FROM: f32 = 0.9;
        pub const EXIT_SCALE_TO: f32 = 0.9;
        pub const SCALE_ANIMATION_DURATION_MS: u16 = 220;
        pub const ALPHA_ANIMATION_DURATION_MS: u16 = 150;
        pub const SCRIM_ANIMATION_DURATION_MS: u16 = 220;
        pub const DECELERATE_CUBIC_FACTOR: f32 = 1.5;
        pub const DECELERATE_QUINT_FACTOR: f32 = 2.5;
        pub const ACTION_LABEL_TEXT: super::super::typography::TypeScale =
            super::super::typography::LABEL_LARGE;
        pub const HEADLINE_TEXT: super::super::typography::TypeScale =
            super::super::typography::HEADLINE_SMALL;
        pub const SUPPORTING_TEXT: super::super::typography::TypeScale =
            super::super::typography::BODY_MEDIUM;
    }

    pub mod date_picker {
        pub const CONTAINER_WIDTH: f32 = 360.0;
        pub const CONTAINER_HEIGHT: f32 = 568.0;
        pub const CONTAINER_SHAPE: f32 = super::super::shape::CORNER_EXTRA_LARGE;
        pub const CONTAINER_ELEVATION_LEVEL: u8 = 3;
        pub const HEADER_CONTAINER_HEIGHT: f32 = 120.0;
        pub const HEADER_TITLE_START_SPACE: f32 = 24.0;
        pub const HEADER_TITLE_END_SPACE: f32 = 12.0;
        pub const HEADER_TITLE_TOP_SPACE: f32 = 16.0;
        pub const HEADER_HEADLINE_START_SPACE: f32 = 24.0;
        pub const HEADER_HEADLINE_END_SPACE: f32 = 12.0;
        pub const HEADER_HEADLINE_BOTTOM_SPACE: f32 = 12.0;
        pub const RANGE_HEADER_CONTAINER_HEIGHT: f32 = 128.0;
        pub const HORIZONTAL_SPACE: f32 = 12.0;
        pub const RANGE_MONTH_SUBHEAD_START_SPACE: f32 = 24.0;
        pub const RANGE_MONTH_SUBHEAD_TOP_SPACE: f32 = 20.0;
        pub const RANGE_MONTH_SUBHEAD_BOTTOM_SPACE: f32 = 8.0;
        pub const DIALOG_ACTIONS_END_SPACE: f32 = 6.0;
        pub const DIALOG_ACTIONS_BOTTOM_SPACE: f32 = 8.0;
        pub const DIALOG_ACTIONS_MAIN_AXIS_SPACE: f32 = 8.0;
        pub const DIALOG_ACTIONS_CROSS_AXIS_SPACE: f32 = 12.0;
        pub const MONTH_YEAR_CONTAINER_HEIGHT: f32 = 56.0;
        pub const WEEKDAY_CONTAINER_HEIGHT: f32 = 48.0;
        pub const DATE_CONTAINER_WIDTH: f32 = 40.0;
        pub const DATE_CONTAINER_HEIGHT: f32 = 40.0;
        pub const DATE_CONTAINER_SHAPE: f32 = super::super::shape::CORNER_FULL;
        pub const DATE_STATE_LAYER_WIDTH: f32 = 40.0;
        pub const DATE_STATE_LAYER_HEIGHT: f32 = 40.0;
        pub const DATE_TODAY_OUTLINE_WIDTH: f32 = 1.0;
        pub const CALENDAR_CELL_SIZE: f32 = 48.0;
        pub const YEAR_CONTAINER_WIDTH: f32 = 72.0;
        pub const YEAR_CONTAINER_HEIGHT: f32 = 36.0;
        pub const YEAR_VERTICAL_SPACE: f32 = 16.0;
        pub const YEARS_IN_ROW: usize = 3;
        pub const MAX_CALENDAR_ROWS: usize = 6;
        pub const TITLE_TEXT: super::super::typography::TypeScale =
            super::super::typography::LABEL_LARGE;
        pub const HEADLINE_TEXT: super::super::typography::TypeScale =
            super::super::typography::HEADLINE_LARGE;
        pub const RANGE_HEADLINE_TEXT: super::super::typography::TypeScale =
            super::super::typography::TITLE_LARGE;
        pub const RANGE_MONTH_SUBHEAD_TEXT: super::super::typography::TypeScale =
            super::super::typography::TITLE_SMALL;
        pub const DATE_LABEL_TEXT: super::super::typography::TypeScale =
            super::super::typography::BODY_LARGE;
        pub const WEEKDAY_LABEL_TEXT: super::super::typography::TypeScale =
            super::super::typography::BODY_LARGE;
        pub const YEAR_LABEL_TEXT: super::super::typography::TypeScale =
            super::super::typography::BODY_LARGE;
    }

    pub mod time_picker {
        pub const CLOCK_DIAL_SIZE: f32 = 256.0;
        pub const CLOCK_DIAL_SHAPE: f32 = super::super::shape::CORNER_FULL;
        pub const CLOCK_DIAL_SELECTOR_CENTER_SIZE: f32 = 8.0;
        pub const CLOCK_DIAL_SELECTOR_HANDLE_SIZE: f32 = 48.0;
        pub const CLOCK_DIAL_SELECTOR_TRACK_WIDTH: f32 = 2.0;
        pub const CONTAINER_SHAPE: f32 = super::super::shape::CORNER_EXTRA_LARGE;
        pub const CONTAINER_ELEVATION_LEVEL: u8 = 3;
        pub const PERIOD_SELECTOR_HORIZONTAL_WIDTH: f32 = 216.0;
        pub const PERIOD_SELECTOR_HORIZONTAL_HEIGHT: f32 = 38.0;
        pub const PERIOD_SELECTOR_VERTICAL_WIDTH: f32 = 52.0;
        pub const PERIOD_SELECTOR_VERTICAL_HEIGHT: f32 = 80.0;
        pub const PERIOD_TOGGLE_MARGIN: f32 = 12.0;
        pub const PERIOD_SELECTOR_START_SPACE: f32 = PERIOD_TOGGLE_MARGIN;
        pub const PERIOD_SELECTOR_ITEM_GAP: f32 = PERIOD_SELECTOR_OUTLINE_WIDTH;
        pub const PERIOD_SELECTOR_SHAPE: f32 = super::super::shape::CORNER_SMALL;
        pub const PERIOD_SELECTOR_OUTLINE_WIDTH: f32 = 1.0;
        pub const TIME_SELECTOR_WIDTH: f32 = 96.0;
        pub const TIME_SELECTOR_24H_WIDTH: f32 = 114.0;
        pub const TIME_SELECTOR_HEIGHT: f32 = 80.0;
        pub const TIME_SELECTOR_SHAPE: f32 = super::super::shape::CORNER_SMALL;
        pub const TIME_SCROLL_FIELD_WIDTH: f32 = 100.0;
        pub const TIME_SCROLL_FIELD_HEIGHT: f32 = 120.0;
        pub const TIME_SCROLL_ITEM_HEIGHT: f32 = 40.0;
        pub const TIME_SCROLL_SEPARATOR_WIDTH: f32 = 16.0;
        pub const RICH_PERIOD_SELECTOR_WIDTH: f32 = 56.0;
        pub const RICH_PERIOD_SELECTOR_HEIGHT: f32 = 120.0;
        pub const RICH_PERIOD_SELECTOR_START_SPACE: f32 = 16.0;
        pub const RICH_PERIOD_SELECTOR_ITEM_GAP: f32 = PERIOD_SELECTOR_OUTLINE_WIDTH;
        pub const CLOCK_DISPLAY_BOTTOM_SPACE: f32 = 36.0;
        pub const CLOCK_FACE_BOTTOM_SPACE: f32 = 24.0;
        pub const DISPLAY_SEPARATOR_WIDTH: f32 = 24.0;
        pub const OUTER_CIRCLE_RADIUS_RATIO: f32 = 101.0 / CLOCK_DIAL_SIZE;
        pub const INNER_CIRCLE_RADIUS_RATIO: f32 = 69.0 / CLOCK_DIAL_SIZE;
        pub const MAX_DISTANCE: f32 = 74.0;
        pub const HEADLINE_TEXT: super::super::typography::TypeScale =
            super::super::typography::LABEL_MEDIUM;
        pub const CLOCK_DIAL_LABEL_TEXT: super::super::typography::TypeScale =
            super::super::typography::BODY_LARGE;
        pub const PERIOD_SELECTOR_LABEL_TEXT: super::super::typography::TypeScale =
            super::super::typography::TITLE_MEDIUM;
        pub const TIME_SELECTOR_LABEL_TEXT: super::super::typography::TypeScale =
            super::super::typography::DISPLAY_LARGE;
    }

    pub mod time_picker_dialog {
        pub const CONTAINER_SHAPE: f32 = super::super::shape::CORNER_EXTRA_LARGE;
        pub const CONTAINER_ELEVATION_LEVEL: u8 = 3;
        pub const CONTENT_PADDING: f32 = 24.0;
        pub const TITLE_TOP_SPACE: f32 = 24.0;
        pub const TITLE_BOTTOM_SPACE: f32 = 20.0;
        pub const ACTIONS_BOTTOM_SPACE: f32 = 24.0;
        pub const ACTIONS_HORIZONTAL_SPACE: f32 = 8.0;
        pub const MIN_HEIGHT_FOR_TIME_PICKER: f32 = 300.0;
        pub const RICH_CONTENT_PADDING: f32 = 12.0;
        pub const RICH_CONTENT_TOP_SPACE: f32 = 12.0;
        pub const RICH_CONTENT_ACTIONS_SPACE: f32 = 12.0;
        pub const RICH_ACTIONS_BOTTOM_SPACE: f32 = 12.0;
        pub const TITLE_TEXT: super::super::typography::TypeScale =
            super::super::typography::LABEL_MEDIUM;
    }

    pub mod time_input {
        pub const CONTAINER_SHAPE: f32 = super::super::shape::CORNER_EXTRA_LARGE;
        pub const CONTAINER_ELEVATION_LEVEL: u8 = 3;
        pub const PERIOD_SELECTOR_CONTAINER_WIDTH: f32 = 52.0;
        pub const PERIOD_SELECTOR_CONTAINER_HEIGHT: f32 = 72.0;
        pub const PERIOD_SELECTOR_CONTAINER_SHAPE: f32 = super::super::shape::CORNER_SMALL;
        pub const PERIOD_SELECTOR_OUTLINE_WIDTH: f32 = 1.0;
        pub const TIME_FIELD_CONTAINER_WIDTH: f32 = 96.0;
        pub const TIME_FIELD_CONTAINER_HEIGHT: f32 = 72.0;
        pub const TIME_FIELD_CONTAINER_SHAPE: f32 = super::super::shape::CORNER_SMALL;
        pub const TIME_FIELD_FOCUS_OUTLINE_WIDTH: f32 = 2.0;
        pub const TIME_FIELD_SUPPORTING_TEXT_TOP_SPACE: f32 = 7.0;
        pub const TIME_FIELD_SUPPORTING_TEXT_LINES: f32 = 2.0;
        pub const DISPLAY_SEPARATOR_WIDTH: f32 = 24.0;
        pub const PERIOD_SELECTOR_LABEL_TEXT: super::super::typography::TypeScale =
            super::super::typography::TITLE_MEDIUM;
        pub const TIME_FIELD_LABEL_TEXT: super::super::typography::TypeScale =
            super::super::typography::DISPLAY_MEDIUM;
        pub const TIME_FIELD_SUPPORTING_TEXT: super::super::typography::TypeScale =
            super::super::typography::BODY_SMALL;
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
        pub const SMALL_CONTAINER_WIDTH: f32 = 40.0;
        pub const SMALL_CONTAINER_HEIGHT: f32 = 40.0;
        pub const SMALL_CONTAINER_SHAPE: f32 = super::super::shape::CORNER_MEDIUM;
        pub const SMALL_ICON_SIZE: f32 = 24.0;
        pub const LARGE_CONTAINER_WIDTH: f32 = 96.0;
        pub const LARGE_CONTAINER_HEIGHT: f32 = 96.0;
        pub const LARGE_CONTAINER_SHAPE: f32 = super::super::shape::CORNER_EXTRA_LARGE;
        pub const LARGE_ICON_SIZE: f32 = 36.0;
        pub const EXTENDED_CONTAINER_HEIGHT: f32 = 56.0;
        pub const EXTENDED_CONTAINER_SHAPE: f32 = super::super::shape::CORNER_LARGE;
        pub const EXTENDED_ICON_SIZE: f32 = 24.0;
        pub const EXTENDED_ICON_LABEL_SPACE: f32 = 12.0;
        pub const EXTENDED_LEADING_SPACE: f32 = 16.0;
        pub const EXTENDED_TRAILING_SPACE: f32 = 20.0;
        pub const EXTENDED_LABEL_TEXT: super::super::typography::TypeScale =
            super::super::typography::LABEL_LARGE;

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
        pub const EXTENDED_ELEVATION: super::button::ElevationLevels =
            super::button::ElevationLevels {
                active: 3,
                hovered: 4,
                pressed: 3,
                disabled: 0,
            };
        pub const EXTENDED_LOWERED_ELEVATION: super::button::ElevationLevels =
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

    pub mod segmented_button {
        pub const CONTAINER_HEIGHT: f32 = 40.0;
        pub const CONTAINER_SHAPE: f32 = super::super::shape::CORNER_FULL;
        pub const OUTLINE_WIDTH: f32 = 1.0;
        pub const WITH_ICON_ICON_SIZE: f32 = 18.0;
        pub const LEADING_SPACE: f32 = 12.0;
        pub const TRAILING_SPACE: f32 = 12.0;
        pub const ICON_LABEL_SPACE: f32 = 8.0;
        pub const LABEL_TEXT: super::super::typography::TypeScale =
            super::super::typography::LABEL_LARGE;
        pub const DISABLED_ICON_OPACITY: f32 = 0.38;
        pub const DISABLED_LABEL_TEXT_OPACITY: f32 = 0.38;
        pub const DISABLED_OUTLINE_OPACITY: f32 = 0.12;
        pub const SELECT_TRANSITION_DURATION_MS: u16 = super::super::motion::DURATION_SHORT4_MS;
        pub const SELECT_TRANSITION_EASING: super::super::motion::CubicBezier =
            super::super::motion::EASING_EMPHASIZED;
        pub const FOCUS_STATE_LAYER_OPACITY: f32 = super::super::state::FOCUS_STATE_LAYER_OPACITY;
        pub const HOVER_STATE_LAYER_OPACITY: f32 = super::super::state::HOVER_STATE_LAYER_OPACITY;
        pub const PRESSED_STATE_LAYER_OPACITY: f32 = super::super::state::FOCUS_STATE_LAYER_OPACITY;
    }

    pub mod snackbar {
        pub const ICON_SIZE: f32 = 24.0;
        pub const WITH_SINGLE_LINE_CONTAINER_HEIGHT: f32 = 48.0;
        pub const WITH_TWO_LINES_CONTAINER_HEIGHT: f32 = 68.0;
        pub const MAX_WIDTH: f32 = 568.0;
        pub const HORIZONTAL_MARGIN: f32 = 16.0;
        pub const BOTTOM_MARGIN: f32 = 16.0;
        pub const CONTAINER_ELEVATION_LEVEL: u8 = 3;
        pub const CONTAINER_SHAPE: f32 = super::super::shape::CORNER_EXTRA_SMALL;
        pub const SUPPORTING_TEXT: super::super::typography::TypeScale =
            super::super::typography::BODY_MEDIUM;
        pub const ACTION_LABEL_TEXT: super::super::typography::TypeScale =
            super::super::typography::LABEL_LARGE;
        pub const SLIDE_ANIMATION_DURATION_MS: u16 = 250;
        pub const CONTENT_FADE_ANIMATION_DURATION_MS: u16 = 180;
        pub const LONG_DURATION_MS: u16 = 2750;
        pub const SLIDE_ANIMATION_EASING: super::super::motion::CubicBezier =
            super::super::motion::EASING_LEGACY;
        pub const CONTENT_FADE_ANIMATION_EASING: super::super::motion::CubicBezier =
            super::super::motion::EASING_LEGACY;
    }

    pub mod search_bar {
        pub const AVATAR_SIZE: f32 = 30.0;
        pub const CONTAINER_HEIGHT: f32 = 56.0;
        pub const ICON_SIZE: f32 = 24.0;
        pub const LEADING_SPACE: f32 = 16.0;
        pub const TRAILING_SPACE: f32 = 16.0;
        pub const LEADING_ICON_LABEL_SPACE: f32 = 16.0;
        pub const TRAILING_ICON_LABEL_SPACE: f32 = 16.0;
        pub const CONTAINER_ELEVATION_LEVEL: u8 = 3;
        pub const CONTAINER_SHAPE: f32 = super::super::shape::CORNER_FULL;
        pub const INPUT_TEXT: super::super::typography::TypeScale =
            super::super::typography::BODY_LARGE;
        pub const SUPPORTING_TEXT: super::super::typography::TypeScale =
            super::super::typography::BODY_LARGE;
        pub const HOVER_STATE_LAYER_OPACITY: f32 = super::super::state::HOVER_STATE_LAYER_OPACITY;
        pub const PRESSED_STATE_LAYER_OPACITY: f32 =
            super::super::state::PRESSED_STATE_LAYER_OPACITY;
    }

    pub mod search_view {
        pub const DOCKED_HEADER_CONTAINER_HEIGHT: f32 = 56.0;
        pub const FULL_SCREEN_HEADER_CONTAINER_HEIGHT: f32 = 72.0;
        pub const LEADING_SPACE: f32 = 16.0;
        pub const TRAILING_SPACE: f32 = 16.0;
        pub const LEADING_ICON_LABEL_SPACE: f32 = 16.0;
        pub const TRAILING_ICON_LABEL_SPACE: f32 = 16.0;
        pub const CONTAINER_ELEVATION_LEVEL: u8 = 3;
        pub const DOCKED_CONTAINER_SHAPE: f32 = super::super::shape::CORNER_EXTRA_LARGE;
        pub const FULL_SCREEN_CONTAINER_SHAPE: f32 = super::super::shape::CORNER_NONE;
        pub const HEADER_INPUT_TEXT: super::super::typography::TypeScale =
            super::super::typography::BODY_LARGE;
        pub const HEADER_SUPPORTING_TEXT: super::super::typography::TypeScale =
            super::super::typography::BODY_LARGE;
    }

    pub mod app_bar {
        pub const AVATAR_SIZE: f32 = 32.0;
        pub const ICON_BUTTON_SPACE: f32 = 0.0;
        pub const ICON_SIZE: f32 = 24.0;
        pub const LEADING_SPACE: f32 = 4.0;
        pub const TRAILING_SPACE: f32 = 4.0;
        pub const CONTAINER_ELEVATION_LEVEL: u8 = 0;
        pub const CONTAINER_SHAPE: f32 = super::super::shape::CORNER_NONE;
        pub const ON_SCROLL_CONTAINER_ELEVATION_LEVEL: u8 = 2;

        pub const SMALL_CONTAINER_HEIGHT: f32 = 64.0;
        pub const SMALL_SEARCH_CONTAINER_HEIGHT: f32 = 56.0;
        pub const SMALL_SEARCH_CONTAINER_SHAPE: f32 = super::super::shape::CORNER_FULL;
        pub const SMALL_TITLE_TEXT: super::super::typography::TypeScale =
            super::super::typography::TITLE_LARGE;
        pub const SMALL_SUBTITLE_TEXT: super::super::typography::TypeScale =
            super::super::typography::LABEL_MEDIUM;

        pub const MEDIUM_CONTAINER_HEIGHT: f32 = 112.0;
        pub const MEDIUM_TITLE_TEXT: super::super::typography::TypeScale =
            super::super::typography::HEADLINE_SMALL;
        pub const MEDIUM_SUBTITLE_TEXT: super::super::typography::TypeScale =
            super::super::typography::LABEL_LARGE;

        pub const LARGE_CONTAINER_HEIGHT: f32 = 152.0;
        pub const LARGE_TITLE_TEXT: super::super::typography::TypeScale =
            super::super::typography::HEADLINE_MEDIUM;
        pub const LARGE_SUBTITLE_TEXT: super::super::typography::TypeScale =
            super::super::typography::TITLE_MEDIUM;
    }

    pub mod bottom_app_bar {
        pub const CONTAINER_HEIGHT: f32 = 80.0;
        pub const CONTAINER_ELEVATION_LEVEL: u8 = 2;
        pub const CONTAINER_SHAPE: f32 = super::super::shape::CORNER_NONE;
    }

    pub mod toolbar {
        pub const DOCKED_CONTAINER_HEIGHT: f32 = 64.0;
        pub const DOCKED_LEADING_SPACE: f32 = 16.0;
        pub const DOCKED_TRAILING_SPACE: f32 = 16.0;
        pub const DOCKED_CONTAINER_SHAPE: f32 = super::super::shape::CORNER_NONE;

        pub const FLOATING_HORIZONTAL_CONTAINER_HEIGHT: f32 = 64.0;
        pub const FLOATING_VERTICAL_CONTAINER_WIDTH: f32 = 64.0;
        pub const FLOATING_CONTAINER_LEADING_SPACE: f32 = 8.0;
        pub const FLOATING_CONTAINER_TRAILING_SPACE: f32 = 8.0;
        pub const FLOATING_CONTAINER_SHAPE: f32 = super::super::shape::CORNER_FULL;
        pub const FLOATING_CONTAINER_ELEVATION_LEVEL: u8 = 3;

        pub const ACTION_CONTAINER_WIDTH: f32 = 48.0;
        pub const ACTION_CONTAINER_HEIGHT: f32 = 48.0;
        pub const ACTION_CONTAINER_SHAPE: f32 = super::super::shape::CORNER_FULL;
        pub const ACTION_SELECTED_CONTAINER_SHAPE: f32 = super::super::shape::CORNER_MEDIUM;
        pub const ACTION_ICON_SIZE: f32 = 24.0;
        pub const ACTION_SPACE: f32 = 0.0;
        pub const FAB_SPACE: f32 = 8.0;
        pub const DISABLED_ICON_OPACITY: f32 = 0.38;
    }

    pub mod bottom_sheet {
        pub const CONTAINER_SHAPE_TOP: f32 = super::super::shape::CORNER_EXTRA_LARGE;
        pub const CONTAINER_SHAPE_BOTTOM: f32 = super::super::shape::CORNER_NONE;
        pub const HIDDEN_CONTAINER_SHAPE: f32 = super::super::shape::CORNER_NONE;
        pub const MODAL_CONTAINER_ELEVATION_LEVEL: u8 = 1;
        pub const STANDARD_CONTAINER_ELEVATION_LEVEL: u8 = 1;
        pub const DRAG_HANDLE_WIDTH: f32 = 32.0;
        pub const DRAG_HANDLE_HEIGHT: f32 = 4.0;
        pub const DRAG_HANDLE_VERTICAL_PADDING: f32 = 22.0;
        pub const CONTENT_PADDING: f32 = 24.0;
        pub const SHEET_PEEK_HEIGHT: f32 = 56.0;
        pub const SHEET_MAX_WIDTH: f32 = 640.0;
        pub const SCRIM_OPACITY: f32 = 0.32;
        pub const POSITIONAL_THRESHOLD: f32 = 56.0;
        pub const VELOCITY_THRESHOLD: f32 = 125.0;
        pub const ANIMATION_DURATION_MS: u16 = super::super::motion::DURATION_MEDIUM2_MS;
        pub const ANIMATION_EASING: super::super::motion::CubicBezier =
            super::super::motion::EASING_LEGACY;
    }

    pub mod side_sheet {
        pub const DOCKED_CONTAINER_WIDTH: f32 = 256.0;
        pub const DETACHED_MARGIN: f32 = 16.0;
        pub const CONTENT_PADDING: f32 = 24.0;
        pub const DOCKED_STANDARD_CONTAINER_SHAPE: f32 = super::super::shape::CORNER_NONE;
        pub const DOCKED_MODAL_CONTAINER_SHAPE: f32 = super::super::shape::CORNER_LARGE;
        pub const DETACHED_CONTAINER_SHAPE: f32 = super::super::shape::CORNER_LARGE;
        pub const MODAL_CONTAINER_ELEVATION_LEVEL: u8 = 1;
        pub const STANDARD_CONTAINER_ELEVATION_LEVEL: u8 = 0;
        pub const SCRIM_OPACITY: f32 = 0.32;
        pub const ANIMATION_DURATION_MS: u16 = 275;
        pub const ANIMATION_EASING: super::super::motion::CubicBezier =
            super::super::motion::EASING_EMPHASIZED;
    }

    pub mod primary_tab {
        pub const CONTAINER_HEIGHT: f32 = 48.0;
        pub const WITH_ICON_AND_LABEL_TEXT_CONTAINER_HEIGHT: f32 = 64.0;
        pub const CONTAINER_ELEVATION_LEVEL: u8 = 0;
        pub const CONTAINER_SHAPE: f32 = super::super::shape::CORNER_NONE;
        pub const ACTIVE_INDICATOR_HEIGHT: f32 = 3.0;
        pub const ACTIVE_INDICATOR_SHAPE_TOP: f32 = 3.0;
        pub const ACTIVE_INDICATOR_SHAPE_BOTTOM: f32 = 0.0;
        pub const ICON_SIZE: f32 = 24.0;
        pub const HORIZONTAL_SPACE: f32 = 16.0;
        pub const INLINE_ICON_LABEL_SPACE: f32 = 8.0;
        pub const STACKED_ICON_LABEL_SPACE: f32 = 2.0;
        pub const LABEL_TEXT: super::super::typography::TypeScale =
            super::super::typography::TITLE_SMALL;
        pub const ACTIVE_HOVER_STATE_LAYER_OPACITY: f32 =
            super::super::state::HOVER_STATE_LAYER_OPACITY;
        pub const ACTIVE_PRESSED_STATE_LAYER_OPACITY: f32 =
            super::super::state::PRESSED_STATE_LAYER_OPACITY;
        pub const INACTIVE_HOVER_STATE_LAYER_OPACITY: f32 =
            super::super::state::HOVER_STATE_LAYER_OPACITY;
        pub const INACTIVE_PRESSED_STATE_LAYER_OPACITY: f32 =
            super::super::state::PRESSED_STATE_LAYER_OPACITY;
        pub const INDICATOR_ANIMATION_DURATION_MS: u16 = super::super::motion::DURATION_MEDIUM1_MS;
        pub const INDICATOR_ANIMATION_EASING: super::super::motion::CubicBezier =
            super::super::motion::EASING_EMPHASIZED;
    }

    pub mod secondary_tab {
        pub const CONTAINER_HEIGHT: f32 = 48.0;
        pub const CONTAINER_ELEVATION_LEVEL: u8 = 0;
        pub const CONTAINER_SHAPE: f32 = super::super::shape::CORNER_NONE;
        pub const ACTIVE_INDICATOR_HEIGHT: f32 = 2.0;
        pub const ACTIVE_INDICATOR_SHAPE: f32 = super::super::shape::CORNER_NONE;
        pub const ICON_SIZE: f32 = 24.0;
        pub const HORIZONTAL_SPACE: f32 = 16.0;
        pub const ICON_LABEL_SPACE: f32 = 8.0;
        pub const LABEL_TEXT: super::super::typography::TypeScale =
            super::super::typography::TITLE_SMALL;
        pub const HOVER_STATE_LAYER_OPACITY: f32 = super::super::state::HOVER_STATE_LAYER_OPACITY;
        pub const PRESSED_STATE_LAYER_OPACITY: f32 =
            super::super::state::PRESSED_STATE_LAYER_OPACITY;
        pub const INDICATOR_ANIMATION_DURATION_MS: u16 = super::super::motion::DURATION_MEDIUM1_MS;
        pub const INDICATOR_ANIMATION_EASING: super::super::motion::CubicBezier =
            super::super::motion::EASING_EMPHASIZED;
    }

    pub mod tooltip {
        pub const SPACING_BETWEEN_TOOLTIP_AND_ANCHOR: f32 = 4.0;
        pub const PLAIN_MIN_HEIGHT: f32 = 24.0;
        pub const PLAIN_MIN_WIDTH: f32 = 40.0;
        pub const PLAIN_MAX_WIDTH: f32 = 200.0;
        pub const PLAIN_HORIZONTAL_SPACE: f32 = 8.0;
        pub const PLAIN_VERTICAL_SPACE: f32 = 4.0;
        pub const PLAIN_CONTAINER_SHAPE: f32 = super::super::shape::CORNER_EXTRA_SMALL;
        pub const PLAIN_SUPPORTING_TEXT: super::super::typography::TypeScale =
            super::super::typography::BODY_SMALL;

        // AndroidX Compose Material3 Tooltip.kt animateTooltip.
        pub const FADE_IN_DURATION_MS: u16 = 150;
        pub const FADE_OUT_DURATION_MS: u16 = 75;
        pub const SCALE_START: f32 = 0.8;
        pub const ANIMATION_DURATION_MS: u16 = FADE_IN_DURATION_MS;

        pub const RICH_MAX_WIDTH: f32 = 320.0;
        pub const RICH_MIN_HEIGHT: f32 = 24.0;
        pub const RICH_MIN_WIDTH: f32 = 40.0;
        pub const RICH_HORIZONTAL_SPACE: f32 = 16.0;
        pub const RICH_TEXT_VERTICAL_SPACE_WITHOUT_TITLE_OR_ACTION: f32 = 4.0;
        pub const RICH_HEIGHT_TO_SUBHEAD_FIRST_LINE: f32 = 28.0;
        pub const RICH_HEIGHT_FROM_SUBHEAD_TO_TEXT_FIRST_LINE: f32 = 24.0;
        pub const RICH_TEXT_BOTTOM_PADDING: f32 = 16.0;
        pub const RICH_ACTION_LABEL_MIN_HEIGHT: f32 = 36.0;
        pub const RICH_ACTION_LABEL_BOTTOM_PADDING: f32 = 8.0;
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
        assert_eq!(state::FOCUS_STATE_LAYER_OPACITY, 0.10);
        assert_eq!(state::PRESSED_STATE_LAYER_OPACITY, 0.10);
        assert_eq!(state::DRAGGED_STATE_LAYER_OPACITY, 0.16);
    }

    #[test]
    fn m3_motion_tokens_match_google_values() {
        assert_eq!(motion::DURATION_SHORT4_MS, 200);
        assert_eq!(motion::DURATION_MEDIUM2_MS, 300);
        assert_eq!(motion::DURATION_EXTRA_LONG4_MS, 1000);
        assert_eq!(motion::SPRING_DEFAULT_DISPLACEMENT_THRESHOLD, 0.01);
        assert_eq!(motion::EXPRESSIVE_DEFAULT_SPATIAL.damping_ratio, 0.8);
        assert_eq!(motion::EXPRESSIVE_DEFAULT_SPATIAL.stiffness, 380.0);
        assert_eq!(motion::EXPRESSIVE_DEFAULT_EFFECTS.damping_ratio, 1.0);
        assert_eq!(motion::EXPRESSIVE_DEFAULT_EFFECTS.stiffness, 1600.0);
        assert_eq!(motion::EXPRESSIVE_FAST_SPATIAL.damping_ratio, 0.6);
        assert_eq!(motion::EXPRESSIVE_FAST_SPATIAL.stiffness, 800.0);
        assert_eq!(motion::EXPRESSIVE_FAST_EFFECTS.damping_ratio, 1.0);
        assert_eq!(motion::EXPRESSIVE_FAST_EFFECTS.stiffness, 3800.0);
        assert_eq!(motion::EXPRESSIVE_SLOW_SPATIAL.damping_ratio, 0.8);
        assert_eq!(motion::EXPRESSIVE_SLOW_SPATIAL.stiffness, 200.0);
        assert_eq!(motion::EXPRESSIVE_SLOW_EFFECTS.damping_ratio, 1.0);
        assert_eq!(motion::EXPRESSIVE_SLOW_EFFECTS.stiffness, 800.0);
        assert_eq!(
            motion::EASING_EMPHASIZED_DECELERATE,
            motion::CubicBezier::new(0.05, 0.7, 0.1, 1.0)
        );
    }

    #[test]
    fn cubic_bezier_transform_clamps_and_reaches_endpoints() {
        assert_eq!(motion::EASING_EMPHASIZED_DECELERATE.transform(-1.0), 0.0);
        assert_eq!(motion::EASING_EMPHASIZED_DECELERATE.transform(0.0), 0.0);
        assert_eq!(motion::EASING_EMPHASIZED_DECELERATE.transform(1.0), 1.0);
        assert_eq!(motion::EASING_EMPHASIZED_DECELERATE.transform(2.0), 1.0);
        assert!(
            motion::EASING_EMPHASIZED_DECELERATE.transform(0.5)
                > motion::EASING_LINEAR.transform(0.5)
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
        assert_eq!(component::badge::SMALL_SIZE, 6.0);
        assert_eq!(component::badge::LARGE_CONTAINER_HEIGHT, 16.0);
        assert_eq!(component::badge::LARGE_CONTAINER_MIN_WIDTH, 16.0);
        assert_eq!(component::badge::LARGE_CONTAINER_MAX_WIDTH, 34.0);
        assert_eq!(component::badge::LARGE_CONTAINER_SHAPE, 8.0);
        assert_eq!(component::badge::LARGE_HORIZONTAL_SPACE, 4.0);
        assert_eq!(component::badge::ICON_ONLY_OFFSET, 6.0);
        assert_eq!(component::badge::WITH_CONTENT_HORIZONTAL_OFFSET, 12.0);
        assert_eq!(component::badge::WITH_CONTENT_VERTICAL_OFFSET, 14.0);
        assert_eq!(component::badge::LABEL_TEXT, typography::LABEL_SMALL);
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
        assert_eq!(component::date_picker::CONTAINER_WIDTH, 360.0);
        assert_eq!(component::date_picker::CONTAINER_HEIGHT, 568.0);
        assert_eq!(component::date_picker::DATE_CONTAINER_WIDTH, 40.0);
        assert_eq!(component::date_picker::DATE_CONTAINER_HEIGHT, 40.0);
        assert_eq!(component::date_picker::HEADER_CONTAINER_HEIGHT, 120.0);
        assert_eq!(component::date_picker::RANGE_HEADER_CONTAINER_HEIGHT, 128.0);
        assert_eq!(
            component::date_picker::RANGE_MONTH_SUBHEAD_START_SPACE,
            24.0
        );
        assert_eq!(component::date_picker::RANGE_MONTH_SUBHEAD_TOP_SPACE, 20.0);
        assert_eq!(
            component::date_picker::RANGE_MONTH_SUBHEAD_BOTTOM_SPACE,
            8.0
        );
        assert_eq!(component::date_picker::DIALOG_ACTIONS_END_SPACE, 6.0);
        assert_eq!(component::date_picker::DIALOG_ACTIONS_BOTTOM_SPACE, 8.0);
        assert_eq!(component::date_picker::DIALOG_ACTIONS_MAIN_AXIS_SPACE, 8.0);
        assert_eq!(
            component::date_picker::DIALOG_ACTIONS_CROSS_AXIS_SPACE,
            12.0
        );
        assert_eq!(component::date_picker::YEAR_CONTAINER_WIDTH, 72.0);
        assert_eq!(component::date_picker::YEAR_CONTAINER_HEIGHT, 36.0);
        assert_eq!(component::time_picker::CLOCK_DIAL_SIZE, 256.0);
        assert_eq!(
            component::time_picker::CLOCK_DIAL_SELECTOR_HANDLE_SIZE,
            48.0
        );
        assert_eq!(component::time_picker::CLOCK_DIAL_SELECTOR_TRACK_WIDTH, 2.0);
        assert_eq!(
            component::time_picker::PERIOD_SELECTOR_HORIZONTAL_WIDTH,
            216.0
        );
        assert_eq!(
            component::time_picker::PERIOD_SELECTOR_VERTICAL_HEIGHT,
            80.0
        );
        assert_eq!(component::time_picker::PERIOD_TOGGLE_MARGIN, 12.0);
        assert_eq!(component::time_picker::PERIOD_SELECTOR_START_SPACE, 12.0);
        assert_eq!(component::time_picker::PERIOD_SELECTOR_ITEM_GAP, 1.0);
        assert_eq!(component::time_picker::TIME_SELECTOR_WIDTH, 96.0);
        assert_eq!(component::time_picker::TIME_SELECTOR_HEIGHT, 80.0);
        assert_eq!(component::time_picker::TIME_SCROLL_FIELD_WIDTH, 100.0);
        assert_eq!(component::time_picker::TIME_SCROLL_FIELD_HEIGHT, 120.0);
        assert_eq!(component::time_picker::TIME_SCROLL_SEPARATOR_WIDTH, 16.0);
        assert_eq!(component::time_picker::RICH_PERIOD_SELECTOR_WIDTH, 56.0);
        assert_eq!(component::time_picker::RICH_PERIOD_SELECTOR_HEIGHT, 120.0);
        assert_eq!(component::time_picker::RICH_PERIOD_SELECTOR_ITEM_GAP, 1.0);
        assert_eq!(component::time_picker_dialog::CONTENT_PADDING, 24.0);
        assert_eq!(component::time_picker_dialog::TITLE_TOP_SPACE, 24.0);
        assert_eq!(component::time_picker_dialog::TITLE_BOTTOM_SPACE, 20.0);
        assert_eq!(component::time_picker_dialog::ACTIONS_BOTTOM_SPACE, 24.0);
        assert_eq!(
            component::time_picker_dialog::MIN_HEIGHT_FOR_TIME_PICKER,
            300.0
        );
        assert_eq!(component::time_picker_dialog::RICH_CONTENT_PADDING, 12.0);
        assert_eq!(component::time_picker_dialog::RICH_CONTENT_TOP_SPACE, 12.0);
        assert_eq!(
            component::time_picker_dialog::RICH_CONTENT_ACTIONS_SPACE,
            12.0
        );
        assert_eq!(
            component::time_picker_dialog::RICH_ACTIONS_BOTTOM_SPACE,
            12.0
        );
        assert_eq!(component::time_input::TIME_FIELD_CONTAINER_WIDTH, 96.0);
        assert_eq!(component::time_input::TIME_FIELD_CONTAINER_HEIGHT, 72.0);
        assert_eq!(
            component::time_input::TIME_FIELD_SUPPORTING_TEXT_TOP_SPACE,
            7.0
        );
        assert_eq!(
            component::time_input::PERIOD_SELECTOR_CONTAINER_HEIGHT,
            72.0
        );
        assert_eq!(component::time_input::DISPLAY_SEPARATOR_WIDTH, 24.0);
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
        assert_eq!(component::slider::HANDLE_ELEVATION, 2.0);
        assert_eq!(component::slider::STATE_LAYER_SIZE, 40.0);
        assert_eq!(component::slider::LABEL_CONTAINER_HEIGHT, 28.0);
        assert_eq!(component::linear_progress::TRACK_HEIGHT, 4.0);
        assert_eq!(component::linear_progress::ACTIVE_INDICATOR_HEIGHT, 4.0);
        assert_eq!(component::linear_progress::ACTIVE_WAVE_AMPLITUDE, 3.0);
        assert_eq!(component::linear_progress::ACTIVE_WAVE_WAVELENGTH, 40.0);
        assert_eq!(
            component::linear_progress::INDETERMINATE_ACTIVE_WAVE_WAVELENGTH,
            20.0
        );
        assert_eq!(component::linear_progress::STOP_SIZE, 4.0);
        assert_eq!(component::linear_progress::TRACK_ACTIVE_SPACE, 4.0);
        assert_eq!(component::linear_progress::TRACK_THICKNESS, 4.0);
        assert_eq!(component::linear_progress::WAVE_HEIGHT, 10.0);
        assert_eq!(
            component::linear_progress::DETERMINATE_TRANSITION_DURATION_MS,
            250
        );
        assert_eq!(component::linear_progress::INDETERMINATE_DURATION_MS, 1750);
        assert_eq!(
            component::linear_progress::FIRST_LINE_HEAD_DURATION_MS,
            1000
        );
        assert_eq!(component::linear_progress::FIRST_LINE_TAIL_DELAY_MS, 250);
        assert_eq!(component::linear_progress::SECOND_LINE_HEAD_DELAY_MS, 650);
        assert_eq!(component::linear_progress::SECOND_LINE_TAIL_DELAY_MS, 900);
        assert_eq!(
            component::linear_progress::DETERMINATE_EASING,
            motion::CubicBezier::new(0.4, 0.0, 0.6, 1.0)
        );
        assert_eq!(component::loading_indicator::CONTAINER_WIDTH, 48.0);
        assert_eq!(component::loading_indicator::CONTAINER_HEIGHT, 48.0);
        assert_eq!(
            component::loading_indicator::CONTAINER_SHAPE,
            shape::CORNER_FULL
        );
        assert_eq!(component::loading_indicator::ACTIVE_SIZE, 38.0);
        assert_eq!(component::loading_indicator::MORPH_INTERVAL_MS, 650);
        assert_eq!(
            component::loading_indicator::GLOBAL_ROTATION_DURATION_MS,
            4666
        );
        assert_eq!(component::loading_indicator::INDETERMINATE_SHAPE_COUNT, 7);
        assert_eq!(component::loading_indicator::DETERMINATE_SHAPE_COUNT, 2);
        assert_eq!(
            component::loading_indicator::MORPH_SPRING_DAMPING_RATIO,
            0.6
        );
        assert_eq!(component::loading_indicator::MORPH_SPRING_STIFFNESS, 200.0);
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
        assert_eq!(component::list::TWO_LINE_CONTAINER_HEIGHT, 72.0);
        assert_eq!(component::list::THREE_LINE_CONTAINER_HEIGHT, 88.0);
        assert_eq!(component::list::LEADING_SPACE, 16.0);
        assert_eq!(component::list::TRAILING_SPACE, 16.0);
        assert_eq!(component::list::TOP_SPACE, 12.0);
        assert_eq!(component::list::BOTTOM_SPACE, 12.0);
        assert_eq!(component::list::LEADING_ICON_SIZE, 24.0);
        assert_eq!(component::list::LEADING_AVATAR_SIZE, 40.0);
        assert_eq!(component::list::LABEL_TEXT, typography::BODY_LARGE);
        assert_eq!(component::list::SUPPORTING_TEXT, typography::BODY_MEDIUM);
        assert_eq!(
            component::list::TRAILING_SUPPORTING_TEXT,
            typography::LABEL_SMALL
        );
        assert_eq!(component::list::DISABLED_LABEL_TEXT_OPACITY, 0.30);
        assert_eq!(component::list::DISABLED_ICON_OPACITY, 0.38);
        assert_eq!(component::menu::CONTAINER_ELEVATION_LEVEL, 2);
        assert_eq!(component::menu::TOP_SPACE, 8.0);
        assert_eq!(component::select::MENU_CONTAINER_ELEVATION_LEVEL, 2);
        assert_eq!(component::select::MENU_LIST_ITEM_CONTAINER_HEIGHT, 48.0);
        assert_eq!(component::select::TRAILING_ICON_SIZE, 24.0);
        assert_eq!(component::select::TEXT_FIELD_DISABLED_OUTLINE_WIDTH, 1.0);
        assert_eq!(component::navigation_bar::CONTAINER_HEIGHT, 80.0);
        assert_eq!(component::navigation_bar::CONTAINER_ELEVATION_LEVEL, 2);
        assert_eq!(component::navigation_bar::ACTIVE_INDICATOR_WIDTH, 64.0);
        assert_eq!(component::navigation_bar::ACTIVE_INDICATOR_HEIGHT, 32.0);
        assert_eq!(
            component::navigation_bar::ACTIVE_INDICATOR_SHAPE,
            shape::CORNER_FULL
        );
        assert_eq!(component::navigation_bar::ICON_SIZE, 24.0);
        assert_eq!(
            component::navigation_bar::LABEL_TEXT,
            typography::LABEL_MEDIUM
        );
        assert_eq!(component::navigation_bar::ITEM_HORIZONTAL_PADDING, 8.0);
        assert_eq!(component::navigation_bar::INDICATOR_TO_LABEL_PADDING, 4.0);
        assert_eq!(component::navigation_bar::ITEM_ANIMATION_DURATION_MS, 100);
        assert_eq!(component::navigation_rail::CONTAINER_WIDTH, 96.0);
        assert_eq!(component::navigation_rail::EXPANDED_CONTAINER_WIDTH, 220.0);
        assert_eq!(component::navigation_rail::CONTAINER_ELEVATION_LEVEL, 3);
        assert_eq!(component::navigation_rail::ACTIVE_INDICATOR_WIDTH, 56.0);
        assert_eq!(component::navigation_rail::ACTIVE_INDICATOR_HEIGHT, 32.0);
        assert_eq!(
            component::navigation_rail::EXPANDED_ACTIVE_INDICATOR_HEIGHT,
            56.0
        );
        assert_eq!(
            component::navigation_rail::EXPANDED_ACTIVE_INDICATOR_MARGIN_HORIZONTAL,
            20.0
        );
        assert_eq!(
            component::navigation_rail::EXPANDED_ACTIVE_INDICATOR_PADDING_START,
            16.0
        );
        assert_eq!(
            component::navigation_rail::EXPANDED_ACTIVE_INDICATOR_PADDING_END,
            16.0
        );
        assert_eq!(component::navigation_rail::ICON_SIZE, 24.0);
        assert_eq!(component::navigation_rail::ICON_LABEL_HORIZONTAL_SPACE, 8.0);
        assert_eq!(
            component::navigation_rail::LABEL_TEXT,
            typography::LABEL_MEDIUM
        );
        assert_eq!(component::navigation_rail::ITEM_WIDTH, 96.0);
        assert_eq!(component::navigation_rail::ITEM_HEIGHT, 64.0);
        assert_eq!(component::navigation_rail::VERTICAL_PADDING, 4.0);
        assert_eq!(component::navigation_rail::CONTENT_TOP_MARGIN, 44.0);
        assert_eq!(component::navigation_rail::ITEM_TOP_PADDING, 6.0);
        assert_eq!(component::navigation_rail::HEADER_PADDING, 40.0);
        assert_eq!(component::navigation_rail::ITEM_ANIMATION_DURATION_MS, 150);
        assert_eq!(component::navigation_drawer::CONTAINER_WIDTH, 360.0);
        assert_eq!(component::navigation_drawer::MINIMUM_CONTAINER_WIDTH, 240.0);
        assert_eq!(component::navigation_drawer::ACTIVE_INDICATOR_WIDTH, 336.0);
        assert_eq!(component::navigation_drawer::ACTIVE_INDICATOR_HEIGHT, 56.0);
        assert_eq!(
            component::navigation_drawer::ACTIVE_INDICATOR_SHAPE,
            shape::CORNER_FULL
        );
        assert_eq!(
            component::navigation_drawer::LABEL_TEXT,
            typography::LABEL_LARGE
        );
        assert_eq!(
            component::navigation_drawer::HEADLINE_TEXT,
            typography::TITLE_SMALL
        );
        assert_eq!(component::navigation_drawer::ITEM_HORIZONTAL_PADDING, 12.0);
        assert_eq!(component::navigation_drawer::LABEL_BADGE_SPACE, 12.0);
        assert_eq!(
            component::navigation_drawer::MODAL_CONTAINER_ELEVATION_LEVEL,
            1
        );
        assert_eq!(
            component::navigation_drawer::STANDARD_CONTAINER_ELEVATION_LEVEL,
            0
        );
        assert_eq!(component::adaptive_navigation::WIDTH_COMPACT_MAX, 600.0);
        assert_eq!(component::adaptive_navigation::WIDTH_MEDIUM_MAX, 840.0);
        assert_eq!(component::adaptive_navigation::HEIGHT_COMPACT_MAX, 480.0);
        assert_eq!(component::adaptive_navigation::HEIGHT_MEDIUM_MAX, 900.0);
        assert_eq!(component::dialog::CONTAINER_ELEVATION_LEVEL, 3);
        assert_eq!(component::dialog::CONTAINER_MIN_WIDTH, 280.0);
        assert_eq!(component::dialog::CONTAINER_MAX_WIDTH, 560.0);
        assert_eq!(component::dialog::CONTAINER_PADDING, 24.0);
        assert_eq!(component::dialog::ICON_BOTTOM_PADDING, 16.0);
        assert_eq!(component::dialog::TITLE_BOTTOM_PADDING, 16.0);
        assert_eq!(component::dialog::SUPPORTING_TEXT_BOTTOM_PADDING, 24.0);
        assert_eq!(component::dialog::ACTIONS_HORIZONTAL_SPACING, 8.0);
        assert_eq!(component::dialog::ACTIONS_VERTICAL_SPACING, 8.0);
        assert_eq!(component::dialog::SCRIM_OPACITY, 0.32);
        assert_eq!(component::dialog::ENTER_SCALE_FROM, 0.9);
        assert_eq!(component::dialog::EXIT_SCALE_TO, 0.9);
        assert_eq!(component::dialog::SCALE_ANIMATION_DURATION_MS, 220);
        assert_eq!(component::dialog::ALPHA_ANIMATION_DURATION_MS, 150);
        assert_eq!(component::dialog::SCRIM_ANIMATION_DURATION_MS, 220);
        assert_eq!(component::dialog::DECELERATE_CUBIC_FACTOR, 1.5);
        assert_eq!(component::dialog::DECELERATE_QUINT_FACTOR, 2.5);
        assert_eq!(
            component::dialog::ACTION_LABEL_TEXT,
            typography::LABEL_LARGE
        );
        assert_eq!(component::dialog::HEADLINE_TEXT, typography::HEADLINE_SMALL);
        assert_eq!(component::dialog::SUPPORTING_TEXT, typography::BODY_MEDIUM);
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
        assert_eq!(component::fab::SMALL_CONTAINER_WIDTH, 40.0);
        assert_eq!(component::fab::SMALL_CONTAINER_HEIGHT, 40.0);
        assert_eq!(component::fab::SMALL_CONTAINER_SHAPE, shape::CORNER_MEDIUM);
        assert_eq!(component::fab::SMALL_ICON_SIZE, 24.0);
        assert_eq!(component::fab::LARGE_CONTAINER_WIDTH, 96.0);
        assert_eq!(component::fab::LARGE_CONTAINER_HEIGHT, 96.0);
        assert_eq!(
            component::fab::LARGE_CONTAINER_SHAPE,
            shape::CORNER_EXTRA_LARGE
        );
        assert_eq!(component::fab::LARGE_ICON_SIZE, 36.0);
        assert_eq!(component::fab::EXTENDED_CONTAINER_HEIGHT, 56.0);
        assert_eq!(
            component::fab::EXTENDED_CONTAINER_SHAPE,
            shape::CORNER_LARGE
        );
        assert_eq!(component::fab::EXTENDED_ICON_SIZE, 24.0);
        assert_eq!(component::fab::EXTENDED_ICON_LABEL_SPACE, 12.0);
        assert_eq!(component::fab::EXTENDED_LEADING_SPACE, 16.0);
        assert_eq!(component::fab::EXTENDED_TRAILING_SPACE, 20.0);
        assert_eq!(component::fab::EXTENDED_LABEL_TEXT, typography::LABEL_LARGE);
        assert_eq!(component::fab::ELEVATION.active, 3);
        assert_eq!(component::fab::ELEVATION.hovered, 4);
        assert_eq!(component::fab::EXTENDED_ELEVATION.active, 3);
        assert_eq!(component::fab::EXTENDED_ELEVATION.hovered, 4);
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
        assert_eq!(component::segmented_button::CONTAINER_HEIGHT, 40.0);
        assert_eq!(
            component::segmented_button::CONTAINER_SHAPE,
            shape::CORNER_FULL
        );
        assert_eq!(component::segmented_button::OUTLINE_WIDTH, 1.0);
        assert_eq!(component::segmented_button::WITH_ICON_ICON_SIZE, 18.0);
        assert_eq!(component::segmented_button::LEADING_SPACE, 12.0);
        assert_eq!(component::segmented_button::TRAILING_SPACE, 12.0);
        assert_eq!(component::segmented_button::ICON_LABEL_SPACE, 8.0);
        assert_eq!(
            component::segmented_button::LABEL_TEXT,
            typography::LABEL_LARGE
        );
        assert_eq!(
            component::segmented_button::DISABLED_LABEL_TEXT_OPACITY,
            0.38
        );
        assert_eq!(component::segmented_button::DISABLED_OUTLINE_OPACITY, 0.12);
        assert_eq!(
            component::segmented_button::SELECT_TRANSITION_DURATION_MS,
            200
        );
        assert_eq!(
            component::segmented_button::SELECT_TRANSITION_EASING,
            motion::EASING_EMPHASIZED
        );
        assert_eq!(component::snackbar::ICON_SIZE, 24.0);
        assert_eq!(component::snackbar::WITH_SINGLE_LINE_CONTAINER_HEIGHT, 48.0);
        assert_eq!(component::snackbar::WITH_TWO_LINES_CONTAINER_HEIGHT, 68.0);
        assert_eq!(component::snackbar::MAX_WIDTH, 568.0);
        assert_eq!(component::snackbar::HORIZONTAL_MARGIN, 16.0);
        assert_eq!(component::snackbar::BOTTOM_MARGIN, 16.0);
        assert_eq!(component::snackbar::CONTAINER_ELEVATION_LEVEL, 3);
        assert_eq!(
            component::snackbar::CONTAINER_SHAPE,
            shape::CORNER_EXTRA_SMALL
        );
        assert_eq!(
            component::snackbar::SUPPORTING_TEXT,
            typography::BODY_MEDIUM
        );
        assert_eq!(
            component::snackbar::ACTION_LABEL_TEXT,
            typography::LABEL_LARGE
        );
        assert_eq!(component::snackbar::SLIDE_ANIMATION_DURATION_MS, 250);
        assert_eq!(component::snackbar::CONTENT_FADE_ANIMATION_DURATION_MS, 180);
        assert_eq!(component::snackbar::LONG_DURATION_MS, 2750);
        assert_eq!(
            component::snackbar::SLIDE_ANIMATION_EASING,
            motion::EASING_LEGACY
        );
        assert_eq!(
            component::snackbar::CONTENT_FADE_ANIMATION_EASING,
            motion::EASING_LEGACY
        );
        assert_eq!(component::search_bar::AVATAR_SIZE, 30.0);
        assert_eq!(component::search_bar::CONTAINER_HEIGHT, 56.0);
        assert_eq!(component::search_bar::ICON_SIZE, 24.0);
        assert_eq!(component::search_bar::LEADING_SPACE, 16.0);
        assert_eq!(component::search_bar::TRAILING_SPACE, 16.0);
        assert_eq!(component::search_bar::LEADING_ICON_LABEL_SPACE, 16.0);
        assert_eq!(component::search_bar::CONTAINER_ELEVATION_LEVEL, 3);
        assert_eq!(component::search_bar::CONTAINER_SHAPE, shape::CORNER_FULL);
        assert_eq!(component::search_bar::INPUT_TEXT, typography::BODY_LARGE);
        assert_eq!(component::search_view::DOCKED_HEADER_CONTAINER_HEIGHT, 56.0);
        assert_eq!(
            component::search_view::FULL_SCREEN_HEADER_CONTAINER_HEIGHT,
            72.0
        );
        assert_eq!(
            component::search_view::DOCKED_CONTAINER_SHAPE,
            shape::CORNER_EXTRA_LARGE
        );
        assert_eq!(
            component::search_view::FULL_SCREEN_CONTAINER_SHAPE,
            shape::CORNER_NONE
        );
        assert_eq!(component::app_bar::AVATAR_SIZE, 32.0);
        assert_eq!(component::app_bar::ICON_BUTTON_SPACE, 0.0);
        assert_eq!(component::app_bar::ICON_SIZE, 24.0);
        assert_eq!(component::app_bar::LEADING_SPACE, 4.0);
        assert_eq!(component::app_bar::TRAILING_SPACE, 4.0);
        assert_eq!(component::app_bar::CONTAINER_ELEVATION_LEVEL, 0);
        assert_eq!(component::app_bar::ON_SCROLL_CONTAINER_ELEVATION_LEVEL, 2);
        assert_eq!(component::app_bar::SMALL_CONTAINER_HEIGHT, 64.0);
        assert_eq!(component::app_bar::SMALL_SEARCH_CONTAINER_HEIGHT, 56.0);
        assert_eq!(
            component::app_bar::SMALL_TITLE_TEXT,
            typography::TITLE_LARGE
        );
        assert_eq!(component::app_bar::MEDIUM_CONTAINER_HEIGHT, 112.0);
        assert_eq!(
            component::app_bar::MEDIUM_TITLE_TEXT,
            typography::HEADLINE_SMALL
        );
        assert_eq!(component::app_bar::LARGE_CONTAINER_HEIGHT, 152.0);
        assert_eq!(
            component::app_bar::LARGE_TITLE_TEXT,
            typography::HEADLINE_MEDIUM
        );
        assert_eq!(component::bottom_app_bar::CONTAINER_HEIGHT, 80.0);
        assert_eq!(component::bottom_app_bar::CONTAINER_ELEVATION_LEVEL, 2);
        assert_eq!(
            component::bottom_app_bar::CONTAINER_SHAPE,
            shape::CORNER_NONE
        );
        assert_eq!(component::toolbar::DOCKED_CONTAINER_HEIGHT, 64.0);
        assert_eq!(component::toolbar::DOCKED_LEADING_SPACE, 16.0);
        assert_eq!(component::toolbar::DOCKED_TRAILING_SPACE, 16.0);
        assert_eq!(
            component::toolbar::DOCKED_CONTAINER_SHAPE,
            shape::CORNER_NONE
        );
        assert_eq!(
            component::toolbar::FLOATING_HORIZONTAL_CONTAINER_HEIGHT,
            64.0
        );
        assert_eq!(component::toolbar::FLOATING_VERTICAL_CONTAINER_WIDTH, 64.0);
        assert_eq!(component::toolbar::FLOATING_CONTAINER_LEADING_SPACE, 8.0);
        assert_eq!(component::toolbar::FLOATING_CONTAINER_TRAILING_SPACE, 8.0);
        assert_eq!(
            component::toolbar::FLOATING_CONTAINER_SHAPE,
            shape::CORNER_FULL
        );
        assert_eq!(component::toolbar::FLOATING_CONTAINER_ELEVATION_LEVEL, 3);
        assert_eq!(component::toolbar::ACTION_CONTAINER_WIDTH, 48.0);
        assert_eq!(component::toolbar::ACTION_CONTAINER_HEIGHT, 48.0);
        assert_eq!(component::toolbar::ACTION_ICON_SIZE, 24.0);
        assert_eq!(
            component::bottom_sheet::CONTAINER_SHAPE_TOP,
            shape::CORNER_EXTRA_LARGE
        );
        assert_eq!(component::bottom_sheet::CONTAINER_SHAPE_BOTTOM, 0.0);
        assert_eq!(component::bottom_sheet::MODAL_CONTAINER_ELEVATION_LEVEL, 1);
        assert_eq!(
            component::bottom_sheet::STANDARD_CONTAINER_ELEVATION_LEVEL,
            1
        );
        assert_eq!(component::bottom_sheet::DRAG_HANDLE_WIDTH, 32.0);
        assert_eq!(component::bottom_sheet::DRAG_HANDLE_HEIGHT, 4.0);
        assert_eq!(component::bottom_sheet::DRAG_HANDLE_VERTICAL_PADDING, 22.0);
        assert_eq!(component::bottom_sheet::CONTENT_PADDING, 24.0);
        assert_eq!(component::bottom_sheet::SHEET_PEEK_HEIGHT, 56.0);
        assert_eq!(component::bottom_sheet::SHEET_MAX_WIDTH, 640.0);
        assert_eq!(component::bottom_sheet::SCRIM_OPACITY, 0.32);
        assert_eq!(component::bottom_sheet::POSITIONAL_THRESHOLD, 56.0);
        assert_eq!(component::bottom_sheet::VELOCITY_THRESHOLD, 125.0);
        assert_eq!(component::bottom_sheet::ANIMATION_DURATION_MS, 300);
        assert_eq!(
            component::bottom_sheet::ANIMATION_EASING,
            motion::EASING_LEGACY
        );
        assert_eq!(component::side_sheet::DOCKED_CONTAINER_WIDTH, 256.0);
        assert_eq!(component::side_sheet::DETACHED_MARGIN, 16.0);
        assert_eq!(component::side_sheet::CONTENT_PADDING, 24.0);
        assert_eq!(
            component::side_sheet::DOCKED_STANDARD_CONTAINER_SHAPE,
            shape::CORNER_NONE
        );
        assert_eq!(
            component::side_sheet::DOCKED_MODAL_CONTAINER_SHAPE,
            shape::CORNER_LARGE
        );
        assert_eq!(
            component::side_sheet::DETACHED_CONTAINER_SHAPE,
            shape::CORNER_LARGE
        );
        assert_eq!(component::side_sheet::MODAL_CONTAINER_ELEVATION_LEVEL, 1);
        assert_eq!(component::side_sheet::STANDARD_CONTAINER_ELEVATION_LEVEL, 0);
        assert_eq!(component::side_sheet::SCRIM_OPACITY, 0.32);
        assert_eq!(component::side_sheet::ANIMATION_DURATION_MS, 275);
        assert_eq!(
            component::side_sheet::ANIMATION_EASING,
            motion::EASING_EMPHASIZED
        );
        assert_eq!(component::primary_tab::CONTAINER_HEIGHT, 48.0);
        assert_eq!(
            component::primary_tab::WITH_ICON_AND_LABEL_TEXT_CONTAINER_HEIGHT,
            64.0
        );
        assert_eq!(component::primary_tab::CONTAINER_ELEVATION_LEVEL, 0);
        assert_eq!(component::primary_tab::CONTAINER_SHAPE, shape::CORNER_NONE);
        assert_eq!(component::primary_tab::ACTIVE_INDICATOR_HEIGHT, 3.0);
        assert_eq!(component::primary_tab::ACTIVE_INDICATOR_SHAPE_TOP, 3.0);
        assert_eq!(component::primary_tab::ACTIVE_INDICATOR_SHAPE_BOTTOM, 0.0);
        assert_eq!(component::primary_tab::ICON_SIZE, 24.0);
        assert_eq!(component::primary_tab::HORIZONTAL_SPACE, 16.0);
        assert_eq!(component::primary_tab::INLINE_ICON_LABEL_SPACE, 8.0);
        assert_eq!(component::primary_tab::STACKED_ICON_LABEL_SPACE, 2.0);
        assert_eq!(component::primary_tab::LABEL_TEXT, typography::TITLE_SMALL);
        assert_eq!(component::primary_tab::INDICATOR_ANIMATION_DURATION_MS, 250);
        assert_eq!(
            component::primary_tab::INDICATOR_ANIMATION_EASING,
            motion::EASING_EMPHASIZED
        );
        assert_eq!(component::secondary_tab::CONTAINER_HEIGHT, 48.0);
        assert_eq!(component::secondary_tab::CONTAINER_ELEVATION_LEVEL, 0);
        assert_eq!(
            component::secondary_tab::CONTAINER_SHAPE,
            shape::CORNER_NONE
        );
        assert_eq!(component::secondary_tab::ACTIVE_INDICATOR_HEIGHT, 2.0);
        assert_eq!(
            component::secondary_tab::ACTIVE_INDICATOR_SHAPE,
            shape::CORNER_NONE
        );
        assert_eq!(component::secondary_tab::ICON_SIZE, 24.0);
        assert_eq!(component::secondary_tab::HORIZONTAL_SPACE, 16.0);
        assert_eq!(component::secondary_tab::ICON_LABEL_SPACE, 8.0);
        assert_eq!(
            component::secondary_tab::LABEL_TEXT,
            typography::TITLE_SMALL
        );
        assert_eq!(
            component::secondary_tab::INDICATOR_ANIMATION_DURATION_MS,
            250
        );
        assert_eq!(
            component::secondary_tab::INDICATOR_ANIMATION_EASING,
            motion::EASING_EMPHASIZED
        );
        assert_eq!(component::tooltip::SPACING_BETWEEN_TOOLTIP_AND_ANCHOR, 4.0);
        assert_eq!(component::tooltip::PLAIN_MIN_HEIGHT, 24.0);
        assert_eq!(component::tooltip::PLAIN_MIN_WIDTH, 40.0);
        assert_eq!(component::tooltip::PLAIN_MAX_WIDTH, 200.0);
        assert_eq!(component::tooltip::PLAIN_HORIZONTAL_SPACE, 8.0);
        assert_eq!(component::tooltip::PLAIN_VERTICAL_SPACE, 4.0);
        assert_eq!(component::tooltip::PLAIN_CONTAINER_SHAPE, 4.0);
        assert_eq!(component::tooltip::PLAIN_SUPPORTING_TEXT.size, 12.0);
        assert_eq!(component::tooltip::FADE_IN_DURATION_MS, 150);
        assert_eq!(component::tooltip::FADE_OUT_DURATION_MS, 75);
        assert_eq!(component::tooltip::SCALE_START, 0.8);
        assert_eq!(component::tooltip::ANIMATION_DURATION_MS, 150);
        assert_eq!(component::tooltip::RICH_MAX_WIDTH, 320.0);
        assert_eq!(component::tooltip::RICH_MIN_HEIGHT, 24.0);
        assert_eq!(component::tooltip::RICH_MIN_WIDTH, 40.0);
        assert_eq!(component::tooltip::RICH_HORIZONTAL_SPACE, 16.0);
        assert_eq!(
            component::tooltip::RICH_TEXT_VERTICAL_SPACE_WITHOUT_TITLE_OR_ACTION,
            4.0
        );
        assert_eq!(component::tooltip::RICH_HEIGHT_TO_SUBHEAD_FIRST_LINE, 28.0);
        assert_eq!(
            component::tooltip::RICH_HEIGHT_FROM_SUBHEAD_TO_TEXT_FIRST_LINE,
            24.0
        );
        assert_eq!(component::tooltip::RICH_TEXT_BOTTOM_PADDING, 16.0);
        assert_eq!(component::tooltip::RICH_ACTION_LABEL_MIN_HEIGHT, 36.0);
        assert_eq!(component::tooltip::RICH_ACTION_LABEL_BOTTOM_PADDING, 8.0);
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
