/// Demangle function name, currently supports C++ and Rust
pub fn demangle_function_name(name: impl Into<String>) -> String {
    let name = name.into();

    if let Ok(name) = cpp_demangle::Symbol::new(&name) {
        if let Ok(name) = name.demangle(&cpp_demangle::DemangleOptions::default()) {
            return name;
        }
    } else if let Ok(name) = rustc_demangle::try_demangle(&name) {
        return name.to_string();
    }

    name
}
