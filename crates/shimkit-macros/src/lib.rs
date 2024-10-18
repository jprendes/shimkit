use darling::ast::NestedMeta;
use darling::FromMeta;
use proc_macro::TokenStream;
use quote::quote;
use syn::ItemFn;

#[derive(FromMeta, PartialEq, Eq, Debug)]
enum TokioFlavor {
    MultiThread,
    CurrentThread,
}

#[derive(Debug, FromMeta)]
struct MainArgs {
    shimkit: Option<syn::Path>,
    tokio: Option<syn::Path>,
    flavor: Option<TokioFlavor>,
    worker_threads: Option<u32>,
    start_paused: Option<bool>,
}

#[proc_macro_attribute]
pub fn main(args: TokenStream, input: TokenStream) -> TokenStream {
    main_impl(args, input).unwrap_or_else(|err| err.into_compile_error().into())
}

fn main_impl(args: TokenStream, input: TokenStream) -> syn::Result<TokenStream> {
    let args = NestedMeta::parse_meta_list(args.into())?;
    let args = MainArgs::from_list(&args)?;
    let input: ItemFn = syn::parse(input)?;
    let ident = input.sig.ident.clone();

    let shimkit_path = args.shimkit.unwrap_or(syn::Path::from_string("shimkit")?);
    let tokio_path = args.tokio.unwrap_or(syn::Path::from_string("tokio")?);

    let flavor = match args.flavor.unwrap_or(TokioFlavor::CurrentThread) {
        TokioFlavor::CurrentThread => "new_current_thread",
        TokioFlavor::MultiThread => "new_multi_thread",
    };

    let flavor = syn::Ident::from_string(flavor)?;

    let start_paused = match args.start_paused {
        Some(true) => quote! { .start_paused(true) },
        _ => quote! {},
    };

    let worker_threads = match args.worker_threads {
        Some(n) => quote! { .worker_threads(#n) },
        _ => quote! {},
    };

    let tokens = if input.sig.asyncness.is_none() {
        quote! {
            fn main() -> impl ::std::process::Termination {
                #input
                #shimkit_path::run::run(#ident)
            }
        }
    } else {
        quote! {
            fn main() -> impl ::std::process::Termination {
                fn inner_main(cmd: #shimkit_path::args::Command) -> impl ::std::process::Termination {
                    #input
                    #tokio_path::runtime::Builder::#flavor()
                        #worker_threads
                        .enable_all()
                        #start_paused
                        .build()
                        .unwrap()
                        .block_on(#ident(cmd))
                }
                #shimkit_path::run::run(inner_main)
            }
        }
    };

    Ok(tokens.into())
}
