#include "fpsCounter.hpp"
#include "citra_engine/citra_engine.hpp"
#include <chrono>

using namespace CitraEngine::Scene;

void fpsCounterTick(UI::UIObject* obj, SceneCtx* ctx, CitraEngine::Input::InputState* inputState) {
    obj->data.text = "Delta Time: " + std::to_string((int)(ctx->deltaTime.count() / 1000.0f)) + "ms";
    if (ctx->deltaTime.count() / 1000.0f > 34) obj->data.basecolor = 0xFF0000FF; else obj->data.basecolor = 0xFFFFFFFF;
}
