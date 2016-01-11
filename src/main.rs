extern crate getopts;
extern crate glob;
extern crate toml;

use std::env;
use std::path::Path;
use std::io::prelude::*;
use std::fs::{self, File};



fn main() {
  
  let cwd = env::current_dir().expect("Couldn't get current directory");

  let args: Vec<String> = env::args().collect();
  
  let mut opts = getopts::Options::new();
  
  opts
    .reqopt("o", "out-dir", "Output directory", "DIR")
    .reqopt("c", "crate", "Crate to copy", "CRATE")
    .optopt("l", "cargo-lock", "Path to Cargo.lock", "Cargo.lock")
    .optflag("h", "help", "Print this help menu");

  let program = &args.get(0).cloned().unwrap();

  let matches = opts.parse(&args[1..]).unwrap();
  
  if matches.opt_present("h") {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
    return;
  }

  let pkg = matches.opt_str("c")
    .expect("No crate specified");

  let out_dir = {
    let d = matches
      .opt_str("o")
      .expect("No output directory specified");
    Path::new(&cwd).join(&d).join(&pkg)
  };
 
  let src_dir = {
    
    let cargo_lock = {
      
      let l = matches
        .opt_str("l")
        .unwrap_or_else(|| "Cargo.lock".into() );

      let reading = || -> std::io::Result<String> {
        let mut f = try!(File::open(&l));
        let mut s = String::new();
        let _ = try!(f.read_to_string(&mut s));
        Ok(s)
      };

      reading().ok()
        .and_then(|x| toml::Parser::new(&x).parse() )
        .expect("Could not read Cargo.lock")

    };

    let version = &cargo_lock.get("package").iter()
      .filter_map(|s| s.as_slice() )
      .flat_map(|s| s)
      .filter_map(|s| s.as_table() )
      .filter(|s| s.get("name") == Some(&toml::Value::String(pkg.clone())))
      .filter_map(|s| s.get("version") )
      .filter_map(|s| s.as_str() )
      .next()
      .expect("Could not determine version");

    let cargo_home = env::var("CARGO_HOME").ok()
      .map(|d| d.into() )
      .or_else(|| env::home_dir().map(|p| p.join(".cargo") ))
      .expect("Could not determine $CARGO_HOME");

    let path = &cargo_home
      .join("registry")
      .join("src")
      .join("*")
      .join( format!("{}-{}", &pkg, &version) );

    glob::glob(&path.to_string_lossy()).into_iter()
      .filter_map(|mut x| x.next() )
      .filter_map(|x| x.ok() )
      .next()
      .expect("Error determining source directory")
  };

  env::set_current_dir(&src_dir).expect("Couldn't change directory");

  fs::create_dir(&out_dir).expect("Could not create directory");

  for path in glob::glob("**/*")
    .into_iter()
    .flat_map(|x| x)
    .filter_map(|x| x.ok() ) {
    
    if let Ok(m) = fs::metadata(&path) {
      if m.is_dir() {
        fs::create_dir(&out_dir.join(&path)).expect("Could not create directory");
      } else {
        fs::copy(&path, &out_dir.join(&path)).expect("Could not copy file");
      }
    }
  }

}
