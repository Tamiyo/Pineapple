use pineapple_passes::PassArgs;
use structopt::StructOpt;

pub fn parse_cli() -> Result<(), Box<dyn std::error::Error>> {
    let args = PassArgs::from_args();

    let content = std::fs::read_to_string(&args.input)?;
    build(content.as_str(), args)?;
    Ok(())
}

fn build(buf: &str, args: PassArgs) -> Result<(), String> {
    pineapple_passes::compile(buf, args);
    Ok(())
}
