use anyhow::anyhow;
use libvips::VipsApp;

use crate::Result;

pub fn init() -> Result<VipsApp> {
    VipsApp::new("Caby", false).map_err(|e| anyhow!("could not initialize libvips: {:?}", e))
}
