use std::{cell::RefCell, collections::HashMap, rc::Rc};

use citra_engine::{
    asset_provider::{AssetProvider, AssetType},
    error::CitraError,
    logger::Logger,
};
use ctru::{Error, services::romfs::RomFS};

use crate::logger::N3dsLogger;

pub struct N3dsAssetProvider {
    _romfs: RomFS,
    loggers: HashMap<String, Rc<RefCell<dyn Logger>>>,
}
impl N3dsAssetProvider {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            _romfs: RomFS::new()?,
            loggers: HashMap::new(),
        })
    }
}
impl AssetProvider for N3dsAssetProvider {
    fn get_asset_location(&self, asset_type: AssetType, path: String) -> String {
        get_asset_location(asset_type, path)
    }
    fn read_file_to_string(&self, path: String) -> Result<String, CitraError> {
        std::fs::read_to_string(path.clone())
            .map_err(|e| CitraError::ReadFileError(format!("{}: {}", path, e)))
    }
    fn get_logger(
        &mut self,
        target_file: String,
    ) -> Result<Rc<RefCell<dyn citra_engine::logger::Logger>>, CitraError> {
        if self.loggers.contains_key(&target_file) {
            self.loggers.insert(
                target_file.clone(),
                Rc::new(RefCell::new(
                    N3dsLogger::new(target_file.clone())
                        .map_err(|e| CitraError::LoggerFetchError(e.to_string()))?,
                )),
            );
        }
        Ok(self.loggers.get(&target_file).unwrap().clone()) // this is oki because we ensured that logger at target_file exists
    }
}

pub fn get_asset_location(asset_type: AssetType, path: String) -> String {
    let mut output = "romfs:".to_string();
    if asset_type == AssetType::Logfile {
        output = "sdmc:".to_string();
    }
    match asset_type {
        AssetType::Scene => output += &(path.clone() + ".json")[..],
        AssetType::Texture => output += &(path.clone() + ".t3x")[..],
        AssetType::Model => output += &(path.clone() + ".glb")[..],
        AssetType::Shader => output += &(path.clone() + ".shbin")[..],
        AssetType::ShaderData => output += &(path.clone() + ".json")[..],
        AssetType::Music => output += &(path.clone() + ".ogg")[..],
        _ => todo!(),
    }
    output
}
