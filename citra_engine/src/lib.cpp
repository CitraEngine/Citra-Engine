#include "citra_engine/citra_engine.hpp"

using namespace CitraEngine;

Engine::Engine(std::string platform, void(*softPanic)(std::string), Scene::Scene* topScene, Scene::Scene* bottomScene, AssetProviderInterface* assetProvider): platform(platform), softPanic(softPanic), assetProvider(assetProvider) {
    topScene->ctx.softPanic = softPanic;
    bottomScene->ctx.softPanic = softPanic;
    topScene->ctx.assetProvider = assetProvider;
    bottomScene->ctx.assetProvider = assetProvider;
}

bool Engine::update(Input::InputState inputState, Scene::Scene* topScene, Scene::Scene* bottomScene) { // returns true if app should end
    if (inputState.kDown & Input::KEY_START) {
        return true;
    }
    if (inputState.kDown & Input::KEY_SELECT) {
        softPanic("User initiated test");
    }
    topScene->tick(&inputState);
    bottomScene->tick(&inputState);

    return false;
}