use gen::AxumConnectServiceGenerator;
use std::io::Result;

mod gen;

pub fn compile(
    protos: &[impl AsRef<std::path::Path>],
    includes: &[impl AsRef<std::path::Path>],
    mut config: prost_build::Config,
) -> Result<()> {
    config.service_generator(Box::new(AxumConnectServiceGenerator::new()));
    config.compile_protos(protos, includes)
}
