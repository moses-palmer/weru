use proc_macro::*;
use quote::quote;
use syn::{parse_macro_input, Ident, ItemStruct};

pub fn entity(attr: TokenStream, item: TokenStream) -> TokenStream {
    let table_name = parse_macro_input!(attr as Ident).to_string();
    let struct_definition = parse_macro_input!(item as ItemStruct);
    let name = struct_definition.ident.clone();
    let description_name =
        Ident::new(&format!("{}Description", name), Span::call_site().into());

    let self_ty = &struct_definition.ident;

    let (key_name, key_ty) = struct_definition
        .fields
        .iter()
        .next()
        .map(|f| (f.ident.clone().unwrap(), f.ty.clone()))
        .expect("An entity must have at least one field");
    let key_i = 1;

    let (field_name, field_ty) = struct_definition
        .fields
        .iter()
        .skip(1)
        .map(|f| (f.ident.clone().unwrap(), f.ty.clone()))
        .unzip::<_, _, Vec<_>, Vec<_>>();
    let field_i = (2..=struct_definition.fields.len()).collect::<Vec<_>>();

    quote! {
        #struct_definition

        /// A description of an entity.
        ///
        /// This struct contains all fields of the entity except the key.
        #[derive(
            Clone,
            Debug,
            Default,
            PartialEq,
            ::serde::Deserialize,
            ::serde::Serialize,
        )]
        pub struct #description_name {
            #(
                pub #field_name: Option<#field_ty>,
            )*
        }

        #[allow(unused)]
        impl #description_name {
            /// Merges this description with another.
            ///
            /// All items set in `other` will be copied to a new item.
            ///
            /// # Arguments
            /// *  `other` - Another description.
            pub fn merge(self, other: Self) -> Self {
                Self {
                    #(
                        #field_name: other.#field_name.or(self.#field_name),
                    )*
                }
            }

            /// Attempts to convert this description to an entity.
            ///
            /// Unless all fields are set, this method will return `None`.
            ///
            /// # Arguments
            /// *  `key` - The key value to use.
            pub fn entity(self, key: #key_ty) -> Option<#self_ty> {
                Some(#self_ty {
                    #key_name: key,
                    #(
                        #field_name: self.#field_name?,
                    )*
                })
            }
        }

        #[allow(unused)]
        impl #name {
            /// Creates a new item of this kind.
            pub fn new(
                #key_name: #key_ty,
                #(
                    #field_name: #field_ty,
                )*
            ) -> Self {
                Self {
                    #key_name,
                    #(
                        #field_name,
                    )*
                }
            }

            /// Lists all entities of this kind in the database.
            ///
            /// Please note that this method will read the entire table.
            ///
            /// # Arguments
            /// *  `e` - The database executor.
            ///
            /// # Panics
            /// This method will panic if any entity fails to be read.
            #[cfg(test)]
            pub async fn list<'a, E>(e: E) -> Result<
                Vec<Self>, ::weru::database::Error
            >
                where
                E: ::weru::database::sqlx::Executor<'a, Database
                    = ::weru::database::Database>,
            {
                use ::weru::database::sqlx::FromRow;
                Ok(
                    ::weru::database::sqlx::query(concat!(
                        "SELECT * from ",
                        stringify!(#table_name),
                    ))
                        .fetch_all(e)
                        .await?
                        .iter()
                        .map(Self::from_row)
                        .map(Result::unwrap)
                        .collect(),
                )
            }
        }

        impl<'r> ::weru::database::sqlx::FromRow<
            'r,
            ::weru::database::Row> for #name
        {
            // Required method
            fn from_row(
                row: &'r ::weru::database::Row,
            ) -> Result<Self, ::weru::database::sqlx::Error>
            {
                use ::weru::database::sqlx::Row;
                Ok(Self {
                    #key_name: row.try_get(&stringify!(#key_name))?,
                    #(
                        #field_name: row.try_get(&stringify!(#field_name))?,
                    )*
                })
            }
        }

        #[allow(unused)]
        #[::weru::async_trait::async_trait]
        impl ::weru::database::Entity for #self_ty {
            type Key = #key_ty;
            type Description = #description_name;

            const CREATE: &'static str = concat!(
                "INSERT INTO ", stringify!(#table_name), " (",
                    stringify!(#key_name),
                    #(", ", stringify!(#field_name)),*,
                ") ",
                "VALUES (",
                    ::weru::database::parameter!(#key_i),
                    #(", ", ::weru::database::parameter!(#field_i)),*,
                ")",
            );
            const READ: &'static str = concat!(
                "SELECT ",
                    stringify!(#key_name),
                    #(", ", stringify!(#field_name)),*,
                " ",
                "FROM ", stringify!(#table_name), " ",
                "WHERE ", stringify!(#key_name), " = ",
                    ::weru::database::parameter!(#key_i),
            );
            const UPDATE: &'static str = concat!(
                "UPDATE ", stringify!(#table_name), " ",
                "SET ",
                    stringify!(#key_name),
                    " = ",
                    ::weru::database::parameter!(#key_i),
                    #(
                        ", ",
                        stringify!(#field_name),
                        " = ",
                        ::weru::database::parameter!(#field_i)
                    ),*,
                " ",
                "WHERE ", stringify!(#key_name), " = ",
                    ::weru::database::parameter!(#key_i),
            );
            const DELETE: &'static str = concat!(
                "DELETE FROM ", stringify!(#table_name), " ",
                "WHERE ", stringify!(#key_name), " = ",
                    ::weru::database::parameter!(1),
            );

            /// Inserts this item to the database.
            ///
            /// # Arguments
            /// *  `e` - The database executor.
            async fn create<'a, E>(
                &self,
                e: E,
            ) -> Result<(), ::weru::database::Error>
            where
                E: ::weru::database::sqlx::Executor<
                    'a,
                    Database = ::weru::database::Database>
                ,
            {
                let count = ::weru::database::sqlx::query(Self::CREATE)
                    .bind(<#key_ty>::from(self.#key_name.clone()))
                    #(
                        .bind(<#field_ty>::from(self.#field_name.clone()))
                    )*
                    .execute(e)
                    .await?
                    .rows_affected();
                if count != 1 {
                    Err(::weru::database::Error::RowNotFound)
                } else {
                    Ok(())
                }
            }

            /// Loads an item of this kind from the database.
            ///
            /// If no item corresponding to the keys exists, `Ok(None)` is
            /// returned.
            ///
            /// # Arguments
            /// *  `e` - The database executor.
            async fn read<'a, E>(
                e: E,
                key: &#key_ty,
            ) -> Result<Option<Self>, ::weru::database::Error>
            where
                E: ::weru::database::sqlx::Executor<
                    'a,
                    Database = ::weru::database::Database
                >,
            {
                ::weru::database::sqlx::query_as(Self::READ)
                    .bind(key)
                    .fetch_optional(e)
                    .await
            }

            /// Updates this item in the database.
            ///
            /// # Arguments
            /// *  `e` - The database executor.
            async fn update<'a, E>(
                &self,
                e: E,
            ) -> Result<(), ::weru::database::Error>
            where
                E: ::weru::database::sqlx::Executor<
                    'a,
                    Database = ::weru::database::Database
                >,
            {
                let count = ::weru::database::sqlx::query(Self::UPDATE)
                    .bind(self.#key_name.clone())
                    #(
                        .bind(self.#field_name.clone())
                    )*
                    .bind(self.#key_name.clone())
                    .execute(e)
                    .await?
                    .rows_affected();
                if count != 1 {
                    Err(::weru::database::Error::RowNotFound)
                } else {
                    Ok(())
                }
            }

            /// The database key identifying this entity.
            fn key(&self) -> &Self::Key {
                &self.#key_name
            }

            fn merge(mut self, description: Self::Description) -> Self {
                #(
                    if let Some(#field_name) = description.#field_name {
                        self.#field_name = #field_name;
                    }
                )*
                self
            }
        }
    }
    .into()
}
