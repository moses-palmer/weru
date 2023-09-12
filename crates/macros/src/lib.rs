use proc_macro::*;

mod database;

#[proc_macro_attribute]
pub fn database_entity(attr: TokenStream, item: TokenStream) -> TokenStream {
    self::database::entity(attr, item)
}
