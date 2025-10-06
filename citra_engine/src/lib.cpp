#include "citra_engine/citra_engine.hpp"

using namespace CitraEngine;

Engine::Engine(std::string platform, void(*softPanic)(std::string), Scene::Scene* scene, AssetProviderInterface* assetProvider): platform(platform), softPanic(softPanic), assetProvider(assetProvider) {
    attachScene(scene);
}

void Engine::attachScene(Scene::Scene* scene) {
    scene->ctx.softPanic = softPanic;
    scene->ctx.assetProvider = assetProvider;
    scene->engine = &*this;
    this->scene = scene;
}

void Engine::detachScene() {
    delete scene;
}

bool Engine::update(Input::InputState inputState) { // returns true if app should end
    if (inputState.kDown & Input::KEY_START) {
        return true;
    }
    if (inputState.kDown & Input::KEY_SELECT) {
        softPanic("User initiated test");
    }
    scene->tick(&inputState);

    return false;
}