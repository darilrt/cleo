#[proc_macro]
pub fn by_two(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input_str = input.to_string();
    let output_str = format!("{} * 2", input_str);
    output_str.parse().unwrap()
}
