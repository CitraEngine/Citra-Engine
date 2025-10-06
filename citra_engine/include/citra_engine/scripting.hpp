#pragma once
#ifndef CITRA_ENGINE_SCRIPTING
#include <citra_engine/citra_engine_types.hpp>

namespace CitraEngine {
    namespace Scripting {
        class IScript {
        public:
            void OnStart(Scene::Object*, Scene::SceneCtx*, Input::InputState*) {}
            void OnEnable(Scene::Object*, Scene::SceneCtx*, Input::InputState*) {}
            void OnTick(Scene::Object*, Scene::SceneCtx*, Input::InputState*) {}
            void OnDisable(Scene::Object*, Scene::SceneCtx*, Input::InputState*) {}
            void OnDestroy(Scene::Object*, Scene::SceneCtx*, Input::InputState*) {}
        };
    }
}

#define CITRA_ENGINE_SCRIPTING
#endif