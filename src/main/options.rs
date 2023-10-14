use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CliOptions {
    pub verbose: bool,
    pub input_file: Option<PathBuf>,
    pub section_header: String,
    pub section_id: bool,
}

// clap (unfortunately) panics when options are not good
// for consistency, you should exit in case of errors.
// But I would have prefer the standard Result return type!
pub fn parse_options() -> CliOptions {
    let command = clap::Command::new(clap::crate_name!())
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .about(clap::crate_description!())
        .arg(
            clap::Arg::new("INPUT")
                .help("Sets the input file to use")
                .required(false),
        )
        .arg(
            clap::Arg::new("section_id")
                .long("section-id")
                .help("Add section id"),
        )
        .arg(
            clap::Arg::new("section_header")
                .long("section-header")
                .takes_value(true)
                .default_value("h2")
                .help("Specify section header"),
        )
        .arg(
            clap::Arg::new("verbose")
                .long("verbose")
                .help("Turn verbose mode"),
        );
    let matches = command.clone().get_matches();

    let input_file = match matches.value_of("INPUT") {
        None => None,
        Some(s) => {
            let path = std::path::Path::new(s);
            if !path.exists() {
                eprintln!("The input file {} does not exist", path.display());
                std::process::exit(2);
            }
            Some(path.to_path_buf())
        }
    };

    let verbose = matches.is_present("verbose");
    let section_header = matches.value_of("section_header").unwrap_or_default().to_string();
    let section_id = matches.is_present("section_id");
    if input_file.is_none() && atty::is(atty::Stream::Stdin) {
        command.clone().print_help().unwrap();
        std::process::exit(2);
    }

    CliOptions {
        input_file,
        verbose,
        section_header,
        section_id
    }
}
