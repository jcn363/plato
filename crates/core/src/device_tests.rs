use super::*;

#[test]
fn test_device_canonical_rotation() {
    let forma = Device::new("frost", "377");
    let aura_one = Device::new("daylight", "373");
    for n in 0..4 {
        assert_eq!(forma.from_canonical(forma.to_canonical(n)), n);
    }
    assert_eq!(aura_one.from_canonical(0), aura_one.startup_rotation());
    assert_eq!(
        forma.from_canonical(1) - forma.from_canonical(0),
        aura_one.from_canonical(2) - aura_one.from_canonical(3)
    );
}
