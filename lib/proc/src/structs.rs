#[cfg_attr(feature = "debug", derive(Debug))]
pub struct ModuleData {
    pub name: String,
    pub ident: syn::Ident,
    pub extensions: Extensions,
    pub methods: Vec<MethodData>,
}

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct MethodData {
    pub name: String,
    pub ident: syn::Ident,
    pub extensions: Extensions,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Default)]

pub struct Extensions(pub Vec<KV>);

#[cfg_attr(feature = "debug", derive(Debug))]

pub struct KV(pub String, pub String);
