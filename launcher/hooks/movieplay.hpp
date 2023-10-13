//
// Created by Kuriko on 2023/9/17.
//

#ifndef HOOKTEST_MOVIEPLAY_HPP
#define HOOKTEST_MOVIEPLAY_HPP

#include <Shlwapi.h>
#include <windows.h>
#include <filesystem>

#include "utils/log.h"
#include "hooks/hookbase.h"
#include "anonymouscode_data/src/lib.rs.h"
#include "rust/cxx.h"

namespace Game::HookMoviePlay {
    using namespace data;

    static auto logger = Logger::GetLogger("Game::HookMoviePlay");

    using TFnType = void (__thiscall *)(void* This, char* moviePath);

    void __fastcall DetourFunction(void* This, DWORD EDX, char* moviePath);
    using HFnType = decltype(DetourFunction);

    static class MoviePlayHook : public HookAddressBase<TFnType, HFnType> {
    public:
        MoviePlayHook() : HookAddressBase(nullptr, 0x11c1e0) {
            logger.Debug(L"Init");
        }

        void InitHook() override {
            BaseInitHook(DetourFunction);
            logger.Debug(std::format("MoviePlay Address: 0x{:x}", (intptr_t) this->GetOrigFnPtr()));
        }

    } g_obj;


    void __fastcall DetourFunction(void* This, DWORD EDX, char* moviePath) {
        auto orig_fn = g_obj.GetOrigFnPtr();

        auto path = std::filesystem::path(moviePath);
        auto filename = path.filename().string();

        try {
//            auto mapping_info = kdata::get_mapping_info(filename);
            auto new_file_path = kdata::locate_movie(filename);
            const char* newMoviePath = new_file_path.c_str();

            logger.Debug(std::format("load video: {} from {}", moviePath, newMoviePath));
            orig_fn(This, const_cast<char*>(newMoviePath));

            return;
        } catch (const std::exception& e) {
            logger.Debug(e.what());
        }

        orig_fn(This, moviePath);
    }
}



#endif //HOOKTEST_MOVIEPLAY_HPP
