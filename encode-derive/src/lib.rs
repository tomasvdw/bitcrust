
extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;

#[proc_macro_derive(Encode, attributes(count))]
pub fn derive_encode(input: TokenStream) -> TokenStream {
    // Construct a string representation of the type definition
    let s = input.to_string();
    
    // Parse the string representation
    let ast = syn::parse_derive_input(&s).unwrap();

    // Build the impl
    let gen = impl_encode(&ast);
    
    // Return the generated impl
    gen.parse().unwrap()
}

fn impl_encode(ast: &syn::DeriveInput) -> quote::Tokens {
    let name = &ast.ident;
    let fields = match ast.body {
        syn::Body::Struct(ref data) => data.fields(),
        syn::Body::Enum(_) => panic!("#[derive(Encode)] can only be used with structs"),
    };
    let fields = generate_fields(&fields);
    quote! {
        impl Encode for #name {
            fn encode(&self, mut buff: &mut Vec<u8>) -> Result<(), ::std::io::Error> {
                #(#fields)*
                Ok(())
            }
        }
    }
    
}

fn generate_fields(fields: &[syn::Field]) -> Vec<quote::Tokens> {
    let mut result = Vec::new();
    for field in fields {
        let ident = &field.ident;
        if field.attrs.iter().any(|f| f.value.name() == "count") {
            result.push(quote!{
                VarInt::new(self.#ident.len() as u64).encode(&mut buff)?;
            });    
        }
        result.push(quote!{
            self.#ident.encode(&mut buff)?;
        });
    }
    result
}