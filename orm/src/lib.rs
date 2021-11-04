// ORM Macros for sqlx and mongodb
// Writed by me
// It shoukd be very nicer than what it is now
// TODO: write heavy docs for what your code exactly doing
// Then you will learn macros a lot

use proc_macro::{self, TokenStream};
use quote::{format_ident, quote, ToTokens};
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
                            col_describe = format!(
                                "{} INTEGER,",
                                &f.ident.clone().expect("kklllll").to_string()
                            );
                        }
                        _ => {
                            col_describe = format!(
                                "{} TEXT,",
                                &f.ident.clone().expect("ident.clone()").to_string()
                            )
                        }
                    }

                    for attr in f.attrs.iter() {
                        let attr_name = attr
                            .parse_meta()
                            .expect("parse_meta()")
                            .path()
                            .get_ident()
                            .expect("get_ident()")
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

            pub async fn save(&self, pool: &Pool<sqlx::Any>) -> bool {
                match sqlx::query(#q_save)
                    .#binds.execute(pool).await {
                        Ok(_)=> true,
                        Err(err)=>{
                            // Duplicated shouldn't panic
                            if err.as_database_error().unwrap().code().unwrap() == "23000"
                                || err.as_database_error().unwrap().code().unwrap() == "2067"
                            {
                                "Exists!".warn();
                                false
                            } else {
                                panic!(
                                    "{} {}",
                                    err.as_database_error().unwrap().code().unwrap(),
                                    err
                                );
                            }
                        }
                }

            }

            pub async fn fetch_all(pool: &Pool<sqlx::Any>, condition: String) -> Vec<#ident> {
                let query = format!("{} {};",#q_select, condition);
                sqlx::query_as::<_, #ident>(&query)
                    .fetch_all(pool).await.expect("TODO")//TODO
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

///////////////////////////////////
//////////////////////////////////
/////////////////////////////////

#[allow(unused_assignments)]
#[proc_macro_derive(mongorm, attributes(rel))]
pub fn mongorm(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input);
    let ident_str = format!("{}s", ident).to_lowercase();
    let mut id = quote! {};
    let mut append = quote! {};
    let mut or = quote! {};
    let mut start = quote! {};
    let mut update = quote! {};
    let mut args = quote! {};
    let mut insert = quote! {};
    let mut arg_type = quote! {};
    let mut fields = quote! {};

    match data {
        syn::Data::Struct(s) => match s.fields {
            syn::Fields::Named(FieldsNamed { named, .. }) => {
                // arg_type
                let bind_names = named
                    .iter()
                    .filter(|f| match &f.ty {
                        Type::Path(type_path) => {
                            type_path.clone().into_token_stream().to_string() == "String"
                        }
                        _ => false,
                    })
                    .map(|f| {
                        let name = f.ident.as_ref().unwrap();
                        quote! {#name: String}
                    });
                arg_type = quote! {#(#bind_names),*};

                // id => "name":self.name.clone()
                let bind_names = named
                    .iter()
                    .filter(|f| match &f.ty {
                        Type::Path(type_path) => {
                            type_path.clone().into_token_stream().to_string() == "String"
                        }
                        _ => false,
                    })
                    .map(|f| {
                        let name = f.ident.as_ref().unwrap();
                        if args.is_empty() {
                            args = quote! {self.#name.clone(),};
                        }
                        fields = quote! {#fields #name: #name,};

                        let name_str = format!("{}", name.clone());
                        quote! { #name_str:self.#name.clone()}
                    });
                id = quote! {#(#bind_names),*};

                // append
                let bind_names = named
                    .iter()
                    .filter(|f| match &f.ty {
                        Type::Path(type_path) => {
                            type_path.path.segments.first().unwrap().ident.to_string() == "Vec"
                        }
                        _ => false,
                    })
                    .map(|f| {
                        let ty = match &f.ty {
                            Type::Path(type_path) => {
                                match type_path.path.segments.first().unwrap().arguments.clone() {
                                    syn::PathArguments::AngleBracketed(t) => {
                                        t.args.first().unwrap().clone().to_token_stream()
                                    }

                                    _ => panic!(),
                                }
                            }
                            _ => panic!(),
                        };

                        let name = f.ident.as_ref().unwrap();
                        fields = quote! {#fields #name: vec![],};
                        // if ty.to_string() != "String" {
                        //     insert = quote! {#insert #ty::insert(#args options.clone()).await;};
                        // }

                        // let name_str = format!("{}", name.clone());

                        quote! {
                            self.#name.append(&mut doc.#name);
                            self.#name.sort();
                            self.#name.dedup();
                        }
                    });
                append = quote! {#(#bind_names) *};

                // or
                let bind_names = named
                    .iter()
                    .filter(|f| match &f.ty {
                        Type::Path(type_path) => {
                            type_path.path.segments.first().unwrap().ident.to_string() == "Option"
                        }
                        _ => false,
                    })
                    .map(|f| {
                        let name = &f.ident.to_token_stream().to_string();
                        let date = match &f.ty {
                            Type::Path(type_path) => {
                                match type_path.path.segments.first().unwrap().arguments.clone() {
                                    syn::PathArguments::AngleBracketed(t) => t
                                        .args
                                        .first()
                                        .unwrap()
                                        .clone()
                                        .to_token_stream()
                                        .to_string(),

                                    _ => panic!(),
                                }
                            }
                            _ => panic!(),
                        };

                        // For ProgramPlatform => platform
                        let ident = &ident.to_token_stream().to_string().to_lowercase();
                        let mut name = name.to_lowercase().replace(ident, "");

                        // DateTime
                        if date == "DateTime" && name.contains("update") {
                            if name.contains("update") {
                                update = quote! {self.update = Some(DateTime::now());};
                            } else if name.contains("start") {
                                start = quote! {else {self.started_at = Some(DateTime::now());}}
                            }
                            let field_ident = &f.ident;
                            fields = quote! {#fields #field_ident: Some(DateTime::now()),};
                            quote! {}
                        } else {
                            if name == "type" {
                                name = "ty".to_string();
                            }
                            let name = format_ident!("{}", name);
                            // let name: proc_macro2::TokenStream = name.parse().unwrap();

                            fields = quote! {#fields #name: None,};

                            quote! {self.#name = self.#name.or(doc.#name); }
                        }
                    });
                or = quote! {#(#bind_names) *};

                // insert
                named.iter().for_each(|f| {
                    for attr in f.attrs.iter() {
                        if attr.path.is_ident("rel") {
                            let token: syn::LitStr = attr.parse_args().unwrap();
                            // panic!("{:#?}", token.value());
                            let token = format_ident!("{}", token.value());
                            // panic!("{}", token);
                            let fident = f.ident.as_ref().unwrap().to_token_stream();
                            if let Type::Path(type_path) = &f.ty {
                                if type_path.path.segments.first().unwrap().ident.to_string()
                                    == "Vec"
                                {
                                    insert = quote! {#insert
                                        for s in &self.#fident {
                                            #token::insert(s.to_string(), #args).await;
                                        }
                                    };
                                }
                                if type_path.path.segments.first().unwrap().ident.to_string()
                                    == "Option"
                                {
                                    insert = quote! {#insert
                                        if let Some(a)= self.#fident{
                                            #token::insert(a, #args).await;
                                        }
                                    };
                                }
                            }
                        }
                    }
                });
                // if !f.ident.
                // Host::insert()

                // // ?, ?
                // let q_marks = named.iter().map(|_| quote! {?});
                // values = format!("{}", quote! {#(#q_marks), *});

                // // col_name, col_name
                // let idents = named.iter().map(|f| &f.ident);
                // columns = format!("{}", quote! {#(#idents), *});

                // // self.col , self.col
                // let fmt_names = named.iter().map(|f| {
                //     fmt_brac.push_str("{} ");
                //     let name = &f.ident;
                //     quote! {self.#name}
                // });
                // fmt = quote! {#(#fmt_names),*};
            }
            _ => panic!("unnamed fields"),
        },
        _ => panic!("Only for structs"),
    };

    if arg_type.to_string().matches(":").count() == 1 {
        arg_type = quote! {#arg_type, fake: String,};
    }

    // Implemention
    let output = quote! {

        impl #ident {
            pub async fn update(mut self) {

                // Set options for update query
                let options = mongodb::options::UpdateOptions::builder()
                    .upsert(true)
                    .build();

                // Query for finding document
                let query = doc! {#id}; // #id

                // Define collections
                let collection = get_db().await.collection::<Self>(#ident_str);

                // Find document
                let cursor = collection.find_one(query.clone(), None).await.unwrap();

                // Edit existed fields
                if let Some(mut doc) = cursor {

                    // appends
                    #append

                    // self.started_at = doc.started_at;

                    // or
                    #or
                    // self.platform = self.platform.or(doc.platform);
                    // self.handle = self.handle.or(doc.handle);
                    // self.ty = self.ty.or(doc.ty);
                    // self.url = self.url.or(doc.url);
                    // self.icon = self.icon.or(doc.icon);
                    // self.bounty = self.bounty.or(doc.bounty);
                    // self.state = self.state.or(doc.state);
                }  #start

                // Update document
                // update
                // self.update = Some(DateTime::now());
                #update
                let doc = mongodb::bson::to_document(&self).unwrap();
                let doc = doc! {"$set":doc};
                collection
                    .update_one(query, doc, options.clone())
                    .await
                    .unwrap();


                #insert
                // Insert new things
                // for s in &self.scopes {
                //     Scope {
                //         asset: s.clone(),
                //         ty: None,
                //         eligible_bounty: None,
                //         severity: None,
                //         program: Some(self.name.clone()),
                //         subs: vec![],
                //         update: Some(DateTime::now()),
                //     }
                //     .update(options.clone())
                //     .await;
                // }
            }

            pub async fn insert(#arg_type){
                Self{
                    #fields
                }.update().await;
            }

            pub async fn find(filter: Option<String>,limit: Option<String>,sort: Option<String>) -> Vec<Self> {

                let filter: Document = serde_json::from_str(&filter.unwrap_or("{}".to_string()).replace("'", "\"")).expect("filter");
                let limit= limit.unwrap_or("0".to_string()).parse::<i64>().expect("limit");
                let sort: Document = serde_json::from_str(&sort.unwrap_or("{}".to_string())).expect("sort");

                let find_options = FindOptions::builder().limit(limit).sort(sort).build();

                let cursor = get_db()
                    .await
                    .collection::<Self>(#ident_str)
                    .find(filter, find_options)
                    .await
                    .unwrap();

                cursor.try_collect::<Vec<Self>>().await.unwrap()
            }
        }

    };

    output.into()
}
