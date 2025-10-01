#pragma once
#ifndef LIB_ROTATE_CUBE
#include "citra_engine/citra_engine.hpp"

void rotateCubeTick(CitraEngine::Scene::Object*, CitraEngine::Scene::SceneCtx*, CitraEngine::Input::InputState*);
void rotateCubeTickOneAxis(CitraEngine::Scene::Object*, CitraEngine::Scene::SceneCtx*, CitraEngine::Input::InputState*);

#define LIB_ROTATE_CUBE
#endif