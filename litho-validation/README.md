# Litho Validation

Fully compliant, pure Rust implementation of the GraphQL spec, including
validation with graphical diagnostics, schema merging and backwards
compatibility checking.

## Features

### Diagnostics

#### Graphical Diagnostics

#### JSON Diagnostics

### Schema Merging

### Schema Validation

#### Rules

#### Backwards Compatibility

### Query Validation

#### Rules

- [x] 5.1.1 Executable Definitions (no tests)
- [x] 5.2.1.1 Operation Name Uniqueness (no tests)
- [x] 5.2.2.1 Lone Anonymous Operation (no tests)
- [x] 5.2.3.1 Single Root Field
- [x] 5.3.1 Field Selections (no tests)
- [ ] 5.3.2 Field Selection Merging
- [x] 5.3.3 Leaf Field Selections (no tests)
- [x] 5.4.1 Argument Names
- [x] 5.4.2 Argument Uniqueness (no tests)
- [x] 5.4.2.1 Required Arguments (no tests)
- [x] 5.5.1.1 Fragment Name Uniqueness (no tests)
- [x] 5.5.1.2 Fragment Spread Type Existence (no tests)
- [x] 5.5.1.3 Fragments On Composite Types (no tests)
- [x] 5.5.1.4 Fragments Must Be Used (no tests)
- [x] 5.5.2.1 Fragment Spread Target Defined (no tests)
- [x] 5.5.2.2 Fragment Spreads Must Not Form Cycles (no tests)
- [x] 5.5.2.3 Fragment Spread Is Possible
- [x] 5.6.1 Values of Correct Type
- [x] 5.6.2 Input Object Field Names
- [x] 5.6.3 Input Object Field Uniqueness
- [ ] 5.6.4 Input Object Required Fields
- [x] 5.7.1 Directives Are Defined
- [x] 5.7.2 Directives Are In Valid Locations
- [ ] 5.7.3 Directives Are Unique Per Location
- [ ] 5.8.1 Variable Uniqueness
- [ ] 5.8.2 Variables Are Input Types
- [ ] 5.8.3 All Variables Uses Defined
- [ ] 5.8.4 All Variables Used
- [ ] 5.8.5 All Variables Usages are Allowed
