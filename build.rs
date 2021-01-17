use winres::WindowsResource;
use std::io::Result;

fn main() -> Result<()> {
    let mut rc = WindowsResource::new();
    rc.set_resource_file("NPAxHyp.rc");
    rc.compile()?;
    Ok(())
}