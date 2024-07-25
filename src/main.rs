use clap::{command, Arg, ArgAction, Command};

fn main() {
    let matches = command!()
        .subcommand(
            Command::new("get")
                .alias("install")
                .alias("i")
                .about("Install plugin")
                .arg(Arg::new("plugin").required(true))
                .arg(
                    Arg::new("repo")
                        .short('r')
                        .long("repo")
                        .action(ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("remove")
                .alias("uninstall")
                .about("Uninstall plugin"),
        )
        .subcommand(Command::new("info").about("Get plugin info"))
        .version("v0.1a")
        .arg_required_else_help(true)
        .get_matches();

    if let Some((name, submatches)) = matches.subcommand() {
        match name {
            "get" => {
                println!("get");

                let is_repo = submatches.get_flag("repo");
                let plugin = submatches.get_one::<String>("plugin").unwrap();

                println!("{} {}", is_repo, plugin);
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
