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
add_executable(game_chs
        resources/app.o
        launcher.cpp
)
target_include_directories(game_chs PUBLIC ${DETOURS_INCLUDE_DIRS})
target_link_libraries(game_chs PUBLIC ${DETOURS_LIBRARY})

set_property(TARGET game_chs PROPERTY INTERPROCEDURAL_OPTIMIZATION TRUE)


add_library(anonymouscode_chs SHARED
        dll.cpp
        utils/kutils.cpp
        utils/log.cpp
        #        config.cpp
)
set_property(TARGET anonymouscode_chs PROPERTY INTERPROCEDURAL_OPTIMIZATION TRUE)

target_include_directories(anonymouscode_chs PRIVATE utils hooks)

target_include_directories(anonymouscode_chs PRIVATE ${DETOURS_INCLUDE_DIRS})
target_link_libraries(anonymouscode_chs PRIVATE ${DETOURS_LIBRARY})

#find_package(nlohmann_json CONFIG REQUIRED)
#target_link_libraries(anonymouscode_chs PRIVATE nlohmann_json::nlohmann_json)

set(KDATA_INCLUDE_DIRS
        "${CMAKE_SOURCE_DIR}"
        "${CMAKE_SOURCE_DIR}/../data/target/i686-pc-windows-msvc/cxxbridge"
)

set(KDATA_LIBS kernel32.lib advapi32.lib bcrypt.lib ntdll.lib userenv.lib ws2_32.lib msvcrt.lib)
if (CMAKE_BUILD_TYPE STREQUAL "Release")
    set(KDATA_LIBS ${KDATA_LIBS} "${CMAKE_SOURCE_DIR}/../data/target/i686-pc-windows-msvc/release/kdata.lib")
else()
    set(KDATA_LIBS ${KDATA_LIBS} "${CMAKE_SOURCE_DIR}/../data/target/i686-pc-windows-msvc/debug/kdata.lib")
endif()

target_include_directories(anonymouscode_chs PRIVATE ${KDATA_INCLUDE_DIRS})
target_link_libraries(anonymouscode_chs PRIVATE ${KDATA_LIBS})
