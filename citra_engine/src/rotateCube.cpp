#include "rotateCube.hpp"
#include "citra_engine/citra_engine.hpp"

void rotateCubeTick(CitraEngine::Scene::Object* obj, CitraEngine::Scene::SceneCtx* ctx, CitraEngine::Input::InputState* inputState) {
    obj->rotation.x += 0.5 * (ctx->deltaTime.count() / 1000000.0f);
    obj->rotation.y += 0.5 * (ctx->deltaTime.count() / 1000000.0f);
    obj->markDirty();
}

void rotateCubeTickOneAxis(CitraEngine::Scene::Object* obj, CitraEngine::Scene::SceneCtx* ctx, CitraEngine::Input::InputState* inputState) {
    obj->rotation.y += 0.5 * (ctx->deltaTime.count() / 1000000.0f);
    obj->markDirty();
}