use tiger_parse::*;

fn main() {
    #[tiger_tag(id = 0x11111111)]
    struct VariantA;

    #[tiger_tag(id = 0x22222222)]
    struct VariantB;

    #[tiger_tag(id = 0x33333333)]
    struct VariantC;

    tiger_variant_enum! {
        enum TestEnum {
            VariantA,
            VariantB,
            VariantC
        }
    }

    let a = TestEnum::VariantA(Box::new(VariantA));
    let b = TestEnum::VariantB(Box::new(VariantB));
    let c = TestEnum::VariantC(Box::new(VariantC));

    assert_eq!(c.class_id(), 0x33333333);
    println!("a.class_name = {}", a.class_name());
    println!("b.class_name = {}", b.class_name());
    println!("c.class_name = {}", c.class_name());
}
