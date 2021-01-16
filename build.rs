use winres::WindowsResource;
use std::io::Result;

fn main() -> Result<()> {
    let mut rc = WindowsResource::new();
    rc.set_resource_file("nphypercosm.rc");
    rc.compile()?;
    Ok(())
}