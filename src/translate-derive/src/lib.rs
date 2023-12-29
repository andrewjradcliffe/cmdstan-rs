use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{parse_macro_input, AttrStyle, Attribute, Data, DeriveInput, Fields, Meta};

static UNIT_STRUCT: &str = "`Translate` not supported on unit struct";
static UNNAMED_FIELDS: &str = "`Translate` not supported on struct with unnamed fields";
static ENUM_ZERO_VARIANT: &str = "`Translate` not supported on enum with zero variants";
static UNION: &str = "`Translate` not supported union";
static ENUM_REQ_DECLARE: &str = "enum requires `declare`";

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
    fn is_bool(&self) -> bool {
        match self {
            Self::Bool => true,
            _ => false,
        }
    }
    fn is_number(&self) -> bool {
        match self {
            Self::Int | Self::UInt | Self::Float => true,
            _ => false,
        }
    }

    fn is_string(&self) -> bool {
        match self {
            Self::String => true,
            _ => false,
        }
    }

    // Potentially useful methods, but, otherwise, dead code.
    // fn is_primitive(&self) -> bool {
    //     match self {
    //         Self::NotPrimitive => false,
    //         _ => true,
    //     }
    // }
    // fn is_number_or_string(&self) -> bool {
    //     self.is_primitive() || self.is_string()
    // }
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

#[proc_macro_derive(Translate, attributes(declare))]
pub fn derive_translate(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let decl = get_declare(&input.attrs[..]);
    let append_args = append_args_body(&input.data, decl.clone());
    let write_tree_offset = write_tree_offset_body(&input.data, decl.clone());
    let write_stmt = write_stmt_body(&input.data, decl);
    let expanded = quote! {
        impl crate::translate::private::Sealed for #name {}
        impl Translate for #name {
            fn append_args(&self, v: &mut Vec<OsString>) {
                #append_args
            }

            fn write_tree_offset(&self, n: usize, s: &mut OsString) {
                use std::fmt::Write;
                #write_tree_offset
            }
            fn write_stmt(&self, s: &mut OsString) {
                use std::fmt::Write;
                #write_stmt
            }
        }
    };
    proc_macro::TokenStream::from(expanded)
}

fn struct_append_args(data: &syn::DataStruct, decl: Option<String>) -> TokenStream {
    match &data.fields {
        Fields::Named(_) => {
            let mut q = if let Some(decl) = decl {
                quote! {
                    v.push(OsString::from(#decl));
                }
            } else {
                quote! {}
            };
            let iter = data.fields.iter().map(move |f| {
                let ident = f.ident.as_ref().unwrap();
                let ty_ident = match &f.ty {
                    syn::Type::Path(path) => path.path.get_ident(),
                    _ => unimplemented!("type is not `TypePath`"),
                }
                .unwrap();

                (ident, Type::from(ty_ident))
            });
            for (ident, ty) in iter {
                if ty.is_number() {
                    let lhs = format!("{}={{}}", ident);
                    q = quote! {
                        #q
                        v.push(OsString::from(format!(#lhs, self.#ident)));
                    };
                } else if ty.is_bool() {
                    let lhs = format!("{}={{}}", ident);
                    q = quote! {
                        #q
                        v.push(OsString::from(format!(#lhs, self.#ident as u8)));
                    };
                } else if ty.is_string() {
                    let lhs = format!("{}=", ident);
                    let len = lhs.len();
                    q = quote! {
                        #q
                        v.push({
                            let mut s = OsString::with_capacity(#len + self.#ident.len());
                            s.push(#lhs);
                            s.push(&self.#ident);
                            s
                        });
                    };
                } else {
                    q = quote! {
                        #q
                        self.#ident.append_args(v);
                    };
                }
            }
            q
        }
        Fields::Unnamed(_) => unimplemented!("{}", UNNAMED_FIELDS),
        Fields::Unit => unimplemented!("{}", UNIT_STRUCT),
    }
}

fn enum_variant_append_args_body(var: &syn::Variant, decl: String) -> TokenStream {
    match &var.fields {
        Fields::Named(_) => {
            let mut q = quote! {
                v.push(OsString::from(#decl));
            };

            let iter = var.fields.iter().map(move |f| {
                let ident = f.ident.as_ref().unwrap();
                let ty_ident = match &f.ty {
                    syn::Type::Path(path) => path.path.get_ident(),
                    _ => unimplemented!("type is not `TypePath`"),
                }
                .unwrap();
                (ident, Type::from(ty_ident))
            });

            let mut idents = Vec::new();

            for (ident, ty) in iter {
                if ty.is_number() {
                    let lhs = format!("{}={{}}", ident);
                    q = quote! {
                        #q
                        v.push(OsString::from(format!(#lhs, #ident)));
                    };
                } else if ty.is_bool() {
                    let lhs = format!("{}={{}}", ident);
                    q = quote! {
                        #q
                        v.push(OsString::from(format!(#lhs, *#ident as u8)));
                    };
                } else if ty.is_string() {
                    let lhs = format!("{}=", ident);
                    let len = lhs.len();
                    q = quote! {
                        #q
                        v.push({
                            let mut s = OsString::with_capacity(#len + #ident.len());
                            s.push(#lhs);
                            s.push(#ident);
                            s
                        });
                    };
                } else {
                    q = quote! {
                        #q
                        #ident.append_args(v);
                    };
                }
                idents.push(ident);
            }
            let me = &var.ident;
            quote! {
                Self::#me { #(#idents),* } => {
                    #q
                }
            }
        }
        Fields::Unnamed(_) => unimplemented!("{}", UNNAMED_FIELDS),
        Fields::Unit => {
            let me = &var.ident;
            quote! {
                Self::#me => {
                    v.push(OsString::from(#decl));
                }
            }
        }
    }
}

fn enum_append_args(data: &syn::DataEnum, decl: Option<String>) -> TokenStream {
    let Some(decl) = decl else {
        unimplemented!("{}", ENUM_REQ_DECLARE)
    };
    let decl_ref = decl.trim_matches('"');
    let recurse = data.variants.iter().map(|var| {
        let name = if let Some(name) = get_declare(&var.attrs[..]) {
            name.trim_matches('"').to_string()
        } else {
            var.ident.to_string().to_lowercase()
        };
        let decl = format!("{}={}", decl_ref, name);
        enum_variant_append_args_body(var, decl)
    });
    quote! {
        match self {
            #(#recurse),*
        }
    }
}

fn append_args_body(data: &Data, decl: Option<String>) -> TokenStream {
    match *data {
        Data::Struct(ref data) => struct_append_args(data, decl),
        Data::Enum(ref data) if data.variants.len() != 0 => enum_append_args(data, decl),
        Data::Enum(_) => unimplemented!("{}", ENUM_ZERO_VARIANT),
        Data::Union(_) => unimplemented!("{}", UNION),
    }
}

fn write_stmt_body(data: &Data, decl: Option<String>) -> TokenStream {
    match *data {
        Data::Struct(ref data) => struct_write_stmt(data, decl),
        Data::Enum(ref data) if data.variants.len() != 0 => enum_write_stmt(data, decl),
        Data::Enum(_) => unimplemented!("{}", ENUM_ZERO_VARIANT),
        Data::Union(_) => unimplemented!("{}", UNION),
    }
}

fn write_tree_offset_body(data: &Data, decl: Option<String>) -> TokenStream {
    match *data {
        Data::Struct(ref data) => struct_write_tree_offset(data, decl),
        Data::Enum(ref data) if data.variants.len() != 0 => enum_write_tree_offset(data, decl),
        Data::Enum(_) => unimplemented!("{}", ENUM_ZERO_VARIANT),
        Data::Union(_) => unimplemented!("{}", UNION),
    }
}

fn enum_write_tree_offset(data: &syn::DataEnum, decl: Option<String>) -> TokenStream {
    let Some(decl) = decl else {
        unimplemented!("{}", ENUM_REQ_DECLARE)
    };
    let decl_ref = decl.trim_matches('"');
    if decl_ref == "metric" {
        // Handle special case
        let recurse = data.variants.iter().map(|var| {
            let ident = &var.ident;
            let name = if let Some(name) = get_declare(&var.attrs[..]) {
                name.trim_matches('"').to_string()
            } else {
                var.ident.to_string().to_lowercase()
            };
            let tyvar = format!("metric = {}", name);
            quote! {
                Self::#ident => write!(s, #tyvar).unwrap()
            }
        });
        quote! {
            for _ in 0..n {
                s.push(" ");
            }
            match self {
                #(#recurse),*
            }
        }
    } else {
        let recurse = data
            .variants
            .iter()
            .map(|var| enum_variant_write_tree_offset_body(var, decl_ref));
        // The initial offset is common to all variants, hence,
        // the code need not be specific to any `match` arm.
        // Likewise, the increment of the offset for the next line
        // can be computed as soon as the required whitespace has been
        // written.
        quote! {
            for _ in 0..n {
                s.push(" ");
            }
            let n = n + 2;
            match self {
                #(#recurse),*
            }
        }
    }
}

fn enum_variant_write_tree_offset_body(var: &syn::Variant, decl: &str) -> TokenStream {
    let name = if let Some(name) = get_declare(&var.attrs[..]) {
        name.trim_matches('"').to_string()
    } else {
        var.ident.to_string().to_lowercase()
    };
    let tyvar = format!("{} = {}\n", decl, name);
    match &var.fields {
        Fields::Named(_) => {
            // A variant with named fields is equivalent to a non-unit struct
            // with named fields and is displayed equivalently.
            // The offset of each named field is 2 greater than the offset
            // of the variant-type declaration, equivalent to the named fields
            // of a struct with declared type.
            let variant = format!("{}\n", name);
            let mut q = quote! {
                write!(s, #tyvar).unwrap();
                for _ in 0..n {
                    s.push(" ");
                }
                write!(s, #variant).unwrap();
                let n = n + 2;
            };

            let mut iter = var
                .fields
                .iter()
                .map(move |f| {
                    let ident = f.ident.as_ref().unwrap();
                    let ty_ident = match &f.ty {
                        syn::Type::Path(path) => path.path.get_ident(),
                        _ => unimplemented!("type is not `TypePath`"),
                    }
                    .unwrap();
                    (ident, Type::from(ty_ident))
                })
                .peekable();

            let mut idents = Vec::new();

            while let Some((ident, ty)) = iter.next() {
                let is_not_last = iter.peek().is_some();
                if ty.is_number() {
                    let lhs = format!("{} = {{}}", ident);
                    q = quote! {
                        #q
                        for _ in 0..n {
                            s.push(" ");
                        }
                        write!(s, #lhs, #ident).unwrap();
                    };
                } else if ty.is_bool() {
                    let lhs = format!("{} = {{}}", ident);
                    q = quote! {
                        #q
                        for _ in 0..n {
                            s.push(" ");
                        }
                        write!(s, #lhs, *#ident as u8).unwrap();
                    };
                } else if ty.is_string() {
                    let lhs = format!("{} = ", ident);
                    q = quote! {
                        #q
                        for _ in 0..n {
                            s.push(" ");
                        }
                        write!(s, #lhs).unwrap();
                        s.push(#ident);
                    }
                } else {
                    q = quote! {
                        #q
                        #ident.write_tree_offset(n, s);
                    };
                }
                if is_not_last {
                    q = quote! {
                        #q
                        s.push("\n");
                    }
                }
                idents.push(ident);
            }
            let me = &var.ident;
            quote! {
                Self::#me { #(#idents),* } => {
                    #q
                }
            }
        }
        Fields::Unnamed(_) => unimplemented!("{}", UNNAMED_FIELDS),
        Fields::Unit => {
            // If the variant is the unit variant, then the "variant" declaration
            // is the last line, hence, we exclude the newline.
            let variant = format!("{}", name);
            let me = &var.ident;
            quote! {
                Self::#me => {
                    write!(s, #tyvar).unwrap();
                    for _ in 0..n {
                        s.push(" ");
                    }
                    write!(s, #variant).unwrap();
                }
            }
        }
    }
}

fn struct_write_tree_offset(data: &syn::DataStruct, decl: Option<String>) -> TokenStream {
    match &data.fields {
        Fields::Named(_) => {
            // The key difference here is that a struct without a type declaration
            // prints as a list of fields at the offset of the (missing) type declaration.
            // N.B. It would be unusual to omit the type declaration for any struct
            // which is not top-level, as the ownership of the respective fields would become
            // ambiguous.
            let mut q = if let Some(decl) = decl {
                quote! {
                    for _ in 0..n {
                        s.push(" ");
                    }
                    let n = n + 2;
                    write!(s, #decl).unwrap();
                    s.push("\n");
                }
            } else {
                quote! {}
            };
            let mut iter = data
                .fields
                .iter()
                .map(move |f| {
                    let ident = f.ident.as_ref().unwrap();
                    let ty_ident = match &f.ty {
                        syn::Type::Path(path) => path.path.get_ident(),
                        _ => unimplemented!("type is not `TypePath`"),
                    }
                    .unwrap();
                    (ident, Type::from(ty_ident))
                })
                .peekable();
            while let Some((ident, ty)) = iter.next() {
                let is_not_last = iter.peek().is_some();
                if ty.is_number() {
                    let lhs = format!("{} = {{}}", ident);
                    q = quote! {
                        #q
                        for _ in 0..n {
                            s.push(" ");
                        }
                        write!(s, #lhs, self.#ident).unwrap();
                    };
                } else if ty.is_bool() {
                    let lhs = format!("{} = {{}}", ident);
                    q = quote! {
                        #q
                        for _ in 0..n {
                            s.push(" ");
                        }
                        write!(s, #lhs, self.#ident as u8).unwrap();
                    };
                } else if ty.is_string() {
                    let lhs = format!("{} = ", ident);
                    q = quote! {
                        #q
                        for _ in 0..n {
                            s.push(" ");
                        }
                        write!(s, #lhs).unwrap();
                        s.push(&self.#ident);
                    }
                } else {
                    q = quote! {
                        #q
                        self.#ident.write_tree_offset(n, s);
                    };
                }
                if is_not_last {
                    q = quote! {
                        #q
                        s.push("\n");
                    };
                }
            }
            q
        }
        Fields::Unnamed(_) => unimplemented!("{}", UNNAMED_FIELDS),
        Fields::Unit => unimplemented!("{}", UNIT_STRUCT),
    }
}

fn struct_write_stmt(data: &syn::DataStruct, decl: Option<String>) -> TokenStream {
    match &data.fields {
        Fields::Named(_) => {
            let mut q = if let Some(decl) = decl {
                quote! {
                    write!(s, #decl).unwrap();
                    s.push(" ");
                }
            } else {
                quote! {}
            };
            let mut iter = data
                .fields
                .iter()
                .map(move |f| {
                    let ident = f.ident.as_ref().unwrap();
                    let ty_ident = match &f.ty {
                        syn::Type::Path(path) => path.path.get_ident(),
                        _ => unimplemented!("type is not `TypePath`"),
                    }
                    .unwrap();
                    (ident, Type::from(ty_ident))
                })
                .peekable();
            while let Some((ident, ty)) = iter.next() {
                let is_not_last = iter.peek().is_some();
                if ty.is_number() {
                    let lhs = format!("{}={{}}", ident);
                    q = quote! {
                        #q
                        write!(s, #lhs, self.#ident).unwrap();
                    };
                } else if ty.is_bool() {
                    let lhs = format!("{}={{}}", ident);
                    q = quote! {
                        #q
                        write!(s, #lhs, self.#ident as u8).unwrap();
                    };
                } else if ty.is_string() {
                    let lhs = format!("{}=", ident);
                    q = quote! {
                        #q
                        write!(s, #lhs).unwrap();
                        s.push(&self.#ident);
                    };
                } else {
                    q = quote! {
                        #q
                        self.#ident.write_stmt(s);
                    };
                }
                if is_not_last {
                    q = quote! {
                        #q
                        s.push(" ");
                    }
                }
            }
            quote! {
                #q
            }
        }
        Fields::Unnamed(_) => unimplemented!("{}", UNNAMED_FIELDS),
        Fields::Unit => unimplemented!("{}", UNIT_STRUCT),
    }
}

fn enum_variant_write_stmt_body(var: &syn::Variant, decl: String) -> TokenStream {
    match &var.fields {
        Fields::Named(_) => {
            let mut q = quote! {
                write!(s, #decl).unwrap();
                s.push(" ");
            };

            let mut iter = var
                .fields
                .iter()
                .map(move |f| {
                    let ident = f.ident.as_ref().unwrap();
                    let ty_ident = match &f.ty {
                        syn::Type::Path(path) => path.path.get_ident(),
                        _ => unimplemented!("type is not `TypePath`"),
                    }
                    .unwrap();
                    (ident, Type::from(ty_ident))
                })
                .peekable();

            let mut idents = Vec::new();

            while let Some((ident, ty)) = iter.next() {
                let is_not_last = iter.peek().is_some();
                if ty.is_number() {
                    let lhs = format!("{}={{}}", ident);
                    q = quote! {
                        #q
                        write!(s, #lhs, #ident).unwrap();
                    };
                } else if ty.is_bool() {
                    let lhs = format!("{}={{}}", ident);
                    q = quote! {
                        #q
                        write!(s, #lhs, *#ident as u8).unwrap();
                    };
                } else if ty.is_string() {
                    let lhs = format!("{}=", ident);
                    q = quote! {
                        #q
                        write!(s, #lhs).unwrap();
                        s.push(#ident);
                    };
                } else {
                    q = quote! {
                        #q
                        #ident.write_stmt(s);
                    }
                }
                if is_not_last {
                    q = quote! {
                        #q
                        s.push(" ");
                    }
                }
                idents.push(ident);
            }
            let me = &var.ident;
            quote! {
                Self::#me { #(#idents),* } => {
                    #q
                }
            }
        }
        Fields::Unnamed(_) => unimplemented!("{}", UNNAMED_FIELDS),
        Fields::Unit => {
            let me = &var.ident;
            quote! {
                Self::#me => write!(s, #decl).unwrap()
            }
        }
    }
}
fn enum_write_stmt(data: &syn::DataEnum, decl: Option<String>) -> TokenStream {
    let Some(decl) = decl else {
        unimplemented!("{}", ENUM_REQ_DECLARE)
    };
    let decl_ref = decl.trim_matches('"');
    let recurse = data.variants.iter().map(|var| {
        let name = if let Some(name) = get_declare(&var.attrs[..]) {
            name.trim_matches('"').to_string()
        } else {
            var.ident.to_string().to_lowercase()
        };
        let decl = format!("{}={}", decl_ref, name);
        enum_variant_write_stmt_body(var, decl)
    });
    quote! {
        match self {
            #(#recurse),*
        }
    }
}

fn is_outer(a: &Attribute) -> bool {
    match a.style {
        AttrStyle::Outer => true,
        _ => false,
    }
}
fn is_declare(a: &Attribute) -> bool {
    a.meta.path().is_ident("declare")
}

fn get_declare(input: &[Attribute]) -> Option<String> {
    let mut n: usize = 0;
    let decls = input
        .into_iter()
        .filter(|a| is_outer(*a) && is_declare(*a))
        .inspect(|_| {
            n += 1;
        });
    if let Some(a) = decls.last() {
        if n > 1 {
            unimplemented!("Only a single `#[declare =\"...\"]` is permissible.")
        } else {
            let value = match &a.meta {
                Meta::NameValue(ref x) => match x.value {
                    syn::Expr::Lit(ref x) => match x.lit {
                        syn::Lit::Str(ref x) => x.value(),
                        _ => unimplemented!("`declare` value must be a string literal"),
                    },
                    _ => unimplemented!("`declare` value must be a string literal"),
                },
                _ => unimplemented!("`declare` attribute must be name-value."),
            };
            Some(value)
        }
    } else {
        None
    }
}
