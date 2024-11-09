import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

interface IndexingStatus {
  total_files: number;
  current_drive: string;
  is_complete: boolean;
}

interface FileDetails {
  path: string;
  extension: string;
  size: number;
  last_modified: string;
  created: string;
}

export default function FileSearch() {
  const [keyword, setKeyword] = useState("");
  const [isIndexing, setIsIndexing] = useState(true);
  const [indexingStatus, setIndexingStatus] = useState<IndexingStatus | null>(
    null,
  );
  const [allFiles, setAllFiles] = useState<FileDetails[]>([]);
  const [displayedResults, setDisplayedResults] = useState<FileDetails[]>([]);

  useEffect(() => {
    // Vérifier d'abord si l'indexation est déjà terminée
    invoke<boolean>("is_indexing_complete")
      .then((isComplete) => {
        if (isComplete) {
          setIsIndexing(false);
          // Charger directement les fichiers si l'indexation est déjà terminée
          return invoke<FileDetails[]>("get_all_index_entry")
            .then((files) => {
              console.log("Fichiers déjà indexés récupérés:", files.length);
              setAllFiles(files);
              console.log(
                "Fichiers récupérés avec succès",
                JSON.stringify(files),
              );
            });
        }
      })
      .catch((err) =>
        console.error("Erreur lors de la vérification de l'indexation:", err)
      );

    // Configuration de l'écouteur d'événements pour les nouvelles indexations
    const unsubscribe = listen<IndexingStatus>("indexing-status", (event) => {
      console.log("Statut d'indexation reçu:", event.payload);
      setIndexingStatus(event.payload);
      setIsIndexing(!event.payload.is_complete);

      if (event.payload.is_complete) {
        invoke<FileDetails[]>("get_all_index_entry")
          .then((files) => {
            setAllFiles(files);
            console.log(
              "Fichiers récupérés avec succès",
              JSON.stringify(files),
            );
          })
          .catch((err) =>
            console.error("Erreur lors de la récupération des fichiers:", err)
          );
      }
    });

    return () => {
      unsubscribe.then((fn) => fn());
    };
  }, []);

  // Filtrage des résultats (inchangé)
  useEffect(() => {
    const filterResults = () => {
      if (!keyword.trim()) {
        setDisplayedResults([]);
        return;
      }

      const searchTerm = keyword.toLowerCase();
      const filtered = allFiles
        .filter((file) => file.path.toLowerCase().includes(searchTerm))
        .slice(0, 1000);

      setDisplayedResults(filtered);
    };

    filterResults();
  }, [keyword, allFiles]);

  return (
    <div className="p-4">
      {isIndexing && indexingStatus && (
        <div className="mb-4 p-3 bg-blue-100 rounded">
          <p>Indexation en cours...</p>
          <p>Disque actuel : {indexingStatus.current_drive}</p>
          <p>Fichiers trouvés : {indexingStatus.total_files}</p>
        </div>
      )}

      <input
        type="text"
        value={keyword}
        onChange={(e) => setKeyword(e.target.value)}
        placeholder="Rechercher des fichiers..."
        className="w-full px-4 py-2 border rounded mb-4"
        disabled={isIndexing}
      />

      {!isIndexing && (
        <div className="mt-4">
          <h2 className="text-xl mb-2">
            Résultats ({displayedResults.length})
            {displayedResults.length === 1000 &&
              " (Limité aux 1000 premiers résultats)"}
          </h2>
          <div className="space-y-2">
            {displayedResults.map((file, index) => (
              <div key={index} className="p-3 border rounded hover:bg-gray-50">
                <div className="font-medium truncate">{file.path}</div>
                <div className="text-sm text-gray-600 flex gap-4 mt-1">
                  <span>Extension: {file.extension || "Aucune"}</span>
                  <span>Taille: {formatFileSize(file.size)}</span>
                  <span>
                    Modifié: {new Date(file.last_modified).toLocaleString()}
                  </span>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}

function formatFileSize(size: number): string {
  if (size < 1024) return `${size} B`;
  if (size < 1024 * 1024) return `${(size / 1024).toFixed(2)} KB`;
  if (size < 1024 * 1024 * 1024) {
    return `${(size / (1024 * 1024)).toFixed(2)} MB`;
  }
  return `${(size / (1024 * 1024 * 1024)).toFixed(2)} GB`;
}
