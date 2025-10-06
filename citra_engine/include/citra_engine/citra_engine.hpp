#pragma once
#ifndef CITRA_ENGINE
#include <stdint.h>
#include <string>
#include <array>
#include <glm/glm.hpp>
#include <memory>
#include "channel.hpp"
#include "citra_engine_types.hpp"
#include "audio_interface.hpp"
#include "asset_provider_interface.hpp"
#include "scripting.hpp"

typedef uint32_t u32;

namespace CitraEngine {
    namespace Math {
        struct Plane {
            glm::vec3 normal = {0.0f, 1.0f, 0.0f};
            float distance = 0.0f;
        };

        struct Frustum {
            Plane topFace;
            Plane bottomFace;
            
            Plane rightFace;
            Plane leftFace;

            Plane nearFace;
            Plane farFace;
        };
    }

    namespace Input {
        // taken straight from 3ds/services/hid.h
        #define BIT(n) (1U<<(n))
        enum InputBits {
            KEY_A       = BIT(0),       ///< A
            KEY_B       = BIT(1),       ///< B
            KEY_SELECT  = BIT(2),       ///< Select
            KEY_START   = BIT(3),       ///< Start
            KEY_DRIGHT  = BIT(4),       ///< D-Pad Right
            KEY_DLEFT   = BIT(5),       ///< D-Pad Left
            KEY_DUP     = BIT(6),       ///< D-Pad Up
            KEY_DDOWN   = BIT(7),       ///< D-Pad Down
            KEY_R       = BIT(8),       ///< R
            KEY_L       = BIT(9),       ///< L
            KEY_X       = BIT(10),      ///< X
            KEY_Y       = BIT(11),      ///< Y
            KEY_TOUCH   = BIT(20),      ///< Touch (Not actually provided by HID)
            KEY_CPAD_RIGHT = BIT(28),   ///< Circle Pad Right
            KEY_CPAD_LEFT  = BIT(29),   ///< Circle Pad Left
            KEY_CPAD_UP    = BIT(30),   ///< Circle Pad Up
            KEY_CPAD_DOWN  = BIT(31),   ///< Circle Pad Down

            // Generic catch-all directions
            KEY_UP    = KEY_DUP    | KEY_CPAD_UP,    ///< D-Pad Up or Circle Pad Up
            KEY_DOWN  = KEY_DDOWN  | KEY_CPAD_DOWN,  ///< D-Pad Down or Circle Pad Down
            KEY_LEFT  = KEY_DLEFT  | KEY_CPAD_LEFT,  ///< D-Pad Left or Circle Pad Left
            KEY_RIGHT = KEY_DRIGHT | KEY_CPAD_RIGHT, ///< D-Pad Right or Circle Pad Right
        };

        struct InputState {
            u32 kDown;
            u32 kHeld;
            u32 kUp;
        }__attribute__((packed));
    }

    namespace Scene {

        enum TextAlign {
            ALIGN_LEFT = 0,
            ALIGN_RIGHT = 4,
            ALIGN_CENTER = 8
        };

        enum RenderType {
            RENDER_PLANE,
            RENDER_CUBE,
            RENDER_MODEL,
            RENDER_EMPTY,
            RENDER_TEXT,
            RENDER_CANVAS
        };

        enum ShaderInputType {
            SHADER_INPUT_TYPE_TEXTURE2D,
            SHADER_INPUT_TYPE_FLOAT,
            SHADER_INPUT_TYPE_VEC2,
            SHADER_INPUT_TYPE_VEC3,
            SHADER_INPUT_TYPE_VEC4,
        };

        struct ShaderInput {
            ShaderInputType type;
            std::optional<std::string> texturePath;
            std::optional<glm::vec4> numberData;
        };

        struct Material {
            std::string shaderPath;
            std::vector<ShaderInput> shaderInputs; // each index corresponds to a descriptor in a descriptor set starting at binding 1 in the shader (this is becuase the projection and view matrices get the first binding)
        };

        class RenderData {
        public:
            RenderType type;
            std::shared_ptr<std::string> model;
            std::vector<std::shared_ptr<Material>> materials;

            RenderData();
            RenderData Plane(std::weak_ptr<Material> material);
            RenderData Cube(std::weak_ptr<Material> material);
            RenderData Model(std::weak_ptr<std::string> model, std::weak_ptr<Material> material);
        };

        class Object {
        private:
            glm::mat4x4 transform;
            bool isDirty;
        public:
            Object();
            Object(RenderData, glm::vec3, glm::vec3, glm::vec3, Scripting::IScript* script = nullptr);
            std::weak_ptr<Object> self;
            RenderData data;
            glm::vec3 position;
            glm::vec3 rotation;
            glm::vec3 scale;
            Scripting::IScript* script = nullptr;
            std::weak_ptr<Object> parent;
            std::vector<std::shared_ptr<Object>> children = {};

            static std::shared_ptr<Object> Create();
            static std::shared_ptr<Object> Create(RenderData, glm::vec3, glm::vec3, glm::vec3, Scripting::IScript* script = nullptr);

            ~Object();
            void setPosition(glm::vec3);
            void setRotation(glm::vec3);
            void setScale(glm::vec3);
            glm::mat4x4 getTransform();
            bool isVisible(Math::Frustum*);
            bool addChild(std::shared_ptr<Object>);
            void markDirty();
            void tickAll(SceneCtx*, Input::InputState*);
        };

        class Camera {
        private:
            glm::mat4x4 transform;
            Math::Frustum frustum;
        public:
            glm::vec3 position;
            glm::vec3 rotation;
            float zNear;
            float zFar;
            float fovY;
            float aspect;
            bool isDirty;
            void(*tick)(Camera*, SceneCtx*, Input::InputState*);
            Camera(glm::vec3, glm::vec3, float, float, float, float, void(*)(Camera*, SceneCtx*, Input::InputState*));
            void LookAt(glm::vec3 target);
            glm::mat4x4 getTransform();
            Math::Frustum generateFrustum();
        };

        struct SceneCtx {
            std::chrono::microseconds deltaTime;
            std::chrono::high_resolution_clock::time_point tickStart;
            Camera* camera;
            u32 animationTimer; // ticks once every ms
            AudioInterface* audio;
            void(*softPanic)(std::string);
            AssetProviderInterface* assetProvider;
        };

        class Scene {
        public:
            std::shared_ptr<Object> root;
            std::array<std::shared_ptr<UI::UIObject>, 256> uiObjects;
            std::vector<std::shared_ptr<Material>> materials;
            std::vector<std::shared_ptr<std::string>> models;
            SceneCtx ctx;
            Engine* engine = nullptr;
            Scene(Camera*, AudioInterface*, AssetProviderInterface*);
            ~Scene();
            void tick(Input::InputState*);
            std::weak_ptr<Material> registerMaterial(Material mat);
            std::weak_ptr<std::string> registerModel(std::string path);

            bool fromJson(std::string path, Scene* output);
        };
    }

    namespace Message {
        enum MessageType {
            MESSAGE_PANIC,
            MESSAGE_READY,
            MESSAGE_END,
            MESSAGE_BEGIN_LOADING_SCREEN,
            MESSAGE_STOP_LOADING_SCREEN,
            MESSAGE_LOADING_SCREEN_STOPPED
        };

        struct GameBoundMessage {
            MessageType type;
        };

        struct RenderBoundMessage {
            MessageType type;
        };
    }

    /// @brief The engine itself
    class Engine {
    private:
        std::string platform;
        void(*softPanic)(std::string);
        AssetProviderInterface* assetProvider;
        std::vector<std::string, Scripting::IScript> scripts;
        Scene::Scene* scene = nullptr;

        void attachScene(Scene::Scene* scene);
        void detachScene();
    public:
        /// @brief Runs init 
        /// @param platform a string representing the platform the game is running on
        /// @param softPanic this will be called if the program cannot continue but can safely exit without causing a full panic
        Engine(std::string platform, void(*softPanic)(std::string), Scene::Scene* scene, AssetProviderInterface* assetProvider);
        /// @brief one tick of app logic, should be called on a loop, returns true if app should exit
        /// @param inputState the current input state of the system
        /// @return bool
        bool update(Input::InputState);
        /// @brief Register a script
        void registerScript(std::string name, Scripting::IScript script);
    };
}

#define CITRA_ENGINE
#endif