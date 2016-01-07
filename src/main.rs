extern crate docopt;
extern crate rustc_serialize;
extern crate goethite;

use docopt::Docopt;
use goethite::site;

#[cfg_attr(rustfmt, rustfmt_skip)]
const USAGE: &'static str = "
goethite.

Usage:
  goethite build --source=<src> --dest=<dest>
  goethite (-h | --help)

Options:
  -h --help           Print this message.
  --source=<src>      Source directory.
  --dest=<dest>       Destination directory.
";

#[derive(Debug, RustcDecodable)]
struct Args {
    flag_source: String,
    flag_dest: String,
}

fn main() {
    let args: Args = Docopt::new(USAGE).and_then(|d| d.decode()).unwrap_or_else(|e| e.exit());

    match site::build(args.flag_source, args.flag_dest) {
        Ok(_) => println!("Build successful!"),
        Err(err) => {
            println!("{}\nBuild failed!", err);
            std::process::exit(1);
        }
    }
}
