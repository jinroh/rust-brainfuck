#[macro_use]
macro_rules! prompt {
  () => {{
    print!("> ");
    ::std::io::stdout().flush().unwrap();
  }};
  ($fmt:expr) => {{
    print!(concat!($fmt, "> "));
    ::std::io::stdout().flush().unwrap();
  }};
  ($fmt:expr, $($arg:tt)*) => {{
    print!(concat!($fmt, "> "), $($arg)*);
    ::std::io::stdout().flush().unwrap();
  }};
}
