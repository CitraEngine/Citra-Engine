#pragma once
#ifndef CITRA_ENGINE_TYPES

namespace CitraEngine {
    namespace Math {
        struct Plane;
        struct Frustum;
    }
    namespace Input {
        enum InputBits;
        struct InputState;
    }
    namespace Scene {
        struct SpriteData;
        namespace UI {
            struct UIHandle;
            enum UIRenderType;
            enum TextAlign;
            struct UIRenderData;
            class UIObject;
            struct UIHandle;
        }
        enum RenderType;
        enum ShaderInputType;
        struct ShaderInput;
        struct Material;
        class RenderData;
        class Object;
        class Camera;
        struct SceneCtx;
        class Scene;
    }
    namespace Message {
        enum MessageType;
        struct GameBoundMessage;
        struct RenderBoundMessage;
    }
    class Engine;
}

#define CITRA_ENGINE_TYPES
#endif