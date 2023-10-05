#include <windows.h>

#include "utils/log.h"
#include "hooks/media.hpp"
#include "hooks/readfile.hpp"
#include "hooks/getfilesizeex.hpp"
#include "hooks/findentry.hpp"
#include "hooks/readdisk.hpp"
#include "hooks/movieplay.hpp"

#include "anonymouscode_data/src/lib.rs.h"

const auto* LOG_FILE = L"log.txt";
//const auto LOG_LEVEL = LogLevel::Debug;
const auto LOG_LEVEL = LogLevel::Silent;

void Init() {
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

    Game::HookFindEntry::g_obj.InitHook();
    Game::HookReadDisk::g_obj.InitHook();
    Game::HookMoviePlay::g_obj.InitHook();
}


[[maybe_unused]]
BOOL WINAPI
DllMain(HINSTANCE hWnd, DWORD reason, LPVOID lpReserved) {
    switch (reason) {
        case DLL_PROCESS_ATTACH:
            Init();
            break;
        case DLL_THREAD_ATTACH:
        case DLL_THREAD_DETACH:
        case DLL_PROCESS_DETACH:
        default:
            break;
    }

    return TRUE;
}

// Detour needs at least on exported function
extern "C"
__declspec(dllexport)
int WINAPI AnonymousCodeCHS() { return 0; }
