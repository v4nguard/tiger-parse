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

    let _ = TestEnum::VariantA(Box::new(VariantA));
    let _ = TestEnum::VariantB(Box::new(VariantB));
    let c = TestEnum::VariantC(Box::new(VariantC));

    assert_eq!(c.class_id(), 0x33333333);
}
