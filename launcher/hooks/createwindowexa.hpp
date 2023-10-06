//
// Created by Kuriko on 2023/9/15.
//

#ifndef HOOKTEST_CREATEWINDOWEXA_HPP
#define HOOKTEST_CREATEWINDOWEXA_HPP

#include <windows.h>
#include <filesystem>
#include <tchar.h>

#include "utils/kutils.h"
#include "utils/log.h"
#include "hooks/hookbase.h"

#include "anonymouscode_data/src/lib.rs.h"
#include "rust/cxx.h"

/*
HWND CreateWindowExA(
  [in]           DWORD     dwExStyle,
  [in, optional] LPCSTR    lpClassName,
  [in, optional] LPCSTR    lpWindowName,
  [in]           DWORD     dwStyle,
  [in]           int       X,
  [in]           int       Y,
  [in]           int       nWidth,
  [in]           int       nHeight,
  [in, optional] HWND      hWndParent,
  [in, optional] HMENU     hMenu,
  [in, optional] HINSTANCE hInstance,
  [in, optional] LPVOID    lpParam
);
 */

namespace Window::HookCreateWindowExA {

    HWND  WINAPI
    DetourFunction(
            IN DWORD dwExStyle,
            IN OPTIONAL LPCSTR lpClassName,
            IN OPTIONAL LPCSTR lpWindowName,
            IN DWORD dwStyle,
            IN int X,
            IN int Y,
            IN int nWidth,
            IN int nHeight,
            IN OPTIONAL HWND hWndParent,
            IN OPTIONAL HMENU hMenu,
            IN OPTIONAL HINSTANCE hInstance,
            IN OPTIONAL LPVOID lpParam
    );

    using FnType = decltype(&CreateWindowExA);

    static auto logger = Logger::GetLogger("File::HookCreateWindowExA");

    static class CreateWindowExAHook : public HookBase<FnType> {
    public:
        CreateWindowExAHook() : HookBase("User32.dll", "CreateWindowExA") {}

        void InitHook() override { BaseInitHook(DetourFunction); }


    private:
        std::wstring game_path_;

    } g_obj;


    HWND  WINAPI
    DetourFunction(
            IN DWORD dwExStyle,
            IN OPTIONAL LPCSTR lpClassName,
            IN OPTIONAL LPCSTR lpWindowName,
            IN DWORD dwStyle,
            IN int X,
            IN int Y,
            IN int nWidth,
            IN int nHeight,
            IN OPTIONAL HWND hWndParent,
            IN OPTIONAL HMENU hMenu,
            IN OPTIONAL HINSTANCE hInstance,
            IN OPTIONAL LPVOID lpParam
    ) {
        auto orig_fn = g_obj.GetOrigFnPtr();

        const uint8_t buf[255] = {0};
        const uint8_t append[] = {0x20, 0xa1, 0xb8, 0xc4, 0xe4, 0xc3, 0xfb, 0xd5, 0xdf, 0xba, 0xba, 0xbb, 0xaf, 0xd7, 0xe9, 0xa1, 0xb9};

        const auto sz = strlen(lpWindowName);
        memcpy((void*)buf,  lpWindowName, sz);
        memcpy((void*)(buf + sz), (char*)append, sizeof(append));

        auto hwnd = orig_fn(dwExStyle, lpClassName, (LPCSTR)buf, dwStyle, X, Y, nWidth, nHeight, hWndParent, hMenu, hInstance, lpParam);

        SetWindowTextA(hwnd, (char*)buf);

        return hwnd;
    }
}


#endif //HOOKTEST_CREATEWINDOWEXA_HPP
