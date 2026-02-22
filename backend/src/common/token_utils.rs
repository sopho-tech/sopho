use sha2::{Digest, Sha256};

pub fn hash_token(token: &str) -> String {
    hex::encode(Sha256::digest(token.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_token_produces_expected_output() {
        let token = "/zQEsOPDVtL7yqheVd9tMBywP3JNjzZSpg2kbhOt72R/7/ZWDVEW1D0w5a60YhJbqH4xOcYm31Qplqm5AidSvg==";
        assert_eq!(
            hash_token(token),
            "85681328ae82c001789122548d5f49964132458ed327b0bed0b11fd2293ab206"
        );
    }

    #[test]
    fn hash_token_produces_expected_output_second_token() {
        let token = "/46PtzbHo2wbI8BeVSKWUfZwdjNItyy49vG+h2wxS6I=";
        assert_eq!(
            hash_token(token),
            "c730883c2750e06a9a318c10dcef80ad2c88cf04cf26ec06bba2f4c26786de9f"
        );
    }
}
