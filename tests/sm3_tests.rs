use sm3;
use rand;


#[cfg(test)]
mod sm3_tests {

    use super::*;
    
    struct Cipher {
        clear: &'static str,
        encrypted: &'static str
    }

    const TEST_CIPHERS: &'static [Cipher] = &[
        Cipher{clear: "616263", encrypted: "66c7f0f462eeedd9d1f2d46bdc10e4e24167c4875cf2f7a2297da02b8f4ba8e0"},
    ];

    const OPENSSL_SM3: sm3::Hash = sm3::OPENSSL_SM3;
    const MY_SM3: sm3::Hash = sm3::MY_SM3;

    #[test]
    fn test_openssl_sm3() {
        for cipher in TEST_CIPHERS.iter() {
            assert_eq!(
                OPENSSL_SM3(&hex::decode(cipher.clear).unwrap()).as_ref(),
                hex::decode(cipher.encrypted).unwrap().as_slice()
            )
        }
    }

    #[test]
    fn test_my_sm3() {
        for cipher in TEST_CIPHERS.iter() {
            assert_eq!(
                MY_SM3(&hex::decode(cipher.clear).unwrap()).as_ref(),
                hex::decode(cipher.encrypted).unwrap().as_slice()
            )
        }
    }

    #[test]
    fn random_test_sm3() {
        for _ in 0..100 {
            let len = rand::random::<u16>();
            let random_bytes = (0..len).map(|_| { rand::random::<u8>() }).collect::<Vec<u8>>();
            let bytes = random_bytes.as_slice();
            let my_result = MY_SM3(bytes);
            let openssl_result = OPENSSL_SM3(bytes);
            assert_eq!(my_result.as_ref(), openssl_result.as_ref());
        }
    }
}
