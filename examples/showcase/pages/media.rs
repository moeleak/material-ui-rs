use std::sync::Arc;

use iced::Length;
use iced::widget::{Column, Row, sensor};
use material::widget::{grid, image, menu, page};
use material_ui_rs as material;

use super::super::{CoverAction, Message, Showcase};

/// The covers' titles and the colors of their gradients.
pub(in super::super) const COVERS: [(&str, [u8; 3], [u8; 3]); 8] = [
    ("Lighthouse", [0x4f, 0x9d, 0xde], [0xf6, 0xe2, 0x9c]),
    ("Nightfall", [0x1c, 0x1f, 0x4a], [0x8a, 0x5c, 0xb8]),
    ("Sakura Lane", [0xf4, 0xa7, 0xc0], [0xff, 0xf1, 0xf5]),
    ("Clockwork", [0x5b, 0x3a, 0x1e], [0xd9, 0xa4, 0x41]),
    ("Seaside", [0x0b, 0x6e, 0x79], [0xa8, 0xe6, 0xcf]),
    ("Snowfield", [0x9a, 0xb3, 0xc9], [0xf5, 0xf8, 0xfc]),
    ("Ember", [0x7a, 0x12, 0x12], [0xf2, 0x8c, 0x28]),
    ("Missing art", [0, 0, 0], [0, 0, 0]),
];

const COVER_SIZE: (u32, u32) = (300, 400);
const MAX_COVER_WIDTH: f32 = 168.0;

pub(super) fn view(state: &Showcase) -> material::Element<'_, Message> {
    page::sections([
        page::section("Cover grid", covers(state)).into(),
        page::section("Placeholders", placeholders()).into(),
    ])
    .into()
}

fn covers(state: &Showcase) -> material::Element<'_, Message> {
    let cells = COVERS.iter().enumerate().map(|(index, (title, ..))| {
        let cover = sensor(grid::aspect_ratio(
            image::cached(
                &state.covers,
                &index,
                material::tokens::shape::CORNER_MEDIUM,
            ),
            COVER_SIZE.0 as f32 / COVER_SIZE.1 as f32,
        ))
        .on_show(move |_| Message::CoverShown(index));

        let cell = Column::new()
            .push(cover)
            .push(material::text::type_scale(
                *title,
                material::tokens::typography::TITLE_SMALL,
            ))
            .spacing(8);

        let action = |action| Message::CoverAction(index, action);

        menu::context(cell)
            .item(
                menu::item("Play")
                    .icon("play_arrow")
                    .on_press(action(CoverAction::Play)),
            )
            .item(
                menu::item("Show in folder")
                    .icon("folder_open")
                    .on_press(action(CoverAction::Reveal)),
            )
            .item(menu::item("Rename").icon("edit"))
            .divider()
            .item(
                menu::item("Remove from library")
                    .icon("delete")
                    .destructive()
                    .on_press(action(CoverAction::Remove)),
            )
            .into()
    });

    let status = state
        .cover_action
        .as_deref()
        .unwrap_or("Right-click a cover for its actions.");

    Column::new()
        .push(material::text::body_medium(status))
        .push(grid::grid(MAX_COVER_WIDTH, cells))
        .spacing(16)
        .width(Length::Fill)
        .into()
}

fn placeholders() -> material::Element<'static, Message> {
    let corner = material::tokens::shape::CORNER_MEDIUM;
    let cell = |kind| {
        iced::widget::container(grid::aspect_ratio(
            image::placeholder(kind, corner),
            3.0 / 4.0,
        ))
        .width(Length::Fixed(96.0))
    };

    Row::new()
        .push(cell(image::Placeholder::Loading))
        .push(cell(image::Placeholder::Failed))
        .push(cell(image::Placeholder::Missing))
        .spacing(grid::SPACING)
        .into()
}

/// Where the cover at `index` is decoded from: a gradient encoded as a BMP,
/// so that the cache decodes it in the background as it would a file; the
/// last cover is not an image at all.
pub(in super::super) fn cover_source(index: usize) -> image::Source {
    let Some(&(_, top, bottom)) = COVERS.get(index).filter(|_| index + 1 < COVERS.len()) else {
        return image::Source::Bytes(Arc::from(&b"not an image"[..]));
    };

    image::Source::Bytes(gradient_bmp(COVER_SIZE.0, COVER_SIZE.1, top, bottom).into())
}

fn gradient_bmp(width: u32, height: u32, top: [u8; 3], bottom: [u8; 3]) -> Vec<u8> {
    let row = (width * 3).next_multiple_of(4);
    let pixels = row * height;
    let mut bmp = Vec::with_capacity(54 + pixels as usize);

    bmp.extend_from_slice(b"BM");
    bmp.extend_from_slice(&(54 + pixels).to_le_bytes());
    bmp.extend_from_slice(&0u32.to_le_bytes());
    bmp.extend_from_slice(&54u32.to_le_bytes());
    bmp.extend_from_slice(&40u32.to_le_bytes());
    bmp.extend_from_slice(&(width as i32).to_le_bytes());
    bmp.extend_from_slice(&(height as i32).to_le_bytes());
    bmp.extend_from_slice(&1u16.to_le_bytes());
    bmp.extend_from_slice(&24u16.to_le_bytes());
    bmp.extend_from_slice(&[0; 24]);

    // Rows go from the bottom up, in blue, green, red order.
    for y in (0..height).rev() {
        let t = y as f32 / (height - 1).max(1) as f32;
        let channel = |i: usize| (top[i] as f32 + (bottom[i] as f32 - top[i] as f32) * t) as u8;
        let pixel = [channel(2), channel(1), channel(0)];

        for _ in 0..width {
            bmp.extend_from_slice(&pixel);
        }

        bmp.resize(bmp.len() + (row - width * 3) as usize, 0);
    }

    bmp
}
