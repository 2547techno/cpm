use clap::{arg, command, Command};

fn main() {
    let matches = command!()
        .subcommand(
            Command::new("get")
                .alias("install")
                .alias("i")
                .about("Install plugin")
                .arg(arg!(-r --repo "Install plugin from git repo")),
        )
        .subcommand(
            Command::new("remove")
                .alias("uninstall")
                .about("Uninstall plugin"),
        )
        .subcommand(Command::new("info").about("Get plugin info"))
        .version("v0.1a")
        .get_matches();

    if let Some(command) = matches.subcommand_name() {
        match command {
            "get" => {
                println!("get")
            }
            "remove" => {
                todo!("implement remove")
            }
            "info" => {
                todo!("implement remove")
            }
            _ => {
                println!("none")
            }
        }
    }
}
