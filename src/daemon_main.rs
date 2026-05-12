use persona_system::SystemCommandLine;

fn main() {
    if let Err(error) = SystemCommandLine::from_environment().run() {
        eprintln!("persona-system-daemon: {error}");
        std::process::exit(1);
    }
}
