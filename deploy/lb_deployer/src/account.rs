use cosmwasm_std::Addr;
use secretrs::{
    crypto::{secp256k1::SigningKey, PublicKey},
    AccountId,
};

#[derive(Debug, Clone)]
pub struct Account {
    prvk: bip32::XPrv,
    pubk: PublicKey,
}

impl Account {
    pub fn from_mnemonic(s: &str) -> Option<Account> {
        let mnemonic = bip39::Mnemonic::parse(s).ok()?;
        // empty passphrase
        Some(Account::from_seed(mnemonic.to_seed("")))
    }

    pub fn from_seed(seed: [u8; 64]) -> Account {
        let path = "m/44'/529'/0'/0/0"
            .parse()
            .expect("invalid scrt derivation path");
        let prvk =
            bip32::XPrv::derive_from_path(seed, &path).expect("private key derivation failed");
        let pubk = SigningKey::from(&prvk).public_key();
        Account { prvk, pubk }
    }

    pub fn new_random() -> Account {
        use nanorand::rand::Rng;
        let mut seed = [0; 64];
        let mut rng = nanorand::rand::ChaCha8::new();
        rng.fill_bytes(&mut seed);

        let path = "m/44'/529'/0'/0/0"
            .parse()
            .expect("invalid scrt derivation path");
        let prvk =
            bip32::XPrv::derive_from_path(seed, &path).expect("private key derivation failed");
        let pubk = SigningKey::from(&prvk).public_key();
        Account { prvk, pubk }
    }

    pub fn addr(&self) -> Addr {
        Addr::unchecked(self.id().as_ref())
    }

    pub fn signing_key(&self) -> SigningKey {
        SigningKey::from(&self.prvk)
    }

    pub fn id(&self) -> AccountId {
        self.pubk
            .account_id("secret")
            .expect("invalid public key type")
    }

    fn prv_pub_bytes(&self) -> ([u8; 32], [u8; 32]) {
        let mut secret = [0u8; 32];
        secret.clone_from_slice(&self.prvk.private_key().to_bytes());
        let secret = x25519_dalek::StaticSecret::from(secret);
        let public = x25519_dalek::PublicKey::from(&secret);

        (secret.to_bytes(), public.to_bytes())
    }

    pub fn a() -> Self {
        Account::from_mnemonic(A_MNEMONIC).unwrap()
    }

    pub fn b() -> Self {
        Account::from_mnemonic(B_MNEMONIC).unwrap()
    }

    pub fn c() -> Self {
        Account::from_mnemonic(C_MNEMONIC).unwrap()
    }

    pub fn d() -> Self {
        Account::from_mnemonic(D_MNEMONIC).unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn accounts_from_mnemonic() {
        assert_eq!(
            Account::a().addr(),
            Addr::unchecked("secret1ap26qrlp8mcq2pg6r47w43l0y8zkqm8a450s03")
        );
        assert_eq!(
            Account::b().addr(),
            Addr::unchecked("secret1fc3fzy78ttp0lwuujw7e52rhspxn8uj52zfyne")
        );
        assert_eq!(
            Account::c().addr(),
            Addr::unchecked("secret1ajz54hz8azwuy34qwy9fkjnfcrvf0dzswy0lqq")
        );
        assert_eq!(
            Account::d().addr(),
            Addr::unchecked("secret1ldjxljw7v4vk6zhyduywh04hpj0jdwxsmrlatf")
        );
    }
}

static A_MNEMONIC: &str = "grant rice replace explain federal release fix clever romance raise often wild taxi quarter soccer fiber love must tape steak together observe swap guitar";
static B_MNEMONIC: &str = "jelly shadow frog dirt dragon use armed praise universe win jungle close inmate rain oil canvas beauty pioneer chef soccer icon dizzy thunder meadow";
static C_MNEMONIC: &str = "chair love bleak wonder skirt permit say assist aunt credit roast size obtain minute throw sand usual age smart exact enough room shadow charge";
static D_MNEMONIC: &str = "word twist toast cloth movie predict advance crumble escape whale sail such angry muffin balcony keen move employ cook valve hurt glimpse breeze brick";
