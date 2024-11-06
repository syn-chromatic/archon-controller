// src/lib.rs
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DataEnum, DeriveInput};

#[proc_macro_derive(ValueConverter)]
pub fn value_converter_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let enum_name = input.ident;

    let variants = if let Data::Enum(DataEnum { variants, .. }) = &input.data {
        variants
    } else {
        panic!("ValueConverter can only be derived for enums");
    };

    let match_arms = variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        quote! {
            #enum_name::#variant_name => |_| #enum_name::#variant_name,
        }
    });

    let expanded = quote! {
        impl #enum_name {
            pub fn value_converter(&self) -> fn(ValueEnum) -> Self {
                match self {
                    #(#match_arms)*
                }
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(ToItem)]
pub fn item_generator_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let enum_name = input.ident;

    if let Data::Enum(_) = &input.data {
        let expanded = quote! {
            impl #enum_name {
                pub fn item(&self, value: ValueEnum) -> MenuItem<&str, Self, ValueEnum, true> {
                    let title_text: &str = self.as_str();
                    MenuItem::new(title_text, value).with_value_converter(self.value_converter())
                }
            }
        };

        TokenStream::from(expanded)
    } else {
        panic!("ToItem can only be derived for enums");
    }
}
