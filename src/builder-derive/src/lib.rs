use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{parse_macro_input, AttrStyle, Attribute, Data, DeriveInput, Fields, Meta};

static UNIT_STRUCT: &str = "`Builder` not supported on unit struct";
static UNNAMED_FIELDS: &str =
    "`Builder` not supported on struct or enum variant with unnamed fields";
static ENUM_ZERO_VARIANT: &str = "`Builder` not supported on enum with zero variants";
static UNION: &str = "`Builder` not supported union";

/// Coarse type categorization, sufficient for this procedural macro.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Type {
    Int,
    UInt,
    Float,
    String,
    Bool,
    NotPrimitive,
}

impl Type {
    fn is_number(&self) -> bool {
        match self {
            Self::Int | Self::UInt | Self::Float => true,
            _ => false,
        }
    }
    // fn is_primitive(&self) -> bool {
    //     match self {
    //         Self::NotPrimitive | Self::String => false,
    //         _ => true,
    //     }
    // }
    fn is_bool(&self) -> bool {
        match self {
            Self::Bool => true,
            _ => false,
        }
    }
    fn is_number_or_bool(&self) -> bool {
        self.is_number() || self.is_bool()
    }
    fn is_string(&self) -> bool {
        match self {
            Self::String => true,
            _ => false,
        }
    }
}

impl From<&Ident> for Type {
    fn from(ident: &Ident) -> Self {
        if ident == "i8" || ident == "i16" || ident == "i32" || ident == "i64" || ident == "i128" {
            Self::Int
        } else if ident == "u8"
            || ident == "u16"
            || ident == "u32"
            || ident == "u64"
            || ident == "u128"
        {
            Self::UInt
        } else if ident == "f32" || ident == "f64" {
            Self::Float
        } else if ident == "bool" {
            Self::Bool
        } else if ident == "String" || ident == "OsString" {
            Self::String
        } else {
            Self::NotPrimitive
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct FieldInfo {
    ident: Ident,
    ty: Ident,
    ty_coarse: Type,
    default: Option<syn::Lit>,
}
impl From<&syn::Field> for FieldInfo {
    fn from(f: &syn::Field) -> Self {
        let ident = f.ident.clone().unwrap();
        let ty_ident = match &f.ty {
            syn::Type::Path(path) => path.path.get_ident(),
            _ => unimplemented!("type is not `TypePath`"),
        }
        .unwrap();
        let ty = (*ty_ident).clone();
        let ty_coarse = Type::from(ty_ident);
        let default = get_default(&f.attrs[..]);
        Self {
            ident,
            ty,
            ty_coarse,
            default,
        }
    }
}

#[proc_macro_derive(Builder, attributes(defaults_to))]
pub fn derive_builder(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let imp = derive_impl(&input.data, &name);
    let expanded = quote! {
        impl Builder for #name {}
        #imp
    };
    proc_macro::TokenStream::from(expanded)
}

fn into_fns<'a>(fields: &'a [FieldInfo]) -> impl Iterator<Item = TokenStream> + 'a {
    fields.iter().map(
        |FieldInfo {
             ref ident, ref ty, ..
         }| {
            let doc = format!("Configure `{}` with the given value.", ident);
            quote! {
                #[doc = #doc]
                pub fn #ident<T: Into<#ty>>(mut self, #ident: T) -> Self {
                    self.#ident = Some(#ident.into());
                    self
                }
            }
        },
    )
}
fn build_defaults<'a>(fields: &'a [FieldInfo]) -> impl Iterator<Item = TokenStream> + 'a {
    fields.iter().map(
        |FieldInfo {
             ref ident,
             ref ty_coarse,
             ref default,
             ..
         }| {
            if ty_coarse.is_number_or_bool() {
                let Some(ref default) = default else {
                    unimplemented!("default value required for {}", ident);
                };
                // Below is a compromise of sorts. We can take a string literal
                // if it parses to a valid path (i.e. we are pointing to a language item).
                match default {
                    syn::Lit::Str(s) => {
                        match s.parse::<syn::Path>() {
                            Ok(path) => quote! {
                                let #ident = self.#ident.unwrap_or(#path);
                            },
                            Err(_) => {
                                unimplemented!("String literal for number or boolean field must be a valid path expression")
                            }
                        }
                    }
                    x => {
                        // Special case to handle environment capture
                        if ident == "num_threads" {
                            quote! {
                                let num_threads = self.num_threads.unwrap_or_else(|| {
                                    std::env::var("STAN_NUM_THREADS").map_or(#x, |s| s.parse::<i32>().unwrap_or(#x))
                                });
                            }
                        } else {
                            quote! {
                                let #ident = self.#ident.unwrap_or(#x);
                            }
                        }
                    }
                }
            } else if ty_coarse.is_string() {
                match default {
                    Some(default) => {
                        // Intended behavior: if the string literal parses to a path expr,
                        // then it was a path expression; otherwise, it an arbitrary
                        // string literal.
                        match default {
                            syn::Lit::Str(s) => {
                                match s.parse::<syn::Path>() {
                                    Ok(path) => quote! {
                                        let #ident = self.#ident.unwrap_or_else(|| #path.into());
                                    },
                                    _ => quote! {
                                        let #ident = self.#ident.unwrap_or_else(|| #default.into());
                                    },
                                }
                            }
                            _ => unimplemented!("String literal required for `String` field"),
                        }
                    }
                    _ => {
                        quote! {
                            let #ident = self.#ident.unwrap_or_else(|| "".into());
                        }
                    }
                }
            } else {
                if default.is_some() {
                    unimplemented!("default value not permissible for non-primitive type; got {:?}", default);
                } else {
                    quote! {
                        let #ident = self.#ident.unwrap_or_default();
                    }
                }
            }
        },
    )
}
fn new_impl(fields: &[FieldInfo]) -> TokenStream {
    let idents_new = fields.iter().map(|FieldInfo { ref ident, .. }| {
        quote! {
            #ident: None
        }
    });
    quote! {
        /// Return a builder with all options unspecified.
        pub fn new() -> Self {
            Self {
                #(#idents_new),*
            }
        }
    }
}

fn builder_doc<T: std::fmt::Display>(name: &T) -> String {
    format!("Options builder for [`{}`].\nFor any option left unspecified, the default value indicated on `{}` will be supplied.", name, name)
}

fn builder_fields<'a>(fields: &'a [FieldInfo]) -> impl Iterator<Item = TokenStream> + 'a {
    fields.iter().map(
        |FieldInfo {
             ref ident, ref ty, ..
         }| {
            quote! {
                #ident: Option<#ty>
            }
        },
    )
}

fn derive_struct_impl(data: &syn::DataStruct, name: &Ident) -> TokenStream {
    let fields = struct_fields(data);
    let builder_name = format_ident!("{}Builder", name);
    let decls = builder_fields(&fields);
    let into_fns = into_fns(&fields);
    let default_stmts = build_defaults(&fields);
    let idents = fields.iter().map(|FieldInfo { ref ident, .. }| ident);
    let builder_doc = builder_doc(name);
    let build_doc = format!("Build the `{}` instance.", name);
    let new_imp = new_impl(&fields);
    quote! {
        #[derive(Debug, Clone, PartialEq)]
        #[doc = #builder_doc]
        pub struct #builder_name {
            #(#decls),*
        }
        impl #builder_name {
            #new_imp

            #(#into_fns)*

            #[doc = #build_doc]
            pub fn build(self) -> #name {
                #(#default_stmts)*
                #name {
                    #(#idents),*
                }
            }
        }
        impl From<#builder_name> for #name {
            fn from(x: #builder_name) -> Self {
                x.build()
            }
        }
        impl Default for #builder_name {
            fn default() -> Self {
                Self::new()
            }
        }
        impl Default for #name {
            fn default() -> Self {
                #builder_name::new().build()
            }
        }
        impl #name {
            /// Return a builder with all options unspecified.
            pub fn builder() -> #builder_name {
                #builder_name::new()
            }
        }
    }
}

fn derive_impl(data: &Data, name: &Ident) -> TokenStream {
    match data {
        Data::Struct(ref data) => derive_struct_impl(data, name),
        Data::Enum(ref data) if data.variants.len() == 0 => unimplemented!("{}", ENUM_ZERO_VARIANT),
        Data::Enum(ref data) => derive_enum_impl(data, name),
        Data::Union(_) => unimplemented!("{}", UNION),
    }
}

fn struct_fields(data: &syn::DataStruct) -> Vec<FieldInfo> {
    match &data.fields {
        Fields::Named(_) => data.fields.iter().map(FieldInfo::from).collect(),
        Fields::Unnamed(_) => unimplemented!("{}", UNNAMED_FIELDS),
        Fields::Unit => unimplemented!("{}", UNIT_STRUCT),
    }
}

fn derive_enum_impl(data: &syn::DataEnum, name: &Ident) -> TokenStream {
    let impls = data
        .variants
        .iter()
        .filter_map(|var| derive_variant_impl(var, name));
    quote! {
        #(#impls)*
    }
}

fn variant_fields(var: &syn::Variant) -> Option<Vec<FieldInfo>> {
    match &var.fields {
        Fields::Named(_) => Some(var.fields.iter().map(FieldInfo::from).collect()),
        Fields::Unnamed(_) => unimplemented!("{}", UNNAMED_FIELDS),
        Fields::Unit => None,
    }
}
fn derive_variant_impl(var: &syn::Variant, name: &Ident) -> Option<TokenStream> {
    if let Some(fields) = variant_fields(var) {
        let var_name = &var.ident;
        let builder_name = format_ident!("{}Builder", var_name);
        let decls = builder_fields(&fields);
        let into_fns = into_fns(&fields);
        let default_stmts = build_defaults(&fields);
        let idents = fields.iter().map(|FieldInfo { ref ident, .. }| ident);
        let ty_variant = format!("{}::{}", name, var_name);
        let builder_doc = builder_doc(&ty_variant);
        let build_doc = format!("Build the `{}` instance.", ty_variant);
        let new_imp = new_impl(&fields);
        Some(quote! {
            #[derive(Debug, Clone, PartialEq)]
            #[doc = #builder_doc]
            pub struct #builder_name {
                #(#decls),*
            }
            impl #builder_name {
                #new_imp

                #(#into_fns)*

                #[doc = #build_doc]
                pub fn build(self) -> #name {
                    #(#default_stmts)*
                    #name::#var_name {
                        #(#idents),*
                    }
                }
            }
            impl From<#builder_name> for #name {
                fn from(x: #builder_name) -> Self {
                    x.build()
                }
            }
            impl Default for #builder_name {
                fn default() -> Self {
                    Self::new()
                }
            }
        })
    } else {
        None
    }
}

fn is_outer(a: &Attribute) -> bool {
    match a.style {
        AttrStyle::Outer => true,
        _ => false,
    }
}
fn is_defaults_to(a: &Attribute) -> bool {
    a.meta.path().is_ident("defaults_to")
}

fn get_default(input: &[Attribute]) -> Option<syn::Lit> {
    let mut n: usize = 0;
    let defaults = input
        .into_iter()
        .filter(|a| is_outer(*a) && is_defaults_to(*a))
        .inspect(|_| {
            n += 1;
        });
    if let Some(a) = defaults.last() {
        if n > 1 {
            unimplemented!("Only a single `#[defaults_to = ...]` is permissible per field.")
        } else {
            let value = match &a.meta {
                Meta::NameValue(x) => match &x.value {
                    syn::Expr::Lit(x) => x.lit.clone(),
                    e => unimplemented!("`defaults_to` value must be a literal, got {:?}", e),
                },
                _ => unimplemented!("`defaults_to` attribute must be name-value."),
            };
            Some(value)
        }
    } else {
        None
    }
}
