#pragma once
#ifndef PC_CONTROLLERMAP
#include <SDL3/SDL.h>
#include "citra_engine/citra_engine.hpp"

bool initControllerMap();
void tickControllerMap();
void processSDLKeyDownEvent(SDL_Keycode keyCode);
void processSDLKeyUpEvent(SDL_Keycode keyCode);
void processSDLLostFocusEvent();
bool flipScreen();
CitraEngine::Input::InputState getInputState();

#define PC_CONTROLLERMAP
#endif // PC_CONTROLLERMAP