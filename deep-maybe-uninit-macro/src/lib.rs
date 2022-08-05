use ::proc_macro2::{Ident, TokenStream, TokenTree};
use ::quote::{format_ident, quote};
use ::syn::{Data, DeriveInput, Fields};
use syn::AttrStyle;

fn get_ident_from_stream(tokens: TokenStream) -> Option<Ident> {
    match tokens.into_iter().next() {
        Some(TokenTree::Group(group)) => get_ident_from_stream(group.stream()),
        Some(TokenTree::Ident(ident)) => Some(ident),
        _ => None,
    }
}

#[proc_macro_derive(DeepMaybeUninit)]
pub fn deep_maybe_uninit_macro_derive(
    input: ::proc_macro::TokenStream,
) -> ::proc_macro::TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();

    let mut found_good_repr = false;
    for attr in ast.attrs {
        if let (AttrStyle::Outer, Some(outer_ident), Some(inner_ident)) = (
            &attr.style,
            attr.path.get_ident(),
            get_ident_from_stream(attr.tokens),
        ) {
            if outer_ident.to_string() == "repr"
                && (inner_ident == "C" || inner_ident == "transparent")
            {
                found_good_repr = true;
            }
        }
    }

    if !found_good_repr {
        panic!("`[repr(C)]` or `[repr(transparent)]` is needed for `derive(DeepMaybeUninit)`.");
    }

    let visibility = ast.vis;
    let name = &ast.ident;
    let deep_maybe_uninit_name = format_ident!("DeepMaybeUninit{name}");
    let (full_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let fields = if let Data::Struct(data) = ast.data {
        data.fields
    } else {
        panic!("Only structs are supported for `derive(DeepMaybeUninit)`.")
    };

    let mut struct_decl = match fields {
        Fields::Named(data) => {
            let named_fields = data.named;
            let field_idents = named_fields.iter().map(|p| &p.ident);
            let field_types = named_fields.iter().map(|p| &p.ty);
            quote! {
                #[repr(C)]
                #visibility struct #deep_maybe_uninit_name #full_generics {#(
                    #field_idents: <#field_types as ::deep_maybe_uninit::HasDeepMaybeUninit>::AsDeepMaybeUninit,
                )*}
            }
        }
        Fields::Unnamed(data) => {
            let field_types = data.unnamed.iter().map(|p| &p.ty);
            quote! {
                #[repr(C)]
                struct #deep_maybe_uninit_name #full_generics (#(
                    <#field_types as ::deep_maybe_uninit::HasDeepMaybeUninit>::AsDeepMaybeUninit,
                )*);
            }
        }
        Fields::Unit => {
            quote! {
                #[repr(C)]
                struct #deep_maybe_uninit_name #full_generics;
            }
        }
    };

    let impls = quote! {
        unsafe impl #full_generics ::deep_maybe_uninit::HasDeepMaybeUninit for #name #ty_generics #where_clause {
            type AsDeepMaybeUninit = #deep_maybe_uninit_name #ty_generics;
        }

        unsafe impl #full_generics ::deep_maybe_uninit::IsDeepMaybeUninit for #deep_maybe_uninit_name #ty_generics #where_clause {
            type AsDeepInit = #name #ty_generics;
        }
    };

    struct_decl.extend(impls);
    struct_decl.into()
}
