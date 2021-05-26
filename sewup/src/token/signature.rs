pub const DO_BALANCE_SIGNATURE: [u8; 4] = [153, 147, 2, 26];

pub const DO_TRANSFER_SIGNATURE: [u8; 4] = [93, 53, 159, 189];

pub const NAME_SIGNATURE: [u8; 4] = [6, 253, 222, 3];

pub const SYMBOL_SIGNATURE: [u8; 4] = [149, 216, 155, 65];

pub const DECIMALS_SIGNATURE: [u8; 4] = [49, 60, 229, 103];

pub const TOTAL_SUPPLY_SIGNATURE: [u8; 4] = [24, 22, 13, 221];

pub const APPROVE_SIGNATURE: [u8; 4] = [16, 134, 169, 170];

pub const ALLOWANCE_SIGNATURE: [u8; 4] = [221, 98, 237, 62];

pub const TRANSFER_FROM_SIGNATURE: [u8; 4] = [46, 160, 223, 225];

// 0xaa174ccb mint _mint(address account, uint256 amount)
pub const MINT_SIGNATURE: [u8; 4] = [170, 23, 76, 203];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::get_function_signature;
    #[test]
    fn test_function_signature() {
        // assert_eq!(get_function_signature("doBalance(address)"), DO_BALANCE_SIGNATURE);
        // assert_eq!(get_function_signature("doTransfer(address,value)"), DO_TRANSFER_SIGNATURE);
        assert_eq!(get_function_signature("name()"), NAME_SIGNATURE);
        assert_eq!(get_function_signature("symbol()"), SYMBOL_SIGNATURE);
        assert_eq!(get_function_signature("decimals()"), DECIMALS_SIGNATURE);
        assert_eq!(
            get_function_signature("totalSupply()"),
            TOTAL_SUPPLY_SIGNATURE
        );
        assert_eq!(
            get_function_signature("approve(address,uint64)"),
            APPROVE_SIGNATURE
        );
        assert_eq!(
            get_function_signature("allowance(address,address)"),
            ALLOWANCE_SIGNATURE
        );
        assert_eq!(
            get_function_signature("transferFrom(address,address,uint64)"),
            TRANSFER_FROM_SIGNATURE
        );
        // assert_eq!(get_function_signature("mint(address,uint256)"), MINT_SIGNATURE);
    }
}
