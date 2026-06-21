---
goal: "Implémenter une application desktop Tauri/Deno/Svelte pour lancer et superviser les processus d'un projet"
version: "1.0"
date_created: "2026-06-19"
last_updated: "2026-06-19"
owner: "Project maintainers"
status: "Planned"
tags: ["feature", "desktop", "tauri", "svelte", "deno", "process-runner"]
---

# Introduction

![Status: Planned](https://img.shields.io/badge/status-Planned-blue)

Ce plan décrit l'implémentation d'une application desktop, nommée provisoirement
`devapp`, qui permet de définir, lancer, observer, éditer et arrêter un ensemble
de processus de développement depuis une interface Tauri/Svelte. Le moteur
d'orchestration est implémenté côté Rust afin de centraliser les règles de
configuration, de dépendances, de readiness, de logs et de cycle de vie.

## 1. Requirements & Constraints

- **REQ-001**: L'application doit utiliser Tauri 2, Svelte 5, TypeScript,
  Tailwind CSS et Deno pour les tâches frontend.
- **REQ-002**: Le fichier de configuration projet doit être un fichier YAML
  situé à la racine du projet, nommé `devapp.yml`.
- **REQ-003**: Le format de configuration YAML doit supporter `version`, `env`,
  `processes`, `kind`, `cmd`, `dependsOn`, `ready`, et les conditions de
  dépendance `success` et `ready`.
- **REQ-004**: Un processus avec `kind: task` doit être considéré comme terminé
  lorsque sa commande se termine avec un code de sortie.
- **REQ-005**: Un processus avec `kind: service` doit rester actif jusqu'à arrêt
  explicite, échec du processus, arrêt du projet, ou fermeture de session.
- **REQ-006**: Une dépendance `dependsOn: { setup: success }` doit débloquer le
  processus dépendant uniquement si `setup` termine avec un code de sortie `0`.
- **REQ-007**: Une dépendance `dependsOn: { api: ready }` doit débloquer le
  processus dépendant uniquement lorsque `api` atteint l'état `Ready`.
- **REQ-008**: Les readiness checks requis pour la V1 sont `ready.type: http`,
  `ready.type: log`, `ready.type: delay`, et `ready.type: command`.
- **REQ-008A**: `ready.type: http` doit utiliser une URL absolue et réussir
  lorsque la réponse HTTP a un status code compris entre `200` et `299`.
- **REQ-008B**: `ready.type: log` doit réussir lorsqu'une ligne stdout ou stderr
  du processus correspond au motif texte ou regex configuré.
- **REQ-008C**: `ready.type: delay` doit réussir après une durée configurée en
  millisecondes à partir du démarrage du processus.
- **REQ-008D**: `ready.type: command` doit réussir lorsqu'une commande de
  readiness lancée dans le `workingDir` du projet termine avec un code de sortie
  `0`.
- **REQ-009**: Les variables `env` globales du projet doivent être injectées
  dans tous les processus lancés depuis cette configuration.
- **REQ-010**: Chaque processus doit être lancé avec un `workingDir` égal au
  dossier projet associé à la configuration.
- **REQ-011**: Les projets créés depuis l'application desktop doivent être
  stockés dans le répertoire de configuration applicatif de l'OS via
  `ProjectDirs`, par exemple `$XDG_CONFIG_HOME/devapp/projects.yml` ou
  `~/.config/devapp/projects.yml` sous Linux.
- **REQ-012**: Un projet enregistré depuis l'application doit stocker au minimum
  `id`, `name`, `baseDir`, `configSource`, `configPath`, `createdAt`, et
  `updatedAt`.
- **REQ-013**: Si un projet contient déjà un fichier `devapp.yml` dans son
  dossier racine, l'application doit charger et éditer ce fichier directement.
- **REQ-014**: Si un projet est créé depuis l'application sans fichier
  `devapp.yml` dans son dossier racine, l'application doit stocker sa
  configuration YAML dans le dossier de configuration applicatif OS, par exemple
  `$XDG_CONFIG_HOME/devapp/project-configs/<project-id>.yml`.
- **REQ-014A**: La configuration stockée hors projet doit contenir ou référencer
  le `baseDir` afin que l'application sache à quel dossier elle s'applique.
- **REQ-015**: L'interface desktop doit afficher une liste verticale des
  processus à gauche avec `name`, `status`, `kind`, et un indicateur visuel
  différent pour les tâches.
- **REQ-016**: L'interface desktop doit afficher à droite les logs du processus
  sélectionné.
- **REQ-017**: L'interface desktop doit permettre d'ouvrir un terminal attaché
  au `baseDir` du projet et de l'afficher dans la vue de droite.
- **REQ-018**: Le backend doit émettre les logs stdout et stderr vers le
  frontend via des événements Tauri typés.
- **REQ-019**: Le backend doit permettre les actions `start project`,
  `stop project`, `restart process`, `stop process`, `open terminal`,
  `write terminal input`, `resize terminal`, et `close terminal`.
- **REQ-020**: L'arrêt d'un projet doit arrêter tous les services et terminaux
  enfants créés par la session.
- **REQ-021**: Une fenêtre applicative représente une seule session projet
  active; plusieurs projets peuvent être actifs simultanément si chacun dispose
  de sa propre fenêtre indépendante.
- **REQ-021A**: Si une fenêtre a déjà une session projet active et que
  l'utilisateur lance un autre projet, l'application doit ouvrir une nouvelle
  fenêtre indépendante pour cet autre projet au lieu de remplacer la session de
  la fenêtre courante.
- **REQ-021B**: Les logs, terminaux, processus sélectionnés, erreurs UI et
  actions runtime doivent être isolés par fenêtre projet.
- **REQ-022**: Si une tâche échoue, si un service échoue avant readiness, si un
  readiness check expire, ou si une dépendance requise échoue, l'application
  doit arrêter toute la session.
- **REQ-023**: Les logs doivent être gardés en mémoire en V1, via une interface
  de stockage extensible permettant d'ajouter plus tard une persistance fichier
  sans réécrire l'orchestrateur.
- **REQ-024**: Le champ `cmd` doit être exécuté via le shell OS par défaut en
  V1; le modèle de configuration doit rester extensible pour ajouter plus tard
  un champ `argv` sans shell.
- **SEC-001**: L'application ne doit exécuter que des commandes associées à un
  projet explicitement sélectionné ou détecté par l'utilisateur.
- **SEC-002**: L'application doit afficher le `baseDir` et le fichier de
  configuration avant le lancement depuis l'interface desktop.
- **SEC-003**: Les commandes YAML doivent être traitées comme du shell local
  explicite; aucune tentative d'assainissement ne doit donner une fausse
  garantie de sécurité.
- **SEC-004**: Les chemins de configuration enregistrés doivent être canonisés
  avant stockage pour réduire les ambiguïtés de chemin.
- **CON-001**: Le dépôt actuel est un scaffold Tauri/Svelte minimal situé dans
  `devapp/`.
- **CON-002**: Le fichier `devapp/src-tauri/tauri.conf.json` référence déjà
  `deno task dev` et `deno task build`, mais aucun `deno.json` n'existe
  actuellement.
- **CON-003**: Le plan doit limiter les changements au dossier applicatif
  `devapp/`.
- **CON-004**: La première version ne doit pas implémenter de synchronisation
  cloud, de partage d'équipe, ni de daemon système permanent.
- **PAT-001**: Le code Rust doit être séparé en modules `domain`, `application`,
  `infrastructure`, et `tauri_api`.
- **PAT-002**: Le frontend Svelte doit séparer les appels Tauri, le store d'état
  runtime, et les composants visuels.
- **PAT-003**: Les événements runtime doivent utiliser des payloads
  sérialisables `serde` côté Rust et des types TypeScript équivalents côté
  frontend.
- **PAT-004**: Le styling frontend doit utiliser Tailwind CSS comme système
  principal; le CSS custom doit être limité aux styles globaux indispensables,
  aux imports de librairies comme xterm, et aux cas non exprimables proprement
  avec des utilities.
- **DEC-001**: Le nom du fichier de configuration racine est `devapp.yml`.
- **DEC-002**: Il n'y a pas de runner CLI en V1; l'exécution se fait uniquement
  depuis l'interface desktop Tauri.
- **DEC-003**: Le champ `cmd` est une chaîne shell en V1, car ce format couvre
  les commandes multi-lignes, les variables, les redirections, les pipes et les
  quotes complexes.
- **DEC-004**: Les readiness checks V1 sont `http`, `log`, `delay`, et
  `command`.
- **DEC-005**: L'échec d'une dépendance ou d'un processus requis arrête toute la
  session.
- **DEC-006**: Les logs restent en mémoire en V1, avec une abstraction prévue
  pour une persistance future.
- **DEC-007**: L'application autorise plusieurs projets actifs en V1 via une
  fenêtre indépendante par projet; chaque fenêtre ne pilote qu'une seule session
  projet.
- **DEC-008**: L'application peut éditer le YAML; elle édite `devapp.yml` si le
  fichier existe dans le projet, sinon elle édite une configuration YAML stockée
  dans le dossier applicatif OS.

## 2. Implementation Steps

### Implementation Phase 1

- GOAL-001: Stabiliser la base Tauri/Deno/Svelte et créer les types partagés de
  configuration et runtime.

| Task      | Description                                                                                                                                                                                                                                                                                    | Completed | Date |
| --------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------- | ---- |
| TASK-001  | Créer `devapp/deno.json` avec les tâches `dev`, `build`, `preview`, `check`, et `tauri` afin d'aligner `src-tauri/tauri.conf.json` avec Deno.                                                                                                                                                  |           |      |
| TASK-002  | Mettre à jour `devapp/package.json` uniquement si nécessaire pour conserver les dépendances frontend existantes et éviter un double système de scripts contradictoire.                                                                                                                         |           |      |
| TASK-003  | Ajouter dans `devapp/src-tauri/Cargo.toml` les dépendances Rust `serde_yaml`, `thiserror`, `tokio`, `reqwest`, `directories`, `uuid`, `indexmap`, `portable-pty`, `chrono`, et `regex`.                                                                                                        |           |      |
| TASK-003A | Ajouter dans `devapp/package.json` les dépendances frontend `tailwindcss` et `@tailwindcss/vite`, puis configurer `devapp/vite.config.js` pour charger le plugin Tailwind Vite après le plugin Svelte.                                                                                         |           |      |
| TASK-003B | Créer `devapp/src/app.css` avec `@import "tailwindcss";`, importer ce fichier depuis `devapp/src/routes/+layout.ts`, et retirer les styles globaux du scaffold qui entrent en conflit avec Tailwind.                                                                                           |           |      |
| TASK-004  | Créer `devapp/src-tauri/src/domain/mod.rs` qui exporte les modules `config`, `process`, `project`, `runtime`, et `terminal`.                                                                                                                                                                   |           |      |
| TASK-005  | Créer `devapp/src-tauri/src/domain/config.rs` avec les structs `DevappConfig`, `ProcessConfig`, `ReadyConfig`, `HttpReadyConfig`, `LogReadyConfig`, `DelayReadyConfig`, `CommandReadyConfig`, les enums `ProcessKind`, `DependencyCondition`, `ReadyType`, et la validation de `version == 1`. |           |      |
| TASK-006  | Créer `devapp/src-tauri/src/domain/process.rs` avec les enums `ProcessStatus` (`Pending`, `Blocked`, `Starting`, `Running`, `Ready`, `Succeeded`, `Failed`, `Stopping`, `Stopped`) et `LogStream` (`Stdout`, `Stderr`, `System`).                                                              |           |      |
| TASK-007  | Créer `devapp/src-tauri/src/domain/project.rs` avec `ProjectRecord`, `ProjectId`, `ProjectSource`, et les champs `id`, `name`, `base_dir`, `config_source`, `config_path`, `created_at`, `updated_at`.                                                                                         |           |      |
| TASK-008  | Créer `devapp/src-tauri/src/domain/runtime.rs` avec `RunSessionId`, `ProcessRuntimeId`, `RunSessionSnapshot`, `ProcessSnapshot`, et les payloads de logs.                                                                                                                                      |           |      |
| TASK-009  | Créer `devapp/src/lib/types.ts` avec les types TypeScript équivalents aux payloads Rust exposés au frontend.                                                                                                                                                                                   |           |      |

### Implementation Phase 2

- GOAL-002: Implémenter le chargement, la validation, la découverte et le
  stockage des projets.

| Task      | Description                                                                                                                                                                                                                                                             | Completed | Date |
| --------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------- | ---- |
| TASK-010  | Créer `devapp/src-tauri/src/infrastructure/mod.rs` qui exporte `config_loader`, `project_store`, `shell`, `ready_checks`, `log_store`, et `pty`.                                                                                                                        |           |      |
| TASK-011  | Créer `devapp/src-tauri/src/infrastructure/config_loader.rs` avec `load_config(config_path: &Path) -> Result<LoadedProjectConfig, AppError>` qui lit YAML via `serde_yaml`, canonise `config_path`, définit `base_dir`, et valide le graphe.                            |           |      |
| TASK-012  | Implémenter dans `config_loader.rs` `find_project_config(base_dir: &Path) -> Result<Option<PathBuf>, AppError>` qui cherche uniquement `devapp.yml` à la racine du projet sélectionné.                                                                                  |           |      |
| TASK-013  | Implémenter la validation du graphe dans `config_loader.rs`: erreur si dépendance inconnue, condition non supportée, cycle de dépendances, nom de processus vide, ou `cmd` vide.                                                                                        |           |      |
| TASK-014  | Créer `devapp/src-tauri/src/infrastructure/project_store.rs` avec `ProjectStore::load`, `ProjectStore::save`, `ProjectStore::upsert`, `ProjectStore::remove`, basé sur `directories::ProjectDirs` et les fichiers `projects.yml` et `project-configs/<project-id>.yml`. |           |      |
| TASK-015  | Créer `devapp/src-tauri/src/error.rs` avec `AppError`, conversion vers `String` pour les commandes Tauri, et catégories `Config`, `Io`, `Validation`, `Runtime`, `ProjectStore`, `Terminal`.                                                                            |           |      |
| TASK-016  | Ajouter les tests unitaires Rust dans `devapp/src-tauri/src/infrastructure/config_loader.rs` pour charger l'exemple YAML fourni, détecter un cycle, détecter une dépendance inconnue, et détecter une version non supportée.                                            |           |      |
| TASK-016A | Ajouter les tests unitaires Rust dans `project_store.rs` pour vérifier l'édition d'un `devapp.yml` existant et l'édition d'une configuration stockée sous `project-configs/<project-id>.yml` lorsqu'aucun YAML projet n'existe.                                         |           |      |

### Implementation Phase 3

- GOAL-003: Implémenter l'orchestrateur de processus utilisé par l'application
  desktop.

| Task     | Description                                                                                                                                                                                                                                                                                 | Completed | Date |
| -------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------- | ---- |
| TASK-017 | Créer `devapp/src-tauri/src/application/mod.rs` qui exporte `orchestrator`, `events`, `readiness`, et `command_runner`.                                                                                                                                                                     |           |      |
| TASK-018 | Créer `devapp/src-tauri/src/application/events.rs` avec `RuntimeEvent`, `ProcessLogEvent`, `ProcessStatusEvent`, `SessionStatusEvent`, et `TerminalEvent`.                                                                                                                                  |           |      |
| TASK-019 | Créer `devapp/src-tauri/src/application/command_runner.rs` avec `spawn_process` qui exécute `cmd` dans le shell OS, injecte `env`, définit `current_dir`, capture stdout/stderr, et conserve le handle enfant.                                                                              |           |      |
| TASK-020 | Implémenter la stratégie shell dans `devapp/src-tauri/src/infrastructure/shell.rs`: Unix utilise `sh -c`, Windows utilise `cmd /C` par défaut; documenter l'ajout futur possible d'un champ `argv` sans shell.                                                                              |           |      |
| TASK-021 | Créer `devapp/src-tauri/src/application/readiness.rs` avec `wait_until_ready` qui supporte `ReadyConfig::Http`, `ReadyConfig::Log`, `ReadyConfig::Delay`, `ReadyConfig::Command`, intervalle `500ms` pour les checks pollés, timeout par défaut `60s`, et publication d'événements système. |           |      |
| TASK-022 | Créer `devapp/src-tauri/src/application/orchestrator.rs` avec `ProcessOrchestrator`, `start_session`, `stop_session`, `restart_process`, `stop_process`, `snapshot`, et stockage des sessions par identifiant de fenêtre/projet afin d'isoler les projets actifs.                           |           |      |
| TASK-023 | Implémenter dans `orchestrator.rs` un scheduler DAG qui démarre les processus dont toutes les dépendances sont satisfaites et déclenche l'arrêt complet de la session dès qu'une dépendance requise échoue.                                                                                 |           |      |
| TASK-024 | Implémenter dans `orchestrator.rs` la différence `task`/`service`: une tâche réussie devient `Succeeded`, un service lancé devient `Running`, puis `Ready` si son readiness check passe ou s'il n'a pas de readiness check.                                                                 |           |      |
| TASK-025 | Créer `devapp/src-tauri/src/infrastructure/log_store.rs` avec un trait `LogStore`, une implémentation `InMemoryLogStore`, et un ring buffer configurable de `10000` lignes par défaut pour permettre une future persistance fichier.                                                        |           |      |
| TASK-026 | Ajouter les tests Rust de l'orchestrateur avec commandes courtes: ordre `setup -> api -> worker`, arrêt complet si `setup` échoue, transition `Ready` sur HTTP local, transition `Ready` sur log, transition `Ready` sur delay, et transition `Ready` sur command.                          |           |      |

### Implementation Phase 4

- GOAL-004: Exposer le moteur à Tauri et implémenter le terminal intégré.

| Task     | Description                                                                                                                                                                                                                                                                                                                                         | Completed | Date |
| -------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------- | ---- |
| TASK-027 | Créer `devapp/src-tauri/src/tauri_api/mod.rs` qui exporte `commands` et `state`.                                                                                                                                                                                                                                                                    |           |      |
| TASK-028 | Créer `devapp/src-tauri/src/tauri_api/state.rs` avec `AppState` contenant `ProcessOrchestrator` et `ProjectStore` sous primitives thread-safe.                                                                                                                                                                                                      |           |      |
| TASK-029 | Créer `devapp/src-tauri/src/tauri_api/commands.rs` avec les commandes Tauri `list_projects`, `save_project`, `remove_project`, `load_project_config`, `save_project_config`, `start_project`, `stop_project`, `restart_process`, `stop_process`, `get_session_snapshot`, `open_terminal`, `write_terminal`, `resize_terminal`, et `close_terminal`. |           |      |
| TASK-030 | Modifier `devapp/src-tauri/src/lib.rs` pour enregistrer `AppState`, remplacer la commande `greet`, initialiser le plugin opener existant, et enregistrer toutes les commandes Tauri.                                                                                                                                                                |           |      |
| TASK-031 | Créer `devapp/src-tauri/src/infrastructure/pty.rs` avec `TerminalManager` basé sur `portable-pty`, ouverture du shell utilisateur dans `baseDir`, streaming output, écriture input, resize, et fermeture.                                                                                                                                           |           |      |
| TASK-032 | Ajouter dans `devapp/src-tauri/capabilities/default.json` les permissions Tauri nécessaires aux commandes custom, aux events, et aux dialogs si un plugin dialog est ajouté.                                                                                                                                                                        |           |      |
| TASK-033 | Ajouter les tests Rust de `TerminalManager` limités aux plateformes supportées par l'environnement CI local, avec skip documenté si aucun shell n'est disponible.                                                                                                                                                                                   |           |      |

### Implementation Phase 5

- GOAL-005: Construire l'interface desktop Svelte pour gérer les projets,
  sessions, processus, logs et terminaux.

| Task      | Description                                                                                                                                                                                                                                                                            | Completed | Date |
| --------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------- | ---- |
| TASK-034  | Remplacer `devapp/src/routes/+page.svelte` par l'écran applicatif principal: rail projets, header projet actif, liste processus à gauche, vue logs/terminal à droite, barre d'actions.                                                                                                 |           |      |
| TASK-035  | Créer `devapp/src/lib/tauri/client.ts` avec des fonctions typées qui appellent les commandes Tauri via `invoke`.                                                                                                                                                                       |           |      |
| TASK-036  | Créer `devapp/src/lib/stores/runtime.svelte.ts` avec l'état Svelte 5 pour la session de la fenêtre courante, le projet attaché à cette fenêtre, le processus sélectionné, le terminal sélectionné, les logs, et les erreurs UI.                                                        |           |      |
| TASK-036A | Ajouter le flux multi-fenêtre: lorsqu'un projet différent du projet actif de la fenêtre courante est lancé, appeler une commande Tauri `open_project_window` ou équivalente pour ouvrir une nouvelle fenêtre indépendante initialisée avec ce projet.                                  |           |      |
| TASK-037  | Créer `devapp/src/lib/components/ProjectPicker.svelte` pour choisir un projet enregistré et déclencher `start_project`.                                                                                                                                                                |           |      |
| TASK-038  | Créer `devapp/src/lib/components/ProcessList.svelte` pour afficher `name`, `status`, `kind`, icônes de statut, et sélection.                                                                                                                                                           |           |      |
| TASK-039  | Créer `devapp/src/lib/components/LogViewer.svelte` avec rendu différencié stdout/stderr/system, auto-scroll activable, recherche locale, et bouton clear côté UI.                                                                                                                      |           |      |
| TASK-040  | Créer `devapp/src/lib/components/TerminalPane.svelte` avec rendu terminal basé sur `xterm.js` et pont input/resize vers les commandes Tauri.                                                                                                                                           |           |      |
| TASK-041  | Ajouter `xterm` et `@xterm/addon-fit` dans `devapp/package.json` et intégrer le CSS xterm dans le composant terminal.                                                                                                                                                                  |           |      |
| TASK-042  | Créer `devapp/src/lib/components/ProjectSettingsDialog.svelte` pour enregistrer un projet avec `name`, `baseDir`, `configSource`, et `configPath`; utiliser `devapp.yml` si présent dans `baseDir`, sinon créer une configuration applicative sous `project-configs/<project-id>.yml`. |           |      |
| TASK-043  | Adapter le style global dans `devapp/src/app.css` et les classes Tailwind des composants Svelte pour une UI dense, utilitaire, sans page marketing, compatible 800x600 minimum.                                                                                                        |           |      |
| TASK-044  | Ajouter l'écoute des événements Tauri dans `runtime.svelte.ts`: append logs, update status, update snapshots, append terminal output, et suppression des listeners au teardown.                                                                                                        |           |      |
| TASK-044A | Créer `devapp/src/lib/components/ConfigEditor.svelte` avec édition YAML, validation via `load_project_config`, sauvegarde via `save_project_config`, et affichage de la source active `ProjectFile` ou `AppConfigFile`.                                                                |           |      |

### Implementation Phase 6

- GOAL-006: Finaliser la robustesse, la documentation et les validations
  utilisateur.

| Task     | Description                                                                                                                                                                                                                              | Completed | Date |
| -------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------- | ---- |
| TASK-045 | Créer `devapp/examples/deno-runner.yml` avec un exemple `setup`, `api`, `worker` auto-contenu pour tester le runtime Tauri.                                                                                                              |           |      |
| TASK-046 | Mettre à jour `devapp/README.md` avec installation dev, `deno task dev`, `deno task tauri dev`, format YAML, édition de configuration, readiness checks, et limitations V1.                                                              |           |      |
| TASK-047 | Ajouter `devapp/docs/configuration.md` avec le schéma YAML documenté, les exemples de dépendances `success` et `ready`, les types `http`, `log`, `delay`, `command`, et la décision `cmd` shell.                                         |           |      |
| TASK-048 | Ajouter `devapp/docs/runtime-model.md` avec les états de processus, les transitions, l'arrêt complet de session sur échec, et le modèle d'isolation "une session par fenêtre projet".                                                    |           |      |
| TASK-049 | Exécuter `deno task check` depuis `devapp/` et corriger toutes les erreurs TypeScript/Svelte.                                                                                                                                            |           |      |
| TASK-050 | Exécuter `cargo test` depuis `devapp/src-tauri/` et corriger toutes les erreurs Rust.                                                                                                                                                    |           |      |
| TASK-051 | Exécuter `deno task build` depuis `devapp/` et corriger les erreurs de build frontend.                                                                                                                                                   |           |      |
| TASK-052 | Exécuter un test manuel de bout en bout: lancer l'app avec `examples/deno-runner.yml`, éditer la configuration YAML, lancer le projet, vérifier logs `setup`, readiness log `api`, logs `worker`, ouvrir un terminal, arrêter le projet. |           |      |

## 3. Alternatives

- **ALT-001**: Implémenter l'orchestrateur en TypeScript côté frontend. Rejeté
  parce que le contrôle fiable des processus, des signaux, des PTY et des
  événements Tauri est mieux centralisé côté Rust.
- **ALT-002**: Utiliser directement `mprocs` comme backend. Rejeté pour la V1
  parce que l'application a besoin d'un modèle d'état typé, d'événements Tauri,
  d'une UI dédiée et d'un stockage projet intégré.
- **ALT-003**: Stocker tous les projets uniquement dans des fichiers YAML à la
  racine des projets. Rejeté parce que les projets créés depuis l'application
  doivent pouvoir exister sans modifier immédiatement le dossier projet.
- **ALT-004**: Construire un runner CLI ou une UI terminal interactive complète.
  Rejeté pour la V1 parce que l'exécution doit rester uniquement dans
  l'application desktop Tauri.
- **ALT-005**: Lancer les commandes sans shell et parser `cmd` en argv. Rejeté
  comme comportement par défaut parce que l'exemple utilisateur utilise des
  commandes shell multi-lignes et des quotes complexes.
- **ALT-006**: Ne pas éditer le YAML depuis l'application. Rejeté parce que
  l'application doit pouvoir modifier `devapp.yml` lorsqu'il existe et modifier
  la configuration applicative lorsqu'elle est stockée hors projet.

## 4. Dependencies

- **DEP-001**: `tauri = "2"` pour l'application desktop et les commandes
  backend.
- **DEP-002**: `svelte = "^5.0.0"` et `@sveltejs/kit = "^2.9.0"` pour le
  frontend existant.
- **DEP-003**: `deno.json` pour fournir les tâches Deno attendues par
  `tauri.conf.json`.
- **DEP-003A**: `tailwindcss` et `@tailwindcss/vite` pour intégrer Tailwind CSS
  au build Vite/Svelte.
- **DEP-004**: `serde_yaml` pour parser et sérialiser les fichiers `devapp.yml`
  et les configurations YAML stockées par l'application.
- **DEP-005**: `tokio` pour exécuter les processus, lire les streams, faire
  tourner le scheduler, et gérer les readiness checks asynchrones.
- **DEP-006**: `reqwest` pour `ready.type: http`.
- **DEP-007**: `directories` pour localiser le dossier de configuration
  applicatif par OS.
- **DEP-008**: `portable-pty` pour les terminaux intégrés.
- **DEP-009**: `xterm` et `@xterm/addon-fit` pour le rendu terminal côté Svelte.
- **DEP-010**: `uuid` pour identifier projets, sessions, processus runtime et
  terminaux.
- **DEP-011**: `chrono` pour sérialiser `createdAt`, `updatedAt`, et timestamps
  d'événements.
- **DEP-012**: `thiserror` pour les erreurs Rust structurées.
- **DEP-013**: `indexmap` pour conserver l'ordre de déclaration YAML tout en
  permettant un accès par nom.
- **DEP-014**: `regex` pour `ready.type: log` lorsque la configuration demande
  une correspondance regex.

## 5. Files

- **FILE-001**: `devapp/deno.json` - Tâches Deno frontend et Tauri.
- **FILE-002**: `devapp/package.json` - Dépendances frontend, notamment
  `tailwindcss`, `@tailwindcss/vite`, et `xterm`.
- **FILE-003**: `devapp/src/lib/types.ts` - Types frontend partagés avec les
  payloads Rust.
- **FILE-004**: `devapp/src/lib/tauri/client.ts` - Client typé pour les
  commandes Tauri.
- **FILE-005**: `devapp/src/lib/stores/runtime.svelte.ts` - Store applicatif
  Svelte 5.
- **FILE-006**: `devapp/src/lib/components/ProjectPicker.svelte` - Sélection des
  projets enregistrés.
- **FILE-007**: `devapp/src/lib/components/ProcessList.svelte` - Liste verticale
  des processus.
- **FILE-008**: `devapp/src/lib/components/LogViewer.svelte` - Affichage des
  logs.
- **FILE-009**: `devapp/src/lib/components/TerminalPane.svelte` - Terminal
  intégré.
- **FILE-010**: `devapp/src/lib/components/ProjectSettingsDialog.svelte` -
  Création et modification de projets enregistrés.
- **FILE-011**: `devapp/src/routes/+page.svelte` - Écran principal de
  l'application.
- **FILE-011A**: `devapp/src/app.css` - Import Tailwind CSS et styles globaux
  minimaux.
- **FILE-011B**: `devapp/vite.config.js` - Configuration du plugin
  `@tailwindcss/vite`.
- **FILE-012**: `devapp/src-tauri/Cargo.toml` - Dépendances Rust.
- **FILE-013**: `devapp/src-tauri/src/lib.rs` - Initialisation Tauri et modules
  partagés.
- **FILE-014**: `devapp/src-tauri/src/error.rs` - Erreurs applicatives.
- **FILE-015**: `devapp/src-tauri/src/domain/config.rs` - Modèle de
  configuration YAML.
- **FILE-016**: `devapp/src-tauri/src/domain/process.rs` - Modèle de statut
  processus.
- **FILE-017**: `devapp/src-tauri/src/domain/project.rs` - Modèle de projet
  enregistré.
- **FILE-018**: `devapp/src-tauri/src/domain/runtime.rs` - Modèle de session et
  snapshots.
- **FILE-019**: `devapp/src-tauri/src/infrastructure/config_loader.rs` -
  Chargement, découverte et validation YAML.
- **FILE-020**: `devapp/src-tauri/src/infrastructure/project_store.rs` -
  Stockage OS des projets enregistrés.
- **FILE-021**: `devapp/src-tauri/src/infrastructure/shell.rs` - Stratégie shell
  cross-platform.
- **FILE-022**: `devapp/src-tauri/src/infrastructure/pty.rs` - Gestion terminal
  pseudo-TTY.
- **FILE-022A**: `devapp/src-tauri/src/infrastructure/log_store.rs` - Stockage
  mémoire extensible des logs.
- **FILE-022B**: `devapp/src-tauri/src/infrastructure/ready_checks.rs` - Helpers
  pour readiness HTTP, log, delay et command.
- **FILE-023**: `devapp/src-tauri/src/application/orchestrator.rs` - Scheduler
  et cycle de vie processus.
- **FILE-024**: `devapp/src-tauri/src/application/command_runner.rs` - Spawn et
  capture stdout/stderr.
- **FILE-025**: `devapp/src-tauri/src/application/readiness.rs` - Readiness
  HTTP, log, delay et command.
- **FILE-026**: `devapp/src-tauri/src/application/events.rs` - Événements
  runtime.
- **FILE-027**: `devapp/src-tauri/src/tauri_api/commands.rs` - API exposée au
  frontend.
- **FILE-028**: `devapp/src-tauri/src/tauri_api/state.rs` - État Tauri partagé.
- **FILE-029**: `devapp/src/lib/components/ConfigEditor.svelte` - Édition YAML.
- **FILE-030**: `devapp/src-tauri/capabilities/default.json` - Permissions
  Tauri.
- **FILE-031**: `devapp/examples/deno-runner.yml` - Exemple projet auto-contenu.
- **FILE-032**: `devapp/docs/configuration.md` - Documentation config.
- **FILE-033**: `devapp/docs/runtime-model.md` - Documentation états runtime.
- **FILE-034**: `devapp/README.md` - Documentation utilisateur et développeur.

## 6. Testing

- **TEST-001**: `cargo test config_loader_loads_example` vérifie que l'exemple
  YAML fourni est parsé avec 3 processus et les dépendances attendues.
- **TEST-002**: `cargo test config_loader_rejects_unknown_dependency` vérifie
  qu'une dépendance vers un processus absent échoue.
- **TEST-003**: `cargo test config_loader_rejects_cycles` vérifie qu'un cycle de
  dépendances échoue.
- **TEST-004**: `cargo test config_loader_rejects_unsupported_version` vérifie
  que `version: 2` échoue tant que seule la version 1 est supportée.
- **TEST-005**: `cargo test orchestrator_runs_success_dependency_before_service`
  vérifie que `api` démarre uniquement après succès de `setup`.
- **TEST-006**:
  `cargo test orchestrator_stops_session_when_required_dependency_fails` vérifie
  qu'un échec de dépendance arrête toute la session.
- **TEST-007**: `cargo test orchestrator_marks_http_ready` démarre un serveur
  HTTP local et vérifie la transition `Ready`.
- **TEST-008**: `cargo test orchestrator_stops_all_children_on_session_stop`
  vérifie que l'arrêt de session arrête services et terminaux.
- **TEST-009**: `cargo test project_store_round_trip` vérifie sauvegarde,
  chargement, mise à jour et suppression d'un projet enregistré dans un dossier
  temporaire.
- **TEST-010**: `cargo test project_store_edits_project_yaml_or_app_yaml`
  vérifie l'édition d'un `devapp.yml` projet et d'une configuration YAML stockée
  par l'application.
- **TEST-011**: `deno task check` vérifie TypeScript et Svelte.
- **TEST-012**: `deno task build` vérifie le build frontend.
- **TEST-013**: `cargo test orchestrator_marks_log_ready` vérifie la transition
  `Ready` lorsqu'une ligne de log attendue est observée.
- **TEST-014**: `cargo test orchestrator_marks_delay_ready` vérifie la
  transition `Ready` après la durée configurée.
- **TEST-015**: `cargo test orchestrator_marks_command_ready` vérifie la
  transition `Ready` lorsqu'une commande de readiness retourne `0`.
- **TEST-016**: Test manuel desktop: lancer `examples/deno-runner.yml`, éditer
  le YAML, sélectionner chaque processus, vérifier les logs, ouvrir un terminal,
  arrêter la session.

## 7. Risks & Assumptions

- **RISK-001**: La gestion cross-platform des signaux et des process groups peut
  être différente entre Linux, macOS et Windows; l'implémentation doit
  encapsuler l'arrêt dans `command_runner.rs`.
- **RISK-002**: Les commandes multi-lignes passées au shell peuvent avoir des
  comportements différents selon l'OS; la documentation doit expliciter le shell
  utilisé.
- **RISK-003**: `portable-pty` peut nécessiter des adaptations par OS pour le
  resize, l'encodage et la fermeture propre.
- **RISK-004**: Une UI qui stocke trop de logs en mémoire peut devenir lente; la
  V1 impose un ring buffer par processus.
- **RISK-005**: Les readiness checks sans timeout configurable peuvent bloquer
  la session trop longtemps; la V1 définit un timeout par défaut de `60s`.
- **RISK-006**: `ready.type: command` peut créer des effets de bord si
  l'utilisateur configure une commande mutante; la documentation doit
  recommander des commandes idempotentes.
- **RISK-007**: L'édition YAML dans l'application peut perdre les commentaires
  si la sérialisation réécrit le fichier; la V1 doit préférer sauvegarder le
  texte édité tel quel après validation.
- **ASSUMPTION-001**: Le nom de configuration unique est `devapp.yml`.
- **ASSUMPTION-002**: Il n'y a pas de runner CLI en V1.
- **ASSUMPTION-003**: Les commandes YAML sont exécutées via le shell OS.
- **ASSUMPTION-004**: Les readiness checks V1 sont `http`, `log`, `delay`, et
  `command`.
- **ASSUMPTION-005**: Toute erreur bloquante arrête la session complète.
- **ASSUMPTION-006**: Les logs sont conservés uniquement en mémoire pendant la
  session pour la première version.
- **ASSUMPTION-007**: Une seule session active est autorisée par fenêtre;
  plusieurs fenêtres peuvent exécuter plusieurs projets en parallèle.
- **ASSUMPTION-008**: Le YAML édité dans l'application est sauvegardé dans
  `devapp.yml` si ce fichier existe dans le projet, sinon dans le dossier de
  configuration applicatif OS.

## 8. Related Specifications / Further Reading

- `devapp/README.md`
- `devapp/src-tauri/tauri.conf.json`
- `devapp/package.json`
- `devapp/src-tauri/Cargo.toml`
- `devapp/examples/deno-runner.yml`
- `devapp/docs/configuration.md`
- `devapp/docs/runtime-model.md`
