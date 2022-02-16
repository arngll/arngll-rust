use clap::Parser;
use hamaddr::HamAddr;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Opt {
    /// Silence all output
    #[clap(short, long)]
    quiet: bool,

    /// Verbose mode (-v, -vv, -vvv, etc)
    #[clap(short, long, parse(from_occurrences))]
    verbose: usize,

    #[clap(short, long)]
    callsign: HamAddr,
}

fn main() {
    let opt = Opt::parse();

    println!("Callsign: {}", opt.callsign);
    println!("opt = {:?}", opt);

    stderrlog::new()
        .quiet(opt.quiet)
        .verbosity(opt.verbose)
        .init()
        .unwrap();
}
