#pragma once
#ifndef LIB_FPS_COUNTER
#include "citra_engine/citra_engine.hpp"

void fpsCounterTick(CitraEngine::Scene::UI::UIObject*, CitraEngine::Scene::SceneCtx*, CitraEngine::Input::InputState*);

#define LIB_FPS_COUNTER
#endif