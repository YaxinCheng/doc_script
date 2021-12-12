use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DataStruct, DeriveInput, Fields};

pub(crate) fn implement(input: DeriveInput) -> TokenStream {
    let name = &input.ident;
    let generics = &input.generics;
    field_assert(&input.data);
    return quote! {
        impl #generics Scoped for #name #generics {
            fn scope(&self) -> ScopeId {
                self.scope.expect("Scope id is not set")
            }

            fn set_scope(&mut self, scope: ScopeId) {
                self.scope.replace(scope);
            }
        }
    }
    .into();
}

fn field_assert(data: &Data) {
    match data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => {
            let has_scope_id_field = fields.named.iter().any(|field| {
                field
                    .ident
                    .as_ref()
                    .map(|ident| ident == "scope")
                    .unwrap_or_default()
            });
            if !has_scope_id_field {
                panic!("Struct must have a field named `scope`")
            }
        }
        _ => panic!("Only struct with a field named `scope` can be derived"),
    }
}
