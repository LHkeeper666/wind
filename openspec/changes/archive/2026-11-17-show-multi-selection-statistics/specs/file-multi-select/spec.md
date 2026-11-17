## ADDED Requirements

### Requirement: File information follows explicit selection
The system SHALL use the explicit selection for `i` in a local directory panel or project tree. When no explicit selection exists, it SHALL use the cursor item.

#### Scenario: Selected item differs from cursor
- **WHEN** one or more items are explicitly selected and the cursor is on another item
- **THEN** pressing `i` shows information for the selected item or items without including the cursor item

#### Scenario: No selected items
- **WHEN** no item is explicitly selected
- **THEN** pressing `i` opens information for the cursor item as before
