//
// Created by Kuriko on 2023/9/18.
//

#ifndef HOOKTEST_GETFILESIZEEX_HPP
#define HOOKTEST_GETFILESIZEEX_HPP

#include <windows.h>

#include "utils/kutils.h"
#include "utils/log.hpp"
#include "hooks/hookbase.h"


namespace File::HookGetFilesizeEx {
    using FnType = decltype(&GetFileSizeEx);

    static class GetFileSizeExHook: public HookBase<FnType> {
    public:
        GetFileSizeExHook() : HookBase("Kernel32.dll", "GetFileSizeEx") {}

        void InitHook () override { BaseInitHook(DetourFunction); }

        static
        BOOL WINAPI
        DetourFunction(HANDLE hFile, PLARGE_INTEGER lpFileSize);
    } g_obj;

    BOOL WINAPI
    GetFileSizeExHook::DetourFunction(HANDLE hFile, PLARGE_INTEGER lpFileSize) {
        auto orig_fn = g_obj.GetOrigFnPtr();

        auto filepath = kutils::GetFileNameByHandle(hFile);

        auto msg = std::format(L"Get File Size: {}", filepath);
        Logger::GetInstance().debug(msg);

        return orig_fn(hFile, lpFileSize);
    }
}



#endif //HOOKTEST_GETFILESIZEEX_HPP
