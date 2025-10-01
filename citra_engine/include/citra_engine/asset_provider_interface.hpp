#pragma once
#ifndef CITRA_ENGINE_ASSET_PROVIDER_INTERFACE
#include <string>
#include <memory>
#include "citra_engine_types.hpp"

namespace CitraEngine {
    enum AssetType {
        TEXTURE_ASSET_TYPE,
        MODEL_ASSET_TYPE,
        SHADER_ASSET_TYPE,
        MUSIC_ASSET_TYPE,
        SFX_ASSET_TYPE
    };
    
    struct AssetProviderInterface {
        explicit AssetProviderInterface() {}
        virtual ~AssetProviderInterface() = default;
        virtual std::string getAssetLocation(std::string path, AssetType type) = 0;
        virtual bool loadMaterialAssets(std::shared_ptr<Scene::Material> material) = 0; // Should also load shaders and textures along with material descriptor sets
        virtual bool loadModelAsset(std::string path) = 0;
        virtual bool loadMusicAsset(std::string path) = 0;
        virtual bool loadSFXAsset(std::string path) = 0;
    };
}

#define CITRA_ENGINE_ASSET_PROVIDER_INTERFACE
#endif