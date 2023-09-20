//
// Created by Kuriko on 2023/9/15.
//

#ifndef HOOKTEST_READFILE_HPP
#define HOOKTEST_READFILE_HPP

#include <windows.h>

#include "utils/kutils.h"
#include "utils/log.hpp"
#include "hooks/hookbase.h"

namespace File::HookReadFile {
    using FnType = decltype(&ReadFile);

    static auto logger = Logger::GetLogger("File::HookReadFile");

    static class ReadFileHook : public HookBase<FnType> {
    public:
        ReadFileHook() : HookBase("Kernel32.dll", "ReadFile") {
            WCHAR buffer[255];
            GetCurrentDirectoryW(sizeof(buffer), buffer);
            game_path_ = std::wstring(buffer);
            logger.Info(std::format(L"Current game path: {}", game_path_));
        }

        void InitHook() override { BaseInitHook(DetourFunction); }

        static
        BOOL WINAPI
        DetourFunction(HANDLE hFile, LPVOID lpBuffer, DWORD nNumberOfBytesToRead, LPDWORD lpNumberOfBytesRead,
                       LPOVERLAPPED lpOverlapped);

    private:
        std::wstring game_path_;

    } g_obj;

    BOOL WINAPI
    ReadFileHook::DetourFunction(HANDLE hFile, LPVOID lpBuffer, DWORD nNumberOfBytesToRead, LPDWORD lpNumberOfBytesRead,
                                 LPOVERLAPPED lpOverlapped) {
        auto orig_fn = g_obj.GetOrigFnPtr();

        std::wstring filepath = kutils::GetFileNameByHandle(hFile);

        DWORD offset = 0;
        DWORD offset_high = 0;
        if (lpOverlapped != nullptr) {
            offset = lpOverlapped->Offset;
            offset_high = lpOverlapped->OffsetHigh;
        }

        if (!filepath.starts_with(g_obj.game_path_)) {
            return orig_fn(hFile, lpBuffer, nNumberOfBytesToRead, lpNumberOfBytesRead, lpOverlapped);
        }

        logger.Debug(std::format(
                L"[Raw]"
                "File: {}, offset: {}:{}, size: {}",
                filepath, offset_high, offset, nNumberOfBytesToRead));

        if (nNumberOfBytesToRead == 4937136) {
            HANDLE hFileNew = CreateFileW(L"D:/Projects/1.Galgames/AnonymousCode/AC/motion_new.bin",
                                          GENERIC_READ,
                                          FILE_SHARE_READ,
                                          nullptr,
                                          OPEN_EXISTING,
                                          FILE_ATTRIBUTE_NORMAL,
                                          nullptr);
            logger.Debug(std::format(
                    "[Redirect]"
                    "File: {}, offset: {}:{}, size: {}",
                    R"(D:\Projects\1.Galgames\AnonymousCode\AC\motion_new.bin)",
                    offset_high,
                    offset,
                    nNumberOfBytesToRead));
            if (hFileNew == INVALID_HANDLE_VALUE) {
                logger.Error(std::format(L"CreateFileA failed, fallback to original, errCode: {}", GetLastError()));
                return orig_fn(hFile, lpBuffer, nNumberOfBytesToRead, lpNumberOfBytesRead, lpOverlapped);
            }
            return orig_fn(hFileNew, lpBuffer, nNumberOfBytesToRead, lpNumberOfBytesRead, lpOverlapped);
        }

        // Apply file redirection here
        return orig_fn(hFile, lpBuffer, nNumberOfBytesToRead, lpNumberOfBytesRead, lpOverlapped);
    }
}


#endif //HOOKTEST_READFILE_HPP
