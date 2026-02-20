use auth_crypto::{KeyProvider, SoftKeyProvider};

#[tokio::main]
async fn main() {
    println!("Testing Cryptographic Properties...");

    // Property: Signed data should verify with the same key
    let provider = SoftKeyProvider::new();
    let data = b"test_data_integrity";

    let signature: Vec<u8> = provider.sign(data).await.expect("Sign failed");
    let valid: bool = provider
        .verify(data, &signature)
        .await
        .expect("Verify failed");

    assert!(valid, "Signature verification failed for valid data");
    println!("✅ Signature Round-Trip: PASSED");

    // Property: Modified data should NOT verify
    let corrupted_data = b"test_data_corrupted";
    let valid_corrupt: bool = provider
        .verify(corrupted_data, &signature)
        .await
        .expect("Verify failed");

    assert!(
        !valid_corrupt,
        "Signature verification passed for corrupted data"
    );
    println!("✅ Integrity Check: PASSED");

    println!("\n🎉 All cryptographic property tests PASSED!");
}
