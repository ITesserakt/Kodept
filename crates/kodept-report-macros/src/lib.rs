//! Procedural macros for creating diagnostics in Kodept.
//!
//! This crate provides macros to reduce boilerplate when creating error types
//! and diagnostic messages.

use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

mod diagnostic;

/// Derive macro for creating diagnostic types.
///
/// This macro automatically implements `IntoSpannedReportMessage` for your error type.
///
/// # Examples
///
/// ```rust
/// use kodept_diagnostic_macros::Report;
/// use kodept_core::code_point::Span;
///
/// #[derive(Report)]
/// struct DuplicatedSymbolError {
///     #[primary_label("symbol already defined")]
///     current_def: Span,
///     #[secondary_label("previous declaration")]
///     previous_def: Span,
///     bound_name: String,
/// }
/// ```
#[proc_macro_derive(
    Report,
    attributes(
        primary_label,
        secondary_label,
        note,
        severity,
        message,
        code,
        fail_fast
    )
)]
pub fn derive_diagnostic(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    diagnostic::derive_diagnostic(input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    // Test that the macro generates valid code
    #[test]
    fn test_diagnostic_derive() {
        let input = quote! {
            #[derive(Report)]
            #[severity("error")]
            #[fail_fast("Test")]
            #[message("Hello, {}", self.who)]
            struct TestError {
                #[primary_label("test error")]
                span: Span,
                who: String,
                #[secondary_label]
                span2: Span,
            }
        };

        let parsed = syn::parse2::<DeriveInput>(input).unwrap();
        let result = diagnostic::derive_diagnostic(parsed);
        assert!(result.is_ok(), "Should parse, but {result:?}");
    }
}
