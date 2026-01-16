# DeepFilterNet GUI

A simple, user-friendly GUI wrapper for [DeepFilterNet](https://github.com/Ruliex/DeepFilterNet), written in Rust using the [Iced](https://github.com/iced-rs/iced) framework. This application allows you to easily clean background noise from your `.wav` audio files.

## Features

- **Drag & Drop**: Simply drag your `.wav` files into the window to select them.
- **Automatic Engine Setup**: The app **automatically downloads** the required `deep-filter` engine for you. No manual installation of DeepFilterNet is required.
- **Real-time Progress**: Visual feedback during the one-time download and file processing.
- **Cross-Platform**: Designed for Linux, Windows, and macOS.
- **One-Click Cleaning**: Processes audio using DeepFilterNet's advanced noise suppression model.

## User Guide (For Running the App)

If you have downloaded the pre-built `dfn_gui` executable:

1.  **System Requirements**:
    - **Linux**: A standard desktop environment (requires generic graphics libraries like standard `libxcb`/`wayland` libraries usually present on most systems).
    - **Windows/macOS**: No special dependencies.
2.  **Setup**: Just run the `dfn_gui` file.
    - On the **first run only**, click the "Download Engine" button. The app handles everything else.
3.  **Usage**:
    - Select or Drag & Drop a `.wav` file.
    - Click "Start Processing".
    - Uses "Open File Location" to view your cleaned audio.

## Developer Guide (Building from Source)

If you want to modify the code or build it yourself, you will need the following:

### Build Prerequisites

- **Rust Toolchain**: Install via [rustup.rs](https://rustup.rs/).
- **Linux Build Dependencies**:
  - `pkg-config`, `libssl-dev` (for `reqwest` secure downloads).
  - `libasound2-dev` (standard audio).
  - `libfreetype6-dev`, `libexpat1-dev`, `libxcb-composite0-dev`, `libfontconfig1-dev` (Iced GUI dependencies).

### Building

1.  **Clone the repository**:

    ```bash
    git clone <repository-url>
    cd dfn_gui
    ```

2.  **Run in Development Mode**:

    ```bash
    cargo run
    ```

3.  **Build Release Binary**:
    For a fast, optimized, and smaller executable:
    ```bash
    cargo build --release
    ```
    The binary will be located at `target/release/dfn_gui` (or `.exe`). This single file is all you need to distribute.

## Usage

4. **Process**: Click "Start Processing".
5. **Open Result**: Once finished, click "Open File Location" to find your cleaned audio file (usually in a `dnf_clean` subdirectory).

## Troubleshooting

- **Missing Binary**: If the download fails, check your internet connection.
- **Build Errors**: Ensure you have the necessary system libraries installed (especially `openssl` on Linux).

## License

This project (the **DeepFilterNet GUI Wrapper**) is licensed under the **MIT License**. See the [LICENSE](LICENSE) file for details.

### DeepFilterNet License

This application acts as a frontend for the [DeepFilterNet](https://github.com/Ruliex/DeepFilterNet) engine. The `deep-filter` binary and models downloaded and used by this application are property of their respective authors and are governed by their own licenses (DeepFilterNet is currently dual-licensed under MIT/Apache 2.0).

By using this software, you acknowledge that you are using the DeepFilterNet engine subject to its license terms.
