//
// Created by Kuriko on 2023/9/19.
//

#ifndef HOOKTEST_READDISK_HPP
#define HOOKTEST_READDISK_HPP

#include <Shlwapi.h>
#include <windows.h>

#include "utils/log.h"
#include "hooks/hookbase.h"
#include "hooks/data/data.h"

namespace Game::HookReadDisk {
    using namespace data;

    static auto logger = Logger::GetLogger("Game::HookReadDisk");

    using pEntry = MAG_FileRead_Entry *;
    using TFnType = void (__thiscall *)(pEntry This);

    void __fastcall DetourFunction(pEntry This);

    using HFnType = decltype(DetourFunction);

    static class FindEntryHook : public HookAddressBase<TFnType, HFnType> {
    public:
        FindEntryHook() : HookAddressBase("game.exe", 0x24d590) { }

        void InitHook() override {
            return;
            logger.Debug(L"HookFindEntry::InitHook called");
            BaseInitHook(DetourFunction);
            logger.Debug(std::format("HookFindEntry::InitHook: Address: 0x{:x}", (intptr_t) this->GetOrigFnPtr()));
        }

    } g_obj;

    std::string BuildStdString(StdString *This) {
        if (This->uiStrLen == 0) {
            return "";
        } else if (This->uiStrLen < 16) {
            return {This->aStr, This->uiStrLen};
        } else {
            return {*((char **) This->aStr), This->uiStrLen};
        }
    }

    void __fastcall DetourFunction(pEntry This) {
        std::string msPackName = BuildStdString(&This->msPackName);

        g_obj.GetOrigFnPtr()(This);

        std::string msFileName = BuildStdString(&This->msFileName);
        std::string msFolderName = BuildStdString(&This->msFolderName);

        logger.Debug(
                std::format(
                        "[Raw]"
                        "msPackName: {} msFileName: {} msFolderName: {} "
                        "size: {}:{}, offset: {}:{}",
                        msPackName, msFileName, msFolderName,
                        This->uiSizeLow, This->uiSizeHigh,
                        This->uiOffsetLow, This->uiOffsetHigh
                ));

        if (msPackName.find("ac_title_sc.psb.m") != std::string::npos) {
            logger.Debug("Redirecting ac_title_sc.psb.m");
            This->uiSizeLow = 4937136;
            This->uiSizeHigh = 0;
            This->uiOffsetLow = 329955128;
            This->uiOffsetHigh = 0;
            Logger::Debug(
                    std::format(
                            "[New]"
                            "msPackName: {} msFileName: {} msFolderName: {} "
                            "size: {}:{}, offset: {}:{}",
                            msPackName, msFileName, msFolderName,
                            This->uiSizeLow, This->uiSizeHigh,
                            This->uiOffsetLow, This->uiOffsetHigh
                    ));
        }
    }
}


#endif //HOOKTEST_READDISK_HPP