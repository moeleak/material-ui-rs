# Show a Library of Covers

A library of games, albums or books is a grid of covers, each with the actions
for its item. Three modules make one: `widget::image` loads the covers without
stalling frames, `widget::grid` lays them out for the window's width, and
`widget::menu` gives each cover its context menu.

The showcase's Media page is a complete example.

## Keep a Cache of Covers

Decode covers in the background, down to the size they are shown at, and keep
the most recently shown ones:

```rust
use material::widget::image;

struct Library {
    games: Vec<Game>,
    covers: image::Cache<GameId>,
}

let covers = image::Cache::new(256).max_dimension(512);
```

## Load Covers as They Come Into View

Wrap each cover in an `iced::widget::sensor`, request its image when it shows,
and keep the result:

```rust
#[derive(Debug, Clone)]
enum Message {
    CoverShown(GameId),
    CoverLoaded(image::Loaded<GameId>),
    Play(GameId),
    Remove(GameId),
}

fn update(library: &mut Library, message: Message) -> Task<Message> {
    match message {
        Message::CoverShown(id) => {
            let source = image::Source::Path(library.cover_path(id));
            library.covers.request(id, source, Message::CoverLoaded)
        }
        Message::CoverLoaded(loaded) => {
            library.covers.insert(loaded);
            Task::none()
        }
        // ...
    }
}
```

`request` does nothing for covers that are loading or loaded already, and
`insert` drops results for covers removed meanwhile. Call `remove` when a cover
file changes.

## Lay the Covers Out

`grid::grid` fits as many columns as cells of at most the given width allow.
`grid::aspect_ratio` keeps the cover at the art's aspect ratio above its title:

```rust
use iced::widget::{column, sensor};
use material::widget::{grid, image, menu};

fn view(library: &Library) -> material::Element<'_, Message> {
    let cells = library.games.iter().map(|game| {
        let cover = sensor(grid::aspect_ratio(
            image::cached(&library.covers, &game.id, material::tokens::shape::CORNER_MEDIUM),
            3.0 / 4.0,
        ))
        .on_show(move |_| Message::CoverShown(game.id));

        menu::context(column![cover, material::text::body_medium(&game.title)].spacing(8))
            .item(menu::item("Play").icon("play_arrow").on_press(Message::Play(game.id)))
            .divider()
            .item(
                menu::item("Remove from library")
                    .icon("delete")
                    .destructive()
                    .on_press(Message::Remove(game.id)),
            )
            .into()
    });

    iced::widget::scrollable(grid::grid(180.0, cells)).into()
}
```

`grid::columns_for` gives the number of columns the grid has at a width, for
moving a selection by rows with the keyboard.

## Context Menus

A context menu opens at the pointer when its content is right-clicked, or when
the menu key is pressed over it, towards the side of the window with room for
it. A click outside, <kbd>Esc</kbd> or losing focus closes it; the arrow keys
and <kbd>Enter</kbd> pick an action. Actions without a message are shown
disabled. Use `ContextMenu::on_open` to select the item whose menu opened.
