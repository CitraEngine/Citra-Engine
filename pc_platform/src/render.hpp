#pragma once
#ifndef PC_RENDER
#include "citra_engine/citra_engine.hpp"
#include <SDL3/SDL.h>

void setFramebufferResized();

bool initGfx(SDL_Window*);

void gfxUpdate(CitraEngine::Scene::Scene*, bool);

void gfxQuit();

#define PC_RENDER
#endif