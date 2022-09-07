use proc_macro2::TokenStream;
use quote::quote;

use super::{discard_lifetimes, ContainerMeta, FieldMeta};

pub fn to_xml(input: &syn::DeriveInput) -> proc_macro2::TokenStream {
    let mut body = TokenStream::new();
    let mut attributes = TokenStream::new();
    let meta = ContainerMeta::from_derive(input);
    match &input.data {
        syn::Data::Struct(ref data) => {
            match data.fields {
                syn::Fields::Named(ref fields) => {
                    fields.named.iter().for_each(|field| {
                        process_named_field(field, &mut body, &mut attributes, &meta);
                    });
                }
                syn::Fields::Unnamed(_) => todo!(),
                syn::Fields::Unit => {}
            };
        }
        _ => todo!(),
    };

    let default_namespace = match &meta.ns.uri {
        Some(ns) => quote!(#ns),
        None => quote!(""),
    };

    let cx_len = meta.ns.prefixes.len();
    let mut context = quote!(
        let mut new = ::instant_xml::ser::Context::<#cx_len>::default();
        new.default_ns = #default_namespace;
    );
    for (i, (prefix, ns)) in meta.ns.prefixes.iter().enumerate() {
        context.extend(quote!(
            new.prefixes[#i] = ::instant_xml::ser::Prefix { ns: #ns, prefix: #prefix };
        ));
    }

    let ident = &input.ident;
    let root_name = ident.to_string();
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    quote!(
        impl #impl_generics ToXml for #ident #ty_generics #where_clause {
            fn serialize<W: ::core::fmt::Write + ?::core::marker::Sized>(
                &self,
                serializer: &mut instant_xml::Serializer<W>,
            ) -> Result<(), instant_xml::Error> {
                // Start tag
                match serializer.default_ns() == #default_namespace {
                    true => serializer.write_start(None, #root_name, None)?,
                    false => serializer.write_start(None, #root_name, Some(#default_namespace))?,
                }

                #context
                let old = serializer.push(new)?;

                #attributes
                serializer.end_start()?;

                #body

                // Close tag
                serializer.write_close(None, #root_name)?;
                serializer.pop(old);

                Ok(())
            }

            const KIND: ::instant_xml::Kind = ::instant_xml::Kind::Element(::instant_xml::Id {
                ns: #default_namespace,
                name: #root_name,
            });
        };
    )
}

fn process_named_field(
    field: &syn::Field,
    body: &mut TokenStream,
    attributes: &mut TokenStream,
    meta: &ContainerMeta,
) {
    let name = field.ident.as_ref().unwrap().to_string();
    let field_value = field.ident.as_ref().unwrap();

    let field_meta = FieldMeta::from_field(field);
    if field_meta.attribute {
        attributes.extend(quote!(
            serializer.write_attr(#name, &self.#field_value)?;
        ));
        return;
    }

    let ns = match field_meta.ns.uri {
        Some(ns) => quote!(#ns),
        None => match &meta.ns.uri {
            Some(ns) => quote!(#ns),
            None => quote!(""),
        },
    };

    let mut no_lifetime_type = field.ty.clone();
    discard_lifetimes(&mut no_lifetime_type);
    body.extend(quote!(
        match <#no_lifetime_type as ToXml>::KIND {
            ::instant_xml::Kind::Element(_) => {
                self.#field_value.serialize(serializer)?;
            }
            ::instant_xml::Kind::Scalar => {
                let (prefix, ns) = match serializer.default_ns() == #ns {
                    true => (None, None),
                    false => match serializer.prefix(#ns) {
                        Some(prefix) => (Some(prefix), None),
                        None => (None, Some(#ns)),
                    },
                };

                serializer.write_start(prefix, #name, ns)?;
                serializer.end_start()?;
                self.#field_value.serialize(serializer)?;
                serializer.write_close(prefix, #name)?;
            }
        }
    ));
}
