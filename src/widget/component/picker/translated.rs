fn translated<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
    translation: Vector,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: iced_widget::core::Renderer + 'a,
{
    Element::new(Translated {
        content: content.into(),
        translation,
    })
}

struct Translated<'a, Message, Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
    translation: Vector,
}

impl<Message, Renderer> fmt::Debug for Translated<'_, Message, Renderer> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Translated")
            .field("translation", &self.translation)
            .finish_non_exhaustive()
    }
}

impl<Message, Renderer> Widget<Message, Theme, Renderer> for Translated<'_, Message, Renderer>
where
    Renderer: iced_widget::core::Renderer,
{
    fn tag(&self) -> tree::Tag {
        self.content.as_widget().tag()
    }

    fn state(&self) -> tree::State {
        self.content.as_widget().state()
    }

    fn children(&self) -> Vec<Tree> {
        self.content.as_widget().children()
    }

    fn diff(&self, tree: &mut Tree) {
        self.content.as_widget().diff(tree);
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.content.as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content.as_widget_mut().layout(tree, renderer, limits)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(tree, layout, renderer, operation);
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
        let translation = self.translation;

        self.content.as_widget_mut().update(
            tree,
            event,
            layout,
            cursor - translation,
            renderer,
            clipboard,
            shell,
            &(*viewport - translation),
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
        let translation = self.translation;

        self.content.as_widget().mouse_interaction(
            tree,
            layout,
            cursor - translation,
            &(*viewport - translation),
            renderer,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let Some(viewport) = layout.bounds().intersection(viewport) else {
            return;
        };
        let translation = self.translation;

        renderer.with_layer(viewport, |renderer| {
            renderer.with_translation(translation, |renderer| {
                self.content.as_widget().draw(
                    tree,
                    renderer,
                    theme,
                    style,
                    layout,
                    cursor - translation,
                    &(viewport - translation),
                );
            });
        });
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(
            tree,
            layout,
            renderer,
            viewport,
            translation + self.translation,
        )
    }
}
