//! Material 3 outlined multi-line text field constructors.

use super::*;

pub use iced_text_editor::{Action, Binding, Content, KeyPress};

/// Default height for a compact outlined text area preview.
pub const OUTLINED_AREA_HEIGHT: f32 = tokens::component::text_field::CONTAINER_HEIGHT * 2.0;

/// A Material 3 outlined multi-line text field.
pub struct TextEditor<'a, Message, Renderer = iced_widget::Renderer>
where
    Renderer: core_text::Renderer,
{
    is_enabled: bool,
    inner: IcedTextEditor<'a, core_text::highlighter::PlainText, Message, Theme, Renderer>,
}

impl<Message, Renderer> std::fmt::Debug for TextEditor<'_, Message, Renderer>
where
    Renderer: core_text::Renderer,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextEditor")
            .field("is_enabled", &self.is_enabled)
            .finish_non_exhaustive()
    }
}

impl<'a, Message, Renderer> TextEditor<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: core_text::Renderer + 'a,
{
    pub fn new(content: &'a Content<Renderer>) -> Self {
        let inner = IcedTextEditor::new(content)
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
            .min_height(tokens::component::text_field::CONTAINER_HEIGHT)
            .style(text_editor_style::default);

        Self {
            is_enabled: false,
            inner,
        }
    }

    pub fn id(mut self, id: impl Into<core_widget::Id>) -> Self {
        self.inner = self.inner.id(id);
        self
    }

    pub fn placeholder(mut self, placeholder: impl text::IntoFragment<'a>) -> Self {
        self.inner = self.inner.placeholder(placeholder);
        self
    }

    pub fn width(mut self, width: impl Into<Pixels>) -> Self {
        self.inner = self.inner.width(width);
        self
    }

    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.inner = self.inner.height(height);
        self
    }

    pub fn min_height(mut self, min_height: impl Into<Pixels>) -> Self {
        self.inner = self.inner.min_height(min_height);
        self
    }

    pub fn max_height(mut self, max_height: impl Into<Pixels>) -> Self {
        self.inner = self.inner.max_height(max_height);
        self
    }

    pub fn on_action(mut self, on_edit: impl Fn(Action) -> Message + 'a) -> Self {
        self.is_enabled = true;
        self.inner = self.inner.on_action(on_edit);
        self
    }

    pub fn font(mut self, font: impl Into<Renderer::Font>) -> Self {
        self.inner = self.inner.font(font);
        self
    }

    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.inner = self.inner.size(size);
        self
    }

    pub fn line_height(mut self, line_height: impl Into<core_text::LineHeight>) -> Self {
        self.inner = self.inner.line_height(line_height);
        self
    }

    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.inner = self.inner.padding(padding);
        self
    }

    pub fn wrapping(mut self, wrapping: core_text::Wrapping) -> Self {
        self.inner = self.inner.wrapping(wrapping);
        self
    }

    pub fn key_binding(
        mut self,
        key_binding: impl Fn(KeyPress) -> Option<Binding<Message>> + 'a,
    ) -> Self {
        self.inner = self.inner.key_binding(key_binding);
        self
    }

    pub fn style(
        mut self,
        style: impl Fn(&Theme, iced_text_editor::Status) -> iced_text_editor::Style + 'a,
    ) -> Self
    where
        <Theme as iced_text_editor::Catalog>::Class<'a>: From<iced_text_editor::StyleFn<'a, Theme>>,
    {
        self.inner = self.inner.style(style);
        self
    }
}

#[derive(Debug, Default)]
struct TextEditorState {
    touch_activation: Option<TextFieldTouchActivation>,
    web_input_position: WebInputPositionState,
}

impl<Message, Renderer> Widget<Message, Theme, Renderer> for TextEditor<'_, Message, Renderer>
where
    Renderer: core_text::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<TextEditorState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(TextEditorState::default())
    }

    fn children(&self) -> Vec<Tree> {
        let inner: &dyn Widget<Message, Theme, Renderer> = &self.inner;

        vec![Tree::new(inner)]
    }

    fn diff(&self, tree: &mut Tree) {
        if tree.children.is_empty() {
            tree.children = self.children();
        } else {
            self.inner.diff(&mut tree.children[0]);
            tree.children.truncate(1);
        }
    }

    fn size(&self) -> Size<Length> {
        Widget::<Message, Theme, Renderer>::size(&self.inner)
    }

    fn size_hint(&self) -> Size<Length> {
        Widget::<Message, Theme, Renderer>::size_hint(&self.inner)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let inner = self.inner.layout(&mut tree.children[0], renderer, limits);

        layout::Node::with_children(inner.size(), vec![inner])
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn core_widget::Operation,
    ) {
        self.inner.operate(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            operation,
        );
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
        let translated_event = touch_as_mouse_event(event);
        let inner_event = translated_event.as_ref().unwrap_or(event);
        let activation = {
            let state = tree.state.downcast_mut::<TextEditorState>();
            text_input_activation(
                self.is_enabled,
                &mut state.touch_activation,
                &mut state.web_input_position,
                event,
                visible_bounds,
                cursor,
            )
        };

        let started_focused = {
            let state = tree.children[0]
                .state
                .downcast_ref::<iced_text_editor::State<core_text::highlighter::PlainText>>();

            state.is_focused()
        };

        match activation.inner_touch_handling {
            TextFieldInnerTouchHandling::Forward => {
                self.inner.update(
                    &mut tree.children[0],
                    inner_event,
                    layout.children().next().unwrap(),
                    activation.cursor,
                    renderer,
                    clipboard,
                    shell,
                    viewport,
                );
            }
            TextFieldInnerTouchHandling::Suppress => {}
            TextFieldInnerTouchHandling::ConfirmedOutsideTap => {
                self.inner.update(
                    &mut tree.children[0],
                    &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
                    layout.children().next().unwrap(),
                    mouse::Cursor::Unavailable,
                    renderer,
                    clipboard,
                    shell,
                    viewport,
                );
            }
            TextFieldInnerTouchHandling::ConfirmedTap => {
                let press = Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left));
                self.inner.update(
                    &mut tree.children[0],
                    &press,
                    layout.children().next().unwrap(),
                    activation.cursor,
                    renderer,
                    clipboard,
                    shell,
                    viewport,
                );

                let release = Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left));
                self.inner.update(
                    &mut tree.children[0],
                    &release,
                    layout.children().next().unwrap(),
                    activation.cursor,
                    renderer,
                    clipboard,
                    shell,
                    viewport,
                );
            }
        }

        if activation.request_mobile_keyboard {
            shell.request_redraw();
        }

        normalize_windows_ime_request(shell.input_method_mut(), bounds);

        let is_focused = {
            let state = tree.children[0]
                .state
                .downcast_ref::<iced_text_editor::State<core_text::highlighter::PlainText>>();

            state.is_focused()
        };

        if is_focused && text_caret_refresh_event(event) {
            let state = tree.children[0]
                .state
                .downcast_mut::<iced_text_editor::State<core_text::highlighter::PlainText>>();

            core_widget::operation::Focusable::focus(state);
            shell.request_redraw();
        }

        let input_anchor = web_input_anchor(
            shell.input_method(),
            visible_bounds,
            activation,
            started_focused,
            is_focused,
        );

        sync_mobile_keyboard(
            started_focused,
            is_focused,
            activation.request_mobile_keyboard,
            input_anchor,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.inner.mouse_interaction(
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
        register_mobile_text_region(self.is_enabled, layout.bounds(), viewport);

        self.inner.draw(
            &tree.children[0],
            renderer,
            theme,
            defaults,
            layout.children().next().unwrap(),
            cursor,
            viewport,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.inner.overlay(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message, Renderer> From<TextEditor<'a, Message, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: core_text::Renderer + 'a,
{
    fn from(text_editor: TextEditor<'a, Message, Renderer>) -> Self {
        Element::new(text_editor)
    }
}

pub fn outlined<'a, Message, Renderer>(
    content: &'a Content<Renderer>,
) -> TextEditor<'a, Message, Renderer>
where
    Renderer: core_text::Renderer + 'a,
    Message: 'a,
{
    TextEditor::new(content)
}

pub fn outlined_area<'a, Message, Renderer>(
    content: &'a Content<Renderer>,
) -> TextEditor<'a, Message, Renderer>
where
    Renderer: core_text::Renderer + 'a,
    Message: 'a,
{
    outlined(content).height(Length::Fixed(OUTLINED_AREA_HEIGHT))
}
