fn main() {
    let dirs = directories::ProjectDirs::from("", "", "quokka")
        .expect("could not determine the home directory");
    let paths = quokka_ui::Paths {
        db: dirs.data_dir().join("library.db"),
        settings: dirs.config_dir().join("settings.json"),
        session: dirs.data_dir().join("session.json"),
    };
    if let Err(e) = quokka_ui::run(paths) {
        eprintln!("player: {e}");
        std::process::exit(1);
    }
}
