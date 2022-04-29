use elrond_wasm::elrond_codec::multi_types::MultiValue2;
use elrond_wasm::types::*;
use elrond_wasm_debug::*;
use entity::features::*;
use entity::*;

mod setup;

#[test]
#[ignore]
fn it_sets_features() {
    let mut setup = setup::setup_entity(entity::contract_obj);
    let mut args = MultiValueEncoded::new();
    let feature = b"myfeature";

    args.push(MultiValue2::from((ManagedBuffer::from(feature), ManagedBuffer::from(b"true"))));

    setup
        .blockchain
        .execute_tx(&setup.owner_address, &setup.contract, &rust_biguint!(0), |sc| {
            sc.set_features_endpoint(args);

            assert_eq!(sc.feature_flag(&FeatureName(feature.into())).get(), FEATURE_ON);
        })
        .assert_ok();
}
