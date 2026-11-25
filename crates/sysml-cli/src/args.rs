#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    Parse,
    Validate,
    Inspect,
}

impl Command {
    pub fn name(self) -> &'static str {
        match self {
            Self::Parse => "parse",
            Self::Validate => "validate",
            Self::Inspect => "inspect",
        }
    }
    fn from_name(name: &str) -> Result<Self, String> {
        match name {
            "parse" => Ok(Self::Parse),
            "validate" => Ok(Self::Validate),
            "inspect" => Ok(Self::Inspect),
            _ => Err(format!("Unknown command '{name}'. Run sysml --help.")),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Text,
    Json,
}

#[derive(Debug)]
pub struct Options {
    pub command: Command,
    pub format: Format,
    pub strict: bool,
    pub warnings_as_errors: bool,
    pub request_json: bool,
    pub stdin_name: String,
    pub paths: Vec<String>,
}

pub enum Invocation {
    Help(Option<Command>),
    Version,
    Run(Options),
}

pub fn parse(args: Vec<String>, legacy: bool) -> Result<Invocation, String> {
    let mut args = args.into_iter();
    let Some(first) = args.next() else {
        return Ok(Invocation::Help(None));
    };
    if matches!(first.as_str(), "--version" | "-V") {
        return Ok(Invocation::Version);
    }
    if matches!(first.as_str(), "--help" | "-h" | "help") {
        let command = args.next().map(|s| Command::from_name(&s)).transpose()?;
        return Ok(Invocation::Help(command));
    }
    let command = if legacy {
        Command::Inspect
    } else {
        Command::from_name(&first)?
    };
    let mut args = if legacy { vec![first] } else { Vec::new() }
        .into_iter()
        .chain(args);
    let mut options = Options {
        command,
        format: if command == Command::Inspect {
            Format::Json
        } else {
            Format::Text
        },
        strict: command != Command::Inspect,
        warnings_as_errors: false,
        request_json: false,
        stdin_name: "stdin.sysml".into(),
        paths: Vec::new(),
    };
    let mut literal = false;
    let mut named_stdin = false;
    while let Some(arg) = args.next() {
        if literal {
            options.paths.push(arg);
            continue;
        }
        match arg.as_str() {
            "--" => literal = true,
            "--help" | "-h" => return Ok(Invocation::Help(Some(command))),
            "--strict" if command == Command::Inspect => options.strict = true,
            "--warnings-as-errors" => options.warnings_as_errors = true,
            "--request-json" => options.request_json = true,
            "--format" => set_format(
                &mut options,
                &args.next().ok_or("--format needs text or json")?,
            )?,
            "--stdin-name" => {
                options.stdin_name = args
                    .next()
                    .ok_or("--stdin-name needs a .sysml or .kerml filename")?;
                named_stdin = true;
            }
            _ if arg.starts_with("--format=") => set_format(&mut options, &arg[9..])?,
            _ if arg.starts_with('-') && arg != "-" => {
                return Err(format!("Unknown option: {arg}"))
            }
            _ => options.paths.push(arg),
        }
    }
    let stdin_count = options.paths.iter().filter(|p| *p == "-").count();
    if stdin_count > 1 {
        return Err("Standard input (-) can only be supplied once".into());
    }
    if named_stdin && stdin_count != 1 {
        return Err("--stdin-name requires a - input".into());
    }
    if options.request_json && !options.paths.is_empty() {
        return Err("--request-json cannot be combined with paths or -".into());
    }
    if !options.request_json && options.paths.is_empty() {
        return Err("Supply a file, directory, or - for standard input".into());
    }
    Ok(Invocation::Run(options))
}

fn set_format(options: &mut Options, value: &str) -> Result<(), String> {
    options.format = match value {
        "text" if options.command != Command::Inspect => Format::Text,
        "json" => Format::Json,
        _ => return Err("Use --format text or --format json; inspect supports only json".into()),
    };
    Ok(())
}

pub fn help(command: Option<Command>, legacy: bool) -> String {
    if command.is_none() && !legacy {
        return "SysML — parse, validate, and inspect SysML v2 / KerML files\n\nUsage: sysml <COMMAND> [OPTIONS] <FILE|DIRECTORY|->...\n\nCommands:\n  parse       Check syntax and body structure without name resolution\n  validate    Check syntax, references, and implemented semantic rules\n  inspect     Emit the versioned semantic snapshot as JSON\n\nUse sysml <COMMAND> --help for options. --version prints tool and kernel versions.\n".into();
    }
    let command = command.unwrap_or(Command::Inspect);
    let executable = if legacy {
        "sysml-inspect".into()
    } else {
        format!("sysml {}", command.name())
    };
    let strict = if command == Command::Inspect {
        "  --strict               Exit 1 on error diagnostics (default: export regardless)\n"
    } else {
        ""
    };
    let format = if command == Command::Inspect {
        "json (default)"
    } else {
        "text (default) or json"
    };
    format!("Usage: {executable} [OPTIONS] <FILE|DIRECTORY|->...\n       {executable} [OPTIONS] --request-json < request.json\n\nOptions:\n  --format FORMAT        {format}\n{strict}  --warnings-as-errors   Exit 1 on warnings as well as errors\n  --stdin-name NAME      Dialect/display filename for - (default: stdin.sysml)\n  --request-json         Read an AnalysisRequest from stdin\n  --                     End options (for paths beginning with -)\n  -h, --help             Show help\n\nDirectories: recursively read .sysml/.kerml files, skip hidden directories and\nsymlink entries, deduplicate files. Explicit file symlinks are accepted.\nLimits: 32 documents, 5 MB per file, 20 MB total (kernel policy).\nExit: 0 success, 1 failed checks, 2 input/usage/engine/output failure.\n\nparse checks syntax/body structure; it does not export an AST or resolve names.\nvalidate runs implemented checks; a clean report is not full conformance proof.\ninspect emits snapshot schema v4, including diagnostics, to stdout.\nText diagnostics go to stderr; JSON output is self-contained on stdout.\n")
}
