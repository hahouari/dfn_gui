use futures_util::{Stream, StreamExt};
use iced::widget::{button, column, container, progress_bar, text};
use iced::{Alignment, Element, Length, Task, Theme, window};
use rfd::FileDialog;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;

pub fn main() -> iced::Result {
    iced::application(DfnGui::init, DfnGui::update, DfnGui::view)
        .title(DfnGui::title)
        .subscription(DfnGui::subscription)
        .theme(DfnGui::theme)
        .window(window::Settings {
            size: (600.0, 450.0).into(),
            ..Default::default()
        })
        .run()
}

#[derive(Default)]
struct DfnGui {
    selected_file: Option<PathBuf>,
    status: Status,
    download_progress: f32,
}

#[derive(Debug, Clone, Default)]
enum Status {
    #[default]
    Checking,
    MissingBinary,
    Downloading,
    Idle,
    Ready,
    Processing,
    Done(PathBuf),
    Error(String),
}

#[derive(Debug, Clone)]
enum Message {
    BinaryCheckCompleted(Result<PathBuf, ()>),
    StartDownload,
    DownloadProgress(f32),
    DownloadFinished(Result<PathBuf, String>),
    SelectFile,
    FileSelected(Option<PathBuf>),
    EventOccurred(iced::Event),
    StartProcessing,
    ProcessingFinished(Result<PathBuf, String>),
    OpenLocation(PathBuf),
}

impl DfnGui {
    fn init() -> (Self, Task<Message>) {
        (
            Self::default(),
            Task::perform(
                async { check_binary_exists().ok().ok_or(()) },
                Message::BinaryCheckCompleted,
            ),
        )
    }

    fn title(&self) -> String {
        String::from("DeepFilterNet GUI")
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::BinaryCheckCompleted(Ok(_)) => {
                self.status = Status::Idle;
            }
            Message::BinaryCheckCompleted(Err(_)) => {
                self.status = Status::MissingBinary;
            }
            Message::StartDownload => {
                self.status = Status::Downloading;
                self.download_progress = 0.0;
            }
            Message::DownloadProgress(progress) => {
                self.download_progress = progress;
            }
            Message::DownloadFinished(Ok(_)) => {
                self.status = Status::Idle;
            }
            Message::DownloadFinished(Err(e)) => {
                self.status = Status::Error(format!("Download failed: {}", e));
            }
            Message::SelectFile => {
                return Task::perform(
                    async {
                        FileDialog::new()
                            .add_filter("WAV audio", &["wav"])
                            .pick_file()
                    },
                    Message::FileSelected,
                );
            }
            Message::FileSelected(file) => {
                if let Some(path) = file {
                    self.selected_file = Some(path);
                    self.status = Status::Ready;
                }
            }
            Message::EventOccurred(event) => {
                // Prevent drag-and-drop if binary is missing
                if matches!(
                    self.status,
                    Status::Checking | Status::MissingBinary | Status::Downloading
                ) {
                    return Task::none();
                }

                if let iced::Event::Window(window::Event::FileDropped(path)) = event {
                    if path.extension().map(|s| s == "wav").unwrap_or(false) {
                        self.selected_file = Some(path);
                        self.status = Status::Ready;
                    } else {
                        self.status = Status::Error("Only .wav files are supported".to_string());
                    }
                }
            }
            Message::StartProcessing => {
                if let Some(input_path) = &self.selected_file {
                    if let Ok(bin_path) = check_binary_exists() {
                        self.status = Status::Processing;
                        let path = input_path.clone();
                        return Task::perform(
                            async move {
                                tokio::task::spawn_blocking(move || {
                                    run_deep_filter(&path, &bin_path)
                                })
                                .await
                                .unwrap_or_else(|e| Err(format!("Task join error: {}", e)))
                            },
                            Message::ProcessingFinished,
                        );
                    } else {
                        self.status = Status::Error("Binary missing during processing".to_string());
                    }
                }
            }
            Message::ProcessingFinished(result) => match result {
                Ok(path) => {
                    self.status = Status::Done(path);
                }
                Err(e) => self.status = Status::Error(e),
            },
            Message::OpenLocation(path) => {
                let folder = path.as_path();
                #[cfg(target_os = "linux")]
                let _ = std::process::Command::new("xdg-open").arg(folder).spawn();
                #[cfg(target_os = "windows")]
                let _ = std::process::Command::new("explorer").arg(folder).spawn();
                #[cfg(target_os = "macos")]
                let _ = std::process::Command::new("open").arg(folder).spawn();
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let content = column![
            text("DeepFilterNet Noise Cancellation").size(30),
            self.view_main_area(),
            self.view_status(),
        ]
        .spacing(20)
        .max_width(600)
        .align_x(Alignment::Center);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }

    fn view_main_area(&self) -> Element<'_, Message> {
        match self.status {
            Status::Checking => text("Checking resources...").into(),
            Status::MissingBinary => button("Download Engine (Required)")
                .on_press(Message::StartDownload)
                .padding(20)
                .into(),
            Status::Downloading => column![
                text(format!("Downloading... {:.0}%", self.download_progress)),
                progress_bar(0.0..=100.0, self.download_progress),
            ]
            .spacing(10)
            .align_x(Alignment::Center)
            .into(),
            _ => container(
                column![
                    text(match &self.selected_file {
                        Some(path) =>
                            format!("File: {}", path.file_name().unwrap().to_string_lossy()),
                        None => String::from("Drag and drop a .wav file here or click to select"),
                    }),
                    button("Select WAV File").on_press(Message::SelectFile),
                ]
                .spacing(10)
                .align_x(Alignment::Center),
            )
            .padding(40)
            .width(Length::Fill)
            .style(|_theme: &Theme| container::Style {
                border: iced::Border {
                    color: iced::Color::from_rgb(0.3, 0.3, 0.3),
                    width: 2.0,
                    radius: 10.0.into(),
                },
                ..Default::default()
            })
            .into(),
        }
    }

    fn view_status(&self) -> Element<'_, Message> {
        match &self.status {
            Status::Checking | Status::MissingBinary | Status::Downloading => text("").into(),
            Status::Idle => text("Ready.").into(),
            Status::Ready => button("Clean Audio")
                .on_press(Message::StartProcessing)
                .padding(10)
                .into(),
            Status::Processing => {
                column![text("Cleaning audio..."), progress_bar(0.0..=100.0, 50.0),]
                    .spacing(10)
                    .align_x(Alignment::Center)
                    .into()
            }
            Status::Done(path) => column![
                text("Finished!").color(iced::Color::from_rgb(0.0, 1.0, 0.0)),
                text(format!("Saved to: {}", path.display())).size(12),
                button("Open File Location")
                    .on_press(Message::OpenLocation(path.parent().unwrap().to_path_buf())),
            ]
            .spacing(10)
            .align_x(Alignment::Center)
            .into(),
            Status::Error(e) => column![
                text(format!("Error: {}", e)).color(iced::Color::from_rgb(1.0, 0.0, 0.0)),
                button("Retry").on_press(Message::SelectFile),
            ]
            .spacing(10)
            .align_x(Alignment::Center)
            .into(),
        }
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        let events = iced::event::listen().map(Message::EventOccurred);

        if let Status::Downloading = self.status {
            iced::Subscription::batch(vec![events, iced::Subscription::run(download_process)])
        } else {
            events
        }
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

fn check_binary_exists() -> Result<PathBuf, String> {
    let dirs = directories::ProjectDirs::from("com", "deepfilternet", "deepfilternet-gui")
        .ok_or("Could not find project directories")?;
    let data_dir = dirs.data_local_dir();

    #[cfg(windows)]
    let bin_name = "deep-filter.exe";
    #[cfg(not(windows))]
    let bin_name = "deep-filter";

    let bin_path = data_dir.join(bin_name);
    if bin_path.exists() {
        Ok(bin_path)
    } else {
        Err("Binary not found".to_string())
    }
}

fn download_process() -> impl Stream<Item = Message> {
    futures_util::stream::unfold(State::Start, |state| async move {
        match state {
            State::Start => {
                let dirs = match directories::ProjectDirs::from(
                    "com",
                    "deepfilternet",
                    "deepfilternet-gui",
                ) {
                    Some(d) => d,
                    None => {
                        return Some((
                            Message::DownloadFinished(Err("No download dir".into())),
                            State::Finished,
                        ));
                    }
                };
                let data_dir = dirs.data_local_dir();
                if let Err(e) = std::fs::create_dir_all(data_dir) {
                    return Some((
                        Message::DownloadFinished(Err(e.to_string())),
                        State::Finished,
                    ));
                }

                let (url, bin_name) = match get_binary_url_and_name() {
                    Ok(val) => val,
                    Err(e) => return Some((Message::DownloadFinished(Err(e)), State::Finished)),
                };

                let bin_path = data_dir.join(bin_name);

                match reqwest::get(url).await {
                    Ok(response) => {
                        let total_size = response.content_length().unwrap_or(0);
                        let stream = response.bytes_stream().boxed();
                        let file = match std::fs::File::create(&bin_path) {
                            Ok(f) => f,
                            Err(e) => {
                                return Some((
                                    Message::DownloadFinished(Err(e.to_string())),
                                    State::Finished,
                                ));
                            }
                        };
                        Some((
                            Message::DownloadProgress(0.0),
                            State::Downloading {
                                stream,
                                file,
                                total: total_size,
                                downloaded: 0,
                                path: bin_path,
                            },
                        ))
                    }
                    Err(e) => Some((
                        Message::DownloadFinished(Err(e.to_string())),
                        State::Finished,
                    )),
                }
            }
            State::Downloading {
                mut stream,
                mut file,
                total,
                mut downloaded,
                path,
            } => {
                match stream.next().await {
                    Some(Ok(chunk)) => {
                        if let Err(e) = file.write_all(&chunk) {
                            return Some((
                                Message::DownloadFinished(Err(e.to_string())),
                                State::Finished,
                            ));
                        }
                        downloaded += chunk.len() as u64;
                        let percentage = if total > 0 {
                            (downloaded as f32 / total as f32) * 100.0
                        } else {
                            0.0
                        };
                        Some((
                            Message::DownloadProgress(percentage),
                            State::Downloading {
                                stream,
                                file,
                                total,
                                downloaded,
                                path,
                            },
                        ))
                    }
                    Some(Err(e)) => Some((
                        Message::DownloadFinished(Err(e.to_string())),
                        State::Finished,
                    )),
                    None => {
                        // Done
                        #[cfg(unix)]
                        {
                            use std::os::unix::fs::PermissionsExt;
                            if let Ok(meta) = file.metadata() {
                                let mut perms = meta.permissions();
                                perms.set_mode(0o755);
                                let _ = file.set_permissions(perms);
                            }
                        }
                        Some((Message::DownloadFinished(Ok(path)), State::Finished))
                    }
                }
            }
            State::Finished => None,
        }
    })
}

enum State {
    Start,
    Downloading {
        stream: futures_util::stream::BoxStream<'static, reqwest::Result<bytes::Bytes>>,
        file: std::fs::File,
        total: u64,
        downloaded: u64,
        path: PathBuf,
    },
    Finished,
}

// Rewriting download_process to use BoxStream to handle the type
fn get_binary_url_and_name() -> Result<(&'static str, &'static str), String> {
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    return Ok((
        "https://github.com/Rikorose/DeepFilterNet/releases/download/v0.5.6/deep-filter-0.5.6-x86_64-unknown-linux-musl",
        "deep-filter",
    ));

    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    return Ok((
        "https://github.com/Rikorose/DeepFilterNet/releases/download/v0.5.6/deep-filter-0.5.6-aarch64-unknown-linux-gnu",
        "deep-filter",
    ));

    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    return Ok((
        "https://github.com/Rikorose/DeepFilterNet/releases/download/v0.5.6/deep-filter-0.5.6-aarch64-apple-darwin",
        "deep-filter",
    ));

    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    return Ok((
        "https://github.com/Rikorose/DeepFilterNet/releases/download/v0.5.6/deep-filter-0.5.6-x86_64-pc-windows-msvc.exe",
        "deep-filter.exe",
    ));

    #[allow(unreachable_code)]
    Err("Unsupported OS/Architecture".to_string())
}

fn run_deep_filter(input_path: &Path, bin_path: &Path) -> Result<PathBuf, String> {
    // Prepare output path
    let file_name = input_path
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();
    let input_dir = input_path.parent().unwrap();
    let output_dir = input_dir.join("dnf_clean");
    let output_path = output_dir.join(file_name);

    let status = StdCommand::new(bin_path)
        .arg(input_path)
        .arg("-o")
        .arg(output_dir.clone())
        .status()
        .map_err(|e| format!("Failed to run AI engine: {}", e))?;

    if status.success() {
        Ok(output_path)
    } else {
        Err("DeepFilterNet failed to process the file".to_string())
    }
}
