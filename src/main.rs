use iced::widget::tooltip::Position;
use iced::{application,Length, Task, Theme, Element, highlighter, Font};
use iced::widget::{button, column, container, horizontal_space, pick_list, row, text, text_editor, tooltip};
use std::ffi;
use std::path::{Path, PathBuf};
use std::io::ErrorKind;
use std::sync::Arc;
fn main() -> iced::Result {
    application(Editor::title, Editor::update, Editor::view)
    .theme(Editor::theme)
    .font(include_bytes!("../icon_fonts/text_editor_icon.ttf").as_slice())
    .run_with(Editor::new)
}

struct Editor {
    content: text_editor::Content,
    error: Option<Error>,
    path: Option<PathBuf>,
    theme: highlighter::Theme,
}

#[derive(Debug, Clone)]
enum Message {
    Edit(text_editor::Action),
    FileOpened(Result<(PathBuf, Arc<String>), Error>),
    Open,
    New,
    Save,
    FileSaved(Result<PathBuf, Error>),
    ThemeSelected(highlighter::Theme),
}

fn icon<'a, Message>(unicode_point: char) -> Element<'a, Message> {
    const ICON_FONT: Font = Font::with_name("text_editor_icon");
    text(unicode_point).font(ICON_FONT).into()
}

fn new_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{F04A}')
}

fn save_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{F067}')
}

fn open_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{F068}')
}

fn button_tooltip<'a>(content: Element<'a, Message>, label: &'a str, on_press: Message) -> Element<'a, Message> {
    tooltip(
        button(container(content).center_x(20))
            .padding([5, 6])
            .on_press(on_press), 
        label, 
        Position::FollowCursor
    )
    .style(container::rounded_box)
    .into()
}

impl Editor {

    fn new() -> (Self, Task<Message>) {
        (
            Self {
                // content: text_editor::Content::with_text(include_str!("./main.rs")),
                content: text_editor::Content::new(),
                error: None,
                path: None,
                theme: highlighter::Theme::SolarizedDark,
            },
        Task::perform(load_file(default_load_file()), Message::FileOpened)
        )
    }

    fn title(&self) -> String {
        String::from("Text editor")
    }

    fn update(&mut self, message: Message) ->Task<Message> {
        match message {
            Message::Edit(action) => {
                self.content.perform(action);
                Task::none()
            }
            Message::FileOpened(Ok((path, contents))) => {
                self.content = text_editor::Content::with_text(&contents);
                self.path = Some(path);
                Task::none()
            }
            Message::FileOpened(Err(error)) => {
                self.error = Some(error);
                Task::none()
            }

            Message::Open => {
                Task::perform(pick_file(), Message::FileOpened) 
            }

            Message::New => {
                self.content = text_editor::Content::new();
                self.path = None;
                Task::none()
            }

            Message::Save => {
                let contents = self.content.text();
                Task::perform(save_file(self.path.clone(), contents), Message::FileSaved)
            }

            Message::FileSaved(Ok(path)) => {
                self.path = Some(path);
                Task::none()
            }

            Message::FileSaved(Err(error)) => {
                self.error = Some(error);
                Task::none()
            }
            Message::ThemeSelected(theme) => {
                self.theme = theme;
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let controls = row![
            button_tooltip(open_icon(), "Open File", Message::Open),
            button_tooltip(new_icon(), "New File", Message::New),
            button_tooltip(save_icon(), " Save File", Message::Save),
            horizontal_space(),
            pick_list(highlighter::Theme::ALL, Some(self.theme), Message::ThemeSelected),
        ].spacing(10);
        let input_content = text_editor(&self.content)
            .on_action(Message::Edit)
            .height(Length::Fill)
            .highlight(
                self.path.as_deref()
                .and_then(Path::extension)
                .and_then(ffi::OsStr::to_str)
                .unwrap_or("rs"),
                self.theme
            );
        let position = {
            let (line, column) = &self.content.cursor_position();
            text(format!("{} : {}", line + 1, column + 1))
        };
        let file_path = if let Some(Error::IOFailed(error)) = self.error.as_ref() {
            text(error.to_string())
        } else {
            match self.path.as_deref().and_then(Path::to_str) {
                Some(path) => text(path).size(15),
                None => text("New File"),
            }
        };
        let status_bar = row!(file_path, horizontal_space(), position);
        container(column![controls, input_content, status_bar]).padding(10).into()

        // let text1 = text("text");
        // let text2 = text("text");
        // let text3 = text("text");
        // container(row!(text1,horizontal_space(),text2,horizontal_space(),text3)).into()

    }

    fn theme(&self) -> iced::Theme {
        if self.theme.is_dark() { 
            Theme::Dark
        } else {
            Theme::Light
        }
    }
}

#[derive(Debug, Clone)]
enum Error {
    IOFailed(ErrorKind),
    DialogClosed,
}
// &str String Pathbuf ...
async fn load_file(path: impl AsRef<Path>) -> Result<(PathBuf, Arc<String>), Error> {
     let contents = tokio::fs::read_to_string(path.as_ref()).await
         .map(Arc::new)
         .map_err(|error| Error::IOFailed(error.kind()))?;
    Ok((path.as_ref().to_path_buf(), contents))
}

fn default_load_file() -> PathBuf {
    PathBuf::from(format!("{}/src/main.rs", env!("CARGO_MANIFEST_DIR")))
}

// async fn pick_file() ->Result<(PathBuf, Arc<String>), Error> {
//     let filehandle = rfd::AsyncFileDialog::new().set_title("Choose a file")
//         .pick_file()
//         .await
//         .ok_or(Error::DialogClosed)?;

//     load_file(filehandle.path().to_owned()).await
// }

async fn pick_file() ->Result<(PathBuf, Arc<String>), Error> {
    let file_path = rfd::AsyncFileDialog::new().set_title("Choose a file")
        .pick_file()
        .await
        .ok_or(Error::DialogClosed)
        .map(|filehandle|filehandle.path().to_owned())?;

    load_file(file_path).await
}


async fn save_file(path: Option<PathBuf>, contents: String) -> Result<PathBuf, Error> {
    let path = if let Some(path) = path {
        path 
    } else {
        rfd::AsyncFileDialog::new().set_title("Save a file.")
            .save_file().await
            .ok_or(Error::DialogClosed)
            .map(|filehandle| filehandle.path().to_owned())?
    };

    tokio::fs::write(&path, contents).await
                                     .map_err(|error| Error::IOFailed(error.kind()))?;
    Ok(path)
}