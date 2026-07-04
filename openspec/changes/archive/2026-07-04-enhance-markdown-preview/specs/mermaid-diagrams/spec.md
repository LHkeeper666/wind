## ADDED Requirements

### Requirement: Mermaid code block rendering
The system SHALL render ` ```mermaid ` code blocks as SVG diagrams using the mermaid library.

#### Scenario: Flowchart diagram
- **WHEN** markdown contains a mermaid code block with `graph TD; A-->B;`
- **THEN** the preview displays a rendered flowchart diagram

#### Sequence diagram
- **WHEN** markdown contains a mermaid code block with `sequenceDiagram; A->>B: Hello;`
- **THEN** the preview displays a rendered sequence diagram

### Requirement: Mermaid error fallback
The system SHALL display the original mermaid source code when rendering fails.

#### Scenario: Invalid mermaid syntax
- **WHEN** markdown contains a mermaid code block with invalid syntax
- **THEN** the preview displays the raw code with a visual indicator of the error

### Requirement: Theme consistency
The system SHALL render mermaid diagrams with a theme that matches the current application theme (dark/light).

#### Scenario: Dark theme
- **WHEN** the application is in dark theme and a mermaid diagram is rendered
- **THEN** the diagram uses dark theme colors (light text on dark background)
