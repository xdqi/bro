use std::io;
#[cfg(windows)]
extern crate winres;

fn main() -> io::Result<()> {
    #[cfg(windows)]
    {
        winres::WindowsResource::new()
            // This path can be absolute, or relative to your crate root.
            .set_icon("res/bro.ico")
            .compile()?;
    }
    Ok(())
}
