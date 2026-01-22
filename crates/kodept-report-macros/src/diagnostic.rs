use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Attribute, Data, DeriveInput, Error, Expr, ExprLit, Field, Fields, Ident, Lit, LitInt, LitStr,
    Meta, Token, parse::Parse, punctuated::Punctuated, spanned::Spanned,
};

#[derive(Debug)]
enum FieldVariant {
    PrimaryLabel(Option<FormatArgs>),
    SecondaryLabel(Option<FormatArgs>),
    None,
}

#[derive(Debug)]
struct FormatArgs {
    format: String,
    args: Vec<Expr>,
}

#[derive(Debug)]
struct DiagnosticField {
    name: Ident,
    variant: FieldVariant,
}

#[derive(Debug, Default)]
enum DiagnosticSeverity {
    Note,
    #[default]
    Warning,
    Error,
    Bug,
}

#[derive(Debug, Default)]
struct DiagnosticConfig {
    severity: DiagnosticSeverity,
    error_code: Option<u32>,
    fail_fast_reason: Option<FormatArgs>,
    message_format: Option<FormatArgs>,
    notes: Vec<FormatArgs>,
    fields: Vec<DiagnosticField>,
}

pub fn derive_diagnostic(input: DeriveInput) -> Result<TokenStream, Error> {
    let config = parse_diagnostic_config(&input)?;
    let struct_name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let error_code = config.error_code;
    let severity = match config.severity {
        DiagnosticSeverity::Note => quote! { kodept_report::message::Severity::Note },
        DiagnosticSeverity::Warning => quote! { kodept_report::message::Severity::Warning },
        DiagnosticSeverity::Error => quote! { kodept_report::message::Severity::Error },
        DiagnosticSeverity::Bug => quote! { kodept_report::message::Severity::Bug },
    };
    let message = if let Some(message_args) = config.message_format {
        let args = quote_format_args(&message_args);
        quote! { diagnostic = diagnostic.with_message(#args); }
    } else {
        quote! {}
    };
    let notes = config.notes.into_iter().map(|args| {
        let args = quote_format_args(&args);
        quote! { diagnostic = diagnostic.with_note(#args); }
    });

    let field_decorations = config.fields.into_iter().map(|field| {
        let field_name = &field.name;
        match field.variant {
            FieldVariant::PrimaryLabel(value) => {
                let string = value.as_ref().map(quote_format_args).unwrap_or(quote! { "" });
                quote! {
                    diagnostic = diagnostic.with_label(kodept_report::message::Label::primary(#string, self.#field_name));
                }
            },
            FieldVariant::SecondaryLabel(value) => {
                let string = value.as_ref().map(quote_format_args).unwrap_or(quote! { "" });
                quote! {
                    diagnostic = diagnostic.with_label(kodept_report::message::Label::secondary(#string, self.#field_name));
                }
            },
            FieldVariant::None => quote! {  }
        }
    });
    let code_fn_impl = if let Some(code) = error_code {
        quote! { #[inline] fn code(&self) -> u32 { #code } }
    } else {
        quote! {}
    };
    let behaviour_fn_impl = if let Some(reason) = config.fail_fast_reason {
        let string = quote_format_args(&reason);
        quote! {
            #[inline]
            fn behaviour(&self) -> kodept_report::prelude::MessageBehaviour {
                kodept_report::prelude::MessageBehaviour::fail_fast(#string)
            }
        }
    } else {
        quote! {}
    };

    let expanded = quote! {
        impl #impl_generics kodept_report::traits::IntoSpannedReportMessage for #struct_name #ty_generics #where_clause {
            type Message = kodept_report::message::Diagnostic;

            #code_fn_impl
            #behaviour_fn_impl

            fn into_message(self) -> Self::Message {
                let mut diagnostic = kodept_report::message::Diagnostic::new(
                    #severity
                );

                #(#field_decorations)*
                #message
                #(#notes)*

                diagnostic
            }
        }
    };

    Ok(expanded)
}

fn parse_diagnostic_config(input: &DeriveInput) -> Result<DiagnosticConfig, Error> {
    let mut config = DiagnosticConfig::default();
    let mut severity_set = false;

    for attr in &input.attrs {
        if attr.path().is_ident("code") {
            config.error_code = Some(parse_error_code_attr(attr)?);
        } else if attr.path().is_ident("severity") {
            config.severity = attr.parse_args()?;
            severity_set = true;
        } else if attr.path().is_ident("fail_fast") {
            config.fail_fast_reason = Some(attr.parse_args()?);
        } else if attr.path().is_ident("message") {
            config.message_format = Some(attr.parse_args()?);
        } else if attr.path().is_ident("note") {
            config.notes.push(attr.parse_args()?);
        } else {
            return Err(Error::new(attr.span(), "Unknown attribute"));
        };
    }
    if !severity_set {
        return Err(Error::new_spanned(input, "Missing `severity` attribute"));
    }

    match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => {
                for field in &fields.named {
                    config.fields.push(parse_diagnostic_field(field)?);
                }
            }
            Fields::Unnamed(fields) => {
                return Err(Error::new(
                    fields.span(),
                    "Diagnostic derive macro only supports named fields",
                ));
            }
            Fields::Unit => {}
        },
        Data::Enum(_) => {
            return Err(Error::new(
                input.span(),
                "Diagnostic derive macro only supports structs",
            ));
        }
        Data::Union(_) => {
            return Err(Error::new(
                input.span(),
                "Diagnostic derive macro only supports structs",
            ));
        }
    }

    Ok(config)
}

impl Parse for DiagnosticSeverity {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lit: LitStr = input.parse()?;
        let mut text = lit.value();
        text.make_ascii_lowercase();
        match text.as_str() {
            "bug" => Ok(DiagnosticSeverity::Bug),
            "error" => Ok(DiagnosticSeverity::Error),
            "warning" => Ok(DiagnosticSeverity::Warning),
            "note" => Ok(DiagnosticSeverity::Note),
            _ => Err(Error::new(
                input.span(),
                "Expected one of [bug, error, warning, note]",
            )),
        }
    }
}

fn parse_error_code_attr(attr: &Attribute) -> Result<u32, Error> {
    attr.parse_args_with(|input: syn::parse::ParseStream| {
        let lit: LitInt = input.parse()?;
        lit.base10_parse()
    })
}

fn parse_diagnostic_field(field: &Field) -> Result<DiagnosticField, Error> {
    let name = field
        .ident
        .as_ref()
        .ok_or_else(|| Error::new(field.span(), "Field must have a name"))?;

    let variant = field
        .attrs
        .iter()
        .try_fold(FieldVariant::None, |mut acc, next| {
            if next.path().is_ident("primary_label") {
                acc = FieldVariant::PrimaryLabel(parse_label_attr(next)?);
            } else if next.path().is_ident("secondary_label") {
                acc = FieldVariant::SecondaryLabel(parse_label_attr(next)?);
            } else if next.path().is_ident("note") {
                return Err(Error::new_spanned(
                    next,
                    "`note` should not appear as field attribute",
                ));
            } else if next.path().is_ident("message") {
                return Err(Error::new_spanned(
                    next,
                    "`message` should not appear as field attribute",
                ));
            } else if next.path().is_ident("code") {
                return Err(Error::new_spanned(
                    next,
                    "`code` should not appear as field attribute",
                ));
            } else if next.path().is_ident("severity") {
                return Err(Error::new_spanned(
                    next,
                    "`severity` should not appear as field attribute",
                ));
            }
            Ok(acc)
        })?;

    Ok(DiagnosticField {
        name: name.clone(),
        variant,
    })
}

impl Parse for FormatArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let args: Punctuated<Expr, Token![,]> = Punctuated::parse_separated_nonempty(input)?;
        let mut iter = args.into_iter();
        let Some(Expr::Lit(ExprLit {
            lit: Lit::Str(format_string),
            ..
        })) = iter.next()
        else {
            return Err(input.error("Expected format string literal"));
        };

        Ok(FormatArgs {
            format: format_string.value(),
            args: iter.collect(),
        })
    }
}

fn parse_label_attr(attr: &Attribute) -> Result<Option<FormatArgs>, Error> {
    if let Meta::Path(_) = attr.meta {
        Ok(None)
    } else {
        attr.parse_args().map(Some)
    }
}

fn quote_format_args(FormatArgs { format, args }: &FormatArgs) -> TokenStream {
    if args.is_empty() {
        quote! { #format }
    } else {
        quote! { format!(#format, #(#args, )*) }
    }
}
