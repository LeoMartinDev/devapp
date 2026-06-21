---
goal: "Corriger et refactorer le frontend Svelte apres review UX, composants et robustesse"
version: "1.0"
date_created: "2026-06-20"
last_updated: "2026-06-20"
owner: "Project maintainers"
status: "Planned"
tags: ["refactor", "frontend", "svelte", "ux", "accessibility", "design-system", "bugfix"]
---

# Introduction

![Status: Planned](https://img.shields.io/badge/status-Planned-blue)

Ce plan transforme les constats de review frontend en actions executables. Il vise a corriger les bugs d'affichage du terminal integre, a rendre l'editeur de configuration fiable, a introduire des primitives UI reutilisables, a renforcer l'accessibilite des dialogs, a ameliorer les performances des logs, et a reduire la duplication dans l'interface Svelte/Tailwind.

## 1. Requirements & Constraints

- **REQ-001**: Le terminal doit rester affiche dans l'interface Svelte via `xterm`; aucune fenetre terminal externe ne doit etre ouverte.
- **REQ-002**: L'editeur de configuration ne doit jamais masquer une erreur de chargement YAML par une configuration vide sauvegardable.
- **REQ-003**: L'editeur de configuration doit proteger les champs YAML inconnus, les commentaires, ou au minimum avertir explicitement que le save reconstruit le fichier.
- **REQ-004**: Les formulaires de projet et de configuration doivent valider les champs critiques avant appel backend.
- **REQ-005**: Les dialogs doivent supporter `role="dialog"`, `aria-modal`, focus initial, fermeture Escape, fermeture overlay controlee, et restauration du focus.
- **REQ-006**: Les logs doivent rester utilisables pendant les sessions longues sans rendre toutes les lignes indefiniment.
- **REQ-007**: Les actions processus doivent etre disponibles uniquement lorsque le statut du processus et l'etat `busy` les autorisent.
- **REQ-008**: L'interface doit rester utilisable sur une hauteur de 600px et sur une largeur inferieure a 1024px.
- **REQ-009**: Les composants UI generiques doivent etre reutilisables par `+page.svelte`, `ProjectSettingsDialog.svelte`, `ConfigEditor.svelte`, `ProcessList.svelte`, `LogViewer.svelte`, et `TerminalPane.svelte`.
- **REQ-010**: Les changements doivent rester limites au dossier applicatif `devapp/`.
- **REQ-011**: Une fenetre applicative doit representer un seul projet actif. Si une fenetre a deja un projet lance et que l'utilisateur lance un autre projet, l'application doit ouvrir une nouvelle fenetre independante pour cet autre projet.
- **REQ-012**: Chaque fenetre projet doit avoir son propre etat runtime: session, processus selectionne, terminal selectionne, logs, erreurs UI et configuration en cours d'edition.
- **REQ-013**: Fermer ou arreter un projet dans une fenetre ne doit pas arreter les sessions lancees dans les autres fenetres.
- **REQ-014**: Le terminal `xterm` reste integre dans chaque fenetre projet; la regle multi-fenetre concerne les projets, pas les terminaux.
- **CON-001**: La stack actuelle est Svelte 5, SvelteKit, TypeScript, Tailwind CSS 4, Tauri 2, et xterm.
- **CON-002**: Le projet compile actuellement avec `npm run check`; toutes les phases doivent conserver ce resultat.
- **CON-003**: Le code backend Rust existe deja; ce plan cible prioritairement le frontend et ne modifie le backend que si une validation frontend necessite un contrat plus explicite.
- **CON-004**: Le plan initial `feature-process-runner-desktop-1.md` contenait l'ancienne contrainte "une seule session active"; cette contrainte est remplacee par le modele "une session par fenetre projet".
- **PAT-001**: Utiliser les runes Svelte 5 existantes (`$state`, `$derived`, `$effect`) pour rester coherent avec le code actuel.
- **PAT-002**: Garder Tailwind comme systeme de style principal; limiter le CSS global aux styles de base et aux imports de librairies.
- **PAT-003**: Introduire des composants UI petits, typables, sans logique metier, puis les composer dans les composants metier.
- **PAT-004**: Les messages d'erreur utilisateur doivent etre affiches pres du champ ou de l'action concernee, pas seulement dans `runtimeStore.uiError`.

## 2. Implementation Steps

### Implementation Phase 1

- GOAL-001: Corriger les bugs utilisateur a fort impact avant tout refactor visuel.

| Task | Description | Completed | Date |
|------|-------------|-----------|------|
| TASK-001 | Modifier `devapp/src/lib/components/TerminalPane.svelte`: deplacer l'appel `xterm.open(host)` dans un `$effect` qui s'execute lorsque `xterm !== null`, `host !== null`, et que le terminal n'est pas encore attache au DOM. Ajouter un booleen local `terminalOpened = $state(false)`. | Yes | 2026-06-20 |
| TASK-002 | Modifier `TerminalPane.svelte`: dans le meme `$effect`, appeler `fit()` apres `xterm.open(host)` avec `requestAnimationFrame` pour garantir que le terminal integre occupe la taille disponible dans l'interface. | Yes | 2026-06-20 |
| TASK-003 | Modifier `TerminalPane.svelte`: quand `terminalId` passe de `null` a une valeur, garder l'instance xterm existante, reinitialiser uniquement le contenu via `xterm.reset()`, puis ecrire le `output` courant si disponible. | Yes | 2026-06-20 |
| TASK-004 | Ajouter une verification manuelle documentee: ouvrir l'onglet Terminal avant de cliquer sur `Term`, cliquer sur `Term`, verifier que le terminal apparait dans la vue droite sans ouvrir de fenetre externe. | Yes | 2026-06-20 |
| TASK-005 | Modifier `devapp/src/lib/components/ConfigEditor.svelte`: remplacer le `catch` de `load(projectId)` qui appelle `runtimeStore.clearError()` et `resetEmpty()` par un etat local `loadError = $state<string | null>(null)`. | Yes | 2026-06-20 |
| TASK-006 | Modifier `ConfigEditor.svelte`: lorsque `loadError !== null`, afficher un panneau d'erreur dans le dialog, ne pas afficher le formulaire genere vide, et desactiver `Save settings`. | Yes | 2026-06-20 |
| TASK-007 | Modifier `ConfigEditor.svelte`: recharger la configuration a chaque ouverture du dialog pour le meme projet, sauf si un futur etat `dirty` explicite demande confirmation. Retirer la condition bloquante `projectId === loadedProjectId` ou la remplacer par une cle `openLoadKey`. | Yes | 2026-06-20 |

Manual verification for TASK-004:

1. Open the Terminal tab before creating a terminal.
2. Click `Term`.
3. Verify the terminal appears in the right pane of the app and no external terminal window opens.

### Implementation Phase 1A

- GOAL-001A: Adapter le modele UI au requirement "une fenetre independante par projet lance".

| Task | Description | Completed | Date |
|------|-------------|-----------|------|
| TASK-007A | Modifier le comportement de lancement dans `devapp/src/routes/+page.svelte`: si la fenetre courante a deja une `session` pour un projet different de `selectedProjectId`, ne pas remplacer la session courante; demander au backend/Tauri d'ouvrir une nouvelle fenetre pour `selectedProjectId`. | Yes | 2026-06-20 |
| TASK-007B | Ajouter dans `devapp/src/lib/tauri/client.ts` une fonction typee `openProjectWindow(projectId: ProjectId): Promise<void>` si la commande Tauri n'existe pas encore cote frontend. | Yes | 2026-06-20 |
| TASK-007C | Modifier `devapp/src/lib/stores/runtime.svelte.ts`: ajouter un champ `windowProjectId = $state<ProjectId | null>(null)` ou equivalent pour identifier le projet attache a la fenetre courante. | Yes | 2026-06-20 |
| TASK-007D | Modifier `runtime.svelte.ts`: filtrer ou ignorer les evenements runtime qui ne correspondent pas au `windowProjectId` de la fenetre courante, afin que deux fenetres ne melangent pas logs, terminaux et snapshots. | Yes | 2026-06-20 |
| TASK-007E | Modifier le shell dans `+page.svelte` ou `ProjectSelector.svelte`: quand une session est active dans la fenetre, afficher clairement le projet attache a cette fenetre et traiter la selection d'un autre projet comme une action "Open in new window" plutot que "replace current project". | Yes | 2026-06-20 |
| TASK-007F | Ajouter une action explicite `Open in new window` dans la selection projet ou le menu projet pour permettre de lancer un projet dans une fenetre independante sans perturber la fenetre courante. | Yes | 2026-06-20 |
| TASK-007G | Documenter le contrat backend attendu: la commande `open_project_window(projectId)` cree une fenetre Tauri avec un label unique par projet ou par instance, transmet `projectId` a la nouvelle fenetre, puis cette fenetre initialise son store avec ce projet. | Yes | 2026-06-20 |
| TASK-007H | Verifier que `stopCurrentProject()`, `closeSelectedTerminal()`, `restartSessionProcess()` et `stopSessionProcess()` ciblent uniquement la session de la fenetre courante, jamais une session d'une autre fenetre. | Yes | 2026-06-20 |

Backend/window contract for TASK-007G:

`open_project_window(projectId)` resolves the project, focuses an existing window labeled `project-{projectId}` when present, or creates a new Tauri window with URL parameters `projectId=<id>&autorun=1`. The new window initializes `runtimeStore.windowProjectId` from the URL, selects that project, and starts the session in that window. Runtime session commands continue to target the current Tauri window label via the backend `window_key`, so stop, process actions, and terminals remain scoped to the window that issued the command.

### Implementation Phase 2

- GOAL-002: Rendre l'editeur de configuration fiable et explicite sur la sauvegarde YAML.

| Task | Description | Completed | Date |
|------|-------------|-----------|------|
| TASK-008 | Extraire depuis `ConfigEditor.svelte` les fonctions pures `createProcess`, `toProcessForm`, `buildConfig`, `buildProcessConfig`, `buildReadyConfig`, `serializeConfig`, `yamlScalar`, et `yamlKey` vers `devapp/src/lib/config/editorModel.ts`. | Yes | 2026-06-20 |
| TASK-009 | Creer `devapp/src/lib/config/validation.ts` avec `validateProjectDetails(input)`, `validateConfigForm(formState)`, et types `ValidationIssue`, `ValidationResult`. | Yes | 2026-06-20 |
| TASK-010 | Dans `validation.ts`, valider les projets: `name.trim().length > 0`, `baseDir.trim().length > 0`, `configSource` dans `projectFile | appConfigFile | ""`. | Yes | 2026-06-20 |
| TASK-011 | Dans `validation.ts`, valider les process: nom non vide, noms uniques apres trim, commande non vide, dependencies vers process existants, absence de dependance vers soi-meme. | Yes | 2026-06-20 |
| TASK-012 | Dans `validation.ts`, valider readiness HTTP avec `new URL(value)` et protocoles `http:` ou `https:`. Valider readiness delay, interval et timeout comme nombres finis superieurs ou egaux a `0`. | Yes | 2026-06-20 |
| TASK-013 | Dans `ConfigEditor.svelte`, stocker les erreurs de validation par champ avec une cle stable, par exemple `process.<processId>.name`, `process.<processId>.cmd`, `env.<envId>.key`, `ready.httpUrl`. | Yes | 2026-06-20 |
| TASK-014 | Dans `ConfigEditor.svelte`, bloquer `save()` si `validateConfigForm` retourne des erreurs. Afficher les erreurs inline sous les inputs concernes et un recapitulatif compact dans le footer. | Yes | 2026-06-20 |
| TASK-015 | Ajouter dans `ConfigEditor.svelte` un avertissement visible si le mode actuel continue de serialiser le YAML complet: "Saving rewrites the generated YAML and may remove comments or unsupported fields." | Yes | 2026-06-20 |
| TASK-016 | Etudier puis implementer une option de preservation YAML: ajouter un mode `Raw YAML` avec textarea qui edite `document.yaml` directement, appelle `runtimeStore.loadConfig(project.id, yaml)` pour validation, puis `saveConfig(yaml)`. | Yes | 2026-06-20 |
| TASK-017 | Dans le mode formulaire, conserver une action secondaire `Preview YAML` qui affiche le resultat de `serializeConfig(buildConfig())` en lecture seule avant sauvegarde. | Yes | 2026-06-20 |

### Implementation Phase 3

- GOAL-003: Creer des primitives UI partagees pour reduire la duplication et stabiliser le design.

| Task | Description | Completed | Date |
|------|-------------|-----------|------|
| TASK-018 | Creer `devapp/src/lib/components/ui/Button.svelte` avec props `variant: "primary" | "secondary" | "danger" | "ghost"`, `size: "sm" | "md"`, `disabled`, `type`, et slot contenu. | Yes | 2026-06-20 |
| TASK-019 | Creer `devapp/src/lib/components/ui/IconButton.svelte` avec props `label`, `disabled`, `variant`, `title`, slot icone/texte, et `aria-label={label}`. | Yes | 2026-06-20 |
| TASK-020 | Creer `devapp/src/lib/components/ui/TextField.svelte` avec props `label`, `value`, `placeholder`, `error`, `type`, `autocomplete`, `monospace`, et binding `value`. | Yes | 2026-06-20 |
| TASK-021 | Creer `devapp/src/lib/components/ui/SelectField.svelte` avec props `label`, `value`, `options`, `error`, et binding `value`. | Yes | 2026-06-20 |
| TASK-022 | Creer `devapp/src/lib/components/ui/CheckboxField.svelte` avec props `label`, `checked`, `error`, et binding `checked`. | Yes | 2026-06-20 |
| TASK-023 | Creer `devapp/src/lib/components/ui/Badge.svelte` avec props `tone: "neutral" | "success" | "warning" | "danger" | "info"`. | Yes | 2026-06-20 |
| TASK-024 | Creer `devapp/src/lib/components/ui/StatusDot.svelte` avec prop `status` basee sur `ProcessStatus`, et centraliser le mapping couleur actuellement dans `ProcessList.svelte`. | Yes | 2026-06-20 |
| TASK-025 | Creer `devapp/src/lib/components/ui/SegmentedControl.svelte` pour remplacer le switch `Logs / Terminal` dans `+page.svelte`. | Yes | 2026-06-20 |
| TASK-026 | Remplacer progressivement les boutons, inputs, selects, badges et dots du shell principal et des dialogs par ces primitives sans changer le comportement metier. | Yes | 2026-06-20 |

### Implementation Phase 4

- GOAL-004: Decouper les composants metier pour clarifier les responsabilites.

| Task | Description | Completed | Date |
|------|-------------|-----------|------|
| TASK-027 | Extraire depuis `devapp/src/routes/+page.svelte` un composant `devapp/src/lib/components/AppShell.svelte` qui porte la grille principale, aside, header et zone de contenu. | Yes | 2026-06-20 |
| TASK-028 | Extraire depuis `+page.svelte` un composant `ProjectSelector.svelte` pour la zone projet, le select, l'etat idle/ready, et les metadonnees projet. | Yes | 2026-06-20 |
| TASK-029 | Extraire depuis `+page.svelte` un composant `RuntimeToolbar.svelte` pour les actions `Run`, `Stop`, `Term`, `Config`, `Project`. | Yes | 2026-06-20 |
| TASK-030 | Remplacer `ProjectPicker.svelte` ou le supprimer. Si conserve, le reutiliser dans le shell actuel; sinon supprimer le fichier et les props obsoletes. | Yes | 2026-06-20 |
| TASK-031 | Retirer la fonction morte `handleRemoveProject` de `+page.svelte` ou ajouter une action visible `Remove project` dans `ProjectSelector` avec confirmation via dialog applicatif. | Yes | 2026-06-20 |
| TASK-032 | Extraire depuis `ConfigEditor.svelte` un composant `EnvEditor.svelte` pour les variables globales. | Yes | 2026-06-20 |
| TASK-033 | Extraire depuis `ConfigEditor.svelte` un composant `ConfigProcessList.svelte` pour la liste laterale des processus. | Yes | 2026-06-20 |
| TASK-034 | Extraire depuis `ConfigEditor.svelte` un composant `ProcessForm.svelte` pour nom, commande, kind et suppression. | Yes | 2026-06-20 |
| TASK-035 | Extraire depuis `ConfigEditor.svelte` un composant `DependencyEditor.svelte` pour ajouter, selectionner et supprimer les dependances. | Yes | 2026-06-20 |
| TASK-036 | Extraire depuis `ConfigEditor.svelte` un composant `ReadyCheckEditor.svelte` pour les controles HTTP, log, delay, command, interval et timeout. | Yes | 2026-06-20 |

### Implementation Phase 5

- GOAL-005: Ameliorer accessibilite, navigation clavier et feedback utilisateur.

| Task | Description | Completed | Date |
|------|-------------|-----------|------|
| TASK-037 | Creer `devapp/src/lib/components/ui/Dialog.svelte` avec props `open`, `title`, `description`, `size`, `onClose`, `closeOnOverlay`, et slots `default`, `footer`. | Yes | 2026-06-20 |
| TASK-038 | Dans `Dialog.svelte`, ajouter `role="dialog"`, `aria-modal="true"`, `aria-labelledby`, gestion Escape, focus initial sur le premier champ/action, focus trap minimal, et restauration du focus a la fermeture. | Yes | 2026-06-20 |
| TASK-039 | Remplacer l'overlay manuel de `ProjectSettingsDialog.svelte` par `Dialog.svelte`. | Yes | 2026-06-20 |
| TASK-040 | Remplacer l'overlay manuel de `ConfigEditor.svelte` par `Dialog.svelte`. | Yes | 2026-06-20 |
| TASK-041 | Remplacer `confirm()` natif dans la suppression projet par un `ConfirmDialog.svelte` base sur `Dialog.svelte`. | Yes | 2026-06-20 |
| TASK-042 | Ajouter les attributs `aria-current`, `aria-pressed`, ou `aria-selected` au besoin dans `SegmentedControl`, `ProcessList`, et la liste de process de l'editeur. | Yes | 2026-06-20 |
| TASK-043 | Ajouter des libelles explicites aux boutons courts `+`, `Term`, `Config`, `Close`, `Clear`, `Restart`, `Stop` via texte visible plus clair ou `aria-label` + `title`. | Yes | 2026-06-20 |

### Implementation Phase 6

- GOAL-006: Stabiliser les logs, les actions processus et le responsive.

| Task | Description | Completed | Date |
|------|-------------|-----------|------|
| TASK-044 | Modifier `runtime.svelte.ts`: definir une constante `MAX_LOG_LINES_PER_PROCESS = 10000` et tronquer les logs par process lors de chaque append. | Yes | 2026-06-20 |
| TASK-045 | Modifier `LogViewer.svelte`: ajouter un compteur de lignes affichees et une indication lorsque les logs sont tronques. | Yes | 2026-06-20 |
| TASK-046 | Modifier `LogViewer.svelte`: ajouter une action `Pause` qui bloque l'auto-scroll et fige temporairement la vue sans arreter la collecte dans le store. | Yes | 2026-06-20 |
| TASK-047 | Modifier `LogViewer.svelte`: remplacer `break-all` par une strategie plus lisible, par exemple `break-words whitespace-pre-wrap`, sauf pour les tokens tres longs. | Yes | 2026-06-20 |
| TASK-048 | Modifier `ProcessList.svelte`: ajouter prop `busy` et desactiver `Restart`/`Stop` pendant les operations globales. | Yes | 2026-06-20 |
| TASK-049 | Modifier `ProcessList.svelte`: calculer les actions autorisees par statut. Desactiver `Stop` pour `succeeded`, `failed`, `stopped`; desactiver `Restart` pour `pending`, `blocked`, `starting`, `stopping`. | Yes | 2026-06-20 |
| TASK-050 | Modifier `+page.svelte` et `ConfigEditor.svelte`: remplacer les grilles fixes par des variantes responsive `grid-cols-1 md:grid-cols-*`, et retirer `min-h-[620px]` si cela force un overflow inutile sur petits ecrans. | Yes | 2026-06-20 |
| TASK-051 | Modifier `ConfigEditor.svelte`: sur largeur mobile/tablette, transformer la sidebar des processus en zone horizontale scrollable ou en select, puis afficher le formulaire en une colonne. | Yes | 2026-06-20 |

### Implementation Phase 7

- GOAL-007: Verifier, documenter et verrouiller la qualite.

| Task | Description | Completed | Date |
|------|-------------|-----------|------|
| TASK-052 | Executer `npm run check` depuis `devapp/` et corriger toutes les erreurs TypeScript/Svelte. | Yes | 2026-06-20 |
| TASK-053 | Executer `npm run build` depuis `devapp/` et corriger toutes les erreurs de build. | Yes | 2026-06-20 |
| TASK-054 | Executer un parcours manuel desktop: creer projet, editer projet, ouvrir config, provoquer erreur YAML, corriger, sauvegarder, lancer projet, regarder logs, ouvrir terminal integre, fermer terminal. |  |  |
| TASK-055 | Executer un parcours manuel responsive avec largeur inferieure a 768px: ouvrir shell, config dialog, actions processus, logs, terminal. Verifier absence d'overlap et de contenu inaccessible. |  |  |
| TASK-056 | Ajouter une section `Frontend review remediation` dans `devapp/README.md` ou `devapp/docs/runtime-model.md` avec les decisions: terminal integre, validation config, YAML raw/form, limites logs, composants UI. | Yes | 2026-06-20 |
| TASK-057 | Executer un parcours manuel multi-fenetre: lancer projet A dans une fenetre, selectionner projet B et lancer, verifier qu'une deuxieme fenetre s'ouvre, puis verifier que l'arret de A ne stoppe pas B. |  |  |
| TASK-058 | Executer un parcours manuel multi-fenetre avec terminaux: ouvrir un terminal dans la fenetre A et un terminal dans la fenetre B, verifier que les inputs/outputs ne se melangent pas. |  |  |

Validation note for 2026-06-20:

- `deno task check` passed with `svelte-check found 0 errors and 0 warnings`.
- `deno task build` completed successfully.
- `deno task tauri dev` could not be used for manual desktop and multi-window validation in this agent environment. The Tauri dev flow delegates to the configured frontend dev server on `127.0.0.1:1420`; the sandboxed run failed with `EACCES`, and the unsandboxed run found an existing user-owned dev server on the fixed Tauri port.
- Headless Chrome screenshots against the existing dev server produced a blank SvelteKit bootstrap page, so responsive/manual UI tasks were not marked complete from that evidence.

## 3. Alternatives

- **ALT-001**: Garder tous les styles Tailwind inline et corriger seulement les bugs. Rejete parce que la duplication actuelle augmente le risque d'incoherence entre dialogs, boutons, inputs et badges.
- **ALT-002**: Remplacer xterm par une textarea ou un faux terminal. Rejete parce que l'application a besoin d'un terminal interactif integre avec input, resize et rendu ANSI.
- **ALT-003**: Ouvrir le terminal systeme externe au lieu du terminal integre. Rejete parce que le requirement produit est d'afficher le terminal dans l'interface.
- **ALT-004**: Forcer uniquement l'edition formulaire et supprimer toute edition YAML brute. Rejete parce que le formulaire ne represente pas forcement toutes les possibilites YAML futures et peut supprimer commentaires/champs inconnus.
- **ALT-005**: Introduire une grosse librairie de composants UI. Rejete pour l'instant car les besoins sont simples, l'app est dense, et les primitives locales suffisent.
- **ALT-006**: Virtualiser immediatement les logs avec une dependance externe. Differe; un ring buffer et une limite d'affichage corrigent d'abord le risque principal avec moins de complexite.
- **ALT-007**: Remplacer la session de la fenetre courante lorsqu'un autre projet est lance. Rejete parce que le produit exige une instance/fenetre independante par projet lance.
- **ALT-008**: Gerer plusieurs projets actifs dans une seule fenetre avec des tabs. Rejete pour ce requirement parce que l'isolation attendue est une fenetre independante par projet.

## 4. Dependencies

- **DEP-001**: `xterm` et `@xterm/addon-fit` deja presents pour le terminal integre.
- **DEP-002**: Svelte 5 et runes `$state`, `$derived`, `$effect` deja utilisees dans le code.
- **DEP-003**: Tailwind CSS 4 deja present comme systeme de style.
- **DEP-004**: Contrats Tauri existants dans `devapp/src/lib/tauri/client.ts`.
- **DEP-005**: Types existants dans `devapp/src/lib/types.ts`, notamment `ProcessStatus`, `ReadyConfig`, `ProjectRecord`, `ProcessSnapshot`, `ProcessLogPayload`.
- **DEP-006**: Aucune nouvelle dependance obligatoire pour les phases 1 a 5.
- **DEP-007**: API Tauri de creation de fenetre cote backend ou commande custom `open_project_window(projectId)` exposee au frontend.

## 5. Files

- **FILE-001**: `devapp/src/routes/+page.svelte` - Shell principal, selection projet, toolbar runtime, switch logs/terminal.
- **FILE-002**: `devapp/src/lib/components/TerminalPane.svelte` - Correction attachement xterm au DOM et rendu terminal integre.
- **FILE-003**: `devapp/src/lib/components/ConfigEditor.svelte` - Chargement, validation, sauvegarde, decoupage de l'editeur de configuration.
- **FILE-004**: `devapp/src/lib/components/ProjectSettingsDialog.svelte` - Validation projet et migration vers `Dialog`.
- **FILE-005**: `devapp/src/lib/components/ProcessList.svelte` - Actions conditionnelles, status dot partage, busy state.
- **FILE-006**: `devapp/src/lib/components/LogViewer.svelte` - Troncature, pause, lisibilite, feedback volume.
- **FILE-007**: `devapp/src/lib/components/ProjectPicker.svelte` - Suppression ou reintegration.
- **FILE-008**: `devapp/src/lib/stores/runtime.svelte.ts` - Limite logs, erreurs UI, actions runtime.
- **FILE-009**: `devapp/src/lib/config/editorModel.ts` - Nouveau fichier pour les conversions config/form/YAML.
- **FILE-010**: `devapp/src/lib/config/validation.ts` - Nouveau fichier pour validation projet et configuration.
- **FILE-011**: `devapp/src/lib/components/ui/Button.svelte` - Nouvelle primitive UI.
- **FILE-012**: `devapp/src/lib/components/ui/IconButton.svelte` - Nouvelle primitive UI.
- **FILE-013**: `devapp/src/lib/components/ui/TextField.svelte` - Nouvelle primitive UI.
- **FILE-014**: `devapp/src/lib/components/ui/SelectField.svelte` - Nouvelle primitive UI.
- **FILE-015**: `devapp/src/lib/components/ui/CheckboxField.svelte` - Nouvelle primitive UI.
- **FILE-016**: `devapp/src/lib/components/ui/Badge.svelte` - Nouvelle primitive UI.
- **FILE-017**: `devapp/src/lib/components/ui/StatusDot.svelte` - Nouvelle primitive UI.
- **FILE-018**: `devapp/src/lib/components/ui/SegmentedControl.svelte` - Nouvelle primitive UI.
- **FILE-019**: `devapp/src/lib/components/ui/Dialog.svelte` - Nouvelle primitive UI accessibilite.
- **FILE-020**: `devapp/src/lib/components/ui/ConfirmDialog.svelte` - Nouvelle primitive de confirmation.
- **FILE-021**: `devapp/src/lib/components/EnvEditor.svelte` - Nouveau composant metier extrait de `ConfigEditor`.
- **FILE-022**: `devapp/src/lib/components/ConfigProcessList.svelte` - Nouveau composant metier extrait de `ConfigEditor`.
- **FILE-023**: `devapp/src/lib/components/ProcessForm.svelte` - Nouveau composant metier extrait de `ConfigEditor`.
- **FILE-024**: `devapp/src/lib/components/DependencyEditor.svelte` - Nouveau composant metier extrait de `ConfigEditor`.
- **FILE-025**: `devapp/src/lib/components/ReadyCheckEditor.svelte` - Nouveau composant metier extrait de `ConfigEditor`.
- **FILE-026**: `devapp/README.md` - Documentation des decisions frontend si retenue.
- **FILE-027**: `devapp/src/lib/tauri/client.ts` - Ajout du client `openProjectWindow(projectId)` si necessaire.
- **FILE-028**: `devapp/src-tauri/src/tauri_api/commands.rs` - Commande backend attendue pour ouvrir une fenetre projet independante si elle n'existe pas encore.
- **FILE-029**: `devapp/src-tauri/src/application/orchestrator.rs` - Isolation des sessions par fenetre/projet si le backend est encore limite a une session globale.

## 6. Testing

- **TEST-001**: `npm run check` doit retourner `0 errors and 0 warnings`.
- **TEST-002**: `npm run build` doit terminer sans erreur.
- **TEST-003**: Test manuel terminal: ouvrir l'onglet Terminal avant creation d'un terminal, cliquer sur `Term`, verifier que xterm s'affiche dans la vue droite de l'application.
- **TEST-004**: Test manuel terminal: taper une commande dans xterm, verifier que l'input est envoye via `writeTerminal` et que l'output revient dans le composant.
- **TEST-005**: Test manuel config load error: fournir un YAML invalide, ouvrir `ConfigEditor`, verifier qu'une erreur est affichee et que `Save settings` est desactive.
- **TEST-006**: Test manuel validation: tenter de sauvegarder deux process avec le meme nom, une commande vide, une URL readiness invalide, et une dependance inconnue; verifier les erreurs inline.
- **TEST-007**: Test manuel YAML raw: editer le YAML brut, valider, sauvegarder, rouvrir et verifier que le contenu sauvegarde correspond.
- **TEST-008**: Test manuel logs: generer plus de `MAX_LOG_LINES_PER_PROCESS` lignes et verifier que l'interface reste fluide et signale la troncature.
- **TEST-009**: Test manuel responsive: largeur inferieure a 768px et hauteur proche de 600px, verifier shell, dialogs, logs, terminal et absence de chevauchement.
- **TEST-010**: Test manuel accessibilite clavier: ouvrir chaque dialog au clavier, naviguer avec Tab/Shift+Tab, fermer avec Escape, verifier retour du focus.
- **TEST-011**: Test manuel multi-fenetre: projet A lance dans fenetre A, projet B lance depuis fenetre A, verifier ouverture fenetre B et independance complete des sessions.
- **TEST-012**: Test manuel isolation logs: generer des logs dans A et B, verifier que chaque fenetre affiche uniquement les logs de son projet.
- **TEST-013**: Test manuel isolation terminaux: ouvrir un terminal dans A et B, verifier que resize, input, close et output restent attaches a leur fenetre.

## 7. Risks & Assumptions

- **RISK-001**: Preserver commentaires et champs inconnus YAML avec un formulaire structure peut demander une vraie librairie YAML AST. Le mode Raw YAML est la solution la plus simple si la preservation complete devient prioritaire.
- **RISK-002**: Le focus trap maison peut etre incomplet. Si les besoins accessibilite grandissent, remplacer `Dialog.svelte` par une solution specialisee ou une action Svelte testee.
- **RISK-003**: Les primitives UI peuvent augmenter le nombre de fichiers mais reduisent la duplication et la variance visuelle.
- **RISK-004**: Limiter les logs en memoire peut surprendre si l'utilisateur attend un historique complet. Il faut afficher clairement la troncature et prevoir plus tard un export.
- **RISK-005**: Recharger la config a chaque ouverture peut supprimer des modifications non sauvegardees si l'utilisateur ferme accidentellement. Ajouter un etat `dirty` et une confirmation avant fermeture regle ce risque.
- **RISK-006**: Si le backend conserve une session globale unique, le frontend ne pourra pas garantir l'independance multi-fenetre. Le backend doit isoler les sessions par fenetre/projet ou par identifiant d'instance.
- **RISK-007**: Des labels Tauri non uniques par projet peuvent empecher d'ouvrir deux instances du meme projet. Definir si l'app autorise une seule fenetre par projet ou plusieurs instances du meme projet avant implementation.
- **ASSUMPTION-001**: Le terminal doit etre integre dans la vue principale de droite, comme actuellement prevu par `TerminalPane.svelte`.
- **ASSUMPTION-002**: Le backend Tauri reste la source de verite pour la validation finale et l'execution; la validation frontend ameliore l'UX mais ne remplace pas les controles backend.
- **ASSUMPTION-003**: Aucun framework de composants externe n'est requis pour atteindre le niveau UX attendu.
- **ASSUMPTION-004**: Le modele cible est une session active par fenetre projet, avec plusieurs fenetres possibles dans le meme processus Tauri.

## 8. Related Specifications / Further Reading

- `devapp/plan/feature-process-runner-desktop-1.md`
- `devapp/src/lib/components/TerminalPane.svelte`
- `devapp/src/lib/components/ConfigEditor.svelte`
- `devapp/src/lib/components/ProjectSettingsDialog.svelte`
- `devapp/src/lib/components/ProcessList.svelte`
- `devapp/src/lib/components/LogViewer.svelte`
- `devapp/src/lib/stores/runtime.svelte.ts`
