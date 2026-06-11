use mpc_primitives::field::FieldElement;
use mpc_primitives::shamir::Shamir;
use mpc_primitives::ot::api::OTSession;

fn demo_field() {
    println!("=== Finite Field Arithmetic (F_97) ===");
    let a = FieldElement::new(42, 97);
    let b = FieldElement::new(55, 97);
    println!("a         = {}", a.value());
    println!("b         = {}", b.value());
    println!("a + b     = {}", (a + b).value());
    println!("a * b     = {}", (a * b).value());
    println!("a^(-1)    = {}", a.inverse().value());
    println!("a * a^(-1)= {}", (a * a.inverse()).value());
}

fn demo_shamir() {
    println!("\n=== Shamir's Secret Sharing (3-of-5) ===");
    let secret = 42u64;
    let prime  = 97u64;
    println!("Secret    = {}", secret);

    let shamir = Shamir::new(secret, 5, 3, prime);
    let shares = shamir.split();

    println!("Shares    = {:?}", shares.iter().map(|(x, y)| (x.value(), y.value())).collect::<Vec<_>>());

    let recovered = Shamir::reconstruct(&shares[0..3]);
    println!("Recovered = {}", recovered.value());
    assert_eq!(recovered.value(), secret);
}

fn demo_ot() {
    println!("\n=== 1-of-2 Oblivious Transfer ===");
    let m0: &[u8; 32] = b"this is the first  message!!!!!!";
    let m1: &[u8; 32] = b"this is the second message!!!!!!";

    for bit in [0u8, 1u8] {
        let result = OTSession::run(bit, m0, m1);
        let expected = if bit == 0 { m0 } else { m1 };
        println!("bit={} => decrypted: {:?}", bit, std::str::from_utf8(&result.decrypted).unwrap());
        assert_eq!(&result.decrypted, expected);
    }
}

fn main() {
    demo_field();
    demo_shamir();
    demo_ot();
    println!("\nAll demos passed.");
}