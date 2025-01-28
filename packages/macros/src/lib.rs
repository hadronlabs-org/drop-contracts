use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, DataEnum, DeriveInput, Variant};

/// Adds the necessary fields to an enum such that the enum implements the
/// query interface needed to be bond provider.
///
/// For example:
///
/// ```
/// use drop_macros::bond_provider_query;
/// use cosmwasm_schema::{cw_serde, QueryResponses};
/// use cosmwasm_std::{Coin, Decimal, Uint128};
///
///
/// #[bond_provider_query]
/// #[cw_serde]
/// #[derive(QueryResponses)]
/// enum QueryMsg {}
/// ```
///
/// Will transform the enum to:
///
/// ```
///use cosmwasm_std::{Coin, Decimal};
///
/// enum QueryMsg {
///     /// Returns flag wether this bond provider can be used with this denom.
///     CanBond { denom: String },
///     /// Returns flag wethers this bond provider can be used during idle state of the core.
///     CanProcessOnIdle {},
///     /// Returns amount of drop tokens to be minted with provided coin
///     TokensAmount { coin: Coin, exchange_rate: Decimal },
///     /// Returns amount of locked but not processed tokens in the async bonding provider like LSM shares
///     AsyncTokensAmount {},
/// }
/// ```
///
/// Note that other derive macro invocations must occur after this
/// procedural macro as they may depend on the new fields. For
/// example, the following will fail becase the `Clone` derivation
/// occurs before the addition of the field.
///
/// ```compile_fail
/// use drop_macros::bond_provider_query;
/// use cosmwasm_schema::{cw_serde, QueryResponses};
/// use cosmwasm_std::Empty;
/// use cosmwasm_std::{Coin, Decimal, Uint128};
///
///
/// #[derive(Clone)]
/// #[bond_provider_query]
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
pub fn bond_provider_query(metadata: TokenStream, input: TokenStream) -> TokenStream {
    // Make sure that no arguments were passed in.
    let args = parse_macro_input!(metadata as AttributeArgs);
    if let Some(first_arg) = args.first() {
        return syn::Error::new_spanned(first_arg, "bonding provider cmd macro takes no arguments")
            .to_compile_error()
            .into();
    }

    let mut ast: DeriveInput = parse_macro_input!(input);
    match &mut ast.data {
        syn::Data::Enum(DataEnum { variants, .. }) => {
            let can_bond: Variant = syn::parse2(quote! {
                #[returns(bool)]
                CanBond { denom: String }
            })
            .unwrap();

            let can_process_on_idle: Variant = syn::parse2(quote! {
                #[returns(bool)]
                CanProcessOnIdle {}
            })
            .unwrap();

            let tokens_amount: Variant = syn::parse2(quote! {
                #[returns(cosmwasm_std::Decimal)]
                TokensAmount { coin: cosmwasm_std::Coin, exchange_rate: cosmwasm_std::Decimal }
            })
            .unwrap();

            let async_tokens_amount: Variant = syn::parse2(quote! {
                #[returns(cosmwasm_std::Uint128)]
                AsyncTokensAmount {}
            })
            .unwrap();

            let can_be_removed: Variant = syn::parse2(quote! {
                #[returns(bool)]
                CanBeRemoved {}
            })
            .unwrap();

            variants.push(can_bond);
            variants.push(can_process_on_idle);
            variants.push(tokens_amount);
            variants.push(async_tokens_amount);
            variants.push(can_be_removed);
        }
        _ => {
            return syn::Error::new(
                ast.ident.span(),
                "bonding provider cmd types can only be derived for enums",
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
/// interface needed to be bond provider.
///
/// For example:
///
/// ```
/// use drop_macros::bond_provider;
///
/// #[bond_provider]
/// enum ExecuteMsg {}
/// ```
///
/// Will transform the enum to:
///
/// ```
///
/// enum ExecuteMsg {
///     Bond { },
///     ProcessOnIdle {},
/// }
/// ```
///
/// Note that other derive macro invocations must occur after this
/// procedural macro as they may depend on the new fields. For
/// example, the following will fail becase the `Clone` derivation
/// occurs before the addition of the field.
///
/// ```compile_fail
/// use drop_macros::bond_provider;
///
/// #[derive(Clone)]
/// #[bond_provider]
/// #[allow(dead_code)]
/// enum Test {
///     Foo,
///     Bar(u64),
///     Baz { foo: u64 },
/// }
/// ```
#[proc_macro_attribute]
pub fn bond_provider(metadata: TokenStream, input: TokenStream) -> TokenStream {
    // Make sure that no arguments were passed in.
    let args = parse_macro_input!(metadata as AttributeArgs);
    if let Some(first_arg) = args.first() {
        return syn::Error::new_spanned(first_arg, "bonding provider cmd macro takes no arguments")
            .to_compile_error()
            .into();
    }

    let mut ast: DeriveInput = parse_macro_input!(input);
    match &mut ast.data {
        syn::Data::Enum(DataEnum { variants, .. }) => {
            let bond: Variant = syn::parse2(quote! { Bond { } }).unwrap();
            let process_on_idle: Variant = syn::parse2(quote! {  ProcessOnIdle {} }).unwrap();

            variants.push(bond);
            variants.push(process_on_idle);
        }
        _ => {
            return syn::Error::new(
                ast.ident.span(),
                "bonding provider cmd types can only be derived for enums",
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
