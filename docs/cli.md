# Command Line Interface Reference

Interstellar Triangulum uses a structured CLI to manage the video creation workflow.

## Global Options
- `--help`: Print help information.
- `--version`: Print version information.

## Subcommands

### `render`
Render a script to video.

**Usage**: `interstellar-triangulum render [OPTIONS] <SCRIPT>`

**Arguments**:
- `<SCRIPT>`: Path to the JSON script file.

**Options**:
- `--renderer <ENGINE>`: Choose renderer backend. Values: `native` (default), `blender`.
- `--output <DIR>`: Directory to save frames and video. Default: `output`.
- `--export-report <FILE>`: Save analysis report to a file (JSON or Markdown).
- `--fail-on-low-score <THRESHOLD>`: Exit with error if narrative score is below threshold.

**Example**:
```bash
cargo run -- render my_script.json --renderer blender --output final_render --export-report report.md
```

### `validate`
Run analysis (Narrative + Credibility) without rendering. Useful for CI/CD or quick checks.

**Usage**: `interstellar-triangulum validate [OPTIONS] <SCRIPT>`

**Arguments**:
- `<SCRIPT>`: Path to the JSON script file.

**Options**:
- `--fail-on-warnings`: Exit with error if any warnings are detected (strict mode).

**Example**:
```bash
cargo run -- validate my_script.json --fail-on-warnings
```

### `template`
Generate a starter script programmatically.

**Usage**: `interstellar-triangulum template [OPTIONS] <TYPE>`

**Arguments**:
- `<TYPE>`: Template type. Values: `explainer`, `tutorial`, `storytelling`.

**Options**:
- `-d, --duration <SECONDS>`: Total target duration. Default: `60.0`.

**Example**:
```bash
cargo run -- template tutorial --duration 120 > tutorial.json
```

### `clean`
Remove generated artifacts.

**Usage**: `interstellar-triangulum clean`

**Description**:
Deletes the `output` directory (or configured output) and the `.cache` directory.
