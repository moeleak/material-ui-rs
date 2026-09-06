use super::*;

type TestParagraph = <iced_widget::Renderer as core_text::Renderer>::Paragraph;

fn assert_close(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() < 0.001,
        "expected {expected}, got {actual}",
    );
}

fn assert_rectangle_close(actual: Rectangle, expected: Rectangle) {
    assert_close(actual.x, expected.x);
    assert_close(actual.y, expected.y);
    assert_close(actual.width, expected.width);
    assert_close(actual.height, expected.height);
}

#[test]
fn animated_scalar_retargets_from_last_advanced_value() {
    let start = Instant::now();
    let mut scalar = AnimatedScalar::new(0.0);

    scalar.set_target(1.0, start, duration_ms(100), tokens::motion::EASING_LINEAR);
    assert!(scalar.advance(start + duration_ms(50)));
    scalar.set_target(
        0.0,
        start + duration_ms(80),
        duration_ms(100),
        tokens::motion::EASING_LINEAR,
    );

    assert_close(scalar.value, 0.8);
    assert_eq!(scalar.to, 0.0);

    assert!(scalar.advance(start + duration_ms(130)));
    assert_close(scalar.value, 0.4);
}

#[test]
fn text_field_floating_label_notch_uses_label_width_with_padding() {
    let field = Rectangle {
        x: 10.0,
        y: 20.0,
        width: 160.0,
        height: 56.0,
    };

    let notch = text_field_floating_label_notch(field, 26.0, 80.0, 80.0, 1.0).unwrap();

    assert_rectangle_close(
        notch,
        Rectangle {
            x: 26.0,
            y: 20.0,
            width: 84.0,
            height: 0.0,
        },
    );
}

#[test]
fn text_field_floating_label_notch_expands_from_label_start() {
    let field = Rectangle {
        x: 10.0,
        y: 20.0,
        width: 160.0,
        height: 56.0,
    };

    let notch = text_field_floating_label_notch(field, 26.0, 80.0, 80.0, 0.5).unwrap();

    assert_rectangle_close(
        notch,
        Rectangle {
            x: 26.0,
            y: 20.0,
            width: 42.0,
            height: 0.0,
        },
    );
}

#[test]
fn text_field_floating_label_notch_follows_interpolated_text_width() {
    let field = Rectangle {
        x: 10.0,
        y: 20.0,
        width: 200.0,
        height: 56.0,
    };

    let notch = text_field_floating_label_notch(field, 26.0, 120.0, 80.0, 0.5).unwrap();

    assert_rectangle_close(
        notch,
        Rectangle {
            x: 26.0,
            y: 20.0,
            width: 52.0,
            height: 0.0,
        },
    );
}

#[test]
fn text_field_floating_label_notch_stays_hidden_before_float() {
    let field = Rectangle {
        x: 10.0,
        y: 20.0,
        width: 160.0,
        height: 56.0,
    };

    assert!(text_field_floating_label_notch(field, 26.0, 80.0, 80.0, 0.0).is_none());
}

#[derive(Default)]
struct OutlineTestRenderer {
    layers: Vec<Rectangle>,
    painted: Vec<Rectangle>,
    text_clips: Vec<(String, Rectangle, Rectangle)>,
}

impl core_text::Renderer for OutlineTestRenderer {
    type Font = iced_widget::core::Font;
    type Paragraph = <iced_widget::Renderer as core_text::Renderer>::Paragraph;
    type Editor = <iced_widget::Renderer as core_text::Renderer>::Editor;

    const ICON_FONT: Self::Font = iced_widget::core::Font::DEFAULT;
    const CHECKMARK_ICON: char = 'x';
    const ARROW_DOWN_ICON: char = 'v';
    const SCROLL_UP_ICON: char = '^';
    const SCROLL_DOWN_ICON: char = 'v';
    const SCROLL_LEFT_ICON: char = '<';
    const SCROLL_RIGHT_ICON: char = '>';
    const ICED_LOGO: char = 'i';

    fn default_font(&self) -> Self::Font {
        Self::Font::DEFAULT
    }
    fn default_size(&self) -> iced_widget::core::Pixels {
        iced_widget::core::Pixels(16.0)
    }

    fn fill_paragraph(
        &mut self,
        _paragraph: &Self::Paragraph,
        _position: Point,
        _color: Color,
        clip: Rectangle,
    ) {
        self.text_clips
            .push(("paragraph".into(), clip, *self.layers.last().unwrap()));
    }

    fn fill_editor(
        &mut self,
        _editor: &Self::Editor,
        _position: Point,
        _color: Color,
        clip: Rectangle,
    ) {
        self.text_clips
            .push(("editor".into(), clip, *self.layers.last().unwrap()));
    }

    fn fill_text(
        &mut self,
        text: core_text::Text<String, Self::Font>,
        _position: Point,
        _color: Color,
        clip: Rectangle,
    ) {
        self.text_clips
            .push((text.content, clip, *self.layers.last().unwrap()));
    }
}

impl iced_widget::graphics::geometry::Renderer for OutlineTestRenderer {
    type Geometry = iced_widget::renderer::wgpu::geometry::Geometry;
    type Frame = iced_widget::renderer::wgpu::geometry::Frame;

    fn new_frame(&self, bounds: Rectangle) -> Self::Frame {
        Self::Frame::new(bounds)
    }
    fn draw_geometry(&mut self, _geometry: Self::Geometry) {}
}

fn draw_field(
    mut field: iced_widget::core::Element<'_, (), crate::Theme, OutlineTestRenderer>,
    viewport: Rectangle,
) -> OutlineTestRenderer {
    use iced_widget::core::{Layout, Size, layout, mouse, widget::Tree};
    use renderer::Renderer as _;

    let mut renderer = OutlineTestRenderer::default();
    let mut tree = Tree::new(field.as_widget());
    let node = field
        .as_widget_mut()
        .layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, Size::new(304.0, 800.0)),
        )
        .move_to(Point::new(28.0, 504.0));
    renderer.start_layer(viewport);
    field.as_widget().draw(
        &tree,
        &mut renderer,
        &crate::Theme::Dark,
        &renderer::Style {
            text_color: Color::WHITE,
        },
        Layout::new(&node),
        mouse::Cursor::Unavailable,
        &viewport,
    );
    renderer.end_layer();
    renderer
}

#[test]
fn actual_field_labels_and_values_keep_text_clips_inside_the_scroll_viewport() {
    let options = crate::widget::combobox::State::new(vec!["Assist", "Filter"]);
    let selected = "Assist";
    for viewport in [
        Rectangle {
            x: 0.0,
            y: 72.0,
            width: 360.0,
            height: 434.0,
        },
        Rectangle {
            x: 0.0,
            y: 72.0,
            width: 360.0,
            height: 464.0,
        },
        Rectangle {
            x: 0.0,
            y: 516.0,
            width: 360.0,
            height: 100.0,
        },
        Rectangle {
            x: 0.0,
            y: 72.0,
            width: 360.0,
            height: 400.0,
        },
    ] {
        let fields: [iced_widget::core::Element<'_, (), crate::Theme, OutlineTestRenderer>; 3] = [
            crate::widget::select::outlined(["Assist", "Filter"], Some("Assist"), |_| ())
                .label("Chip type")
                .into(),
            crate::widget::text_input::outlined("Chip type", "Assist")
                .on_input(|_| ())
                .into(),
            crate::widget::combobox::outlined(&options, "Choose", Some(&selected), |_| ())
                .label("Chip type")
                .into(),
        ];
        for (index, field) in fields.into_iter().enumerate() {
            let renderer = draw_field(field, viewport);
            if viewport.y == 72.0 && viewport.height == 400.0 {
                assert!(
                    renderer.text_clips.is_empty(),
                    "fully clipped field {index} drew text"
                );
            }
            for (content, clip, layer) in &renderer.text_clips {
                if let Some(visible) = clip.intersection(layer) {
                    assert_eq!(
                        visible.intersection(&viewport),
                        Some(visible),
                        "field {index} text {content:?} escaped viewport: clip={clip:?}, layer={layer:?}"
                    );
                }
            }
            if viewport.y == 72.0 && viewport.height == 434.0 {
                let (_, clip, layer) = renderer
                    .text_clips
                    .iter()
                    .find(|(text, clip, _)| {
                        text == "Chip type" || (index == 1 && text == "paragraph" && clip.y < 504.0)
                    })
                    .expect("floating label remains partially visible");
                assert!(clip.y < 504.0, "floating label must extend above the field");
                assert_eq!(layer.y + layer.height, 506.0);
                if index == 1 {
                    assert_eq!(clip, layer);
                } else {
                    assert!(
                        clip.y + clip.height > layer.y + layer.height,
                        "cached text must retain its full region so tiny-skia applies the layer mask"
                    );
                    assert_eq!(clip.intersection(layer), Some(*layer));
                }
            }
        }
    }
}

#[test]
fn tiny_skia_field_labels_do_not_paint_below_the_navigation_boundary() {
    use iced_widget::core::{Layout, Pixels, Size, layout, mouse, widget::Tree};
    use renderer::Renderer as _;

    static LOAD_FONTS: std::sync::Once = std::sync::Once::new();
    LOAD_FONTS.call_once(|| {
        let mut fonts = iced_widget::graphics::text::font_system().write().unwrap();
        for font in crate::fonts::all() {
            fonts.load_font(font);
        }
    });

    // Match the reported 720x1270 Android surface at 2x density: the field
    // begins at y=503dp, its centered label crosses the page edge at y=507dp.
    let surface = Size::new(720, 1270);
    let viewport = Rectangle {
        x: 0.0,
        y: 72.0,
        width: 360.0,
        height: 435.0,
    };
    let options = crate::widget::combobox::State::new(vec!["Assist", "Filter"]);
    let selected = "Assist";
    let fields: [iced_widget::core::Element<'_, (), crate::Theme, iced_tiny_skia::Renderer>; 3] = [
        crate::widget::select::outlined(["Assist", "Filter"], Some("Assist"), |_| ())
            .label("Chip type")
            .into(),
        crate::widget::text_input::outlined("Chip type", "Assist")
            .on_input(|_| ())
            .into(),
        crate::widget::combobox::outlined(&options, "Choose", Some(&selected), |_| ())
            .label("Chip type")
            .into(),
    ];

    for (index, mut field) in fields.into_iter().enumerate() {
        let mut renderer = iced_tiny_skia::Renderer::new(crate::fonts::ROBOTO, Pixels(16.0));
        let mut tree = Tree::new(field.as_widget());
        let node = field
            .as_widget_mut()
            .layout(
                &mut tree,
                &renderer,
                &layout::Limits::new(Size::ZERO, Size::new(304.0, 635.0)),
            )
            .move_to(Point::new(28.0, 503.0));
        renderer.start_layer(viewport);
        field.as_widget().draw(
            &tree,
            &mut renderer,
            &crate::Theme::Dark,
            &renderer::Style {
                text_color: Color::WHITE,
            },
            Layout::new(&node),
            mouse::Cursor::Unavailable,
            &viewport,
        );
        renderer.end_layer();

        let mut pixels = tiny_skia::Pixmap::new(surface.width, surface.height).unwrap();
        let mut mask = tiny_skia::Mask::new(surface.width, surface.height).unwrap();
        renderer.draw(
            &mut pixels.as_mut(),
            &mut mask,
            &iced_widget::graphics::Viewport::with_physical_size(surface, 2.0),
            &[Rectangle::with_size(Size::new(360.0, 635.0))],
            Color::BLACK,
        );

        let is_background = |x, y| {
            let pixel = pixels.pixel(x, y).unwrap();
            pixel.red() == 0 && pixel.green() == 0 && pixel.blue() == 0 && pixel.alpha() == 255
        };
        assert!(
            (990..1014).any(|y| (88..184).any(|x| !is_background(x, y))),
            "field {index} did not render the visible part of its floating label"
        );
        for y in 1014..surface.height {
            for x in 0..surface.width {
                assert!(
                    is_background(x, y),
                    "field {index} painted over navigation at ({x}, {y})"
                );
            }
        }
    }
}

impl renderer::Renderer for OutlineTestRenderer {
    fn start_layer(&mut self, bounds: Rectangle) {
        // Match iced's replacement clip semantics, including nested layers.
        self.layers.push(bounds);
    }

    fn end_layer(&mut self) {
        let _ = self.layers.pop();
    }

    fn start_transformation(&mut self, _transformation: iced_widget::core::Transformation) {}
    fn end_transformation(&mut self) {}
    fn reset(&mut self, _new_bounds: Rectangle) {}

    fn fill_quad(&mut self, quad: renderer::Quad, _background: impl Into<Background>) {
        if let Some(visible) = quad.bounds.intersection(self.layers.last().unwrap()) {
            self.painted.push(visible);
        }
    }

    fn allocate_image(
        &mut self,
        _handle: &iced_widget::core::image::Handle,
        _callback: impl FnOnce(
            Result<iced_widget::core::image::Allocation, iced_widget::core::image::Error>,
        ) + Send
        + 'static,
    ) {
    }
}

#[test]
fn outlined_fields_cannot_replace_the_scroll_viewport_with_a_larger_notch_clip() {
    use renderer::Renderer as _;

    let field = Rectangle {
        x: 28.0,
        y: 504.0,
        width: 304.0,
        height: 56.0,
    };
    let notch = text_field_floating_label_notch(field, 44.0, 54.0, 54.0, 1.0);
    for viewport in [
        // The select straddles the bottom navigation bar at y = 536.
        Rectangle {
            x: 0.0,
            y: 72.0,
            width: 360.0,
            height: 464.0,
        },
        Rectangle {
            x: 0.0,
            y: 520.0,
            width: 360.0,
            height: 100.0,
        },
        Rectangle {
            x: 40.0,
            y: 490.0,
            width: 240.0,
            height: 100.0,
        },
        Rectangle {
            x: 0.0,
            y: 72.0,
            width: 360.0,
            height: 400.0,
        },
    ] {
        for notch in [notch, None] {
            let mut renderer = OutlineTestRenderer::default();
            renderer.start_layer(viewport);
            draw_text_field_outline(
                &mut renderer,
                field,
                Background::Color(Color::TRANSPARENT),
                Border {
                    width: 1.0,
                    color: Color::BLACK,
                    ..Border::default()
                },
                notch,
                &viewport,
            );
            renderer.end_layer();
            for painted in &renderer.painted {
                assert_eq!(
                    painted.intersection(&viewport),
                    Some(*painted),
                    "outline escaped {viewport:?}: {painted:?}"
                );
            }
            assert_eq!(
                renderer.painted.is_empty(),
                field.intersection(&viewport).is_none()
            );
        }
    }
}

#[test]
fn outlined_field_clip_preserves_the_label_notch_inside_the_viewport() {
    use renderer::Renderer as _;

    let field = Rectangle {
        x: 10.0,
        y: 20.0,
        width: 160.0,
        height: 56.0,
    };
    let notch = Rectangle {
        x: 26.0,
        y: 20.0,
        width: 84.0,
        height: 0.0,
    };
    let mut renderer = OutlineTestRenderer::default();
    draw_text_field_notched(
        &mut renderer,
        field,
        1.0,
        Some(notch),
        &field,
        |renderer, _visible| {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: field,
                    ..renderer::Quad::default()
                },
                Color::TRANSPARENT,
            );
        },
    );
    assert_eq!(
        renderer.painted,
        vec![
            Rectangle {
                x: 10.0,
                y: 20.0,
                width: 16.0,
                height: 2.0
            },
            Rectangle {
                x: 110.0,
                y: 20.0,
                width: 60.0,
                height: 2.0
            },
            Rectangle {
                x: 10.0,
                y: 22.0,
                width: 160.0,
                height: 54.0
            },
        ]
    );
}

#[test]
fn text_field_state_tracks_active_ime_preedit() {
    let mut state = TextFieldState::<TestParagraph>::new(false);

    assert!(!state.ime_preedit_active);
    assert!(state.set_ime_preedit("pin yin"));
    assert!(state.ime_preedit_active);
    assert!(!state.set_ime_preedit("more"));
    assert!(state.clear_ime_preedit());
    assert!(!state.ime_preedit_active);
    assert!(!state.clear_ime_preedit());
}
