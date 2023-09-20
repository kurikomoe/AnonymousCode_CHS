cmake_minimum_required(VERSION 3.20)

set(CMAKE_GENERATOR_PLATFORM "Win32" CACHE STRING "Platform toolset")
set(CMAKE_CXX_STANDARD 20 CACHE STRING "C++ standard")


set(CMAKE_TOOLCHAIN_FILE ${CMAKE_SOURCE_DIR}/vcpkg/scripts/buildsystems/vcpkg.cmake CACHE STRING "Vcpkg toolchain file")
set(CMAKE_EXPORT_COMPILE_COMMANDS CACHE STRING ON)

# --------------------------------------------------------------
project(HookTest VERSION 0.1.0 LANGUAGES C CXX)

# Find detours
find_path(DETOURS_INCLUDE_DIRS "detours/detours.h")
find_library(DETOURS_LIBRARY detours REQUIRED)

# Main, aka target
add_executable(launcher launcher.cpp)
target_include_directories(launcher PUBLIC ${DETOURS_INCLUDE_DIRS})
target_link_libraries(launcher PUBLIC ${DETOURS_LIBRARY})

find_package(nlohmann_json CONFIG REQUIRED)

add_library(test SHARED dll.cpp
        utils/kutils.cpp
        hooks/hookbase.h
        #        config.cpp
)
target_include_directories(test PRIVATE ${DETOURS_INCLUDE_DIRS})
target_include_directories(test PRIVATE utils hooks)
target_link_libraries(test PRIVATE ${DETOURS_LIBRARY})
target_link_libraries(test PRIVATE nlohmann_json::nlohmann_json)
