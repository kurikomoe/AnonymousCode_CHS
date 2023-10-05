#pragma comment(linker, "/subsystem:\"Windows\" /entry:\"mainCRTStartup\"")
#include <windows.h>
#include <string>
#include <filesystem>

#include <detours/detours.h>
#include <iostream>

namespace fs = std::filesystem;


int main() {
    WCHAR working_path[MAX_PATH];
    GetModuleFileNameW(nullptr, working_path, MAX_PATH);

    fs::path path(working_path);

    std::wcout << path.parent_path() << std::endl;

    LPCSTR dll_path = "anonymouscode_chs.dll";
    LPCWSTR target_exe_path = L"game.exe";

    STARTUPINFOW si;
    PROCESS_INFORMATION pi;

    ZeroMemory(&si, sizeof(si));
    ZeroMemory(&pi, sizeof(pi));

    si.cb = sizeof(si);

    DWORD dwFlags = CREATE_DEFAULT_ERROR_MODE | CREATE_SUSPENDED;

    // Change the working directory to the directory containing the DLL.
    SetCurrentDirectoryW(path.parent_path().wstring().c_str());

    SetLastError(0);
    if (!DetourCreateProcessWithDllExW(
            target_exe_path,
            nullptr,
            nullptr,
            nullptr,
            true,
            dwFlags,
            nullptr,
            nullptr,
            &si,
            &pi,
            dll_path,
            nullptr)) {
        auto dwError = GetLastError();
        printf("DetourCreateProcessWithDllEx failed with error %ld\n", dwError);

        ExitProcess(9009);
    }

    ResumeThread(pi.hThread);

    WaitForSingleObject(pi.hProcess, INFINITE);

    return 0;
}
