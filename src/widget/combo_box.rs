//! Material 3 searchable select constructors with token-backed layout defaults.

use std::cell::RefCell;
use std::fmt::{self, Display};

use iced_widget::core::keyboard::key;
use iced_widget::core::text as core_text;
use iced_widget::core::text::paragraph;
use iced_widget::core::time::Instant;
use iced_widget::core::widget::{self, Widget};
use iced_widget::core::{
    Clipboard, Color, Element, Event, Layout, Length, Padding, Pixels, Point, Rectangle, Shell,
    Size, Vector, keyboard, layout, mouse, overlay, renderer,
};
use iced_widget::overlay::menu as overlay_menu;
use iced_widget::text::{self, LineHeight};
use iced_widget::text_input::{self, Icon, TextInput};

use super::menu_overlay;
use super::{
    MobileTextInputState, absolute_line_height, mobile_text_input_activation,
    register_mobile_text_region, select, sync_mobile_keyboard, update_mobile_text_input,
};
use crate::{Theme, menu as menu_style, text_input as text_input_style, tokens};

#[derive(Clone)]
enum DisplayValue<T> {
    Option(T),
    Input(String),
}

impl<T> fmt::Display for DisplayValue<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Option(option) => option.fmt(f),
            Self::Input(input) => input.fmt(f),
        }
    }
}

/// Searchable select state.
///
/// The inner state keeps the current search query and filtered Material menu
/// options. The public state also stores the original options so selected
/// values and typed input can be mirrored by the demo/application state without
/// replacing user-entered text on blur.
pub struct State<T> {
    options: Vec<T>,
    inner: SearchState<DisplayValue<T>>,
}

impl<T> fmt::Debug for State<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("State")
            .field("options", &self.options)
            .finish_non_exhaustive()
    }
}

impl<T> State<T>
where
    T: fmt::Display + Clone,
{
    /// Creates a new [`State`] for a combo box with the given list of options.
    pub fn new(options: Vec<T>) -> Self {
        Self::with_selection(options, None)
    }

    /// Creates a new [`State`] for a combo box with the given list of options
    /// and selected value.
    pub fn with_selection(options: Vec<T>, selection: Option<&T>) -> Self {
        let inner_options = inner_options(&options);
        let inner_selection = selection.cloned().map(DisplayValue::Option);

        Self {
            options,
            inner: SearchState::with_selection(inner_options, inner_selection.as_ref()),
        }
    }

    /// Returns the original options.
    pub fn options(&self) -> &[T] {
        &self.options
    }

    /// Pushes a new option.
    pub fn push(&mut self, new_option: T) {
        self.inner.push(DisplayValue::Option(new_option.clone()));
        self.options.push(new_option);
    }

    /// Returns ownership of the original options.
    pub fn into_options(self) -> Vec<T> {
        self.options
    }

    /// Synchronizes the internal query with the latest user input.
    pub fn set_input(&mut self, input: impl Into<String>) {
        let input = input.into();
        let inner_selection = if input.is_empty() {
            None
        } else {
            Some(DisplayValue::Input(input))
        };

        self.inner =
            SearchState::with_selection(inner_options(&self.options), inner_selection.as_ref());
    }

    /// Synchronizes the internal query with the selected option.
    pub fn set_selection(&mut self, selection: Option<&T>) {
        let inner_selection = selection.cloned().map(DisplayValue::Option);

        self.inner =
            SearchState::with_selection(inner_options(&self.options), inner_selection.as_ref());
    }

    fn inner(&self) -> &SearchState<DisplayValue<T>> {
        &self.inner
    }
}

impl<T> Default for State<T>
where
    T: fmt::Display + Clone,
{
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

/// Material combo box.
pub struct ComboBox<'a, T, Message, Renderer>
where
    T: fmt::Display + Clone,
    Renderer: core_text::Renderer,
{
    inner: MaterialComboBox<'a, DisplayValue<T>, Message, Renderer>,
}

impl<T, Message, Renderer> fmt::Debug for ComboBox<'_, T, Message, Renderer>
where
    T: fmt::Display + Clone,
    Renderer: core_text::Renderer,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ComboBox").finish_non_exhaustive()
    }
}

impl<'a, T, Message, Renderer> ComboBox<'a, T, Message, Renderer>
where
    T: fmt::Display + Clone + 'static,
    Renderer: core_text::Renderer + 'a,
{
    /// Sets the message that should be produced when text is typed.
    pub fn on_input(mut self, on_input: impl Fn(String) -> Message + 'static) -> Self {
        self.inner = self.inner.on_input(on_input);
        self
    }

    /// Sets the message that will be produced when an option is hovered.
    pub fn on_option_hovered(mut self, on_option_hovered: impl Fn(T) -> Message + 'static) -> Self {
        self.inner = self.inner.on_option_hovered(move |value| match value {
            DisplayValue::Option(option) => on_option_hovered(option),
            DisplayValue::Input(_) => {
                unreachable!("typed input is not a selectable option")
            }
        });
        self
    }

    /// Sets the message that will be produced when the combo box opens.
    pub fn on_open(mut self, message: Message) -> Self {
        self.inner = self.inner.on_open(message);
        self
    }

    /// Sets the message that will be produced when the combo box closes.
    pub fn on_close(mut self, message: Message) -> Self {
        self.inner = self.inner.on_close(message);
        self
    }

    /// Sets the floating label of the combo box.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.inner = self.inner.label(label);
        self
    }

    /// Sets the padding.
    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.inner = self.inner.padding(padding);
        self
    }

    /// Sets the font.
    pub fn font(mut self, font: Renderer::Font) -> Self {
        self.inner = self.inner.font(font);
        self
    }

    /// Sets the trailing icon.
    pub fn icon(mut self, icon: Icon<Renderer::Font>) -> Self {
        self.inner = self.inner.icon(icon);
        self
    }

    /// Sets the text size.
    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.inner = self.inner.size(size);
        self
    }

    /// Sets the text line height.
    pub fn line_height(mut self, line_height: impl Into<LineHeight>) -> Self {
        self.inner = self.inner.line_height(line_height);
        self
    }

    /// Sets the width.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.inner = self.inner.width(width);
        self
    }

    /// Sets the menu height.
    pub fn menu_height(mut self, menu_height: impl Into<Length>) -> Self {
        self.inner = self.inner.menu_height(menu_height);
        self
    }

    /// Sets the text shaping strategy.
    pub fn text_shaping(mut self, shaping: text::Shaping) -> Self {
        self.inner = self.inner.text_shaping(shaping);
        self
    }

    /// Sets the input style.
    pub fn input_style(
        mut self,
        style: impl Fn(&Theme, text_input::Status) -> text_input::Style + 'a,
    ) -> Self
    where
        <Theme as text_input::Catalog>::Class<'a>: From<text_input::StyleFn<'a, Theme>>,
    {
        self.inner = self.inner.input_style(style);
        self
    }

    /// Sets the menu style.
    pub fn menu_style(mut self, style: impl Fn(&Theme) -> overlay_menu::Style + 'a) -> Self
    where
        <Theme as overlay_menu::Catalog>::Class<'a>: From<overlay_menu::StyleFn<'a, Theme>>,
    {
        self.inner = self.inner.menu_style(style);
        self
    }
}

impl<'a, T, Message, Renderer> From<ComboBox<'a, T, Message, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    T: fmt::Display + Clone + 'static,
    Message: Clone + 'a,
    Renderer: core_text::Renderer + 'a,
{
    fn from(combo_box: ComboBox<'a, T, Message, Renderer>) -> Self {
        Element::new(combo_box.inner)
    }
}

pub fn outlined<'a, T, Message, Renderer>(
    state: &'a State<T>,
    placeholder: &str,
    selection: Option<&T>,
    on_selected: impl Fn(T) -> Message + 'static,
) -> ComboBox<'a, T, Message, Renderer>
where
    T: fmt::Display + Clone + 'static,
    Renderer: core_text::Renderer + 'a,
{
    outlined_with_input(state, placeholder, "", selection, on_selected)
}

pub fn outlined_with_input<'a, T, Message, Renderer>(
    state: &'a State<T>,
    placeholder: &str,
    input: &str,
    selection: Option<&T>,
    on_selected: impl Fn(T) -> Message + 'static,
) -> ComboBox<'a, T, Message, Renderer>
where
    T: fmt::Display + Clone + 'static,
    Renderer: core_text::Renderer + 'a,
{
    let display_value = if input.is_empty() {
        selection.cloned().map(DisplayValue::Option)
    } else {
        Some(DisplayValue::Input(input.to_owned()))
    };

    let inner = MaterialComboBox::new(state.inner(), placeholder, display_value.as_ref(), {
        move |value| match value {
            DisplayValue::Option(option) => on_selected(option),
            DisplayValue::Input(_) => {
                unreachable!("typed input is not a selectable option")
            }
        }
    })
    .padding(Padding {
        top: tokens::component::text_field::TOP_SPACE,
        right: tokens::component::text_field::TRAILING_SPACE,
        bottom: tokens::component::text_field::BOTTOM_SPACE,
        left: tokens::component::text_field::LEADING_SPACE,
    })
    .option_padding(select::menu_option_padding())
    .size(tokens::component::text_field::INPUT_TEXT_SIZE)
    .line_height(absolute_line_height(
        tokens::component::text_field::INPUT_TEXT_LINE_HEIGHT,
    ))
    .width(Length::Fill)
    .input_style(text_input_style::default)
    .menu_style(menu_style::outlined_select);

    ComboBox { inner }
}

struct MaterialComboBox<'a, T, Message, Renderer>
where
    T: Display + Clone,
    Renderer: core_text::Renderer,
{
    state: &'a SearchState<T>,
    text_input: TextInput<'a, TextInputEvent, Theme, Renderer>,
    label: Option<String>,
    font: Option<Renderer::Font>,
    selection: text_input::Value,
    on_selected: Box<dyn Fn(T) -> Message>,
    on_option_hovered: Option<Box<dyn Fn(T) -> Message>>,
    on_open: Option<Message>,
    on_close: Option<Message>,
    on_input: Option<Box<dyn Fn(String) -> Message>>,
    input_padding: Padding,
    option_padding: Padding,
    size: Option<Pixels>,
    line_height: LineHeight,
    text_shaping: text::Shaping,
    menu_class: <Theme as overlay_menu::Catalog>::Class<'a>,
    menu_height: Length,
}

impl<T, Message, Renderer> fmt::Debug for MaterialComboBox<'_, T, Message, Renderer>
where
    T: Display + Clone,
    Renderer: core_text::Renderer,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MaterialComboBox").finish_non_exhaustive()
    }
}

impl<'a, T, Message, Renderer> MaterialComboBox<'a, T, Message, Renderer>
where
    T: Display + Clone,
    Renderer: core_text::Renderer,
{
    fn new(
        state: &'a SearchState<T>,
        placeholder: &str,
        selection: Option<&T>,
        on_selected: impl Fn(T) -> Message + 'static,
    ) -> Self {
        let text_input = TextInput::new(placeholder, &state.value())
            .on_input(TextInputEvent::TextChanged)
            .class(<Theme as text_input::Catalog>::default());

        let selection = selection.map(T::to_string).unwrap_or_default();

        Self {
            state,
            text_input,
            label: None,
            font: None,
            selection: text_input::Value::new(&selection),
            on_selected: Box::new(on_selected),
            on_option_hovered: None,
            on_input: None,
            on_open: None,
            on_close: None,
            input_padding: text_input::DEFAULT_PADDING,
            option_padding: select::menu_option_padding(),
            size: None,
            line_height: LineHeight::default(),
            text_shaping: text::Shaping::default(),
            menu_class: <Theme as overlay_menu::Catalog>::default(),
            menu_height: Length::Shrink,
        }
    }

    fn on_input(mut self, on_input: impl Fn(String) -> Message + 'static) -> Self {
        self.on_input = Some(Box::new(on_input));
        self
    }

    fn on_option_hovered(mut self, on_option_hovered: impl Fn(T) -> Message + 'static) -> Self {
        self.on_option_hovered = Some(Box::new(on_option_hovered));
        self
    }

    fn on_open(mut self, message: Message) -> Self {
        self.on_open = Some(message);
        self
    }

    fn on_close(mut self, message: Message) -> Self {
        self.on_close = Some(message);
        self
    }

    fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.input_padding = padding.into();
        self.text_input = self.text_input.padding(self.input_padding);
        self
    }

    fn option_padding(mut self, padding: impl Into<Padding>) -> Self {
        self.option_padding = padding.into();
        self
    }

    fn font(mut self, font: Renderer::Font) -> Self {
        self.text_input = self.text_input.font(font);
        self.font = Some(font);
        self
    }

    fn icon(mut self, icon: Icon<Renderer::Font>) -> Self {
        self.text_input = self.text_input.icon(icon);
        self
    }

    fn size(mut self, size: impl Into<Pixels>) -> Self {
        let size = size.into();

        self.text_input = self.text_input.size(size);
        self.size = Some(size);

        self
    }

    fn line_height(mut self, line_height: impl Into<LineHeight>) -> Self {
        self.line_height = line_height.into();
        self.text_input = self.text_input.line_height(self.line_height);
        self
    }

    fn width(mut self, width: impl Into<Length>) -> Self {
        self.text_input = self.text_input.width(width);
        self
    }

    fn menu_height(mut self, menu_height: impl Into<Length>) -> Self {
        self.menu_height = menu_height.into();
        self
    }

    fn text_shaping(mut self, shaping: text::Shaping) -> Self {
        self.text_shaping = shaping;
        self
    }

    fn input_style(
        mut self,
        style: impl Fn(&Theme, text_input::Status) -> text_input::Style + 'a,
    ) -> Self
    where
        <Theme as text_input::Catalog>::Class<'a>: From<text_input::StyleFn<'a, Theme>>,
    {
        self.text_input = self.text_input.style(style);
        self
    }

    fn menu_style(mut self, style: impl Fn(&Theme) -> overlay_menu::Style + 'a) -> Self
    where
        <Theme as overlay_menu::Catalog>::Class<'a>: From<overlay_menu::StyleFn<'a, Theme>>,
    {
        self.menu_class = (Box::new(style) as overlay_menu::StyleFn<'a, Theme>).into();
        self
    }

    fn intrinsic_menu_height(&self, option_count: usize, renderer: &Renderer) -> f32 {
        let text_size = self.size.unwrap_or_else(|| renderer.default_size());
        let option_height =
            f32::from(self.line_height.to_absolute(text_size)) + self.option_padding.y();

        option_height * option_count as f32
    }
}

#[derive(Debug, Clone)]
struct SearchState<T> {
    options: Vec<T>,
    inner: RefCell<Inner<T>>,
}

#[derive(Debug, Clone)]
struct Inner<T> {
    value: String,
    option_matchers: Vec<String>,
    filtered_options: Filtered<T>,
}

#[derive(Debug, Clone)]
struct Filtered<T> {
    options: Vec<T>,
    updated: Instant,
}

impl<T> SearchState<T>
where
    T: Display + Clone,
{
    fn with_selection(options: Vec<T>, selection: Option<&T>) -> Self {
        let value = selection.map(T::to_string).unwrap_or_default();
        let option_matchers = build_matchers(&options);
        let filtered_options = Filtered::new(
            search(&options, &option_matchers, &value)
                .cloned()
                .collect(),
        );

        Self {
            options,
            inner: RefCell::new(Inner {
                value,
                option_matchers,
                filtered_options,
            }),
        }
    }

    fn push(&mut self, new_option: T) {
        let mut inner = self.inner.borrow_mut();

        inner.option_matchers.push(build_matcher(&new_option));
        self.options.push(new_option);

        inner.filtered_options = Filtered::new(
            search(&self.options, &inner.option_matchers, &inner.value)
                .cloned()
                .collect(),
        );
    }

    fn value(&self) -> String {
        let inner = self.inner.borrow();

        inner.value.clone()
    }

    fn with_inner<O>(&self, f: impl FnOnce(&Inner<T>) -> O) -> O {
        let inner = self.inner.borrow();

        f(&inner)
    }

    fn with_inner_mut(&self, f: impl FnOnce(&mut Inner<T>)) {
        let mut inner = self.inner.borrow_mut();

        f(&mut inner);
    }

    fn sync_filtered_options(&self, options: &mut Filtered<T>) {
        let inner = self.inner.borrow();

        inner.filtered_options.sync(options);
    }
}

impl<T> Filtered<T>
where
    T: Clone,
{
    fn new(options: Vec<T>) -> Self {
        Self {
            options,
            updated: Instant::now(),
        }
    }

    fn empty() -> Self {
        Self {
            options: Vec::new(),
            updated: Instant::now(),
        }
    }

    fn update(&mut self, options: Vec<T>) {
        self.options = options;
        self.updated = Instant::now();
    }

    fn sync(&self, other: &mut Filtered<T>) {
        if other.updated != self.updated {
            *other = self.clone();
        }
    }
}

struct MenuState<T, P: core_text::Paragraph> {
    menu: menu_overlay::State,
    mobile_input: MobileTextInputState,
    hovered_option: Option<usize>,
    new_selection: Option<T>,
    filtered_options: Filtered<T>,
    label: paragraph::Plain<P>,
}

#[derive(Debug, Clone)]
enum TextInputEvent {
    TextChanged(String),
}

impl<T, Message, Renderer> Widget<Message, Theme, Renderer>
    for MaterialComboBox<'_, T, Message, Renderer>
where
    T: Display + Clone + 'static,
    Message: Clone,
    Renderer: core_text::Renderer,
{
    fn size(&self) -> Size<Length> {
        Widget::<TextInputEvent, Theme, Renderer>::size(&self.text_input)
    }

    fn layout(
        &mut self,
        tree: &mut widget::Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        if let Some(label) = &self.label {
            let state = tree
                .state
                .downcast_mut::<MenuState<T, Renderer::Paragraph>>();
            let label_size = Pixels(tokens::component::text_field::LABEL_TEXT_POPULATED_SIZE);
            let label_line_height = LineHeight::Absolute(Pixels(
                tokens::component::text_field::LABEL_TEXT_POPULATED_LINE_HEIGHT,
            ));

            let _ = state.label.update(core_text::Text {
                content: label,
                bounds: Size::new(
                    f32::INFINITY,
                    f32::from(label_line_height.to_absolute(label_size)),
                ),
                size: label_size,
                line_height: label_line_height,
                font: self.font.unwrap_or_else(|| renderer.default_font()),
                align_x: text::Alignment::Default,
                align_y: iced_widget::core::alignment::Vertical::Center,
                shaping: self.text_shaping,
                wrapping: text::Wrapping::None,
            });
        }

        let is_focused = {
            let text_input_state = tree.children[0]
                .state
                .downcast_ref::<text_input::State<Renderer::Paragraph>>();

            text_input_state.is_focused()
        };

        self.text_input.layout(
            &mut tree.children[0],
            renderer,
            limits,
            (!is_focused).then_some(&self.selection),
        )
    }

    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<MenuState<T, Renderer::Paragraph>>()
    }

    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(MenuState::<T, Renderer::Paragraph> {
            menu: menu_overlay::State::new(),
            mobile_input: MobileTextInputState::default(),
            filtered_options: Filtered::empty(),
            hovered_option: Some(0),
            new_selection: None,
            label: paragraph::Plain::default(),
        })
    }

    fn children(&self) -> Vec<widget::Tree> {
        vec![widget::Tree::new(&self.text_input as &dyn Widget<_, _, _>)]
    }

    fn diff(&self, _tree: &mut widget::Tree) {}

    fn update(
        &mut self,
        tree: &mut widget::Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let menu = tree
            .state
            .downcast_mut::<MenuState<T, Renderer::Paragraph>>();
        let activation = mobile_text_input_activation(
            true,
            &mut menu.mobile_input,
            event,
            layout.bounds().intersection(viewport),
            cursor,
        );

        let started_focused = {
            let text_input_state = tree.children[0]
                .state
                .downcast_ref::<text_input::State<Renderer::Paragraph>>();

            text_input_state.is_focused()
        };
        let mut published_message_to_shell = false;

        let mut local_messages = Vec::new();
        let mut local_shell = Shell::new(&mut local_messages);

        update_mobile_text_input(
            &mut self.text_input,
            &mut tree.children[0],
            event,
            layout,
            activation,
            renderer,
            clipboard,
            &mut local_shell,
            viewport,
        );

        if local_shell.is_event_captured() {
            shell.capture_event();
        }

        shell.request_redraw_at(local_shell.redraw_request());
        shell.request_input_method(local_shell.input_method());

        for message in local_messages {
            let TextInputEvent::TextChanged(new_value) = message;

            if let Some(on_input) = &self.on_input {
                shell.publish((on_input)(new_value.clone()));
            }

            self.state.with_inner_mut(|state| {
                menu.hovered_option = Some(0);
                state.value = new_value;

                state.filtered_options.update(
                    search(&self.state.options, &state.option_matchers, &state.value)
                        .cloned()
                        .collect(),
                );
            });
            shell.invalidate_layout();
            shell.request_redraw();
        }

        let is_focused = {
            let text_input_state = tree.children[0]
                .state
                .downcast_ref::<text_input::State<Renderer::Paragraph>>();

            text_input_state.is_focused()
        };

        if is_focused {
            self.state.with_inner(|state| {
                if !started_focused && let Some(on_option_hovered) = &mut self.on_option_hovered {
                    let hovered_option = menu.hovered_option.unwrap_or(0);

                    if let Some(option) = state.filtered_options.options.get(hovered_option) {
                        shell.publish(on_option_hovered(option.clone()));
                        published_message_to_shell = true;
                    }
                }

                if let Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(named_key),
                    modifiers,
                    ..
                }) = event
                {
                    match (named_key, modifiers.shift()) {
                        (key::Named::Enter, _) => {
                            if let Some(index) = &menu.hovered_option
                                && let Some(option) = state.filtered_options.options.get(*index)
                            {
                                menu.new_selection = Some(option.clone());
                            }

                            shell.capture_event();
                            shell.request_redraw();
                        }
                        (key::Named::ArrowUp, _) | (key::Named::Tab, true) => {
                            if let Some(index) = &mut menu.hovered_option {
                                if *index == 0 {
                                    *index = state.filtered_options.options.len().saturating_sub(1);
                                } else {
                                    *index = index.saturating_sub(1);
                                }
                            } else {
                                menu.hovered_option = Some(0);
                            }

                            if let Some(on_option_hovered) = &mut self.on_option_hovered
                                && let Some(option) = menu
                                    .hovered_option
                                    .and_then(|index| state.filtered_options.options.get(index))
                            {
                                shell.publish((on_option_hovered)(option.clone()));
                                published_message_to_shell = true;
                            }

                            shell.capture_event();
                            shell.request_redraw();
                        }
                        (key::Named::ArrowDown, _) | (key::Named::Tab, false)
                            if !modifiers.shift() =>
                        {
                            if let Some(index) = &mut menu.hovered_option {
                                if *index >= state.filtered_options.options.len().saturating_sub(1)
                                {
                                    *index = 0;
                                } else {
                                    *index = index.saturating_add(1).min(
                                        state.filtered_options.options.len().saturating_sub(1),
                                    );
                                }
                            } else {
                                menu.hovered_option = Some(0);
                            }

                            if let Some(on_option_hovered) = &mut self.on_option_hovered
                                && let Some(option) = menu
                                    .hovered_option
                                    .and_then(|index| state.filtered_options.options.get(index))
                            {
                                shell.publish((on_option_hovered)(option.clone()));
                                published_message_to_shell = true;
                            }

                            shell.capture_event();
                            shell.request_redraw();
                        }
                        _ => {}
                    }
                }
            });
        }

        self.state.with_inner_mut(|state| {
            if let Some(selection) = menu.new_selection.take() {
                state.value = String::new();
                state.filtered_options.update(self.state.options.clone());
                menu.menu = menu_overlay::State::default();

                shell.publish((self.on_selected)(selection));
                published_message_to_shell = true;

                let mut local_messages = Vec::new();
                let mut local_shell = Shell::new(&mut local_messages);
                self.text_input.update(
                    &mut tree.children[0],
                    &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
                    layout,
                    mouse::Cursor::Unavailable,
                    renderer,
                    clipboard,
                    &mut local_shell,
                    viewport,
                );
                shell.request_input_method(local_shell.input_method());
            }
        });

        let is_focused = {
            let text_input_state = tree.children[0]
                .state
                .downcast_ref::<text_input::State<Renderer::Paragraph>>();

            text_input_state.is_focused()
        };

        sync_mobile_keyboard(
            started_focused,
            is_focused,
            activation.request_mobile_keyboard,
        );

        if started_focused != is_focused {
            shell.invalidate_widgets();

            if is_focused {
                self.state.with_inner(|state| {
                    menu.menu
                        .start_open(state.filtered_options.options.len(), Instant::now());
                });
            }

            if !published_message_to_shell {
                if is_focused {
                    if let Some(on_open) = self.on_open.take() {
                        shell.publish(on_open);
                    }
                } else if let Some(on_close) = self.on_close.take() {
                    shell.publish(on_close);
                }
            }
        }
    }

    fn mouse_interaction(
        &self,
        tree: &widget::Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.text_input
            .mouse_interaction(&tree.children[0], layout, cursor, viewport, renderer)
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        register_mobile_text_region(true, layout.bounds(), viewport);

        let is_focused = {
            let text_input_state = tree.children[0]
                .state
                .downcast_ref::<text_input::State<Renderer::Paragraph>>();

            text_input_state.is_focused()
        };

        let selection = if is_focused || self.selection.is_empty() {
            None
        } else {
            Some(&self.selection)
        };

        self.text_input.draw(
            &tree.children[0],
            renderer,
            theme,
            layout,
            cursor,
            selection,
            viewport,
        );

        if let Some(label) = &self.label {
            let state = tree
                .state
                .downcast_ref::<MenuState<T, Renderer::Paragraph>>();
            let bounds = layout.bounds();
            let is_hovered = cursor.is_over(bounds);
            let label_width = state.label.min_width();
            let label_size = Pixels(tokens::component::text_field::LABEL_TEXT_POPULATED_SIZE);
            let label_line_height = LineHeight::Absolute(Pixels(
                tokens::component::text_field::LABEL_TEXT_POPULATED_LINE_HEIGHT,
            ));

            draw_combo_label_notch(
                renderer,
                bounds,
                label_width,
                combo_label_notch_height(is_focused),
                theme.colors().surface.container.high,
            );

            renderer.fill_text(
                core_text::Text {
                    content: label.clone(),
                    size: label_size,
                    line_height: label_line_height,
                    font: self.font.unwrap_or_else(|| renderer.default_font()),
                    bounds: Size::new(
                        label_width,
                        f32::from(label_line_height.to_absolute(label_size)),
                    ),
                    align_x: text::Alignment::Default,
                    align_y: iced_widget::core::alignment::Vertical::Center,
                    shaping: self.text_shaping,
                    wrapping: text::Wrapping::None,
                },
                Point::new(
                    bounds.x + tokens::component::text_field::LEADING_SPACE,
                    bounds.y,
                ),
                combo_label_color(theme, is_focused, is_hovered),
                *viewport,
            );
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut widget::Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let is_focused = {
            let text_input_state = tree.children[0]
                .state
                .downcast_ref::<text_input::State<Renderer::Paragraph>>();

            text_input_state.is_focused()
        };

        if is_focused {
            let MenuState {
                menu,
                filtered_options,
                hovered_option,
                ..
            } = tree
                .state
                .downcast_mut::<MenuState<T, Renderer::Paragraph>>();

            self.state.sync_filtered_options(filtered_options);

            if filtered_options.options.is_empty() {
                None
            } else {
                let bounds = layout.bounds();

                let mut menu = menu_overlay::Menu::new(
                    menu,
                    &filtered_options.options,
                    hovered_option,
                    |selection| {
                        self.state.with_inner_mut(|state| {
                            state.value = String::new();
                            state.filtered_options.update(self.state.options.clone());
                        });

                        tree.children[0]
                            .state
                            .downcast_mut::<text_input::State<Renderer::Paragraph>>()
                            .unfocus();

                        (self.on_selected)(selection)
                    },
                    self.on_option_hovered.as_deref(),
                    &self.menu_class,
                )
                .width(bounds.width)
                .padding(self.option_padding)
                .text_line_height(self.line_height)
                .text_shaping(self.text_shaping);

                if let Some(font) = self.font {
                    menu = menu.font(font);
                }

                if let Some(size) = self.size {
                    menu = menu.text_size(size);
                }

                let anchor = select::prefer_down_when_menu_fits(
                    layout.position() + translation,
                    *viewport,
                    bounds.height,
                    select::resolved_menu_height(
                        self.menu_height,
                        self.intrinsic_menu_height(filtered_options.options.len(), renderer),
                        viewport.height,
                    ),
                );

                Some(menu.overlay(
                    anchor.position,
                    *viewport,
                    anchor.target_height,
                    self.menu_height,
                ))
            }
        } else {
            None
        }
    }
}

impl<'a, T, Message, Renderer> From<MaterialComboBox<'a, T, Message, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    T: Display + Clone + 'static,
    Message: Clone + 'a,
    Renderer: core_text::Renderer + 'a,
{
    fn from(combo_box: MaterialComboBox<'a, T, Message, Renderer>) -> Self {
        Self::new(combo_box)
    }
}

fn draw_combo_label_notch<Renderer>(
    renderer: &mut Renderer,
    bounds: Rectangle,
    label_width: f32,
    notch_height: f32,
    notch_background: Color,
) where
    Renderer: iced_widget::core::Renderer,
{
    if label_width <= 0.0 || notch_height <= 0.0 {
        return;
    }

    let notch_width = label_width + tokens::component::text_field::OUTLINE_LABEL_PADDING * 2.0;
    let notch_x = bounds.x + tokens::component::text_field::LEADING_SPACE
        - tokens::component::text_field::OUTLINE_LABEL_PADDING;

    renderer.fill_quad(
        renderer::Quad {
            bounds: Rectangle {
                x: notch_x,
                y: bounds.y,
                width: notch_width.min((bounds.x + bounds.width - notch_x).max(0.0)),
                height: notch_height,
            },
            ..renderer::Quad::default()
        },
        notch_background,
    );
}

fn combo_label_notch_height(is_focused: bool) -> f32 {
    let outline_width = if is_focused {
        tokens::component::text_field::FOCUS_OUTLINE_WIDTH
    } else {
        tokens::component::text_field::OUTLINE_WIDTH
    };

    outline_width.ceil() + 1.0
}

fn combo_label_color(theme: &Theme, is_focused: bool, is_hovered: bool) -> Color {
    let colors = theme.colors();

    if is_focused {
        colors.primary.color
    } else if is_hovered {
        colors.surface.text
    } else {
        colors.surface.text_variant
    }
}

fn search<'a, T, A>(
    options: impl IntoIterator<Item = T> + 'a,
    option_matchers: impl IntoIterator<Item = &'a A> + 'a,
    query: &'a str,
) -> impl Iterator<Item = T> + 'a
where
    A: AsRef<str> + 'a,
{
    let query: Vec<String> = query
        .to_lowercase()
        .split(|c: char| !c.is_ascii_alphanumeric())
        .map(String::from)
        .collect();

    options
        .into_iter()
        .zip(option_matchers)
        .filter_map(move |(option, matcher)| {
            if query.iter().all(|part| matcher.as_ref().contains(part)) {
                Some(option)
            } else {
                None
            }
        })
}

fn build_matchers<'a, T>(options: impl IntoIterator<Item = T> + 'a) -> Vec<String>
where
    T: Display + 'a,
{
    options.into_iter().map(build_matcher).collect()
}

fn build_matcher<T>(option: T) -> String
where
    T: Display,
{
    let mut matcher = option.to_string();
    matcher.retain(|c| c.is_ascii_alphanumeric());
    matcher.to_lowercase()
}

fn inner_options<T>(options: &[T]) -> Vec<DisplayValue<T>>
where
    T: Clone,
{
    options.iter().cloned().map(DisplayValue::Option).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_value_uses_typed_query_text() {
        assert_eq!(DisplayValue::<&str>::Input("xxx".into()).to_string(), "xxx");
    }

    #[test]
    fn state_preserves_original_options() {
        let mut state = State::new(vec!["Assist", "Suggestion"]);

        state.push("Filter");

        assert_eq!(state.options(), &["Assist", "Suggestion", "Filter"]);
    }

    #[test]
    fn combo_box_option_padding_produces_m3_menu_item_height() {
        let state = State::new(vec!["Assist", "Suggestion", "Filter"]);
        let combo: ComboBox<'_, _, (), iced_widget::Renderer> =
            outlined_with_input(&state, "Search", "Suggestion", None, |_| ());
        let padding = combo.inner.option_padding;

        assert_eq!(
            tokens::component::text_field::INPUT_TEXT_LINE_HEIGHT + padding.y(),
            tokens::component::select::MENU_LIST_ITEM_CONTAINER_HEIGHT
        );
    }
}
