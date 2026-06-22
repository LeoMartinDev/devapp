# TODO

## [DONE] Bug : Auto-scroll agressif dans la vue des logs

### Symptôme
Quand l'utilisateur scrolle manuellement vers le haut dans `LogViewer` pour
relire d'anciennes lignes, l'arrivée d'une nouvelle ligne de log le ramène
immédiatement tout en bas, ce qui empêche toute lecture.

### Cause racine
Dans `src/lib/components/LogViewer.svelte:107-116`, l'effet réactif utilise la
variable `autoScroll` (booléen) pour décider s'il faut scroller vers le bas
quand `filteredLogs.length` change. Or `autoScroll` n'est modifié que par le
clic sur le bouton toggle (ligne 158-172). Le défilement manuel de
l'utilisateur n'est pas détecté, donc `autoScroll` reste `true` et chaque
nouvelle ligne de log déclenche `viewport.scrollTo({ top: viewport.scrollHeight })`.

```ts
// LogViewer.svelte:107-116 — état actuel
$effect(() => {
    filteredLogs.length;
    autoScroll;
    paused;
    if (autoScroll && !paused && viewport) {
      requestAnimationFrame(() => {
        viewport?.scrollTo({ top: viewport.scrollHeight });
      });
    }
  });
```

### Solution
Remplacer le toggle manuel d'`autoScroll` par une détection automatique de la
position de scroll, en supprimant le bouton toggle existant (ou en le gardant
comme forçage manuel complémentaire).

#### Étape 1 — Ajouter un handler `onscroll` sur le viewport

Sur le `<div bind:this={viewport}>` (ligne 212), ajouter un handler `onscroll`
qui compare la position actuelle au bas du conteneur :

```ts
function handleScroll() {
  if (!viewport) return;
  const threshold = 4; // px de tolérance pour le floating-point
  const atBottom =
    viewport.scrollTop + viewport.clientHeight >=
    viewport.scrollHeight - threshold;
  autoScroll = atBottom;
}
```

#### Étape 2 — Mettre à jour le template

```svelte
<div
  bind:this={viewport}
  onscroll={handleScroll}
  class="min-h-0 flex-1 overflow-auto px-3 py-2 font-mono text-[12px] leading-[1.45]"
>
```

#### Étape 3 — Conserver ou retirer le bouton toggle

Deux options :
- **Recommandé** : garder le bouton comme overrride explicite (si l'utilisateur
  réactive `autoScroll`, le handler `onscroll` le remettra à `false` dès qu'il
  scrolle à nouveau).
- **Minimal** : supprimer le bouton et ne garder que la détection automatique.

#### Étape 4 — Vérifier l'interaction avec `paused`

Le `paused` n'est pas impacté car il gèle le flux de logs, pas le scroll. Les
deux mécanismes sont orthogonaux.

#### Fichiers modifiés

| Fichier | Changement |
|---|---|
| `src/lib/components/LogViewer.svelte` | Ajouter `handleScroll()`, lier `onscroll` au viewport, ajuster ou retirer le bouton toggle |

---

## Feature : Renforcer la distinction Task / Service dans l'UI

### Problème
Le type `kind: task` existe déjà dans le modèle de données (`ProcessKind::Task`
dans `src-tauri/src/domain/config.rs:70`), et le backend distingue correctement
le comportement :
- Une **task** qui sort avec status 0 passe à `succeeded` et débloque ses
  dépendants (`orchestrator.rs:586-604`)
- Un **service** qui sort inopinément est marqué `failed` et stoppe la session
  (`orchestrator.rs:608-614`)

Cependant, dans l'interface (`ProcessList.svelte`), une task est affichée
exactement comme un service : elle a des boutons **Stop** (quand elle tourne),
**Start** et **Restart** (quand elle est terminée). Or une task est censée
s'exécuter une fois jusqu'à la fin — on ne devrait ni l'arrêter manuellement,
ni la relancer depuis le bouton. De plus, rien ne la distingue visuellement
d'un service.

### Solution

#### Partie A — Supprimer les actions Stop / Start / Restart pour les tasks

Modifier `src/lib/components/ProcessList.svelte` pour que la fonction
`rowAction()` retourne `null` quand le process est de type `task` :

```ts
// Actuel (ligne 44-57)
function rowAction(status: ProcessStatus): RowAction {
    switch (status) {
      case "running":
      case "ready":
      case "starting":
        return "stop";
      case "stopped":
      case "failed":
      case "succeeded":
        return "start";
      default:
        return null;
    }
  }

// Nouveau
function rowAction(process: ProcessSnapshot): RowAction {
    // Les tasks ne sont jamais interactives
    if (process.kind === "task") return null;
    switch (process.status) {
      case "running":
      case "ready":
      case "starting":
        return "stop";
      case "stopped":
      case "failed":
      case "succeeded":
        return "start";
      default:
        return null;
    }
  }
```

Adapter tous les appels à `rowAction()` dans le template (lignes 80, 105) pour
passer l'objet `process` au lieu de `process.status`.

Adapter `actionEnabled()` (ligne 59-61) de la même façon.

#### Partie B — Icône spécifique pour les tasks (comme les terminaux)

Les terminaux ont leur propre icône SVG (lignes 181-184) à la place du
`StatusDot`. Les tasks doivent suivre le même principe : remplacer le
`<StatusDot>` par une icône SVG dédiée qui exprime visuellement la notion
de « tâche ponctuelle » (one-shot).

Ligne actuelle dans le template (ligne 93) :
```svelte
<StatusDot status={process.status} />
```

Remplacer par un bloc conditionnel :
```svelte
{#if process.kind === "task"}
  <TaskIcon />
{:else}
  <StatusDot status={process.status} />
{/if}
```

Icône proposée — un éclair (⚡) stylisé, évocateur d'une exécution rapide
et unique, dans le même style que l'icône terminal :

```svelte
<!-- TaskIcon : éclair / one-shot -->
<svg
  class="h-3.5 w-3.5 shrink-0 text-text-subtle"
  viewBox="0 0 24 24"
  fill="currentColor"
  aria-hidden="true"
>
  <path d="M13 2L3 14h8l-2 8 10-12h-8l2-8z" />
</svg>
```

Alternative possible : une icône « script » (lignes empilées) ou « check in
box ». L'important est que l'icône soit **visuellement distincte** du rond
coloré des services et de l'invite de commande des terminaux.

Note : le `StatusDot` est retiré pour les tasks car l'information de statut
reste lisible via le label texte (`statusLabel()`) affiché en dessous du nom.

#### Partie C (optionnelle) — Garde backend

Ajouter une vérification dans `orchestrator.rs` pour que `stop_process` et
`restart_process` refusent les tasks :

```rust
// orchestrator.rs, dans stop_process() (ligne 188)
let managed = state.sessions.get_mut(window_key)...;
let process = managed.processes.get(process_name)...;
if matches!(process.config.kind, ProcessKind::Task) {
    return Err(AppError::runtime("cannot stop a task process"));
}
```

Idem dans `restart_process()` (ligne 162) pour la double protection.

#### Fichiers modifiés

| Fichier | Changement |
|---|---|
| `src/lib/components/ProcessList.svelte` | `rowAction()` prend `ProcessSnapshot`, retourne `null` pour les tasks ; icône SVG dédiée (éclair) remplaçant `StatusDot` pour les tasks |
| `src-tauri/src/application/orchestrator.rs` | (optionnel) Rejet de `stop`/`restart` pour `ProcessKind::Task` |

#### Exemple de configuration cible

```yaml
version: 1
processes:
  docker:
    kind: task
    cmd: docker compose up -d

  api:
    kind: service
    cmd: deno task dev
    dependsOn:
      docker: success
    ready:
      type: log
      pattern: "listening"
```

Avec ces changements, `docker` :
- apparaît avec une icône d'éclair dédiée (distincte du rond coloré des services et de l'icône terminal)
- n'a **pas** de boutons Stop / Start / Restart
- s'exécute au lancement de la session, se termine, et passe à `succeeded`
- débloque `api` automatiquement via la dépendance `success`
