#include "playMusic.hpp"
#include "citra_engine/error.hpp"

using namespace CitraEngine::Scene;

bool hasRan = false;

void playMusic(Object* self, SceneCtx* ctx, CitraEngine::Input::InputState* inputState) {
    if (!hasRan) {
        if (ctx->audio != nullptr && !ctx->audio->bgm_play(0, 100, ctx->assetProvider->getAssetLocation("/music/Rest", CitraEngine::MUSIC_ASSET_TYPE))) {
            ctx->softPanic(getErr());
        }
        hasRan = true;
    }
}