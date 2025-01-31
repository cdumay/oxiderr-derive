//! [![License: BSD-3-Clause](https://img.shields.io/badge/license-BSD--3--Clause-blue)](./LICENSE)
//! [![oxiderr-derive on crates.io](https://img.shields.io/crates/v/oxiderr-derive)](https://crates.io/crates/oxiderr-derive)
//! [![oxiderr-derive on docs.rs](https://docs.rs/oxiderr-derive/badge.svg)](https://docs.rs/oxiderr-derive)
//! [![Source Code Repository](https://img.shields.io/badge/Code-On%20GitHub-blue?logo=GitHub)](https://github.com/cdumay/oxiderr-derive)
//!
//! The `oxiderr-derive` crate provides procedural macros to simplify the creation of custom error types in Rust. By leveraging these macros,
//! developers can efficiently define error structures that integrate seamlessly with the `oxiderr` error management ecosystem.
//!
//! # Overview
//!
//! Error handling in Rust often involves creating complex structs to represent various error kinds and implementing traits to provide context and
//! conversions. The `oxiderr-derive` crate automates this process by offering macros that generate the necessary boilerplate code, allowing for
//! more readable and maintainable error definitions.
//!
//! # Features
//!
//! * **Macros**: Automatically generate implementations for custom error types.
//! * **Integration with oxiderr**: Designed to work cohesively with the `oxiderr` crate, ensuring consistent error handling patterns.
//!
//! # Usage
//!
//! To utilize `oxiderr-derive` in your project, follow these steps:
//!
//! 1. **Add Dependencies**: Include `oxiderr` with the feature `derive` in your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! oxiderr = { version = "0.1", features = ["derive"] }
//! ```
//!
//! 2. **Define Error**: Use the provided derive macros to define your error and error kind structs:
//!
//! ```rust
//! use oxiderr::{define_errors, define_kinds, AsError};
//!
//! define_kinds! {
//!     UnknownError = ("Err-00001", 500, "Unexpected error"),
//!     IoError = ("Err-00001", 400, "IO error")
//! }
//! define_errors! {
//!     Unexpected = UnknownError,
//!     FileRead = IoError,
//!     FileNotExists = IoError
//! }
//! ```
//! In this example:
//!
//! * define_kinds create `oxiderr::ErrorKind` structs representing different categories of errors.
//! * define_errors create `oxiderr::Error` struct that contains an ErrorKind and metadata.
//!
//! 3. **Implementing Error Handling**: With the above definitions, you can now handle errors in your application as follows:
//!
//! ```rust
//! use std::fs::File;
//! use std::io::Read;
//!
//! fn try_open_file(path: &str) -> oxiderr::Result<String> {
//!     let mut file = File::open(path).map_err(|err| FileNotExists::new().set_message(err.to_string()))?;
//!     let mut content = String::new();
//!     file.read_to_string(&mut content).map_err(|err| FileRead::new().set_message(err.to_string()))?;
//!     Ok(content)
//! }
//!
//! fn main() {
//!     let path = "example.txt";
//!
//!     match try_open_file(path) {
//!         Ok(content) => println!("File content:\n{}", content),
//!         Err(e) => eprintln!("{}", e),
//!     }
//! }
//! ```
//! This will output:
//!
//! ```text
//! [Err-00001] Client::IoError::FileNotExists (400) - No such file or directory (os error 2)
//! ```
extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{parenthesized, parse_macro_input, Ident, LitInt, LitStr, Token, Type};

struct ErrorKindArgs {
    const_name: Ident,
    _eq: Token![=],
    _parens: syn::token::Paren,
    message: LitStr,
    _comma1: Token![,],
    code: LitInt,
    _comma2: Token![,],
    description: LitStr,
}

impl Parse for ErrorKindArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let const_name: Ident = input.parse()?;
        let _eq: Token![=] = input.parse()?;

        let content;
        let _parens = parenthesized!(content in input);

        let message: LitStr = content.parse()?;
        let _comma1: Token![,] = content.parse()?;
        let code: LitInt = content.parse()?;
        let _comma2: Token![,] = content.parse()?;
        let description: LitStr = content.parse()?;

        Ok(ErrorKindArgs {
            const_name,
            _eq,
            _parens,
            message,
            _comma1,
            code,
            _comma2,
            description,
        })
    }
}

struct ErrorKindArgsList {
    items: Punctuated<ErrorKindArgs, Comma>,
}

impl Parse for ErrorKindArgsList {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(ErrorKindArgsList {
            items: Punctuated::parse_terminated(input)?,
        })
    }
}

/// The `define_kinds` macro is a procedural macro that generates constants of type `oxiderr::ErrorKind`. This macro simplifies the definition
/// of structured error kinds by allowing developers to declare them using a concise syntax. It takes a list of error definitions and expands
/// them into properly structured `oxiderr::ErrorKind` constants.
///
/// # Usage Example
///
/// ## Macro Input
///
/// ```rust
/// define_kinds! {
///     FileNotFound = ("File not found", 404, "The requested file could not be located"),
///     PermissionDenied = ("Permission denied", 403, "The user lacks the necessary permissions")
/// }
/// ```
/// ## Macro Expansion (Generated Code)
///
/// ```rust
/// #[allow(non_upper_case_globals)]
/// pub const FileNotFound: oxiderr::ErrorKind = oxiderr::ErrorKind(
///     "FileNotFound",
///     "File not found",
///     404,
///     "The requested file could not be located"
/// );
///
/// #[allow(non_upper_case_globals)]
/// pub const PermissionDenied: oxiderr::ErrorKind = oxiderr::ErrorKind(
///     "PermissionDenied",
///     "Permission denied",
///     403,
///     "The user lacks the necessary permissions"
/// );
/// ```
#[proc_macro]
pub fn define_kinds(input: TokenStream) -> TokenStream {
    let args_list = parse_macro_input!(input as ErrorKindArgsList);

    let constants = args_list.items.iter().map(|args| {
        let const_name = &args.const_name;
        let message = &args.message;
        let code = &args.code;
        let description = &args.description;

        quote! {
            #[allow(non_upper_case_globals)]
            pub const #const_name: oxiderr::ErrorKind = oxiderr::ErrorKind(stringify!(#const_name), #message, #code, #description);
        }
    });

    TokenStream::from(quote! {
        #(#constants)*
    })
}

struct ErrorDefinition {
    name: Ident,
    kind: Type,
}

struct ErrorDefinitions {
    definitions: Vec<ErrorDefinition>,
}

impl Parse for ErrorDefinitions {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut definitions = Vec::new();

        while !input.is_empty() {
            let name: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            let kind: Type = input.parse()?;
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
            definitions.push(ErrorDefinition { name, kind });
        }

        Ok(ErrorDefinitions { definitions })
    }
}

/// The `define_errors` macro is a procedural macro that generates structured error types implementing `oxiderr::AsError`. This macro simplifies
/// error handling by defining error structures with relevant metadata, serialization, and error conversion logic.
///
/// Each generated struct:
///
/// * Implements `oxiderr::AsError` for interoperability with `oxiderr::ErrorKind`.
/// * Provides methods for setting error messages and details.
/// * Supports conversion from `oxiderr::Error`.

/// # Usage Example
///
/// ## Macro Input
///
/// ```rust
/// define_errors! {
///     NotFoundError = FileNotFound,
///     UnauthorizedError = PermissionDenied
/// }
/// ```
/// ## Macro Expansion (Generated Code for NotFoundError)
///
/// ```rust
/// #[derive(Debug, Clone)]
/// pub struct NotFoundError {
///     class: String,
///     message: String,
///     details: Option<std::collections::BTreeMap<String, serde_value::Value>>,
/// }
///
/// impl NotFoundError {
///     pub const kind: oxiderr::ErrorKind = FileNotFound;
///
///     pub fn new() -> Self {
///         Self {
///             class: format!("{}::{}::{}", Self::kind.side(), Self::kind.name(), "NotFoundError"),
///             message: Self::kind.description().into(),
///             details: None,
///         }
///     }
///
///     pub fn set_message(mut self, message: String) -> Self {
///         self.message = message;
///         self
///     }
///
///     pub fn set_details(mut self, details: std::collections::BTreeMap<String, serde_value::Value>) -> Self {
///         self.details = Some(details);
///         self
///     }
///
///     pub fn convert(error: oxiderr::Error) -> Self {
///         let mut err_clone = error.clone();
///         let mut details = error.details.unwrap_or_default();
///         err_clone.details = None;
///         details.insert("origin".to_string(), serde_value::to_value(err_clone).unwrap());
///
///         Self {
///             class: format!("{}::{}::{}", Self::kind.side(), Self::kind.name(), "NotFoundError"),
///             message: Self::kind.description().into(),
///             details: Some(details),
///         }
///     }
/// }
///
/// impl oxiderr::AsError for NotFoundError {
///     fn kind() -> oxiderr::ErrorKind {
///         Self::kind
///     }
///     fn class(&self) -> String {
///         self.class.clone()
///     }
///     fn message(&self) -> String {
///         self.message.clone()
///     }
///     fn details(&self) -> Option<std::collections::BTreeMap<String, serde_value::Value>> {
///         self.details.clone()
///     }
/// }
///
/// impl std::error::Error for NotFoundError {}
///
/// impl std::fmt::Display for NotFoundError {
///     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
///         write!(f, "[{}] {} ({}): {}", Self::kind.message_id(), "NotFoundError", Self::kind.code(), self.message())
///     }
/// }
/// ```
#[proc_macro]
pub fn define_errors(input: TokenStream) -> TokenStream {
    let definitions = parse_macro_input!(input as ErrorDefinitions);

    let generated_structs = definitions.definitions.iter().map(|definition| {
        let name = &definition.name;
        let kind = &definition.kind;

        quote! {
            #[derive(Debug, Clone)]
            pub struct #name {
                class: String,
                message: String,
                details: Option<std::collections::BTreeMap<String, serde_value::Value>>,
            }

            impl #name {
                pub const kind: oxiderr::ErrorKind = #kind;
                pub fn new() -> Self {
                    Self {
                        class: format!("{}::{}::{}", Self::kind.side(), Self::kind.name(), stringify!(#name)),
                        message: Self::kind.description().into(),
                        details: None,
                    }
                }
                pub fn set_message(mut self, message: String) -> Self {
                    self.message = message;
                    self
                }
                pub fn set_details(mut self, details: std::collections::BTreeMap<String, serde_value::Value>) -> Self {
                    self.details = Some(details);
                    self
                }
                pub fn convert(error: oxiderr::Error) -> Self {
                    let mut err_clone = error.clone();
                    let mut details = error.details.unwrap_or_default();
                    err_clone.details = None;
                    details.insert("origin".to_string(), serde_value::to_value(err_clone).unwrap());
                    Self {
                        class: format!("{}::{}::{}", Self::kind.side(), Self::kind.name(), stringify!(#name)),
                        message: Self::kind.description().into(),
                        details: Some(details),
                    }
                }
            }
            impl oxiderr::AsError for #name {
                fn kind()-> oxiderr::ErrorKind {
                    Self::kind
                }
                fn class(&self) -> String {
                    self.class.clone()
                }
                fn message(&self) -> String {
                    self.message.clone()
                }
                fn details(&self) -> Option<std::collections::BTreeMap<String, serde_value::Value>> {
                    self.details.clone()
                }
            }

            impl std::error::Error for #name {}

            impl std::fmt::Display for #name {
                fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    write!(f, "[{}] {} ({}): {}", Self::kind.message_id(), stringify!(#name), Self::kind.code(), self.message())
                }
            }
        }
    });

    TokenStream::from(quote! {
        #(#generated_structs)*
    })
}
