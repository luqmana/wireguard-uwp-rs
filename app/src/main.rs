//! This crate contains the foreground portion of our UWP VPN plugin app.
//!
//! We use XAML programmatically to generate the UI.

#![windows_subsystem = "windows"]
#![allow(non_snake_case)] // Windows naming conventions

use windows::{
    self as Windows,
    core::*,
    ApplicationModel::Activation::LaunchActivatedEventArgs,
    Foundation::Uri,
    Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED},
    UI::Xaml::{Application, ApplicationInitializationCallback},
};

/// Encapsulates our app and overrides the relevant lifecycle management methods.
#[implement(
    extend Windows::UI::Xaml::Application,
    override OnLaunched
)]
struct App;

impl App {
    /// This method get invoked when the app is initially launched.
    fn OnLaunched(&self, _args: &Option<LaunchActivatedEventArgs>) -> Result<()> {
        use Windows::{
            UI::Xaml::Controls::{Grid, Page, TextBlock},
            UI::Xaml::Documents::{Hyperlink, LineBreak, Run},
            UI::Xaml::Media::SolidColorBrush,
            UI::Xaml::Thickness,
            UI::Xaml::Window,
        };

        // Create the initial UI
        let content = TextBlock::new()?;
        let inline_content = content.Inlines()?;

        inline_content.Append({
            let run = Run::new()?;
            run.SetFontSize(32.)?;
            let color = SolidColorBrush::new()?;
            color.SetColor(Windows::UI::Color {
                A: 0xFF,
                R: 0xFC,
                G: 51,
                B: 0x85,
            })?;
            run.SetForeground(color)?;
            run.SetText("WireGuard + UWP + Rust")?;
            run
        })?;
        inline_content.Append(LineBreak::new()?)?;
        inline_content.Append(LineBreak::new()?)?;
        inline_content.Append({
            let run = Run::new()?;
            run.SetText("No profiles found ")?;
            run
        })?;
        inline_content.Append({
            let add_link = Hyperlink::new()?;
            add_link.Inlines()?.Append({
                let run = Run::new()?;
                run.SetText("add one")?;
                run
            })?;
            add_link.SetNavigateUri(Uri::CreateUri("ms-settings:network-vpn")?)?;
            add_link
        })?;
        inline_content.Append({
            let run = Run::new()?;
            run.SetText("!")?;
            run
        })?;

        let root = Page::new()?;
        root.SetContent({
            let grid = Grid::new()?;
            grid.SetPadding(Thickness {
                Left: 40.,
                Top: 40.,
                Right: 40.,
                Bottom: 40.,
            })?;
            grid.Children()?.Append(content)?;
            grid
        })?;

        // Grab the ambient Window created for our UWP app and set the content
        let window = Window::Current()?;
        window.SetContent(root)?;
        window.Activate()
    }
}

fn main() -> Result<()> {
    // We must initialize a COM MTA before initializing the rest of the App
    unsafe {
        CoInitializeEx(std::ptr::null_mut(), COINIT_MULTITHREADED)?;
    }

    // Go ahead with the XAML application initialization.
    // `Windows::UI::Xaml::Application` (which `App` derives from) is responsible for setting up
    // the CoreWindow and Dispatcher for us before calling our overridden OnLaunched/OnActivated.
    Application::Start(ApplicationInitializationCallback::new(|_| {
        App.new().map(|_| ())
    }))
}
