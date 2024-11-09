# Suptool

## Description

Suptool est une application de bureau développée avec Tauri, React et
TypeScript. Elle permet de rechercher et d'indexer des fichiers sur votre
système, offrant une interface utilisateur moderne et réactive.

## Prérequis

Avant de commencer, assurez-vous d'avoir installé les éléments suivants :

- [Rust](https://www.rust-lang.org/tools/install) (version stable)
- [Node.js](https://nodejs.org/) (version 18 ou supérieure)
- [pnpm](https://pnpm.io/installation) pour la gestion des paquets
- [Deno v2](https://deno.land/manual/getting_started/installation) pour exécuter
  des scripts
- [Tauri CLI v2](https://tauri.app/v1/guides/getting-started/installation/) pour
  construire et exécuter l'application

### Installation de Rust et Cargo

Pour installer Rust & Cargo, exécutez la commande suivante :

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Installation

1. **Clonez le dépôt :**

   ```bash
   git clone https://github.com/votre-utilisateur/suptool.git
   cd suptool
   ```

2. **Installez les dépendances Deno :**

   Exécutez la commande suivante pour installer les dépendances :

   ```bash
   deno task setup
   ```

3. **Démarrez l'application en mode développement :**

   Exécutez la commande suivante dans le répertoire racine :

   ```bash
   deno task tauri dev
   ```

   Cela lancera l'application et ouvrira une fenêtre de navigateur pour le
   développement.

4. **Construisez l'application :**

   Pour construire l'application, utilisez la commande suivante :

   ```bash
   deno task tauri build
   ```

   Ou, si vous souhaitez spécifier le runner et la cible :

   ```bash
   deno task tauri build --runner cargo-xwin --target x86_64-pc-windows-msvc
   ```

## Utilisation

Une fois l'application lancée, vous pouvez :

- Indexer vos fichiers en cliquant sur le bouton d'indexation.
- Rechercher des fichiers en utilisant la barre de recherche.
- Et encore beaucoup à venir!

## Contribuer

Les contributions sont les bienvenues ! Si vous souhaitez contribuer, veuillez
suivre ces étapes :

## License

Ce projet est sous licence MIT. Pour plus de détails, veuillez consulter le
fichier [LICENSE](LICENSE).

## Aide

Pour toute question ou problème, n'hésitez pas à ouvrir une issue sur le dépôt
GitHub.
