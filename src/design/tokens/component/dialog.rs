pub const CONTAINER_SHAPE: f32 = super::super::shape::CORNER_EXTRA_LARGE;
pub const CONTAINER_ELEVATION_LEVEL: u8 = 3;
pub const CONTAINER_MIN_WIDTH: f32 = 280.0;
pub const CONTAINER_MAX_WIDTH: f32 = 560.0;
pub const CONTAINER_PADDING: f32 = 24.0;
/// Recommended clearance from the safe window edges for a floating dialog.
pub const WINDOW_MARGIN: f32 = 24.0;
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
