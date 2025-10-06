#include "assetProvider.hpp"
#include <sstream>
#include <filesystem>
#include <fstream>
#include <3ds.h>
#include "citra_engine/error.hpp"

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

bool AssetProvider::loadMaterialAssets(std::shared_ptr<CitraEngine::Scene::Material> material) {
    setErr("Not implemented");
    return false;
}

bool AssetProvider::loadModelAsset(std::string path) {
    setErr("Not implemented");
    return false;
}

bool AssetProvider::loadMusicAsset(std::string path) {
    setErr("Not implemented");
    return false;
}

bool AssetProvider::loadSFXAsset(std::string path) {
    setErr("Not implemented");
    return false;
}

char* AssetProvider::readFileToBuffer(std::string path, size_t* size) {
    std::ifstream file(path, std::ios::binary | std::ios::ate);
    if (!file.is_open()) {
        setErr("AssetProvider::readFileToPointer: File does not exist at: " + path);
        return nullptr;
    }

    *size = static_cast<size_t>(file.tellg());
    file.seekg(0, std::ios::beg);

    char* buffer = (char*)linearAlloc(*size);
    file.read(buffer, *size);
    if (file.fail()) {
        setErr("AssetProvider::readFileToPointer: Error reading file at: " + path);
        linearFree(buffer);
        return nullptr;
    }

    return buffer;
}

void AssetProvider::freeBuffer(void* ptr) {
    linearFree(ptr);
}