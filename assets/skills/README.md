# Built-in Broken Skills Pack

`builtin-skills.zip` is the Claude++ built-in Claude Code skills pack.

- Source import: user-provided `skills.zip`
- Pack version: `2026.07.27`
- Included skills: 17
- Included files: 216
- Install target: `~/.claude/skills/`

Only top-level directories containing `SKILL.md` are packaged. Unrelated
repositories, Git metadata, journals, and root documentation from the source
archive are excluded.

Two metadata-only compatibility fixes are applied while packaging:

- `attack-chain/SKILL.md` receives the required YAML `name` and `description`.
- `reverse-engineering-codex/SKILL.md` uses a unique name matching its directory.

Claude++ never executes scripts from this archive during installation. Skill
scripts remain available for Claude to invoke later under the user's normal
Claude permissions and confirmation policy.
