extern crate rustc_serialize;
extern crate docopt;

use docopt::Docopt;

static USAGE: &'static str = "
daemon manager

Usage:
    dm -r [-n <name>] [-g <group>] [-p <pid-file>] [-o <log-file>] <cmd>...
    dm [-n <name>] [-g <group>]
    dm -k [-n <name>] [-g <group>] [-i]
    dm (-h | --help)
    dm --version

Options:
    -n <name>, --name <name>         daemon name
    -g <group>, --group <group>      daemon group
    -p <pid-file>, --pid <pid-file>  pid file
    -o <log-file>, --log <log-file>  log to
    -s, --shell                      run with shell
    -i, --ignore                     ignore if no daemon matches
    -r, --run                        run cmd
    -k, --kill                       kill daemon
    -q, --quiet                      be quiet
    -h, --help                       Show this help
    --version                        Show version
";

#[derive(RustcDecodable, Debug)]
struct Args {
    arg_cmd: Vec<String>,
    flag_name: Option<String>,
    flag_group: Option<String>,
    flag_pid: Option<String>,
    flag_log: Option<String>,
    flag_quiet: bool,
    flag_ignore: bool,
    flag_run: bool,
    flag_kill: bool,
    flag_help: bool,
    flag_version: bool
}

static VERSION: &'static str = "0.1";

fn main() {
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d|
                                d.options_first(true)
                                 .version(Some(VERSION.to_string()))
                                 .decode())
                            .unwrap_or_else(|e| e.exit());
    if args.flag_run {
        println!("to run `{}` name:{:?}, group:{:?}, pid:{:?}, log:{:?}",
                 args.arg_cmd.connect(" "),
                 args.flag_name, args.flag_group, args.flag_pid, args.flag_log);
    } else if args.flag_kill {
        println!("to kill name:{:?}, group:{:?}, quiet:{}",
                args.flag_name, args.flag_group, args.flag_quiet);
    } else {
        println!("to list name:{:?}, group:{:?}, ignore:{}",
                 args.flag_name, args.flag_group, args.flag_ignore)
    }
}
