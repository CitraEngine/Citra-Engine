#include "assetProvider.hpp"
#include <sstream>
#include <filesystem>

AssetProvider::AssetProvider() {}

std::string AssetProvider::getAssetLocation(std::string path, CitraEngine::AssetType type) {
    std::stringstream ss;
    ss << "romfs:" << path;
    switch (type) {
        case CitraEngine::TEXTURE_ASSET_TYPE:
            ss << ".t3x";
            break;
        case CitraEngine::MODEL_ASSET_TYPE:
            ss << ".glb";
            break;
        case CitraEngine::SHADER_ASSET_TYPE:
            return std::filesystem::path(path).filename().string();
            break;
        case CitraEngine::MUSIC_ASSET_TYPE:
            ss << ".ogg";
            break;
        case CitraEngine::SFX_ASSET_TYPE:
            ss << ".pcm";
            break;
    }
    return ss.str();
}