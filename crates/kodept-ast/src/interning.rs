pub trait Attribute {}

pub trait Backend<A: Attribute> {
}

pub struct AttributeTable<B> {
    backend: B
}
