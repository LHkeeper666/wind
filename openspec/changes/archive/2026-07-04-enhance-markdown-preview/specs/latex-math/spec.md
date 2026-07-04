## ADDED Requirements

### Requirement: Inline LaTeX with dollar signs
The system SHALL render inline LaTeX formulas delimited by single dollar signs `$...$` using katex.

#### Scenario: Simple inline formula
- **WHEN** markdown contains `$E=mc^2$`
- **THEN** the preview displays a rendered inline formula with the equation E=mc²

#### Scenario: Dollar sign in non-formula context
- **WHEN** markdown contains `The price is $5.00`
- **THEN** the preview displays the literal text "$5.00" without formula rendering

### Requirement: Block LaTeX with double dollar signs
The system SHALL render block LaTeX formulas delimited by double dollar signs `$$...$$` using katex, displayed as a centered block element.

#### Scenario: Block formula
- **WHEN** markdown contains `$$\int_0^1 x^2 dx = \frac{1}{3}$$`
- **THEN** the preview displays a centered block formula with the integral rendered

### Requirement: LaTeX with paren/bracket delimiters
The system SHALL render inline formulas with `\(...\)` and block formulas with `\[...\]`.

#### Scenario: Inline formula with parentheses
- **WHEN** markdown contains `\(E=mc^2\)`
- **THEN** the preview displays a rendered inline formula

#### Scenario: Block formula with brackets
- **WHEN** markdown contains `\[E=mc^2\]`
- **THEN** the preview displays a centered block formula

### Requirement: Graceful error handling
The system SHALL display the original LaTeX source text when katex fails to parse a formula, without breaking the rest of the document.

#### Scenario: Invalid LaTeX syntax
- **WHEN** markdown contains `$\invalid{syntax}$`
- **THEN** the preview displays the raw text `$\invalid{syntax}$` without an error overlay
