use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct Parameters {
    pub hooks: String,
    pub before_swap: bool,
    pub after_swap: bool,
    pub before_flash_loan: bool,
    pub after_flash_loan: bool,
    pub before_mint: bool,
    pub after_mint: bool,
    pub before_burn: bool,
    pub after_burn: bool,
    pub before_batch_transfer_from: bool,
    pub after_batch_transfer_from: bool,
}
