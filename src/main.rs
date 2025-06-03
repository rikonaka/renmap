use iced::Background;
use iced::Border;
use iced::Color;
use iced::Length;
use iced::Size;
use iced::Theme;
use iced::alignment::Horizontal;
use iced::alignment::Vertical;
use iced::widget::Button;
use iced::widget::Column;
use iced::widget::Container;
use iced::widget::Row;
use iced::widget::Text;
use iced::widget::TextInput;
use iced::widget::container;
use iced::window;
use iced::overlay::menu;

pub fn style_box_1(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    container::Style {
        background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.0))),
        border: Border {
            width: 1.0,
            radius: 0.0.into(),
            color: palette.background.strong.color,
        },
        ..container::Style::default()
    }
}

pub fn main() -> iced::Result {
    let mut window = window::Settings::default();
    let icon = window::icon::from_file("./assets/corgi.png").expect("can not load icon");
    window.icon = Some(icon);
    window.size = Size::new(800.0, 600.0);
    iced::application(App::title, App::update, App::view)
        .window(window)
        .run()
}

#[derive(Default)]
struct App {
    scan_info: String,
    target_ip: String,
    target_port: String,
}

#[derive(Debug, Clone)]
enum Message {
    StartScan,
    UpdateIP(String),
    UpdatePort(String),
}

impl App {
    fn title(&self) -> String {
        format!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
    }
    fn update(&mut self, message: Message) {
        match message {
            Message::StartScan => {
                self.scan_info =
                    format!("Starting scan on {}:{}", self.target_ip, self.target_port);
            }
            Message::UpdateIP(ip) => {
                self.target_ip = ip;
            }
            Message::UpdatePort(port) => {
                self.target_port = port;
            }
        }
    }
    fn view(&self) -> Column<Message> {
        let menu = menu::Menu::new()
            .add_item("File")
            .add_item("Edit")
            .add_item("View")
            .add_item("Help");

        let input_ip_text = Text::new("Target")
            .line_height(1.0)
            .align_x(Horizontal::Center);

        let input_ip = TextInput::new("127.0.0.1", &self.target_ip)
            .on_input(Message::UpdateIP)
            .padding(10)
            .line_height(1.0)
            .width(Length::Fill)
            .align_x(Horizontal::Left);

        let input_port_text = Text::new("Port")
            .line_height(1.0)
            .align_x(Horizontal::Center);

        let input_port = TextInput::new("80", &self.target_port)
            .on_input(Message::UpdatePort)
            .padding(10)
            .line_height(1.0)
            .width(Length::Fill)
            .align_x(Horizontal::Left);

        let start_button_text = Text::new("Start")
            .line_height(1.0)
            .width(Length::Fill)
            .align_x(Horizontal::Center);
        let start_button = Button::new(start_button_text)
            .width(Length::Fill)
            .on_press(Message::StartScan);

        let row_1 = Row::new()
            .padding(20)
            .spacing(20)
            .align_y(Vertical::Center)
            .push(input_ip_text)
            .push(input_ip)
            .push(input_port_text)
            .push(input_port)
            .push(start_button);

        let group_1 = Container::new(row_1)
            .width(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .style(style_box_1);

        let target_check_line = Text::new(&self.scan_info)
            .width(Length::Fill)
            .align_x(Horizontal::Left);

        Column::new()
            .padding(20)
            .spacing(20)
            .align_x(Horizontal::Center)
            .push(group_1)
            .push(target_check_line)
            .into()
    }
}
