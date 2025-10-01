#pragma once
#ifndef AMIUS_ADVENTURE_CAMERA_MOVE
#include "citra_engine/citra_engine.hpp"

void moveCamera(CitraEngine::Scene::Camera*, CitraEngine::Scene::SceneCtx*, CitraEngine::Input::InputState*);

#define AMIUS_ADVENTURE_CAMERA_MOVE
#endif