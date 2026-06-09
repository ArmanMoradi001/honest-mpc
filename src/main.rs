mod field;
mod shamir;
mod ot;
use api::OTSession;


fn main() {
    // Example usage
    let chosen_bit = 1;
    let m0 = 12345u64;
    let m1 = 67890u64;

    let (_, decrypted) = OTSession::run(chosen_bit, m0, m1);
    println!("Decrypted: {}", decrypted);
}
