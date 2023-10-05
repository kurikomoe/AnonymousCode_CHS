//
// Created by Kuriko on 2023/9/17.
//

#ifndef HOOKTEST_MEDIA_HPP
#define HOOKTEST_MEDIA_HPP

#include <Shlwapi.h>
#include <windows.h>

#include "utils/log.h"
#include "hooks/hookbase.h"

namespace Media::HookSHCreateMemStream {
    using FnType = decltype(&SHCreateMemStream);

    static auto logger = Logger::GetLogger("Media::HookSHCreateMemStream");

    static class SHCreateMemStreamHook : public HookBase<FnType> {
    public:
        SHCreateMemStreamHook() : HookBase("Shlwapi.dll", "SHCreateMemStream") {}

        void InitHook() override { BaseInitHook(DetourFunction); }

        static
        IStream * WINAPI
        DetourFunction(const unsigned char* pInit, UINT cbInit);

    } g_obj;

    IStream * WINAPI
    SHCreateMemStreamHook::DetourFunction(const unsigned char* pInit, UINT cbInit) {
        // Recalculate file header
        size_t size = 0;
        // Hope that header is less than 0xFF
        for (size = 0x10; size <= 0xFF; size++) {
            if (*(pInit+size) == 0x00) {
                break;
            }
        }

        auto ptr = const_cast<unsigned char*>(pInit) + 3;
        *ptr = size;
        logger.Debug(std::format(L"Fix movie header: 0x{:x}", size));

        return g_obj.GetOrigFnPtr()(pInit, cbInit);
    }
}



#endif //HOOKTEST_MEDIA_HPP
