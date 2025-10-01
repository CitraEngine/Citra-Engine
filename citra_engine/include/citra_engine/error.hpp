#pragma once
#ifndef CITRA_ENGINE_ERROR
#include <string>

void setErr(std::string);
std::string getErr();

#define CITRA_ENGINE_ERROR
#endif