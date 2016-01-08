## cargo copy-dep

A simple tool to copy the source code of a specified package into a local directory.

Reads a `Cargo.lock` to determine version.

You can currently do:

    wget https://crates.io/api/v1/crates/<crate>/<version>/download

But this appears to be undocumented and requires you to look up the required version.

You may want to use this tool when "overriding dependencies".

The [crates guide](http://doc.crates.io/guide.html) suggests using `git clone` to download the source code you want to modify. However, this requires tracking down the specific branch that corresponds to the crate/version that you have been using (and it seems like most crate authors don't bother adding tags).

    Usage: cargo-copy-dep [options]

    Options:
        -o, --output DIR    Output directory
        -c, --crate CRATE   Crate to copy
        -l, --cargo-lock Cargo.lock
                            Path to Cargo.lock
        -h, --help          Print this help menu