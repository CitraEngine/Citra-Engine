#pragma once
#ifndef N3DS_ASSET_PROVIDER
#include "citra_engine/asset_provider_interface.hpp"

class AssetProvider : public CitraEngine::AssetProviderInterface {
    public:
        AssetProvider();
        std::string getAssetLocation(std::string, CitraEngine::AssetType);
};

#define N3DS_ASSET_PROVIDER
#endif