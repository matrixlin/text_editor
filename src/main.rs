use iced::{Length, Sandbox, Settings};
use iced::widget::{column, container, horizontal_space, row, text, text_editor};
use iced::Theme;
fn main() -> iced::Result {
    Editor::run(Settings::default())
}

struct Editor {
    content: text_editor::Content, 
}

#[derive(Debug, Clone)]
enum Message {
    Edit(text_editor::Action)
}


impl Sandbox for Editor {
    type Message = Message;

    fn new() -> Self {
        Self {
            content: text_editor::Content::new(),
        }
    }

    fn title(&self) -> String {
        String::from("Text editor")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Edit(action) => self.content.perform(action),
        }
    }

    fn view(&self) -> iced::Element<'_, Message> {
        let input_content = text_editor(&self.content).on_action(Message::Edit).height(Length::Fill);
        let position = {
            let (line, column) = &self.content.cursor_position();
            text(format!("{} : {}", line + 1, column + 1))
        };
        let status_bar = row!(horizontal_space(), position);
        container(column!(input_content, status_bar)).padding(10).into()

        // let text1 = text("text");
        // let text2 = text("text");
        // let text3 = text("text");
        // container(row!(text1,horizontal_space(),text2,horizontal_space(),text3)).into()

    }

    fn theme(&self) -> iced::Theme {
        Theme::Dark
    }
}
