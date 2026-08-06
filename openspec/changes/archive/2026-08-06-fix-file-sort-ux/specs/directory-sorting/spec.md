# Directory Sorting

## MODIFIED Requirements

### Requirement: Sort-by-name uses natural numeric ordering

When sorting files by name, embedded numbers MUST be compared numerically rather than lexicographically.

#### Scenario: Files with numeric suffixes sort naturally

Given files named `file1`, `file2`, `file10`, `file11`, `file21`
When the user sorts by name (ascending)
Then the order is `file1`, `file2`, `file10`, `file11`, `file21`

#### Scenario: Files without numbers are unaffected

Given files named `abc`, `def`, `ghi`
When the user sorts by name
Then standard alphabetical ordering is preserved

### Requirement: Sort prefix keys use dedicated state

The sort prefix key `s` MUST use a dedicated pending-state variable with a 1000ms timeout, instead of reusing the general `lastKey` tracker.

#### Scenario: Sort prefix correctly handles sub-keys

Given the user is in a directory panel
When they press `s` followed by `n` within 1000ms
Then files are sorted by name (ascending)

#### Scenario: Sort prefix with Shift reverses order

Given the user is in a directory panel
When they press `s` followed by `Shift+N` within 1000ms
Then files are sorted by name (descending)

#### Scenario: Sort prefix times out

Given the user pressed `s`
And more than 1000ms elapsed
When they press `n`
Then the `n` key is handled as a normal key press (no sort change)

#### Scenario: Second `s` press triggers size sort

Given the sort prefix is pending (user pressed `s` once)
When they press `s` again within 1000ms
Then files are sorted by size
And the prefix state is cleared
