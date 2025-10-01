#include "assetProvider.hpp"
#include <sstream>

AssetProvider::AssetProvider() {}

std::string AssetProvider::getAssetLocation(std::string path, CitraEngine::AssetType type) {
    std::stringstream ss;
    ss << "./data" << path;
    switch (type) {
        case CitraEngine::TEXTURE_ASSET_TYPE:
            ss << ".png";
            break;
        case CitraEngine::MODEL_ASSET_TYPE:
            ss << ".glb";
            break;
        case CitraEngine::SHADER_ASSET_TYPE:
            ss << ".spv";
            break;
        case CitraEngine::MUSIC_ASSET_TYPE:
            ss << ".ogg";
            break;
        case CitraEngine::SFX_ASSET_TYPE:
            ss << ".ogg";
            break;
    }
    return ss.str();
}