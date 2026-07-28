use iced::widget::column;
use iced::{Size, Theme};
use material_ui_rs as material;

pub fn run() -> iced::Result {
    material::application(boot, update, view)
        .title("{{label}}")
        .theme(|_| Theme::Dark)
        .window(material::window_with_min_size(
            Size::new(960.0, 720.0),
            Size::new(360.0, 640.0),
        ))
        .run()
}

#[derive(Default)]
struct App {
    count: i32,
}

#[derive(Debug, Clone)]
enum Message {
    Increment,
    Decrement,
}

fn boot() -> App {
    App::default()
}

fn update(app: &mut App, message: Message) {
    match message {
        Message::Increment => app.count += 1,
        Message::Decrement => app.count -= 1,
    }
}

fn view(app: &App) -> material::Element<'_, Message> {
    material::widget::page::surface(
        material::widget::page::header(
            "{{label}}",
            "A multi-platform Material app",
        ),
        column![
            material::text::headline_medium(app.count.to_string()),
            material::widget::button::button(
                "Increment",
                material::widget::button::ButtonVariant::Filled,
            )
            .on_press(Message::Increment),
            material::widget::button::button(
                "Decrement",
                material::widget::button::ButtonVariant::Outlined,
            )
            .on_press(Message::Decrement),
        ]
        .spacing(12),
    )
    .into()
}
