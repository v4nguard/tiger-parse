use tiger_parse::*;

fn main() {
    #[tiger_tag(id = 0x11111111)]
    struct VariantA;

    #[tiger_tag(id = 0x22222222)]
    struct VariantB;

    #[tiger_tag(id = 0x33333333)]
    struct VariantC;

    tiger_variant_enum! {
        [offset = 0x10]
        enum TestEnum {
            VariantA,
            VariantB,
            VariantC
        }
    }

    let a = TestEnum::VariantA(Box::new(VariantA));
    let b = TestEnum::VariantB(Box::new(VariantB));
    let c = TestEnum::VariantC(Box::new(VariantC));

    assert_eq!(a.class_id(), 0x11111111);
    assert_eq!(b.class_id(), 0x22222222);
    assert_eq!(c.class_id(), 0x33333333);
    assert_eq!(TestEnum::EXTRA_OFFSET, 16);
    println!("a.class_name = {}", a.class_name());
    println!("b.class_name = {}", b.class_name());
    println!("c.class_name = {}", c.class_name());
}
