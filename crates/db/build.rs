fn main() {
    // rustc does not reliably track newly added files inside a directory used by
    // sqlx::migrate!(). Make Cargo rebuild the embedded migrator whenever the
    // migration set changes, not only when an existing SQL file is edited.
    println!("cargo:rerun-if-changed=migrations");
}
