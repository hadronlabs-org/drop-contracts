use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, DataEnum, DeriveInput, Variant};

/// Adds the necessary fields to an enum such that the enum implements the
/// query interface needed to be paused/unpaused.
///
/// For example:
///
/// ```
/// use drop_macros::pausable_query;
/// use cosmwasm_schema::{cw_serde, QueryResponses};
///
/// #[cw_serde]
/// struct PauseInfoResponse{}
///
/// #[pausable_query]
/// #[cw_serde]
/// #[derive(QueryResponses)]
/// enum QueryMsg {}
/// ```
///
/// Will transform the enum to:
///
/// ```
/// enum QueryMsg {
///     /// Returns information about if the contract is currently paused.
///     PauseInfo {},
/// }
/// ```
///
/// Note that other derive macro invocations must occur after this
/// procedural macro as they may depend on the new fields. For
/// example, the following will fail becase the `Clone` derivation
/// occurs before the addition of the field.
///
/// ```compile_fail
/// use drop_macros::pausable_query;
/// use cosmwasm_schema::{cw_serde, QueryResponses};
/// use cosmwasm_std::Empty;
///
/// struct PauseInfoResponse{}
///
/// #[derive(Clone)]
/// #[pausable_query]
/// #[cw_serde]
/// #[derive(QueryResponses)]
/// #[allow(dead_code)]
/// enum Test {
///     #[returns(Empty)]
///     Foo,
///     #[returns(Empty)]
///     Bar(u64),
///     #[returns(Empty)]
///     Baz { foo: u64 },
/// }
/// ```
#[proc_macro_attribute]
pub fn pausable_query(metadata: TokenStream, input: TokenStream) -> TokenStream {
    // Make sure that no arguments were passed in.
    let args = parse_macro_input!(metadata as AttributeArgs);
    if let Some(first_arg) = args.first() {
        return syn::Error::new_spanned(first_arg, "pausing cmd macro takes no arguments")
            .to_compile_error()
            .into();
    }

    let mut ast: DeriveInput = parse_macro_input!(input);
    match &mut ast.data {
        syn::Data::Enum(DataEnum { variants, .. }) => {
            let pause_info: Variant = syn::parse2(quote! {
                #[returns(PauseInfoResponse)]
                PauseInfo {}
            })
            .unwrap();

            variants.push(pause_info);
        }
        _ => {
            return syn::Error::new(
                ast.ident.span(),
                "pausing cmd types can only be derived for enums",
            )
            .to_compile_error()
            .into()
        }
    };

    quote! {
    #ast
    }
    .into()
}

/// Adds the necessary fields to an enum such that the enum implements the
/// interface needed to be paused/unpaused.
///
/// For example:
///
/// ```
/// use drop_macros::pausable;
///
/// #[pausable]
/// enum ExecuteMsg {}
/// ```
///
/// Will transform the enum to:
///
/// ```
/// enum ExecuteMsg {
///     Pause {},
///     Unpause {},
/// }
/// ```
///
/// Note that other derive macro invocations must occur after this
/// procedural macro as they may depend on the new fields. For
/// example, the following will fail becase the `Clone` derivation
/// occurs before the addition of the field.
///
/// ```compile_fail
/// use drop_macros::pausable;
///
/// #[derive(Clone)]
/// #[pausable]
/// #[allow(dead_code)]
/// enum Test {
///     Foo,
///     Bar(u64),
///     Baz { foo: u64 },
/// }
/// ```
#[proc_macro_attribute]
pub fn pausable(metadata: TokenStream, input: TokenStream) -> TokenStream {
    // Make sure that no arguments were passed in.
    let args = parse_macro_input!(metadata as AttributeArgs);
    if let Some(first_arg) = args.first() {
        return syn::Error::new_spanned(first_arg, "pausing cmd macro takes no arguments")
            .to_compile_error()
            .into();
    }

    let mut ast: DeriveInput = parse_macro_input!(input);
    match &mut ast.data {
        syn::Data::Enum(DataEnum { variants, .. }) => {
            let pause: Variant = syn::parse2(quote! { Pause {} }).unwrap();
            let unpause: Variant = syn::parse2(quote! { Unpause {} }).unwrap();

            variants.push(pause);
            variants.push(unpause);
        }
        _ => {
            return syn::Error::new(
                ast.ident.span(),
                "pausing cmd types can only be derived for enums",
            )
            .to_compile_error()
            .into()
        }
    };

    quote! {
    #ast
    }
    .into()
}
