use auth_crypto::kms::{KeyProvider, SoftKeyProvider};

#[tokio::test]
async fn test_rsa_signing_verification() {
    println!("Testing Cryptographic Properties...");
    
    // Property: Signed data should verify with the same key
    let provider = SoftKeyProvider::new();
    let data = b"test_data_integrity";
    
    let signature = provider.sign(data).await.expect("Sign failed");
    let valid = provider.verify(data, &signature).await.expect("Verify failed");
    
    assert!(valid, "Signature verification failed for valid data");
    println!("✅ Signature Round-Trip: PASSED");

    // Property: Modified data should NOT verify
    let corrupted_data = b"test_data_corrupted";
    let valid_corrupt = provider.verify(corrupted_data, &signature).await.expect("Verify failed");
    
    assert!(!valid_corrupt, "Signature verification passed for corrupted data");
    println!("✅ Integrity Check: PASSED");
}
