use hello_macro::HelloMacro;
use hello_macro_derive::HelloMacro;

#[derive(HelloMacro)]
struct PanCakes;

fn main() {
    PanCakes::hello_macro()
}
