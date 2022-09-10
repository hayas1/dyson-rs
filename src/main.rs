use anyhow::bail;
use clap::{App, Args, Parser, Subcommand};
use dyson::{Indent, Value};
use std::io::{stdin, stdout};

#[derive(Parser)]
struct Arg {
    #[clap(subcommand)]
    action: Action,
}

#[derive(Debug, Subcommand)]
enum Action {
    /// format given json
    Format(FormatArg),

    /// compare two json
    Compare(CompareArg),
    // Edit { edit: Vec<String> },
}

fn main() -> anyhow::Result<()> {
    let cli = Arg::parse();
    match cli.action {
        Action::Format(arg) => format(arg),
        Action::Compare(arg) => compare(arg),
        // Action::Edit { edit } => todo!(),
    }
}

#[derive(Debug, Args)]
struct FormatArg {
    /// input json file path
    path: Option<String>,

    /// output json indent level
    ///
    /// - 0(minified): no unnecessary space and linefeed is included.
    /// - 1(basically): normal json indent. 1 line, 1 element.
    #[clap(short = 'd', long = "indent", default_value = "1", verbatim_doc_comment)]
    indent: u8,
}
fn format(arg: FormatArg) -> anyhow::Result<()> {
    let json = if let Some(path) = arg.path {
        Value::load(&path)?
    } else if atty::is(atty::Stream::Stdin) {
        FormatArg::augment_args(App::new(format!("{} {}", env!("CARGO_PKG_NAME"), "format"))).print_help()?;
        return Ok(());
    } else {
        Value::read(stdin())?
    };

    match arg.indent {
        0 => json.write_with::<_, Indent<0>>(stdout())?,
        1 => json.write_with::<_, Indent<1>>(stdout())?,
        _ => bail!("indent argument must be 0 or 1"),
    };
    println!();
    Ok(())
}

#[derive(Debug, Args)]
struct CompareArg {
    /// input json file path
    path1: String,

    /// input json file path2 (Optional)
    ///
    /// if omit this argument, compare with stdin.
    path2: Option<String>,
}
fn compare(arg: CompareArg) -> anyhow::Result<()> {
    let json1 = Value::load(arg.path1)?;
    let json2 = if let Some(path) = arg.path2 {
        Value::load(&path)?
    } else if atty::is(atty::Stream::Stdin) {
        FormatArg::augment_args(App::new(format!("{} {}", env!("CARGO_PKG_NAME"), "compare"))).print_help()?;
        return Ok(());
    } else {
        Value::read(stdin())?
    };
    println!("{}", json1 == json2);
    Ok(())
}
