use std::{
    fs,
    io,
    io::{
        Error,
        ErrorKind,
    },
    env,
    path::{
        PathBuf,
    },
};

extern crate regex;
use regex::Regex;

fn usage(msg : &str) -> io::Result<()> {
    println!("Usage: renumber [--commit] [--match file_regex] <dir> <output_prefix> <output_suffix>");
    Err(Error::new(ErrorKind::Other, msg))
}

fn get_file_name<'p>(pathbuf : &'p PathBuf) -> &'p str {
    pathbuf.file_name().unwrap().to_str().unwrap()
}

fn main() -> io::Result<()> {
    let args : Vec<String> = env::args().collect();

    let mut do_rename = false;
    let mut file_pattern = ".*";

    let mut arg_i = 1;
    for mut i in 1 .. args.len() {
        if args[i] == "--commit" {
            do_rename = true;
            arg_i = i + 1;
        } else if args[i] == "--match" {
            i += 1;
            if i < args.len() {
                file_pattern = &args[i];
                arg_i = i + 1;
            } else {
                return usage("no value supplied for --match");
            }
        }
    }

    let re_matching_file = Regex::new(file_pattern).expect("failed to compile regex");

    if arg_i >= args.len() {
        return usage("missing dir to run in");
    }

    let dir = &args[arg_i];
    arg_i += 1;

    if arg_i >= args.len() {
        return usage("missing output prefix");
    }

    let prefix = &args[arg_i];
    arg_i += 1;

    if arg_i >= args.len() {
        return usage("missing output suffix");
    }

    let suffix = &args[arg_i];

    println!("file_pattern: {}, dir: {}, prefix: {}, suffix: {}", file_pattern, dir, prefix, suffix);

    let mut entries = fs::read_dir(dir)?
        .map(|res| res.map(|e| e.path()))
        .filter(|e| {
            let pb = e.as_ref().unwrap();

            pb.is_file() &&
            re_matching_file.find(get_file_name(pb)).is_some()
        })
        .collect::<Result<Vec<_>, io::Error>>()?;

    // The order in which `read_dir` returns entries is not guaranteed. If reproducible
    // ordering is required the entries should be explicitly sorted.

    entries.sort();

    // The entries have now been sorted by their path.

    if entries.len() > 1_000_000 {
        return Err(Error::new(ErrorKind::Other, "currently not supporting more than 1,000,000 entries"));
    }

    let mut inpath = PathBuf::new();
    inpath.push(dir);
    let mut outpath = inpath.clone();

    for i in 0 .. entries.len() {
        inpath.push(get_file_name(&entries[i]));
        outpath.push(format!("{}{:06}{}", prefix, i, suffix));
        println!("renumber \"{}\" -> \"{}\"", inpath.to_str().unwrap(), outpath.to_str().unwrap());

        if do_rename {
            fs::rename(&inpath, &outpath)?;
        }

        inpath.pop();
        outpath.pop();
    }

    Ok(())
}
