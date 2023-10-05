//
// Created by Kuriko on 2023/9/15.
//

#ifndef HOOKTEST_READFILE_HPP
#define HOOKTEST_READFILE_HPP

#include <windows.h>
#include <filesystem>

#include "utils/kutils.h"
#include "utils/log.h"
#include "hooks/hookbase.h"

#include "anonymouscode_data/src/lib.rs.h"
#include "rust/cxx.h"

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

        OVERLAPPED localOverlapped {};

        if (lpOverlapped != nullptr) {
            offset = lpOverlapped->Offset;
            offset_high = lpOverlapped->OffsetHigh;
        }

        if (!filepath.starts_with(g_obj.game_path_)) {
            return orig_fn(hFile, lpBuffer, nNumberOfBytesToRead, lpNumberOfBytesRead, lpOverlapped);
        }

        logger.Debug(std::format(
                L"[Raw] File: {}, offset: {}:{}, size: {}",
                filepath, offset_high, offset, nNumberOfBytesToRead));

        // Check media redirect
        auto uid = offset_high;

        kdata::MappingInfo mappingInfo{};

        try {
            if (uid & kutils::UID_MARK) {
                mappingInfo = kdata::get_mapping_info_by_idx(uid & (kutils::UID_MARK - 1));
                nNumberOfBytesToRead = mappingInfo.size;

                auto resource_dat = kdata::get_resource_dat_file();

                HANDLE hFileNew = CreateFileA(resource_dat.c_str(),
                                              GENERIC_READ,
                                              FILE_SHARE_READ,
                                              nullptr,
                                              OPEN_EXISTING,
                                              FILE_ATTRIBUTE_NORMAL,
                                              nullptr);
                if (hFileNew == INVALID_HANDLE_VALUE) {
                    logger.Error(std::format(L"CreateFileA failed, fallback to original, errCode: {}", GetLastError()));
                    return orig_fn(hFile, lpBuffer, nNumberOfBytesToRead, lpNumberOfBytesRead, lpOverlapped);
                }
                OVERLAPPED localOverlappedNew {};
                if (lpOverlapped == nullptr) {
                    lpOverlapped = &localOverlappedNew;
                }

                lpOverlapped->OffsetHigh = mappingInfo.offset >> 32;
                lpOverlapped->Offset = mappingInfo.offset & 0xFFFFFFFF;

                logger.Debug(std::format(
                        "[Redir] File: {}, offset: {}:{}, size: {}",
                        resource_dat.c_str(), lpOverlapped->OffsetHigh, lpOverlapped->Offset, nNumberOfBytesToRead));

                auto ret = orig_fn(hFileNew, lpBuffer, nNumberOfBytesToRead, lpNumberOfBytesRead, lpOverlapped);
                kdata::decrypt_buffer(
                        rust::Slice((uint8_t*)lpBuffer, (size_t)nNumberOfBytesToRead),
                        mappingInfo);
                return ret;
            } else {

            }
        } catch (const std::exception &e) {
//            logger.Debug(std::format(L"No mapping info for file {}", filepath));
        }

        return orig_fn(hFile, lpBuffer, nNumberOfBytesToRead, lpNumberOfBytesRead, lpOverlapped);
    }
}


#endif //HOOKTEST_READFILE_HPP
