# Version Backup (shared module)

Before overwriting any document file, create a backup:

1. If `<file>.v1.md` doesn't exist → copy current file to `<file>.v1.md`
2. If `<file>.v1.md` exists → copy current to `<file>.v2.md` (keep v1 as original)
3. Maximum 3 backups: `.v1`, `.v2`, `.v3` — rotate oldest out when all three slots are full

**Rotation rule (when .v1, .v2, .v3 all exist):**
- Delete `.v3.md` (oldest backup)
- Rename `.v2.md` → `.v3.md`
- Rename `.v1.md` → `.v2.md`
- Copy current file → `.v1.md`

This module is referenced by all content workflows (prd, story, ideate, research, prioritize, prototype).

## Usage in workflows

Before writing the file, apply version backup from `core/shared/version-backup.md`:

```
Before writing the file, apply version backup from `core/shared/version-backup.md`.
```

## Backup naming convention

| Slot   | File name              | Meaning                  |
|--------|------------------------|--------------------------|
| v1     | `<doc>.v1.md`          | Most recent backup       |
| v2     | `<doc>.v2.md`          | One version older        |
| v3     | `<doc>.v3.md`          | Oldest kept backup       |

## Notes

- Backup files are **never shown in normal file listings** — they are safety copies only.
- The `compass:undo` workflow uses these backups to restore previous versions.
- Backup files live alongside the original document (same folder, same base name).
- Do not create backups of backup files (i.e., do not back up `.v1.md` → `.v1.v1.md`).
