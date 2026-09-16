#[cfg(any(target_os = "macos", test))]
mod app;
#[cfg(any(target_os = "macos", test))]
mod applications;
#[cfg(any(target_os = "macos", test))]
mod cli;
#[cfg(any(target_os = "macos", test))]
mod error;
#[cfg(any(target_os = "macos", test))]
mod native;
#[cfg(any(target_os = "macos", test))]
mod output;
mod platform;
#[cfg(any(target_os = "macos", test))]
mod process;
#[cfg(any(target_os = "macos", test))]
mod secure_fs;
#[cfg(any(target_os = "macos", test))]
mod settings;

fn main() {
    if let Err(message) = platform::check() {
        eprintln!("error: {message}");
        std::process::exit(78);
    }
    #[cfg(target_os = "macos")]
    if unsafe { libc::geteuid() } == 0 {
        eprintln!("error: Run mac without sudo. Administrator operations request authentication when needed.");
        std::process::exit(77);
    }
    #[cfg(target_os = "macos")]
    std::process::exit(app::run());
}
