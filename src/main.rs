extern crate getopts;
extern crate glob;
extern crate toml;

use getopts::Options;

use std::env;
use std::path::{Path, PathBuf};
use std::io::prelude::*;
use std::fs::{self, File};

fn main() {
  
  let cwd = env::current_dir().expect("Couldn't get current directory");

  let args: Vec<String> = env::args().collect();
  
  let mut opts = Options::new();
  
  opts
    .optopt("o", "output", "Output directory", "DIR")
    .optopt("c", "crate", "Crate to copy", "CRATE")
    .optopt("l", "cargo-lock", "Path to Cargo.lock", "Cargo.lock")
    .optflag("h", "help", "Print this help menu");

  let program = &args.get(0).cloned().unwrap();

  let matches = opts.parse(&args[1..]).unwrap();
  
  if matches.opt_present("h") {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
    return;
  }

  let pkg = matches
    .opt_str("c")
    .expect("No crate specified");

  let out = matches
    .opt_str("o")
    .expect("No output directory specified");

  let cargo_home = env::var("CARGO_HOME").ok()
    .map(PathBuf::from)
    .or_else(|| env::home_dir().map(|p| p.join(".cargo") ))
    .expect("Could not determine CARGO_HOME");
        
  let cargo_lock = matches
    .opt_str("l")
    .unwrap_or_else(|| "Cargo.lock".into() );

  let _toml = {
    let mut f = File::open(&cargo_lock).expect("No Cargo.lock");
    let mut s = String::new();
    f.read_to_string(&mut s).expect("Error reading Cargo.lock");
    toml::Parser::new(&s).parse().expect("Error parsing Cargo.lock")
  };

  let version = _toml.get("package").iter()
    .filter_map(|s| s.as_slice() )
    .flat_map(|s| s)
    .filter_map(|s| s.as_table() )
    .filter(|s| s.get("name") == Some(&toml::Value::String(pkg.clone())))
    .filter_map(|s| s.get("version") )
    .filter_map(|s| s.as_str() )
    .next()
    .expect("Could not determine version");

  let src_path = {
    let path = &cargo_home
      .join("registry")
      .join("src")
      .join("*")
      .join( format!("{}-{}", &pkg, version) );

    let path_str = path
      .to_str()
      .expect("Error converting path to string");

    glob::glob(path_str).into_iter()
      .filter_map(|mut x| x.next() )
      .filter_map(|x| x.ok() )
      .next()
      .expect("Error determining source directory")
  };

  let out_path = Path::new(&cwd).join(&out).join(&pkg);

  env::set_current_dir(&src_path).expect("Couldn't change directory");

  fs::create_dir(&out_path).expect("Could not create directory");

  for path in glob::glob("**/*")
    .into_iter()
    .flat_map(|x| x)
    .filter_map(|x| x.ok() ) {
    
    if let Ok(m) = fs::metadata(&path) {
      if m.is_dir() {
        fs::create_dir(&out_path.join(&path)).expect("Could not create directory");
      } else {
        fs::copy(&path, &out_path.join(&path)).expect("Could not copy file");
      }
    }
  }

}
