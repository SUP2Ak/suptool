#include "pch.h"
#include <iostream>
#include <vector>
#include <mutex>
#include <unordered_map>
#include <filesystem>
#include <future>  // Pour std::async
#include <string.h>  // Pour les opérations sur C-string

namespace fs = std::filesystem;

// Mutex pour synchroniser l'accès aux résultats
std::mutex resultsMutex;
std::unordered_map<std::string, std::vector<std::string>> cache;  // Cache pour les résultats de recherche

// Convertir std::wstring en std::string en utilisant MultiByteToWideChar
std::string wstringToString(const std::wstring& wstr) {
    int size_needed = WideCharToMultiByte(CP_UTF8, 0, &wstr[0], (int)wstr.size(), nullptr, 0, nullptr, nullptr);
    std::string strTo(size_needed, 0);
    WideCharToMultiByte(CP_UTF8, 0, &wstr[0], (int)wstr.size(), &strTo[0], size_needed, nullptr, nullptr);
    return strTo;
}

// Fonction pour récupérer tous les disques durs du PC
std::vector<std::wstring> getAllDrives() {
    std::vector<std::wstring> drives;
    DWORD driveMask = GetLogicalDrives();
    for (wchar_t letter = 'A'; letter <= 'Z'; ++letter) {
        DWORD bit = 1 << (letter - 'A');
        if (driveMask & bit) {
            std::wstring drive = { letter, L':', L'\\' }; // Utiliser un wide char
            if (GetDriveType(drive.c_str()) == DRIVE_FIXED) { // Vérifie que c'est un disque dur physique
                drives.push_back(drive);
            }
        }
    }
    return drives;
}

bool isDriveAccessible(const fs::path& path) {
    try {
        return fs::exists(path) && fs::is_directory(path);
    }
    catch (const std::filesystem::filesystem_error&) {
        return false;
    }
}

bool isSystemDirectory(const fs::path& dir) {
    // Vérifie si le répertoire est un répertoire système ou protégé
    std::string dirStr = dir.string();
    return (dirStr.find("WindowsApps") != std::string::npos);
}

// Recherche des fichiers dans un répertoire donné
void searchFiles(const fs::path& dirPath, const std::string& keyword, std::vector<std::string>& results) {
    try {
        for (const auto& entry : fs::recursive_directory_iterator(dirPath, fs::directory_options::skip_permission_denied)) {
            if (entry.is_directory()) {
                // Ignorer les répertoires protégés
                if (isSystemDirectory(entry.path())) {
                    continue;
                }
            }
            if (entry.path().string().find(keyword) != std::string::npos) {
                std::lock_guard<std::mutex> lock(resultsMutex);
                results.push_back(entry.path().string());
            }
        }
    }
    catch (const std::filesystem::filesystem_error& e) {
        std::cerr << "Erreur lors de la lecture du répertoire " << dirPath << ": " << e.what() << std::endl;
    }
}

extern "C" __declspec(dllexport) void searchFilesInDrives(const char* keyword, char** results, int* resultCount) {
    std::string keywordStr(keyword);
    std::vector<std::wstring> drives = getAllDrives();
    std::vector<std::string> allResults;

    // Recherche dans les disques
    for (const auto& drive : drives) {
        if (!isDriveAccessible(drive)) {
            continue;
        }
        searchFiles(drive, keywordStr, allResults);
    }

    // Remplir les résultats dans le tableau C
    *resultCount = allResults.size();
    for (size_t i = 0; i < allResults.size(); ++i) {
        results[i] = _strdup(allResults[i].c_str()); // Utiliser _strdup pour allouer de la mémoire pour chaque chaîne
    }
}

// Ajoutez une fonction pour libérer la mémoire des résultats côté C++
extern "C" __declspec(dllexport) void freeResults(char** results, int resultCount) {
    for (int i = 0; i < resultCount; ++i) {
        free(results[i]);
    }
}
