# suptool

suptool est un utilitaire Windows qui offre plusieurs fonctionnalités pour améliorer votre productivité.

## Fonctionnalités

### EverySup
- Recherche rapide de fichiers et dossiers sur votre système
- Interface intuitive avec tableau de résultats
- Indexation des fichiers pour des recherches plus rapides
- Affichage des informations détaillées (taille, date de modification, type)

### ClearTool
- Nettoyage des fichiers temporaires
- Suppression des fichiers inutiles
- Interface intuitive avec tableau de résultats

## Installation

1. Téléchargez la dernière version depuis la [page des releases](https://github.com/SUP2Ak/suptool/releases)
2. Exécutez le fichier d'installation
3. Suivez les instructions à l'écran
4. Les mises à jour sont automatiques proposées quand vous lancer l'application

## Prérequis

- Windows 10 ou plus récent
- Droits administrateur pour l'installation et l'indexation

## Développement

### Technologies utilisées
- Rust
- Slint UI

### Structure du projet

```
suptool/
├── src/
│   ├── app.rs
│   ├── everything/
│   ├── pages/
│   │   ├── about.rs
│   │   ├── everysup.rs
│   │   ├── features/
│   │   ├── home.rs
│   │   └── settings.rs
│   ├── updater/
│   └── widgets/
├── ui/
│   ├── icons/
│   ├── main.slint
│   ├── pages/
│   └── widgets/
```

### Compilation

1. Cloner le projet

```bash
git clone https://github.com/SUP2Ak/suptool.git
cd suptool
```

2. Compiler en mode debug

```bash
cargo run
```

3. Compiler en mode release

```bash
cargo build --release
```

## Licence

Ce projet est sous licence MIT. Voir le fichier `LICENSE` pour plus de détails.

## Auteur

- SUP2Ak

## Contribution

Les contributions sont les bienvenues ! N'hésitez pas à :
1. Forker le projet
2. Créer une branche pour votre fonctionnalité
3. Commiter vos changements
4. Pousser vers la branche
5. Ouvrir une Pull Request

## Support

Si vous rencontrez des problèmes ou avez des suggestions :
1. Ouvrez une issue sur GitHub
2. Décrivez clairement le problème ou la suggestion
3. Ajoutez des captures d'écran si nécessaire