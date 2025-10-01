#include <GL/gl.h>
#include <SDL3/SDL.h>
#include <iostream>
#include "exitfuncs.hpp"
#include "controllermap.hpp"
#include "citra_engine/citra_engine.hpp"
#include "citra_engine/quikmath.hpp"
#include "citra_engine/error.hpp"
#include "render.hpp"
#include "assetProvider.hpp"
#include "audio.hpp"

SDL_Window* mainWindow = nullptr;
AudioEngine* audioEngine = nullptr;

void exitGame(int exitCode) {
    if (audioEngine != nullptr) {
        delete audioEngine;
    }
    gfxQuit();
    if (mainWindow) {
        SDL_DestroyWindow(mainWindow);
    }
    SDL_Quit();
    exit(exitCode);
}

void exitWithErrorWindow(std::string err) {
    if (!SDL_ShowSimpleMessageBox(SDL_MESSAGEBOX_ERROR, "Fatal Error", err.c_str(), nullptr)) {
        // If the message box fails, we can only log the error
        std::cerr << "Error: " << err << std::endl;
    }
    exitGame(-1);
}

int main(int argc, char** argv) {
    if (!initControllerMap()) {
        exitWithErrorWindow("Failed to initialize controller map:" + getErr());
    }
    
    if (!SDL_Init(SDL_INIT_VIDEO | SDL_INIT_AUDIO | SDL_INIT_JOYSTICK | SDL_INIT_HAPTIC)) {
        exitWithErrorWindow(std::string("Failed to initialize SDL: ") + SDL_GetError());
    }
    
    SDL_Window* mainWindow = SDL_CreateWindow("Amius Adventure", 800, 600, SDL_WINDOW_VULKAN | SDL_WINDOW_INPUT_FOCUS | SDL_WINDOW_RESIZABLE);
    if (!mainWindow) {
        exitWithErrorWindow(std::string("Failed to create SDL mainWindow: ") + SDL_GetError());
    }
    float aspectRatio = 800.0f / 600.0f; // Default aspect ratio
    
    SDL_SetWindowPosition(mainWindow, SDL_WINDOWPOS_CENTERED, SDL_WINDOWPOS_CENTERED);
    
    if (!initGfx(mainWindow)) {
        exitWithErrorWindow("Failed to initialize graphics: " + getErr());
    }
    
    SDL_ShowWindow(mainWindow);
    
    audioEngine = new AudioEngine();
    
    CitraEngine::Scene::Camera* topCamera = new CitraEngine::Scene::Camera(glm::vec3{0, 0, 0}, glm::vec3{0, 0, 0}, 0.01f, 1000.0f, DegreesToRadians(40.0f), aspectRatio, nullptr);
    CitraEngine::Scene::Camera* bottomCamera = new CitraEngine::Scene::Camera(glm::vec3{0, 0, 0}, glm::vec3{0, 0, 0}, 0.01f, 1000.0f, DegreesToRadians(40.0f), aspectRatio, nullptr);
    
    CitraEngine::Scene::Scene* topScene = new CitraEngine::Scene::Scene(topCamera, audioEngine);
    CitraEngine::Scene::Scene* bottomScene = new CitraEngine::Scene::Scene(bottomCamera, audioEngine);
    
    AssetProvider* assetProvider = new AssetProvider();
    CitraEngine::Engine* engine = new CitraEngine::Engine("PC", exitWithErrorWindow, topScene, bottomScene, assetProvider);
    
    bool running = true;
    bool useSecondScene = false;
    SDL_Event event;
    bool minimized = false;
    while (running) {
        tickControllerMap();
        while (SDL_PollEvent(&event)) {
            if (event.type == SDL_EVENT_KEY_DOWN) {
                processSDLKeyDownEvent(event.key.key);
            }
            else if (event.type == SDL_EVENT_KEY_UP) {
                processSDLKeyUpEvent(event.key.key);
            }
            else if (event.type == SDL_EVENT_WINDOW_FOCUS_LOST) {
                processSDLLostFocusEvent();
            }
            else if (event.type == SDL_EVENT_WINDOW_RESIZED || event.type == SDL_EVENT_WINDOW_METAL_VIEW_RESIZED) {
                setFramebufferResized();
            }
            else if (event.type == SDL_EVENT_WINDOW_MINIMIZED) {
                minimized = true;
            }
            else if (event.type == SDL_EVENT_WINDOW_RESTORED) {
                minimized = false;
            }
            else if (event.type == SDL_EVENT_QUIT) {
                running = false;
            }
        }
        
        if (engine->update(getInputState(), topScene, bottomScene)) {
            break;
        }

        if (!minimized) { // dont render if window is minimized
            if (flipScreen()) {
                useSecondScene = !useSecondScene;
            }
            gfxUpdate(useSecondScene ? bottomScene : topScene, !useSecondScene);
        }
    }

    exitGame(0);
}