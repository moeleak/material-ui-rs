use iced::widget::column;
use iced::{Size, Subscription};
#[cfg(target_os = "android")]
use iced::Padding;
use material_ui_rs as material;

pub fn run() -> iced::Result {
    material::application(boot, update, view)
        .title({{label_rust}})
        .theme(theme)
        .subscription(subscription)
        .window(material::window_with_min_size(
            Size::new(960.0, 720.0),
            Size::new(360.0, 640.0),
        ))
        .run()
}

#[derive(Default)]
struct App {
    count: i32,
    #[cfg(target_os = "android")]
    safe_area: material::android::SafeAreaInsets,
}

#[derive(Debug, Clone)]
enum Message {
    Increment,
    Decrement,
    #[cfg(target_os = "android")]
    Android(material::android::Event),
}

fn boot() -> App {
    App {
        #[cfg(target_os = "android")]
        safe_area: material::android::safe_area_insets(),
        ..App::default()
    }
}

fn update(app: &mut App, message: Message) {
    match message {
        Message::Increment => app.count += 1,
        Message::Decrement => app.count -= 1,
        #[cfg(target_os = "android")]
        Message::Android(material::android::Event::InsetsChanged(safe_area)) => {
            app.safe_area = safe_area;
        }
    }
}

fn subscription(_app: &App) -> Subscription<Message> {
    #[cfg(target_os = "android")]
    {
        material::android::events().map(Message::Android)
    }
    #[cfg(not(target_os = "android"))]
    {
        Subscription::none()
    }
}

fn theme(_app: &App) -> material::Theme {
    material::Theme::Dark
}

fn view(app: &App) -> material::Element<'_, Message> {
    let page = material::widget::page::surface(
        material::widget::page::header(
            {{label_rust}},
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
    );

    #[cfg(target_os = "android")]
    {
        let safe_area = app.safe_area.content();
        iced::widget::container(page)
            .padding(Padding {
                top: safe_area.top,
                right: safe_area.right,
                bottom: safe_area.bottom,
                left: safe_area.left,
            })
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .into()
    }
    #[cfg(not(target_os = "android"))]
    {
        page.into()
    }
}
