## ADDED Requirements

### Requirement: Shiki highlighter instance caching
The system SHALL cache a single Shiki highlighter instance at the `MarkdownPreviewer` class level and reuse it across all `render()` calls and code blocks within a call.

#### Scenario: Multiple code blocks in one markdown file
- **WHEN** a markdown file contains 5 code blocks with different languages
- **THEN** the system SHALL create the highlighter instance only once and reuse it for all 5 blocks

#### Scenario: Subsequent render calls
- **WHEN** `render()` is called multiple times (e.g., switching between files)
- **THEN** the system SHALL reuse the cached highlighter instance without recreating it

#### Scenario: Language not loaded
- **WHEN** a code block uses a language not yet loaded in the cached highlighter
- **THEN** the system SHALL load the language dynamically via `loadLanguage` and fall back to `text` on failure

#### Scenario: Cleanup on dispose
- **WHEN** `dispose()` is called
- **THEN** the system SHALL call `highlighter.dispose()` to release resources

### Requirement: Parallel image loading
The system SHALL load all local images in parallel using `Promise.all` instead of sequential awaits.

#### Scenario: Multiple local images
- **WHEN** a markdown file contains 3 local images (relative paths)
- **THEN** the system SHALL invoke `read_binary_file` for all 3 images concurrently

#### Scenario: Mixed local and remote images
- **WHEN** a markdown file contains both remote URLs and local image paths
- **THEN** the system SHALL skip remote URLs and load all local images in parallel

### Requirement: Mermaid module caching
The system SHALL cache the dynamically imported mermaid module reference at the class level.

#### Scenario: Multiple mermaid blocks
- **WHEN** a markdown file contains 2 mermaid code blocks
- **THEN** the system SHALL perform `import('mermaid')` only once and reuse the module for the second block

### Requirement: Parallel post-processing
The system SHALL execute code block highlighting and image loading concurrently using `Promise.all`.

#### Scenario: Code blocks and images present
- **WHEN** a markdown file has code blocks and local images
- **THEN** the system SHALL start highlighting code blocks and loading images in parallel, not sequentially
