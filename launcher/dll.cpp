#include <windows.h>

#include "utils/log.h"
#include "hooks/media.hpp"
#include "hooks/readfile.hpp"
#include "hooks/getfilesizeex.hpp"
#include "hooks/findentry.hpp"
#include "hooks/readdisk.hpp"
#include "hooks/movieplay.hpp"
#include "hooks/createwindowexa.hpp"

#include "anonymouscode_data/src/lib.rs.h"

void Init() {
    bool is_debug = kdata::is_debug_mode();

    const auto* LOG_FILE = L"log.txt";
    const auto LOG_LEVEL = is_debug ? LogLevel::Debug : LogLevel::Silent;

    Logger::GetInstance().init(LOG_FILE, LOG_LEVEL);

    Logger::Info("AnonymousCode CHS Started");

    kdata::say_hello();

    auto ret = (uint8_t)kdata::load_resource_dat();
    if (ret != 0) {
        Logger::Debug(std::format("Load Resource Data Failed: {}", ret));
    }

    Media::HookSHCreateMemStream::g_obj.InitHook();

    File::HookReadFile::g_obj.InitHook();
//    File::HookGetFilesizeEx::g_obj.InitHook();

    Window::HookCreateWindowExA::g_obj.InitHook();

    Game::HookFindEntry::g_obj.InitHook();
    Game::HookReadDisk::g_obj.InitHook();
    Game::HookMoviePlay::g_obj.InitHook();
}

void Destroy() {
    kdata::release_resource();
}


[[maybe_unused]]
BOOL WINAPI
DllMain(HINSTANCE hWnd, DWORD reason, LPVOID lpReserved) {
    switch (reason) {
        case DLL_PROCESS_ATTACH:
            Init();
            break;
        case DLL_PROCESS_DETACH:
            Destroy();
            break;
        case DLL_THREAD_ATTACH:
        case DLL_THREAD_DETACH:
        default:
            break;
    }

    return TRUE;
}

// Detour needs at least on exported function
extern "C"
__declspec(dllexport)
int WINAPI AnonymousCodeCHS() { return 0; }
