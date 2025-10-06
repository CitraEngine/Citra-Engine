#include "citra_engine/citra_engine.hpp"
#include <simdjson.h>
#include <fstream>
#include "citra_engine/error.hpp"

using namespace CitraEngine::Scene;

simdjson::ondemand::parser parser;

bool parseJsonV1(simdjson::ondemand::document_reference json, Scene* output) {
    /* ASSETS */
    // Scripts
    auto scriptsRes = json["scripts"].get_array();
    if (json["scripts"].error()) {
        setErr(std::string("parseJsonV1: 'scripts' field could not be found: ") + simdjson::error_message(json["scripts"].error()));
        return false;
    }
    auto scripts = scriptsRes.value_unsafe();
    auto scriptsCountRes = scripts.count_elements();
    if (scriptsCountRes.error()) {
        setErr(std::string("parseJsonV1: 'scripts' count could not be determined: ") + simdjson::error_message(scriptsCountRes.error()));
        return false;
    }
    std::size_t scriptsCount = scriptsCountRes.value_unsafe();
    for (std::size_t i = 0; i < scriptsCount; i++) {
        auto scriptPathRes = scripts.at(i).get_string();
        if (scriptPathRes.error()) {
            setErr(std::string("parseJsonV1: 'scripts[") + std::to_string(i) + "]' entry could not be read: " + simdjson::error_message(scriptPathRes.error()));
            return false;
        }
        std::string_view scriptPath = scriptPathRes.value_unsafe();
        std::string truePath = output->ctx.assetProvider->getAssetLocation(std::string(scriptPath), CitraEngine::SCRIPT_ASSET_TYPE);
        if (!output->ctx.assetProvider->loadScriptAsset(std::string(truePath))) {
            setErr("parseJsonV1: 'scripts[" + std::to_string(i) + "]' entry could not be loaded '" + std::string(scriptPath) + "' -> '" + truePath + "': " + getErr());
            return false;
        }
    }

    return true;
}

bool Scene::fromJson(std::string path, Scene* output) {
    size_t fileSize;
    char* fileData = this->ctx.assetProvider->readFileToBuffer(path, &fileSize);
    if (fileData == nullptr) {
        setErr("Scene::FromJson: error reading file: " + getErr());
        return false;
    }

    simdjson::padded_string jsonString(fileData, fileSize);
    auto jsonRes = parser.iterate(jsonString);
    if (jsonRes.error()) {
        this->ctx.assetProvider->freeBuffer(fileData);
        setErr("Scene::FromJson: Json parse failed for '" + path + "': " + simdjson::error_message(jsonRes.error()));
        return false;
    }
    simdjson::ondemand::document_reference json = jsonRes.value_unsafe();

    auto version = json["version"].get_uint64();
    if (version.error()) {
        this->ctx.assetProvider->freeBuffer(fileData);
        setErr("Scene::FromJson: version field could not be found '" + path + "': " + simdjson::error_message(version.error()));
        return false;
    }
    switch (version.value_unsafe()) {
        case 1:
            if (!parseJsonV1(json, output)) {
                this->ctx.assetProvider->freeBuffer(fileData);
                setErr("Scene::FromJson: parse failed for '" + path + "': " + getErr());
                return false;
            }
            break;
        default:
            this->ctx.assetProvider->freeBuffer(fileData);
            setErr("Scene::FromJson: unsupported version '" + std::to_string(version.value_unsafe()) + "' in '" + path + "'");
            return false;
    }

    this->ctx.assetProvider->freeBuffer(fileData);
    return true;
}