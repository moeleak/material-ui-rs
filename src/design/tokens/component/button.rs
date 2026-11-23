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
pub const ICON_LABEL_SPACE: f32 = 8.0;
pub const TEXT_WITH_LEADING_ICON_LEADING_SPACE: f32 = 12.0;
pub const TEXT_WITH_LEADING_ICON_TRAILING_SPACE: f32 = 16.0;
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
