use iced::executor;
use iced::widget::{button, column, container};
use iced::{Alignment, Application, Command, Element, Length, Settings, Theme};

#[derive(Default)]
struct Filepack {}

#[derive(Debug, Clone, Copy)]
enum Message {}

impl Application for Filepack {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (Self::default(), Command::none())
    }

    fn title(&self) -> String {
        String::from("Filepack")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {}
    }

    fn view(&self) -> Element<Message> {
        let content = column!["Click the button to exit", button("Exit").padding([10, 20]),]
            .spacing(10)
            .align_items(Alignment::Center);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .center_x()
            .center_y()
            .into()
    }
}

pub fn main() -> iced::Result {
    Filepack::run(Settings::default())
}
