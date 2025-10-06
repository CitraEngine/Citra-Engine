#pragma once
#ifndef N3DS_ASSET_PROVIDER
#include "citra_engine/asset_provider_interface.hpp"
#include "citra_engine/citra_engine.hpp"

class AssetProvider : public CitraEngine::AssetProviderInterface {
    public:
        AssetProvider();
        std::string getAssetLocation(std::string, CitraEngine::AssetType);
        bool loadMaterialAssets(std::shared_ptr<CitraEngine::Scene::Material> material);
        bool loadModelAsset(std::string path);
        bool loadMusicAsset(std::string path);
        bool loadSFXAsset(std::string path);

        char* readFileToBuffer(std::string path, size_t* size);
        void freeBuffer(void* ptr);
};

#define N3DS_ASSET_PROVIDER
#endif