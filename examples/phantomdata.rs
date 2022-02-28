use std::marker::PhantomData;




#[derive(Debug, Default, PartialEq, Eq)]
pub struct Identifier<T> {
    inner: u64,
    _tag: PhantomData<T>,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Product {
    id: Identifier<Self>,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct User {
    id: Identifier<Self>,
}


fn main() {
    let user = User::default();

    let product = Product::default();


    assert_eq!(user.id.inner, product.id.inner)
}