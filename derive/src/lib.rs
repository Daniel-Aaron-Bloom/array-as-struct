use itertools::multiunzip;
use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro_crate::{crate_name, FoundCrate};
use proc_macro_error::{abort, abort_if_dirty, emit_error, proc_macro_error};
use quote::quote;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, Data, DeriveInput, GenericParam, Ident, LifetimeParam, Member, Token, Type,
    TypeParam, TypeTuple,
};

/// A derive-like macro which replaces a field-struct declaration with a
/// tuple-struct declaration containing a single array. All fields in the
/// original declaration must share the same type.
///
/// This attribute should almost always come before to any `derive` macros.
#[proc_macro_error]
#[proc_macro_attribute]
pub fn array_as_struct(attr: TokenStream, item: TokenStream) -> TokenStream {
    array_as_struct_helper(attr, item, false)
}

#[doc(hidden)]
#[proc_macro_error]
#[proc_macro_attribute]
pub fn array_as_struct_doctest(attr: TokenStream, item: TokenStream) -> TokenStream {
    array_as_struct_helper(attr, item, true)
}

fn array_as_struct_helper(_attr: TokenStream, item: TokenStream, doctest: bool) -> TokenStream {
    let found_crate =
        crate_name("array-as-struct").expect("array-as-struct is present in `Cargo.toml`");
    let found_crate = match found_crate {
        FoundCrate::Name(name) => Ident::new(&name, Span::call_site()),
        FoundCrate::Itself if doctest => Ident::new("array_as_struct", Span::call_site()),
        FoundCrate::Itself => <Token![crate]>::default().into(),
    };

    let ast = parse_macro_input!(item as DeriveInput);
    let ast_span = ast.span();

    let DeriveInput {
        ident,
        generics,
        attrs,
        vis,
        data,
    } = ast;

    let data = match data {
        Data::Struct(data) => data,
        _ => abort!(ast_span, "only named-field structs are supported"),
    };

    // Converts `<F, const D: usize>` (sans `<` and `>`) to
    //          `<F, D>` (sans `<` and `>`)
    let generic_params = generics.params;
    let generic_params_no_attr: Punctuated<GenericParam, Token![,]> = generic_params
        .iter()
        .map(|gen| match gen {
            GenericParam::Lifetime(x) => GenericParam::Lifetime(LifetimeParam {
                attrs: vec![],
                lifetime: x.lifetime.clone(),
                colon_token: None,
                bounds: Punctuated::new(),
            }),
            GenericParam::Type(x) => GenericParam::Type(x.clone()),
            GenericParam::Const(x) => GenericParam::Type(TypeParam {
                ident: x.ident.clone(),
                attrs: vec![],
                colon_token: None,
                bounds: Punctuated::new(),
                eq_token: None,
                default: None,
            }),
        })
        .collect();

    let mut field_ty = None;
    let field_info = data.fields.into_iter().map(|field| {
        let ident = match field.ident {
            Some(ident) => Member::Named(ident.clone()),
            None => abort!(ast_span, "only named-field structs are supported"),
        };
        match field_ty.take() {
            None => field_ty = Some(field.ty),
            Some(field_ty) if field_ty != field.ty => {
                emit_error!(field_ty, "type did not match future fields");
                abort!(field.ty, "type did not match previous fields");
            }
            Some(x) => field_ty = Some(x),
        }
        (field.attrs, field.vis, ident)
    });
    let (attr_fields, vis_fields, ident_fields): (Vec<_>, Vec<_>, Vec<_>) = multiunzip(field_info);
    let field_ty = field_ty.unwrap_or(Type::Tuple(TypeTuple {
        paren_token: Default::default(),
        elems: Punctuated::new(),
    }));

    let field_index = 0usize..;
    let field_count = vis_fields.len();
    let field_count_str = field_count.to_string();

    abort_if_dirty();

    let v = quote!(
        #(#attrs)*
        #[repr(transparent)]
        #vis struct #ident<#generic_params>(
            /// The array of
            #[doc = #field_count_str]
            /// values
            pub [#field_ty; #field_count]
        );

        impl<#generic_params> #ident<#generic_params_no_attr> {
            #[inline(always)]
            /// Construct the tuple-struct type from the named-field type
            #vis const fn from_val(value: <Self as #found_crate::ArrayStruct>::Value) -> Self {
                #(#attrs)*
                #vis struct Value<#generic_params>{#(
                    #(#attr_fields)*
                    #vis_fields #ident_fields: #field_ty
                ),*};
                #[allow(dead_code)]
                #vis struct Refs<'__array_as_struct, #generic_params>{#(
                    #(#attr_fields)*
                    #vis_fields #ident_fields: &'__array_as_struct #field_ty),*
                };
                #[allow(dead_code)]
                #vis struct Muts<'__array_as_struct, #generic_params>{#(
                    #(#attr_fields)*
                    #vis_fields #ident_fields: &'__array_as_struct mut #field_ty),*
                };
                #[allow(dead_code)]
                #vis struct Index;

                impl<#generic_params> Value<#generic_params_no_attr> {
                    ///
                    #[inline(always)]
                    pub const fn to_array_struct(self) -> #ident<#generic_params_no_attr> {
                        #ident::from_val(self)
                    }
                }

                impl Index {#(
                    #[inline(always)]
                    pub const fn #ident_fields() -> usize { #field_index }
                )*}

                impl<#generic_params> #found_crate::ArrayStruct for #ident<#generic_params_no_attr> {
                    type Value = Value<#generic_params_no_attr>;
                    type Array = [#field_ty; #field_count];
                    type Refs<'__array_as_struct> = Refs<'__array_as_struct, #generic_params_no_attr>;
                    type Muts<'__array_as_struct> = Muts<'__array_as_struct, #generic_params_no_attr>;
                    type Index = Index;
                    #[inline(always)]
                    fn from_val(value: Self::Value) -> Self {
                        <#ident::<#generic_params_no_attr>>::from_val(value)
                    }
                    #[inline(always)]
                    fn val(self) -> Self::Value {
                        <#ident::<#generic_params_no_attr>>::val(self)
                    }
                    #[inline(always)]
                    fn to_array(self) -> Self::Array {
                        self.0
                    }
                    #[inline(always)]
                    fn from_array(array: Self::Array) -> Self {
                        Self(array)
                    }
                    #[inline(always)]
                    fn refs(&'_ self) -> Self::Refs<'_> {
                        <#ident::<#generic_params_no_attr>>::refs(self)
                    }
                    #[inline(always)]
                    fn muts(&'_ mut self) -> Self::Muts<'_> {
                        <#ident::<#generic_params_no_attr>>::muts(self)
                    }

                }

                Self([#(value.#ident_fields),*])
            }

            #[inline(always)]
            /// Construct the named-field type from the tuple-struct type
            #vis const fn val(self) -> <Self as #found_crate::ArrayStruct>::Value {
                let Self([#(#ident_fields),*]) = self;
                type Value = <#ident::<#generic_params_no_attr> as #found_crate::ArrayStruct>::Value;
                Value {
                    #(#ident_fields),*
                }
            }

            #[inline(always)]
            /// Construct the reference-named-field type from the tuple-struct type.
            #vis const fn refs(&'_ self) -> <Self as #found_crate::ArrayStruct>::Refs<'_> {
                let Self([#(#ident_fields),*]) = self;
                type Refs<'__array_as_struct> = <#ident::<#generic_params_no_attr> as #found_crate::ArrayStruct>::Refs<'__array_as_struct>;
                Refs {
                    #(#ident_fields),*
                }
            }

            #[inline(always)]
            /// Construct the mutable-reference-named-field type from the tuple-struct type
            #vis fn muts(&'_ mut self) -> <Self as #found_crate::ArrayStruct>::Muts<'_> {
                let Self([#(#ident_fields),*]) = self;
                type Muts<'__array_as_struct> = <#ident::<#generic_params_no_attr> as #found_crate::ArrayStruct>::Muts<'__array_as_struct>;
                Muts {
                    #(#ident_fields),*
                }
            }
        }

        impl<#generic_params> ::core::convert::From<<#ident::<#generic_params_no_attr> as #found_crate::ArrayStruct>::Value> for #ident<#generic_params_no_attr> {
            #[inline(always)]
            fn from(value: <#ident::<#generic_params_no_attr> as #found_crate::ArrayStruct>::Value) -> Self {
                Self::from_val(value)
            }
        }
        impl<#generic_params> ::core::convert::From<#ident<#generic_params_no_attr>> for <#ident::<#generic_params_no_attr> as #found_crate::ArrayStruct>::Value {
            #[inline(always)]
            fn from(strct: #ident<#generic_params_no_attr>) -> Self {
                strct.val()
            }
        }

        impl<#generic_params> ::core::convert::From<<#ident::<#generic_params_no_attr> as #found_crate::ArrayStruct>::Array> for #ident<#generic_params_no_attr> {
            #[inline(always)]
            fn from(array: <#ident::<#generic_params_no_attr> as #found_crate::ArrayStruct>::Array) -> Self {
                Self(array)
            }
        }
        impl<#generic_params> ::core::convert::From<#ident<#generic_params_no_attr>> for <#ident::<#generic_params_no_attr> as #found_crate::ArrayStruct>::Array {
            #[inline(always)]
            fn from(strct: #ident<#generic_params_no_attr>) -> Self {
                strct.0
            }
        }

        impl<#generic_params> ::core::convert::AsRef<<#ident::<#generic_params_no_attr> as #found_crate::ArrayStruct>::Array> for #ident<#generic_params_no_attr> {
            #[inline(always)]
            fn as_ref(&self) -> &<#ident::<#generic_params_no_attr> as #found_crate::ArrayStruct>::Array {
                &self.0
            }
        }
        impl<#generic_params> ::core::convert::AsMut<<#ident::<#generic_params_no_attr> as #found_crate::ArrayStruct>::Array> for #ident<#generic_params_no_attr> {
            #[inline(always)]
            fn as_mut(&mut self) -> &mut <#ident::<#generic_params_no_attr> as #found_crate::ArrayStruct>::Array {
                &mut self.0
            }
        }

        impl<#generic_params> ::core::borrow::Borrow<<#ident::<#generic_params_no_attr> as #found_crate::ArrayStruct>::Array> for #ident<#generic_params_no_attr> {
            #[inline(always)]
            fn borrow(&self) -> &<#ident::<#generic_params_no_attr> as #found_crate::ArrayStruct>::Array {
                &self.0
            }
        }
        impl<#generic_params> ::core::borrow::BorrowMut<<#ident::<#generic_params_no_attr> as #found_crate::ArrayStruct>::Array> for #ident<#generic_params_no_attr> {
            #[inline(always)]
            fn borrow_mut(&mut self) -> &mut <#ident::<#generic_params_no_attr> as #found_crate::ArrayStruct>::Array {
                &mut self.0
            }
        }

        impl<#generic_params> ::core::ops::Deref for #ident<#generic_params_no_attr> {
            type Target = [#field_ty; #field_count];
            #[inline(always)]
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
        impl<#generic_params> ::core::ops::DerefMut for #ident<#generic_params_no_attr> {
            #[inline(always)]
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }

        impl<I> core::ops::Index<I> for #ident<#generic_params_no_attr>
        where [#field_ty; #field_count]: core::ops::Index<I> {
            type Output = <[#field_ty; #field_count] as core::ops::Index<I>>::Output;

            #[inline(always)]
            fn index(&self, index: I) -> &Self::Output {
                &self.0[index]
            }
        }

        impl<I> core::ops::IndexMut<I> for #ident<#generic_params_no_attr>
        where [#field_ty; #field_count]: core::ops::IndexMut<I> {
            #[inline(always)]
            fn index_mut(&mut self, index: I) -> &mut Self::Output {
                &mut self.0[index]
            }
        }
    );

    v.into()
}
