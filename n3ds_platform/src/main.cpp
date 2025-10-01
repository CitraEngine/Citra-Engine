#include <iostream>
#include <fstream>
#include <tuple>
#include <3ds.h>
#include <c3d/maths.h>
#include <stdlib.h>
#include "citra_engine/citra_engine.hpp"
#include "citra_engine/channel.hpp"
#include "n3dslink.hpp"
#include "exitfuncs.hpp"
#include "render.hpp"
#include "citra_engine/error.hpp"
#include "audio.hpp"
#include "assetProvider.hpp"

#ifndef TESTING_BUILD
int main() {
    cfguInit();
    if (!initGfx()) {
        softPanic("could not init GFX: " + getErr());
    }

    if (!init3dslinkStdio()) {
        softPanic("could not init SOC");
    }

    unlockCore1(); // necessary for sound

    Result rc = romfsInit();
    if (rc) {
        softPanic("Romfs init err: " + std::to_string(rc));
    }
    atexit([](){romfsExit();});

    AudioEngine* audioEngine = new AudioEngine();

    CitraEngine::Scene::Camera* topCamera = new CitraEngine::Scene::Camera(glm::vec3{0, 0, 0}, glm::vec3{0, 0, 0}, 0.01f, 1000.0f, (float)C3D_AngleFromDegrees(40.0f), C3D_AspectRatioTop, nullptr);
    CitraEngine::Scene::Camera* bottomCamera = new CitraEngine::Scene::Camera(glm::vec3{0, 0, 0}, glm::vec3{0, 0, 0}, 0.01f, 1000.0f, (float)C3D_AngleFromDegrees(40.0f), C3D_AspectRatioBot, nullptr);

    CitraEngine::Scene::Scene* topScene = new CitraEngine::Scene::Scene(topCamera, audioEngine);
    CitraEngine::Scene::Scene* bottomScene = new CitraEngine::Scene::Scene(bottomCamera, audioEngine);

    AssetProvider* assetProvider = new AssetProvider();
    CitraEngine::Engine* engine = new CitraEngine::Engine("3DS", softPanic, topScene, bottomScene, assetProvider);

    while (aptMainLoop()) {
        hidScanInput();
        float iod = osGet3DSliderState();

        if (engine->update(CitraEngine::Input::InputState {
            .kDown = hidKeysDown(),
            .kHeld = hidKeysHeld(),
            .kUp = hidKeysUp()
        },
        topScene,
        bottomScene
        )) {
            break;
        }

        gfxUpdate(topScene, bottomScene, iod / 32);

        checkPanic();

        gspWaitForVBlank();
    }

    exitGame();

    delete engine;
    delete bottomScene;
    delete topScene;
    delete audioEngine;

    return 0;
}
#endif
