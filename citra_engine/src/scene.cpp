#include "citra_engine/citra_engine.hpp"
#include <glm/gtc/matrix_transform.hpp>
#include <glm/gtc/quaternion.hpp>
#include "citra_engine/error.hpp"

using namespace CitraEngine::Scene;

Scene::Scene(Camera* camera, AudioInterface* audio, AssetProviderInterface* assetProvider) {
    root = Object::Create();
    std::fill(uiObjects.begin(), uiObjects.end(), nullptr);
    this->ctx = SceneCtx {
        .deltaTime = std::chrono::microseconds(),
        .tickStart = std::chrono::high_resolution_clock::now(),
		.camera = camera,
        .animationTimer = 0,
        .audio = audio,
        .softPanic = nullptr,
        .assetProvider = assetProvider
    };
}

Scene::~Scene() {
	delete this->ctx.camera;
}

void Scene::tick(Input::InputState* inputState) {
    this->ctx.deltaTime = std::chrono::duration_cast<std::chrono::microseconds>(std::chrono::high_resolution_clock::now() - this->ctx.tickStart);
    this->ctx.tickStart = std::chrono::high_resolution_clock::now();
    this->root->tickAll(&this->ctx, inputState);
    for (int i = 0; i < this->uiObjects.size(); i++) {
        if (this->uiObjects[i].get() != nullptr && (*this->uiObjects[i]).tick != nullptr) {
            (*this->uiObjects[i]).tick(&(*this->uiObjects[i]), &this->ctx, inputState);
        }
    }
    if (this->ctx.camera->tick != nullptr) {
        this->ctx.camera->tick(this->ctx.camera, &this->ctx, inputState);
    }
    this->ctx.animationTimer += this->ctx.deltaTime.count();
}

Object::Object() : data(RenderData()), position({0, 0, 0}), rotation({0, 0, 0}), scale({1, 1, 1}), script(nullptr), isDirty(true) {}

Object::~Object() {
    
}

Object::Object(RenderData data, glm::vec3 position = glm::vec3{0, 0, 0}, glm::vec3 rotation = glm::vec3{0, 0, 0}, glm::vec3 scale = glm::vec3{1, 1, 1}, Scripting::IScript* script = nullptr) : 
data(data), position{position.x, position.y, position.z}, rotation{rotation.x, rotation.y, rotation.z}, scale{scale.x, scale.y, scale.z}, script(script), isDirty(true) {}

void Object::setPosition(glm::vec3 position) {
    this->position.x = position.x;
    this->position.y = position.y;
    this->position.z = position.z;
    this->isDirty = true;
}

void Object::setRotation(glm::vec3 rotation) {
    this->rotation.r = rotation.x;
    this->rotation.g = rotation.y;
    this->rotation.b = rotation.z;
    this->isDirty = true;
}

void Object::setScale(glm::vec3 scale) {
    this->scale.x = scale.x;
    this->scale.y = scale.y;
    this->scale.z = scale.z;
    this->isDirty = true;
}

glm::mat4x4 Object::getTransform() {
    if (this->isDirty) {
        if (parent.lock() == nullptr) {
            this->transform = glm::mat4x4(1.0f);
        }
        else {
            this->transform = parent.lock()->getTransform();
        }
        this->transform = glm::translate(this->transform, this->position);
        this->transform = glm::rotate(this->transform, this->rotation.x, glm::vec3(1.0, 0.0, 0.0));
        this->transform = glm::rotate(this->transform, this->rotation.y, glm::vec3(0.0, 1.0, 0.0));
        this->transform = glm::rotate(this->transform, this->rotation.z, glm::vec3(0.0, 0.0, 1.0));
        this->transform = glm::scale(this->transform, this->scale);
        this->isDirty = false;
    }
    return this->transform;
}

bool Object::isVisible(Math::Frustum* frustum) {
    return true;
}

std::shared_ptr<Object> Object::Create() {
    auto output = std::make_shared<Object>();
    output->self = output;
    return output;
}

std::shared_ptr<Object> Object::Create(RenderData data, glm::vec3 pos, glm::vec3 rot, glm::vec3 scale, Scripting::IScript* script = nullptr) {
    auto output = std::make_shared<Object>(data, pos, rot, scale, script);
    output->self = output;
    return output;
}

bool Object::addChild(std::shared_ptr<Object> object) {
    if (object.get() != nullptr) {
        if (object->parent.lock() == nullptr) {
            object->parent = this->self;
            this->children.push_back(object);
        }
        else {
            setErr("Cannot add child that already has a parent");
            return false;
        }
    }
    else {
        setErr("Cannot add child that is null");
        return false;
    }
    return true;
}

void Object::markDirty() {
    this->isDirty = true;
    for (int i = 0; i < this->children.size(); i++) {
        if (this->children[i] != nullptr) {
            this->children[i]->markDirty();
        }
    }
}

void Object::tickAll(SceneCtx* ctx, CitraEngine::Input::InputState* inputState) {
    if (this->script != nullptr) {
        this->script->OnTick(this, ctx, inputState);
    }
    for (int i = 0; i < this->children.size(); i++) {
        if (this->children[i] != nullptr) {
            this->children[i]->tickAll(ctx, inputState);
        }
    }
}

std::weak_ptr<Material> Scene::registerMaterial(Material mat) {
    this->materials.push_back(std::make_shared<Material>(mat));
    return this->materials.back();
}

std::weak_ptr<std::string> Scene::registerModel(std::string path) {
    this->models.push_back(std::make_shared<std::string>(path));
    return this->models.back();
}