# Coding Modes

Synerix adapts to how you work. Choose the mode that fits the task.

## Act Mode

**Direct execution. No context switches.**

Run commands, build projects, run tests, and manage files — all inside the terminal with AI-assisted safety.

| Feature | Detail |
|---|---|
| Sandbox | Auto-approves safe ops, previews risky ones |
| Commands | Full shell access with AI context |
| Use when | Building, testing, debugging, Git workflows |

```
❯ Build the project and run tests
   ✓ cargo build --release
   ✓ 142 tests passed
```

## Vibe Mode

**Immersive flow state.**

Describe what you want. Synerix handles the rest: generate, compile, test, and fix — automatically iterating until it works.

| Feature | Detail |
|---|---|
| Auto-approve | Low-risk file edits and builds |
| Auto-fix | Compilation errors fed back to AI |
| Use when | Implementing features, prototyping, refactoring |

```
❯ Add user preferences to the API
   ✓ Code generated
   ✓ cargo check passes
   ✓ Tests pass
   ✓ Done in 1.8s
```

## Chat Mode

**Conversational AI assistant.**

Ask questions, get explanations, brainstorm ideas, and review code — like pairing with a senior engineer.

| Feature | Detail |
|---|---|
| Context aware | Understands your codebase |
| Streaming | Real-time responses |
| Use when | Learning, explaining, debugging, planning |

```
❯ Explain how the authorization middleware works
   AI: The middleware sits between the router and handlers...
```

## Architect Mode

**Design and review at scale.**

Focus on architecture, module boundaries, design patterns, and long-term maintainability.

| Feature | Detail |
|---|---|
| Analysis | Reads full file structure and dependencies |
| Review | Code quality, patterns, performance |
| Use when | Designing systems, reviews, planning migrations |

```
❯ Review the auth module for potential improvements
   AI: Consider separating concerns into...
```

## Plan Mode

**Break it down before building.**

Decompose complex tasks into actionable, prioritized steps with estimated effort.

| Feature | Detail |
|---|---|
| Decomposition | Hierarchical task breakdown |
| Dependencies | Identifies blockers and ordering |
| Use when | Large features, migrations, unknowns |

```
❯ Plan a migration from REST to GraphQL
   1. Schema design (2h)
   2. Query resolver stubs (3h)
   3. Client migration (4h)
   ...
```

## Switching Modes

| Command | Mode |
|---|---|
| `/mode act` | Act |
| `/mode vibe` | Vibe |
| `/mode chat` | Chat |
| `/mode architect` | Architect |
| `/mode plan` | Plan |

Press `Tab` to cycle through modes without typing.

## Next Steps

- [Configuration](/guide/configuration) — Fine-tune mode behavior
- [Troubleshooting](/guide/troubleshooting) — Common issues
