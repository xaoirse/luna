use proc_macro::{self, TokenStream};
use quote::{quote, ToTokens};
use syn::{parse_macro_input, DeriveInput, FieldsNamed, Type};

#[allow(unused_assignments)]
#[proc_macro_derive(orm, attributes(unique))]
pub fn orm(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input);

    let mut columns = String::new(); // col_name, col_name
    let mut values = String::new(); //  ?, ?
    let mut binds = quote! {}; // .bind(self.col.to_owned()).bind(self.col.to_owned())
    let mut create_cols = String::new(); // col_name TEXT, col_name INTEGER,
    let mut uniques = String::new(); // col, col
    let mut fmt = quote! {}; // self.col, self.col
    let mut fmt_brac = String::new(); // {} {}

    match data {
        syn::Data::Struct(s) => match s.fields {
            syn::Fields::Named(FieldsNamed { named, .. }) => {
                // create_cols = col_name TEXT, col_name Text,
                // uniques = col, col
                let unique_cols = named.iter().filter_map(|f| {
                    let mut filter = None;
                    let mut col_describe = String::new();

                    // https://stackoverflow.com/questions/66906261/rust-proc-macro-derive-how-do-i-check-if-a-field-is-of-a-primitive-type-like-b/66906918#66906918
                    match &f.ty {
                        Type::Path(type_path)
                            if type_path.clone().into_token_stream().to_string() == "i64" =>
                        {
                            col_describe =
                                format!("{} INTEGER,", &f.ident.clone().unwrap().to_string());
                        }
                        _ => {
                            col_describe =
                                format!("{} TEXT,", &f.ident.clone().unwrap().to_string())
                        }
                    }

                    for attr in f.attrs.iter() {
                        let attr_name = attr
                            .parse_meta()
                            .unwrap()
                            .path()
                            .get_ident()
                            .unwrap()
                            .to_string();
                        match attr_name.to_lowercase().as_str() {
                            "unique" => filter = Some(&f.ident),
                            // "integer" => col_describe = col_describe.replace("TEXT", "INTEGER"),
                            _ => (),
                        }
                    }
                    create_cols.push_str(&col_describe);
                    filter
                });
                uniques = format!("{}", quote! {#(#unique_cols), *});

                // .bind(self.col.to_owned())
                let bind_names = named.iter().map(|f| {
                    let name = &f.ident;
                    quote! {bind(self.#name.to_owned())}
                });
                binds = quote! {#(#bind_names).*};

                // ?, ?
                let q_marks = named.iter().map(|_| quote! {?});
                values = format!("{}", quote! {#(#q_marks), *});

                // col_name, col_name
                let idents = named.iter().map(|f| &f.ident);
                columns = format!("{}", quote! {#(#idents), *});

                // self.col , self.col
                let fmt_names = named.iter().map(|f| {
                    fmt_brac.push_str("{} ");
                    let name = &f.ident;
                    quote! {self.#name}
                });
                fmt = quote! {#(#fmt_names),*};
            }
            _ => panic!("unnamed fields"),
        },
        _ => panic!("Only for structs"),
    };

    // Query for creating table
    let mut q_create = format!("CREATE TABLE IF NOT EXISTS {}s ({});", ident, create_cols);
    match uniques.is_empty() {
        false => q_create = q_create.replace(",)", &format!(",UNIQUE({}))", uniques)),
        true => q_create = q_create.replace(",)", ")"),
    }

    // Query for save data
    let q_save = format!("INSERT INTO {}s ({}) VALUES ({});", ident, columns, values);

    // Query for select data
    let q_select = format!("SELECT * FROM {}s WHERE ", ident);

    // Query for update data
    let q_update = format!("UPDATE {}s SET ", ident);

    // Query for delete data
    let q_delete = format!("DELETE FROM {}s WHERE ", ident);

    // String for fmt
    let fmt_pattern = format!("{}", fmt_brac);

    // Implemention
    let output = quote! {

        impl #ident {

            pub async fn init(pool: &Pool<sqlx::Any>) -> Result<AnyQueryResult, Error> {
                sqlx::query(#q_create)
                    .execute(pool).await
            }

            pub async fn save(&self, pool: &Pool<sqlx::Any>) -> Result<AnyQueryResult, Error>{
                sqlx::query(#q_save)
                    .#binds.execute(pool).await
            }

            pub async fn fetch_all(pool: &Pool<sqlx::Any>, condition: String) -> Vec<#ident> {
                let query = format!("{} {};",#q_select, condition);
                sqlx::query_as::<_, #ident>(&query)
                    .fetch_all(pool).await.expect("rrrrrrrrr")
            }

            pub async fn fetch_optional(pool: &Pool<sqlx::Any>, condition: String) -> Option<#ident> {
                let query = format!("{} {};",#q_select, condition);
                sqlx::query_as::<_, #ident>(&query)
                    .fetch_optional(pool).await.unwrap()
            }

            pub async fn update(pool: &Pool<sqlx::Any>, set: String, condition: String) -> Result<AnyQueryResult, Error>{
                let query = format!("{} {} WHERE {};",#q_update, set, condition);
                sqlx::query(&query)
                    .execute(pool).await
            }

            pub async fn delete(pool: &Pool<sqlx::Any>, condition: String) -> Result<AnyQueryResult, Error>{
                let query = format!("{} {};",#q_delete, condition);
                sqlx::query(&query)
                    .execute(pool).await
            }

            pub fn to_string(&self) -> String{
                format!(#fmt_pattern, #fmt)
            }
        }
    };

    output.into()
}
