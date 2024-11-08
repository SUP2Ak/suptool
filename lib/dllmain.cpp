#include "pch.h"
#include <iostream>
#include <vector>
#include <mutex>
#include <unordered_map>
#include <filesystem>
#include <future>
#include <string>
#include <chrono>
#include <atomic>
#include <thread>
#include <condition_variable>
#include <functional>

namespace fs = std::filesystem;

// Structure pour les options de recherche
struct SearchOptions {
    std::vector<std::string> whitelistExtensions;
    std::vector<std::string> blacklistExtensions;
    std::vector<std::string> whitelistFolders;
    std::vector<std::string> blacklistFolders;
    bool searchInArchives;
    std::chrono::system_clock::time_point dateFrom;
    std::chrono::system_clock::time_point dateTo;
    bool checkCreationDate;
    bool checkModificationDate;
    std::vector<std::wstring> specificDrives;
};

// Structure pour les détails d'un fichier
struct FileDetails {
    std::string path;
    std::string extension;
    uint64_t size;
    std::chrono::system_clock::time_point lastModified;
    std::chrono::system_clock::time_point created;
};

// Structure pour le cache
struct CacheEntry {
    std::vector<FileDetails> files;
    std::chrono::system_clock::time_point timestamp;
};

// Mutex global pour le cache
std::mutex globalCacheMutex;
std::unordered_map<std::string, CacheEntry> globalCache;

// Convertir std::wstring en std::string
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
            std::wstring drive = { letter, L':', L'\\' };
            if (GetDriveType(drive.c_str()) == DRIVE_FIXED) {
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
    std::string dirStr = dir.string();
    return (dirStr.find("WindowsApps") != std::string::npos);
}

class FileSearcher {
private:
    // Variables d'instance
    std::atomic<bool> isSearching;
    std::string currentKeyword;
    SearchOptions options;
    std::function<void(const FileDetails&)> resultCallback;
    std::condition_variable cv;
    std::mutex searchMutex;
    std::vector<std::thread> searchThreads;
    
    // Pour la gestion du refiltering
    std::atomic<bool> needRefiltering;
    std::thread filterThread;
    std::vector<FileDetails> currentResults;
    std::mutex resultsLock;

    static const int CACHE_EXPIRATION_MINUTES = 15;

public:
    FileSearcher() : isSearching(false), needRefiltering(false) {}
    
    ~FileSearcher() {
        stopSearch();
    }

    void startSearch(const std::string& keyword, 
                    const SearchOptions& searchOptions,
                    std::function<void(const FileDetails&)> callback) {
        std::lock_guard<std::mutex> lock(searchMutex);
        stopSearch();
        
        currentKeyword = keyword;
        options = searchOptions;
        resultCallback = callback;
        isSearching = true;
        needRefiltering = false;

        // Démarrer les threads de recherche
        startSearchThreads();
    }

    void stopSearch() {
        isSearching = false;
        cv.notify_all();
        
        for(auto& thread : searchThreads) {
            if(thread.joinable()) thread.join();
        }
        searchThreads.clear();
        
        if(filterThread.joinable()) filterThread.join();
    }

    void updateKeyword(const std::string& newKeyword) {
        std::lock_guard<std::mutex> lock(searchMutex);
        if(currentKeyword == newKeyword) return;
        
        currentKeyword = newKeyword;
        needRefiltering = true;
        
        // Démarrer le thread de refiltering
        startFilteringThread();
    }

private:
    size_t calculateOptimalThreadCount(const fs::path& directory) {
        size_t directorySize = 0;
        try {
            for(const auto& entry : fs::recursive_directory_iterator(directory)) {
                directorySize += fs::file_size(entry);
            }
        } catch(...) {}
        
        return std::max<size_t>(1, std::min<size_t>(
            std::thread::hardware_concurrency(),
            directorySize / (1024 * 1024 * 100)  // 100MB par thread
        ));
    }

    void startSearchThreads() {
        auto drives = getAllDrives();
        for(const auto& drive : drives) {
            if(!isDriveAccessible(drive)) continue;
            
            size_t threadCount = calculateOptimalThreadCount(drive);
            for(size_t i = 0; i < threadCount; ++i) {
                searchThreads.emplace_back(&FileSearcher::searchWorker, this, drive, i, threadCount);
            }
        }
    }

    FileDetails getFileDetails(const fs::directory_entry& entry) {
        FileDetails details;
        details.path = entry.path().string();
        details.extension = entry.path().extension().string();
        
        try {
            details.size = entry.file_size();
            auto ftime = entry.last_write_time();
            details.lastModified = std::chrono::system_clock::now() + 
                (ftime - fs::file_time_type::clock::now());
            details.created = details.lastModified;
        } catch(...) {
            details.size = 0;
            details.lastModified = std::chrono::system_clock::now();
            details.created = std::chrono::system_clock::now();
        }
        
        return details;
    }

    bool matchesCriteria(const fs::directory_entry& entry, const SearchOptions& opts) {
        // Vérifier les extensions
        if(!opts.whitelistExtensions.empty()) {
            auto ext = entry.path().extension().string();
            if(std::find(opts.whitelistExtensions.begin(), 
                        opts.whitelistExtensions.end(), ext) == opts.whitelistExtensions.end()) {
                return false;
            }
        }

        // Vérifier les dossiers blacklistés
        for(const auto& blacklistedFolder : opts.blacklistFolders) {
            if(entry.path().string().find(blacklistedFolder) != std::string::npos) {
                return false;
            }
        }

        return true;
    }

    void searchWorker(const fs::path& drive, size_t threadId, size_t totalThreads) {
        try {
            for(const auto& entry : fs::recursive_directory_iterator(drive, 
                fs::directory_options::skip_permission_denied)) {
                
                if(!isSearching) return;

                if(!matchesCriteria(entry, options)) continue;

                FileDetails details = getFileDetails(entry);
                
                if(details.path.find(currentKeyword) != std::string::npos) {
                    // Ajouter au cache global
                    updateCache(details);
                    
                    // Notifier le callback
                    if(resultCallback) resultCallback(details);
                    
                    // Ajouter aux résultats actuels
                    std::lock_guard<std::mutex> lock(resultsLock);
                    currentResults.push_back(details);
                }
            }
        } catch(const std::exception& e) {
            std::cerr << "Error in search worker: " << e.what() << std::endl;
        }
    }

    void updateCache(const FileDetails& details) {
        std::lock_guard<std::mutex> lock(globalCacheMutex);
        
        // Nettoyer le cache expiré
        auto now = std::chrono::system_clock::now();
        for(auto it = globalCache.begin(); it != globalCache.end();) {
            if(now - it->second.timestamp > std::chrono::minutes(CACHE_EXPIRATION_MINUTES)) {
                it = globalCache.erase(it);
            } else {
                ++it;
            }
        }
        
        // Mettre à jour le cache
        auto& entry = globalCache[details.path];
        entry.files.push_back(details);
        entry.timestamp = now;
    }

    void startFilteringThread() {
        if(filterThread.joinable()) filterThread.join();
        filterThread = std::thread(&FileSearcher::filterWorker, this);
    }

    void filterWorker() {
        std::vector<FileDetails> filteredResults;
        {
            std::lock_guard<std::mutex> lock(resultsLock);
            for(const auto& result : currentResults) {
                if(result.path.find(currentKeyword) != std::string::npos) {
                    filteredResults.push_back(result);
                    if(resultCallback) resultCallback(result);
                }
            }
        }
        
        // Mettre à jour les résultats actuels
        {
            std::lock_guard<std::mutex> lock(resultsLock);
            currentResults = std::move(filteredResults);
        }
    }
};

// Structure pour passer les résultats au code C#
struct SearchResult {
    char* path;
    char* extension;
    uint64_t size;
    int64_t lastModified;  // timestamp
    int64_t created;       // timestamp
};

// Callback type definition
typedef void (*ResultCallback)(const SearchResult*);

// Fonction helper pour convertir FileDetails en SearchResult
SearchResult convertToSearchResult(const FileDetails& details) {
    SearchResult result;
    result.path = _strdup(details.path.c_str());
    result.extension = _strdup(details.extension.c_str());
    result.size = details.size;
    result.lastModified = std::chrono::system_clock::to_time_t(details.lastModified);
    result.created = std::chrono::system_clock::to_time_t(details.created);
    return result;
}

// Fonctions exportées
extern "C" {
    __declspec(dllexport) void* createSearcher() {
        return new FileSearcher();
    }

    __declspec(dllexport) void destroySearcher(void* searcher) {
        if (searcher) {
            delete static_cast<FileSearcher*>(searcher);
        }
    }

    __declspec(dllexport) void startSearch(void* searcher, 
                                         const char* keyword,
                                         const SearchOptions* options,
                                         ResultCallback callback) {
        if (!searcher || !keyword || !options) return;

        auto* fs = static_cast<FileSearcher*>(searcher);
        fs->startSearch(
            std::string(keyword),
            *options,
            [callback](const FileDetails& details) {
                if (callback) {
                    SearchResult result = convertToSearchResult(details);
                    callback(&result);
                    // Libérer la mémoire allouée par convertToSearchResult
                    free(result.path);
                    free(result.extension);
                }
            }
        );
    }

    __declspec(dllexport) void stopSearch(void* searcher) {
        if (!searcher) return;
        auto* fs = static_cast<FileSearcher*>(searcher);
        fs->stopSearch();
    }

    __declspec(dllexport) void updateKeyword(void* searcher, const char* newKeyword) {
        if (!searcher || !newKeyword) return;
        auto* fs = static_cast<FileSearcher*>(searcher);
        fs->updateKeyword(std::string(newKeyword));
    }

    // Fonction helper pour créer des SearchOptions depuis C#
    __declspec(dllexport) SearchOptions* createSearchOptions() {
        return new SearchOptions();
    }

    __declspec(dllexport) void destroySearchOptions(SearchOptions* options) {
        if (options) {
            delete options;
        }
    }

    // Fonctions pour configurer les options de recherche
    __declspec(dllexport) void addWhitelistExtension(SearchOptions* options, const char* extension) {
        if (options && extension) {
            options->whitelistExtensions.push_back(std::string(extension));
        }
    }

    __declspec(dllexport) void addBlacklistExtension(SearchOptions* options, const char* extension) {
        if (options && extension) {
            options->blacklistExtensions.push_back(std::string(extension));
        }
    }

    __declspec(dllexport) void addWhitelistFolder(SearchOptions* options, const char* folder) {
        if (options && folder) {
            options->whitelistFolders.push_back(std::string(folder));
        }
    }

    __declspec(dllexport) void addBlacklistFolder(SearchOptions* options, const char* folder) {
        if (options && folder) {
            options->blacklistFolders.push_back(std::string(folder));
        }
    }

    __declspec(dllexport) void setSearchInArchives(SearchOptions* options, bool enable) {
        if (options) {
            options->searchInArchives = enable;
        }
    }

    __declspec(dllexport) void setDateRange(SearchOptions* options, 
                                          int64_t fromTimestamp, 
                                          int64_t toTimestamp) {
        if (options) {
            options->dateFrom = std::chrono::system_clock::from_time_t(fromTimestamp);
            options->dateTo = std::chrono::system_clock::from_time_t(toTimestamp);
        }
    }

    __declspec(dllexport) void setDateChecking(SearchOptions* options, 
                                             bool checkCreation, 
                                             bool checkModification) {
        if (options) {
            options->checkCreationDate = checkCreation;
            options->checkModificationDate = checkModification;
        }
    }
}

/* OLD WORKING CODE

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

    // Search in drives
    for (const auto& drive : drives) {
        if (!isDriveAccessible(drive)) {
            continue;
        }
        searchFiles(drive, keywordStr, allResults);
    }

    // Fill the results in the C array
    *resultCount = allResults.size();
    for (size_t i = 0; i < allResults.size(); ++i) {
        // Use _strdup to allocate memory for each string (I use _strdup because malloc do not convert the string to a C-string)
        // But I know malloc is better because _strdup allocate memory for each string and malloc allocate memory for the whole array of strings
        // Anyway I don't know if it's a big deal because, with malloc, I need to use more loops to fill the array of string with strcpy and use more cpu
        // So I think it's better to use _strdup but you can tell me if I'm wrong :)
        results[i] = _strdup(allResults[i].c_str());
    }
}


extern "C" __declspec(dllexport) void freeResults(char** results, int resultCount) {
    for (int i = 0; i < resultCount; ++i) {
        free(results[i]);
    }
}

*/