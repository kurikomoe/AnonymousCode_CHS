#include <windows.h>
#include <string>

#include <detours/detours.h>


int main() {
    const char* dll_path = R"(D:\Projects\1.Galgames\AnonymousCode\Launcher\launcher\build\Debug\test.dll)";
    const char* working_path = R"(D:\Projects\1.Galgames\AnonymousCode\AC)";
    const char* target_exe_path = R"(D:\Projects\1.Galgames\AnonymousCode\AC\game.exe)";

    STARTUPINFOA si;
    PROCESS_INFORMATION pi;

    ZeroMemory(&si, sizeof(STARTUPINFOA));
    ZeroMemory(&pi, sizeof(PROCESS_INFORMATION));

    si.cb = sizeof(si);

    DWORD dwFlags = CREATE_DEFAULT_ERROR_MODE | CREATE_SUSPENDED;

    SetLastError(0);
    if (!DetourCreateProcessWithDllEx(
            target_exe_path,
            nullptr,
            nullptr,
            nullptr,
            true,
            dwFlags,
            nullptr,
            working_path,
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
