pub fn exit_with_error(err: &anyhow::Error) -> ! {
    eprintln!("Error: {err}");
    std::process::exit(1);
}
