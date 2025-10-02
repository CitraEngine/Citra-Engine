#pragma once
#ifndef N3DS_ASSET_PROVIDER
#include "citra_engine/asset_provider_interface.hpp"
#include "citra_engine/citra_engine.hpp"

class AssetProvider : public CitraEngine::AssetProviderInterface {
    public:
        AssetProvider();
        std::string getAssetLocation(std::string, CitraEngine::AssetType);
        virtual bool loadMaterialAssets(std::shared_ptr<CitraEngine::Scene::Material> material);
        virtual bool loadModelAsset(std::string path);
        virtual bool loadMusicAsset(std::string path);
        virtual bool loadSFXAsset(std::string path);
};

#define N3DS_ASSET_PROVIDER
#endif