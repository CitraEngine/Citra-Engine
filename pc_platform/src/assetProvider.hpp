#pragma once
#ifndef PC_ASSET_PROVIDER
#include "citra_engine/asset_provider_interface.hpp"

class AssetProvider : public CitraEngine::AssetProviderInterface {
    public:
        AssetProvider();
        std::string getAssetLocation(std::string, CitraEngine::AssetType);
};

#define PC_ASSET_PROVIDER
#endif