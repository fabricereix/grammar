pub enum ExitCode {
    Success,
    ErrorParsing,
    ErrorValidation,
}

impl ExitCode {
    pub fn exit(self) -> ! {
        let value = match self {
            ExitCode::Success => 0,
            // exit code 2 is reserved for command-line options parsing
            // used by default by clap
            ExitCode::ErrorParsing => 3,
            ExitCode::ErrorValidation => 4,
        };
        std::process::exit(value)
    }
}
