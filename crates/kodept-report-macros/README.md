# Kodept Diagnostic Macros

This crate provides procedural macros to reduce boilerplate when creating diagnostic error types in the Kodept compiler.

## Features

- **`#[derive(IntoMessage)]`**: Automatically implement `IntoMessage` for types
- **Field attributes**: Mark fields as primary/secondary labels or notes
- **Custom message formatting**: Generate error messages from field values
- **Error codes**: Assign numeric error codes to diagnostics
- **Fail-fast behavior**: Configure diagnostics to cause immediate compilation failure

## Usage

### Basic Diagnostic

```rust
use kodept_diagnostic_macros::Diagnostic;
use kodept_core::code_point::Span;

#[derive(IntoMessage)]
#[severity("Error")]
struct DuplicatedSymbolError {
    #[primary_label("symbol already defined")]
    current_def: Span,
    #[secondary_label("previous declaration")]
    previous_def: Span,
    #[note]
    bound_name: String,
}
```

### Field Attributes

- `#[primary_label("message")]`: Creates a primary label with the given message
- `#[secondary_label("message")]`: Creates a secondary label with the given message
- `#[note]`: Adds the field value as a note to the diagnostic

### Struct-Level Attributes

- `#[severity("Error"|"Warning"|"Note"|"Bug")]`: Sets the diagnostic severity (required)
- `#[code(1234)]`: Assigns a numeric error code
- `#[message("custom message with {field}")]`: Custom message formatting
- `#[fail_fast("reason")]`: Makes the diagnostic cause immediate compilation failure

### Complex Example

```rust
#[derive(IntoMessage)]
#[severity("Error")]
#[code(1001)]
#[message("Type mismatch: expected {expected_type}, found {found_type}", self.expected_type, self.found_type)]
struct TypeMismatchError {
    #[primary_label("expected type")]
    expected_span: Span,
    #[secondary_label("found type")]
    found_span: Span,
    expected_type: String,
    found_type: String,
    context: String, // Won't be included in the main message
}
```

### Message Formatting

You can use field values in custom messages:

```rust
#[derive(IntoMessage)]
#[severity("Error")]
#[message("Invalid syntax: {message}", self.message)]
struct InvalidSyntaxError {
    #[primary_label("syntax error")]
    error_span: Span,
    message: String,
    #[note]
    suggestion: String,
}
```

## Generated Code

The `#[derive(IntoMessage)]` macro generates:

1. Implementation of `IntoMessage` trait
2. Diagnostic construction with labels and notes based on field attributes
3. Message generation from non-annotated fields or custom message format
4. Error code and severity handling
5. Optional fail-fast behavior

## Integration

These macros integrate seamlessly with the existing Kodept reporting system:

```rust
use kodept_report::{FileId, report::Report};

let file_id: FileId = todo!();

let error = DuplicatedSymbolError {
current_def: span1,
previous_def: span2,
bound_name: "my_function".to_string(),
};

// Can be used directly with the reporting system
let report = Report::from_message(file_id, error);
```

## Requirements

- All diagnostic structs must have named fields (no tuple structs)
- The `#[severity]` attribute is required

## Benefits

- **Reduced boilerplate**: No need to manually implement `IntoMessage`
- **Type safety**: Compile-time checking of diagnostic structure
- **Consistency**: Standardized error reporting across the codebase
- **Flexibility**: Custom message formatting and field handling
- **Integration**: Works with the existing Kodept reporting infrastructure
