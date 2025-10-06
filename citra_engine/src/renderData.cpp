#include "citra_engine/citra_engine.hpp"

using namespace CitraEngine::Scene;

RenderData::RenderData() : type(RENDER_EMPTY), model(nullptr), material(nullptr) {}

RenderData RenderData::Plane(std::weak_ptr<Material> material) {
    RenderData output;
    output.type = RENDER_PLANE;
    output.material = material.lock();
    return output;
}

RenderData RenderData::Cube(std::weak_ptr<Material> material) {
    RenderData output;
    output.type = RENDER_CUBE;
    output.material = material.lock();
    return output;
}

RenderData RenderData::Model(std::weak_ptr<std::string> model, std::weak_ptr<Material> material) {
    RenderData output;
    output.type = RENDER_MODEL;
    output.model = model.lock();
    output.material = material.lock();
    return output;
}

void RenderData::changeMaterial(std::weak_ptr<Material> material) {
    this->material = material.lock();
}