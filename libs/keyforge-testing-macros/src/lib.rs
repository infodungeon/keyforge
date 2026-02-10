extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_quote, Item, ItemMod};

/// The Universal Oracle for KeyForge Test Targets.
///
/// Automatically applies Law-grace to modules or items.
#[proc_macro_attribute]
pub fn kf_test(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let common = common_grace();

    // Case A: Applied to a module
    if let Ok(mut input_mod) = syn::parse::<ItemMod>(item.clone()) {
        input_mod.attrs.push(parse_quote!(#[cfg(test)]));
        input_mod.attrs.push(common);
        return quote!(#input_mod).into();
    }

    // Case B: Applied to a function or other item
    let input_item = parse_macro_input!(item as Item);
    let expanded = quote! {
        #[cfg(test)]
        #common
        #input_item
    };
    TokenStream::from(expanded)
}

fn common_grace() -> syn::Attribute {
    parse_quote! {
        #[allow(
            dead_code,
            unused_imports,
            unused_variables,
            unused_mut,
            clippy::unwrap_used,
            clippy::expect_used,
            clippy::panic,
            clippy::cast_possible_truncation,
            clippy::cast_precision_loss,
            clippy::cast_sign_loss,
            clippy::cast_possible_wrap,
            clippy::print_stdout,
            clippy::print_stderr,
            clippy::float_cmp,
            clippy::field_reassign_with_default,
            clippy::uninlined_format_args,
            clippy::needless_range_loop,
            clippy::large_stack_arrays,
            clippy::clone_on_copy,
            clippy::used_underscore_binding,
            clippy::wildcard_imports,
            clippy::needless_pass_by_value,
            clippy::type_complexity,
            clippy::too_many_lines,
            clippy::items_after_statements,
            clippy::similar_names,
            clippy::semicolon_if_nothing_returned,
            clippy::module_inception,
            clippy::too_many_arguments,
            clippy::unused_async,
            clippy::unnecessary_wraps,
            clippy::many_single_char_names
        )]
    }
}
