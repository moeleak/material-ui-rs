//! Material 3 outlined text field constructors with floating label support.

use super::*;
use iced_widget::core::text::Paragraph;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LabelMode {
    Placeholder,
    Floating,
}

fn input_layer_style_alpha(
    theme: &Theme,
    status: iced_text_input::Status,
    content_alpha: f32,
) -> iced_text_input::Style {
    let mut style = text_input_style::default(theme, status);

    style.background = Background::Color(Color::TRANSPARENT);
    style.border.width = 0.0;
    style.border.color = Color::TRANSPARENT;
    style.icon = alpha_color(style.icon, content_alpha);
    style.placeholder = Color::TRANSPARENT;
    style.value = alpha_color(style.value, content_alpha);
    style.selection = alpha_color(style.selection, content_alpha);

    style
}

fn status_style(
    theme: &Theme,
    is_enabled: bool,
    is_error: bool,
    is_focused: bool,
    is_hovered: bool,
) -> (Color, f32, Color) {
    let colors = theme.colors();

    if !is_enabled {
        (
            alpha_color(
                colors.surface.text,
                tokens::component::text_field::DISABLED_OUTLINE_OPACITY,
            ),
            tokens::component::text_field::OUTLINE_WIDTH,
            alpha_color(
                colors.surface.text,
                tokens::component::text_field::DISABLED_LABEL_TEXT_OPACITY,
            ),
        )
    } else if is_error {
        let outline_width = if is_focused {
            tokens::component::text_field::FOCUS_OUTLINE_WIDTH
        } else if is_hovered {
            tokens::component::text_field::HOVER_OUTLINE_WIDTH
        } else {
            tokens::component::text_field::OUTLINE_WIDTH
        };

        (colors.error.color, outline_width, colors.error.color)
    } else if is_focused {
        (
            colors.primary.color,
            tokens::component::text_field::FOCUS_OUTLINE_WIDTH,
            colors.primary.color,
        )
    } else if is_hovered {
        (
            colors.surface.text,
            tokens::component::text_field::HOVER_OUTLINE_WIDTH,
            colors.surface.text,
        )
    } else {
        (
            colors.outline.color,
            tokens::component::text_field::OUTLINE_WIDTH,
            colors.surface.text_variant,
        )
    }
}

/// A Material 3 outlined text field with an animated floating label.
pub struct TextInput<'a, Message, Renderer = iced_widget::Renderer>
where
    Renderer: iced_widget::core::Renderer + core_text::Renderer,
{
    label: text::Fragment<'a>,
    value: String,
    is_populated: bool,
    is_enabled: bool,
    is_secure: bool,
    is_error: bool,
    content_alpha: f32,
    width: Length,
    font: Option<Renderer::Font>,
    label_mode: LabelMode,
    input: IcedTextInput<'a, Message, Theme, Renderer>,
}

impl<Message, Renderer> std::fmt::Debug for TextInput<'_, Message, Renderer>
where
    Renderer: iced_widget::core::Renderer + core_text::Renderer,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextInput")
            .field("is_populated", &self.is_populated)
            .field("is_enabled", &self.is_enabled)
            .field("is_secure", &self.is_secure)
            .field("is_error", &self.is_error)
            .field("content_alpha", &self.content_alpha)
            .field("width", &self.width)
            .field("label_mode", &self.label_mode)
            .finish_non_exhaustive()
    }
}

impl<'a, Message, Renderer> TextInput<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
{
    pub fn new(label: impl text::IntoFragment<'a>, value: &str) -> Self {
        Self::with_mode(label, value, LabelMode::Floating)
    }

    fn placeholder(label: impl text::IntoFragment<'a>, value: &str) -> Self {
        Self::with_mode(label, value, LabelMode::Placeholder)
    }

    fn with_mode(label: impl text::IntoFragment<'a>, value: &str, label_mode: LabelMode) -> Self {
        let input = IcedTextInput::new("", value)
            .width(Length::Fill)
            .padding(Padding {
                top: tokens::component::text_field::TOP_SPACE,
                right: tokens::component::text_field::TRAILING_SPACE,
                bottom: tokens::component::text_field::BOTTOM_SPACE,
                left: tokens::component::text_field::LEADING_SPACE,
            })
            .size(tokens::component::text_field::INPUT_TEXT_SIZE)
            .line_height(absolute_line_height(
                tokens::component::text_field::INPUT_TEXT_LINE_HEIGHT,
            ))
            .style(|theme, status| input_layer_style_alpha(theme, status, 1.0));

        Self {
            label: label.into_fragment(),
            value: value.to_owned(),
            is_populated: !value.is_empty(),
            is_enabled: false,
            is_secure: false,
            is_error: false,
            content_alpha: 1.0,
            width: Length::Fill,
            font: None,
            label_mode,
            input,
        }
    }

    pub fn id(mut self, id: impl Into<core_widget::Id>) -> Self {
        self.input = self.input.id(id);
        self
    }

    pub fn secure(mut self, is_secure: bool) -> Self {
        self.is_secure = is_secure;
        self.input = self.input.secure(is_secure);
        self
    }

    pub fn on_input(mut self, on_input: impl Fn(String) -> Message + 'a) -> Self {
        self.is_enabled = true;
        self.input = self.input.on_input(on_input);
        self
    }

    pub fn on_input_maybe(mut self, on_input: Option<impl Fn(String) -> Message + 'a>) -> Self {
        self.is_enabled = on_input.is_some();
        self.input = self.input.on_input_maybe(on_input);
        self
    }

    pub fn on_submit(mut self, message: Message) -> Self {
        self.input = self.input.on_submit(message);
        self
    }

    pub fn on_paste(mut self, on_paste: impl Fn(String) -> Message + 'a) -> Self {
        self.input = self.input.on_paste(on_paste);
        self
    }

    pub fn error(mut self, is_error: bool) -> Self {
        self.is_error = is_error;
        self
    }

    pub fn alpha(mut self, content_alpha: f32) -> Self {
        let content_alpha = content_alpha.clamp(0.0, 1.0);

        self.content_alpha = content_alpha;
        self.input = self
            .input
            .style(move |theme, status| input_layer_style_alpha(theme, status, content_alpha));

        self
    }

    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self.input = self.input.width(self.width);
        self
    }

    pub fn font(mut self, font: impl Into<Renderer::Font>) -> Self {
        let font = font.into();
        self.font = Some(font);
        self.input = self.input.font(font);
        self
    }
}

impl<Message, Renderer> Widget<Message, Theme, Renderer> for TextInput<'_, Message, Renderer>
where
    Message: Clone,
    Renderer: iced_widget::core::Renderer + core_text::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<TextFieldState<Renderer::Paragraph>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(TextFieldState::<Renderer::Paragraph>::new(
            self.label_mode == LabelMode::Floating && self.is_populated,
        ))
    }

    fn children(&self) -> Vec<Tree> {
        let input: &dyn Widget<Message, Theme, Renderer> = &self.input;

        vec![Tree::new(input)]
    }

    fn diff(&self, tree: &mut Tree) {
        let state = tree
            .state
            .downcast_mut::<TextFieldState<Renderer::Paragraph>>();

        let target = if self.label_mode == LabelMode::Floating {
            bool_value(self.is_populated || state.is_focused)
        } else {
            0.0
        };

        state.label_float.set_target(
            target,
            Instant::now(),
            duration_ms(tokens::component::text_field::LABEL_TRANSITION_DURATION_MS),
            tokens::component::text_field::LABEL_TRANSITION_EASING,
        );

        if tree.children.is_empty() {
            tree.children = self.children();
        } else {
            self.input.diff(&mut tree.children[0]);
            tree.children.truncate(1);
        }
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: Length::Fixed(tokens::component::text_field::CONTAINER_HEIGHT),
        }
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let state = tree
            .state
            .downcast_mut::<TextFieldState<Renderer::Paragraph>>();

        let label_size = tokens::component::text_field::LABEL_TEXT_SIZE;
        let label_line_height = tokens::component::text_field::LABEL_TEXT_LINE_HEIGHT;
        let floating_label_size = tokens::component::text_field::LABEL_TEXT_POPULATED_SIZE;
        let floating_label_line_height =
            tokens::component::text_field::LABEL_TEXT_POPULATED_LINE_HEIGHT;

        let label_node = core_widget::text::layout(
            &mut state.label,
            renderer,
            &layout::Limits::NONE,
            self.label.as_ref(),
            core_widget::text::Format {
                width: Length::Shrink,
                height: Length::Shrink,
                line_height: absolute_line_height(label_line_height),
                size: Some(Pixels(label_size)),
                font: self.font,
                align_x: text::Alignment::Default,
                align_y: alignment::Vertical::Top,
                shaping: text::Shaping::Advanced,
                wrapping: text::Wrapping::None,
            },
        );

        let floating_label_node = core_widget::text::layout(
            &mut state.floating_label,
            renderer,
            &layout::Limits::NONE,
            self.label.as_ref(),
            core_widget::text::Format {
                width: Length::Shrink,
                height: Length::Shrink,
                line_height: absolute_line_height(floating_label_line_height),
                size: Some(Pixels(floating_label_size)),
                font: self.font,
                align_x: text::Alignment::Default,
                align_y: alignment::Vertical::Top,
                shaping: text::Shaping::Advanced,
                wrapping: text::Wrapping::None,
            },
        );

        let intrinsic = Size::new(
            label_node
                .size()
                .width
                .max(floating_label_node.size().width)
                + tokens::component::text_field::LEADING_SPACE
                + tokens::component::text_field::TRAILING_SPACE,
            tokens::component::text_field::CONTAINER_HEIGHT,
        );
        let size = limits.resolve(
            self.width,
            Length::Fixed(tokens::component::text_field::CONTAINER_HEIGHT),
            intrinsic,
        );
        let child_limits = layout::Limits::new(size, size);
        let input = <IcedTextInput<'_, Message, Theme, Renderer> as Widget<
            Message,
            Theme,
            Renderer,
        >>::layout(
            &mut self.input,
            &mut tree.children[0],
            renderer,
            &child_limits,
        );

        layout::Node::with_children(size, vec![input.move_to(Point::ORIGIN)])
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn core_widget::Operation,
    ) {
        operation.text(None, layout.bounds(), self.label.as_ref());
        operation.traverse(&mut |operation| {
            self.input.operate(
                &mut tree.children[0],
                layout.children().next().unwrap(),
                renderer,
                operation,
            );
        });
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let visible_bounds = bounds.intersection(viewport);
        let input_layout = layout.children().next().unwrap();

        let was_focused = {
            let state = tree
                .state
                .downcast_ref::<TextFieldState<Renderer::Paragraph>>();

            state.is_focused
        };

        let activation = {
            let state = tree
                .state
                .downcast_mut::<TextFieldState<Renderer::Paragraph>>();
            let activation = text_input_activation(
                self.is_enabled,
                &mut state.touch_activation,
                event,
                visible_bounds,
                cursor,
            );

            match event {
                Event::Keyboard(iced_widget::core::keyboard::Event::KeyPressed { key, .. })
                    if matches!(
                        key.as_ref(),
                        iced_widget::core::keyboard::Key::Named(
                            iced_widget::core::keyboard::key::Named::Escape
                        )
                    ) =>
                {
                    state.is_focused = false;
                    if state.clear_ime_preedit() {
                        shell.request_redraw();
                    }
                }
                Event::InputMethod(input_method::Event::Preedit(content, _))
                    if state.set_ime_preedit(content) =>
                {
                    shell.request_redraw();
                }
                Event::InputMethod(
                    input_method::Event::Opened
                    | input_method::Event::Closed
                    | input_method::Event::Commit(_),
                ) if state.clear_ime_preedit() => {
                    shell.request_redraw();
                }
                Event::Window(window::Event::RedrawRequested(now))
                    if state.label_float.advance(*now) =>
                {
                    shell.request_redraw();
                }
                _ => {}
            }

            activation
        };

        update_mobile_text_input(
            &mut self.input,
            &mut tree.children[0],
            event,
            input_layout,
            activation,
            renderer,
            clipboard,
            shell,
            viewport,
        );

        normalize_windows_ime_request(shell.input_method_mut(), bounds);

        let is_focused = {
            let input_state = tree.children[0]
                .state
                .downcast_ref::<iced_text_input::State<Renderer::Paragraph>>();

            input_state.is_focused()
        };

        {
            let state = tree
                .state
                .downcast_mut::<TextFieldState<Renderer::Paragraph>>();
            state.is_focused = is_focused;

            if was_focused != is_focused {
                sync_mobile_keyboard(was_focused, is_focused, activation.request_mobile_keyboard);

                shell.request_redraw();
            }

            let target = if self.label_mode == LabelMode::Floating {
                bool_value(self.is_populated || state.is_focused)
            } else {
                0.0
            };
            state.label_float.set_target(
                target,
                Instant::now(),
                duration_ms(tokens::component::text_field::LABEL_TRANSITION_DURATION_MS),
                tokens::component::text_field::LABEL_TRANSITION_EASING,
            );

            if state.is_animating() {
                shell.request_redraw();
            }
        }

        if was_focused == is_focused {
            sync_mobile_keyboard(was_focused, is_focused, activation.request_mobile_keyboard);
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.input.mouse_interaction(
            &tree.children[0],
            layout.children().next().unwrap(),
            cursor,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        defaults: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree
            .state
            .downcast_ref::<TextFieldState<Renderer::Paragraph>>();
        let bounds = layout.bounds();
        register_mobile_text_region(self.is_enabled, bounds, viewport);

        let progress = if self.label_mode == LabelMode::Floating {
            state.label_float.value.clamp(0.0, 1.0)
        } else {
            0.0
        };
        let is_hovered = cursor.is_over(bounds);
        let (outline_color, outline_width, label_color) = status_style(
            theme,
            self.is_enabled,
            self.is_error,
            state.is_focused,
            is_hovered,
        );
        let outline_color = alpha_color(outline_color, self.content_alpha);
        let label_color = alpha_color(label_color, self.content_alpha);
        let label_width = state.label.raw().min_bounds().width;
        let floating_label_width = state.floating_label.raw().min_bounds().width;
        let label_line_height = tokens::component::text_field::LABEL_TEXT_LINE_HEIGHT;
        let floating_label_line_height =
            tokens::component::text_field::LABEL_TEXT_POPULATED_LINE_HEIGHT;
        let floating_label_y = -floating_label_line_height / 2.0;
        let label_y = bounds.y
            + lerp(
                tokens::component::text_field::TOP_SPACE,
                floating_label_y,
                progress,
            );
        let label_x = bounds.x + tokens::component::text_field::LEADING_SPACE;
        let label_notch = text_field_floating_label_notch(
            bounds,
            label_x,
            label_width,
            floating_label_width,
            progress,
        );

        draw_text_field_outline(
            renderer,
            bounds,
            Background::Color(Color::TRANSPARENT),
            Border {
                color: outline_color,
                width: outline_width,
                radius: tokens::component::text_field::CONTAINER_SHAPE.into(),
            },
            label_notch,
        );

        let input_layout = layout.children().next().unwrap();
        let caretless_input;
        let input = if state.ime_preedit_active && self.is_enabled && should_suppress_ime_caret() {
            let content_alpha = self.content_alpha;
            // Keep iced_winit's IME preedit overlay, but suppress iced's own
            // blinking caret so composition does not show two insertion marks.
            let mut input = IcedTextInput::new("", self.value.as_str())
                .width(Length::Fill)
                .padding(Padding {
                    top: tokens::component::text_field::TOP_SPACE,
                    right: tokens::component::text_field::TRAILING_SPACE,
                    bottom: tokens::component::text_field::BOTTOM_SPACE,
                    left: tokens::component::text_field::LEADING_SPACE,
                })
                .size(tokens::component::text_field::INPUT_TEXT_SIZE)
                .line_height(absolute_line_height(
                    tokens::component::text_field::INPUT_TEXT_LINE_HEIGHT,
                ))
                .style(move |theme, status| input_layer_style_alpha(theme, status, content_alpha));

            if self.is_secure {
                input = input.secure(true);
            }

            if let Some(font) = self.font {
                input = input.font(font);
            }

            caretless_input = input;
            &caretless_input
        } else {
            &self.input
        };

        <IcedTextInput<'_, Message, Theme, Renderer> as Widget<Message, Theme, Renderer>>::draw(
            input,
            &tree.children[0],
            renderer,
            theme,
            defaults,
            input_layout,
            cursor,
            viewport,
        );

        if self.label_mode == LabelMode::Placeholder && self.is_populated {
            return;
        }

        if progress < 0.99 {
            core_widget::text::draw(
                renderer,
                defaults,
                Rectangle {
                    x: label_x,
                    y: label_y,
                    width: label_width,
                    height: label_line_height,
                },
                state.label.raw(),
                core_widget::text::Style {
                    color: Some(alpha_color(label_color, 1.0 - progress)),
                },
                viewport,
            );
        }

        if progress > 0.01 {
            core_widget::text::draw(
                renderer,
                defaults,
                Rectangle {
                    x: label_x,
                    y: label_y,
                    width: floating_label_width,
                    height: floating_label_line_height,
                },
                state.floating_label.raw(),
                core_widget::text::Style {
                    color: Some(alpha_color(label_color, progress)),
                },
                viewport,
            );
        }
    }
}

impl<'a, Message, Renderer> From<TextInput<'a, Message, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
{
    fn from(text_input: TextInput<'a, Message, Renderer>) -> Self {
        Element::new(text_input)
    }
}

pub fn outlined<'a, Message, Renderer>(
    label: impl text::IntoFragment<'a>,
    value: &str,
) -> TextInput<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
{
    TextInput::new(label, value)
}

pub fn outlined_floating<'a, Message, Renderer>(
    label: impl text::IntoFragment<'a>,
    value: &str,
) -> TextInput<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
{
    TextInput::new(label, value)
}

pub fn outlined_placeholder<'a, Message, Renderer>(
    label: impl text::IntoFragment<'a>,
    value: &str,
) -> TextInput<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced_widget::core::Renderer + core_text::Renderer + 'a,
{
    TextInput::placeholder(label, value)
}

#[cfg(test)]
#[path = "../../../tests/widget/component/text_input.rs"]
mod tests;
