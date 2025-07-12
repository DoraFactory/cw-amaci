use crate::error::ContractError;
use crate::groth16_parser::parse_groth16_vkey;
use crate::{
    msg::Groth16VKeyType,
    state::{Groth16VkeyStr, MaciParameters},
};
use cosmwasm_std::Uint256;
use pairing_ce::bn256::Bn256;

pub struct OracleVkeyParams {
    pub process_vkey: Groth16VkeyStr,
    pub tally_vkey: Groth16VkeyStr,
}

pub fn format_vkey(groth16_vkey: &Groth16VKeyType) -> Result<Groth16VkeyStr, ContractError> {
    // Create a process_vkeys struct from the process_vkey in the message
    let groth16_vkey_formatted = Groth16VkeyStr {
        alpha_1: hex::decode(&groth16_vkey.vk_alpha1)
            .map_err(|_| ContractError::HexDecodingError {})?,
        beta_2: hex::decode(&groth16_vkey.vk_beta_2)
            .map_err(|_| ContractError::HexDecodingError {})?,
        gamma_2: hex::decode(&groth16_vkey.vk_gamma_2)
            .map_err(|_| ContractError::HexDecodingError {})?,
        delta_2: hex::decode(&groth16_vkey.vk_delta_2)
            .map_err(|_| ContractError::HexDecodingError {})?,
        ic0: hex::decode(&groth16_vkey.vk_ic0).map_err(|_| ContractError::HexDecodingError {})?,
        ic1: hex::decode(&groth16_vkey.vk_ic1).map_err(|_| ContractError::HexDecodingError {})?,
    };
    parse_groth16_vkey::<Bn256>(groth16_vkey_formatted.clone())
        .map_err(|_| ContractError::InvalidVKeyError {})?;

    Ok(groth16_vkey_formatted)
}

pub fn calculate_circuit_params(
    max_voters: u128,
    max_vote_options: u128,
) -> Result<MaciParameters, ContractError> {
    // Select the minimum circuit parameters that can meet the requirements based on max_voters and max_vote_options
    if max_voters <= 25 && max_vote_options <= 5 {
        // 2-1-1-5 scale: supports up to 25 voters, 5 options
        Ok(MaciParameters {
            state_tree_depth: Uint256::from_u128(2u128),
            int_state_tree_depth: Uint256::from_u128(1u128),
            vote_option_tree_depth: Uint256::from_u128(1u128),
            message_batch_size: Uint256::from_u128(5u128),
        })
    } else if max_voters <= 625 && max_vote_options <= 25 {
        // 4-2-2-25 scale: supports up to 625 voters, 25 options
        Ok(MaciParameters {
            state_tree_depth: Uint256::from_u128(4u128),
            int_state_tree_depth: Uint256::from_u128(2u128),
            vote_option_tree_depth: Uint256::from_u128(2u128),
            message_batch_size: Uint256::from_u128(25u128),
        })
    } else if max_voters <= 15625 && max_vote_options <= 125 {
        // 6-3-3-125 scale: supports up to 15625 voters, 125 options
        Ok(MaciParameters {
            state_tree_depth: Uint256::from_u128(6u128),
            int_state_tree_depth: Uint256::from_u128(3u128),
            vote_option_tree_depth: Uint256::from_u128(3u128),
            message_batch_size: Uint256::from_u128(125u128),
        })
    } else {
        Err(ContractError::UnsupportedCircuitSize {})
    }
}

pub fn match_oracle_vkeys(parameters: &MaciParameters) -> Result<OracleVkeyParams, ContractError> {
    if parameters.state_tree_depth == Uint256::from_u128(2)
        && parameters.int_state_tree_depth == Uint256::from_u128(1)
        && parameters.vote_option_tree_depth == Uint256::from_u128(1)
        && parameters.message_batch_size == Uint256::from_u128(5)
    {
        // vkey for 2-1-1-5 scale
        let groth16_process_vkey = Groth16VKeyType {
            vk_alpha1: "2d4d9aa7e302d9df41749d5507949d05dbea33fbb16c643b22f599a2be6df2e214bedd503c37ceb061d8ec60209fe345ce89830a19230301f076caff004d1926".to_string(),
            vk_beta_2: "0967032fcbf776d1afc985f88877f182d38480a653f2decaa9794cbc3bf3060c0e187847ad4c798374d0d6732bf501847dd68bc0e071241e0213bc7fc13db7ab304cfbd1e08a704a99f5e847d93f8c3caafddec46b7a0d379da69a4d112346a71739c1b1a457a8c7313123d24d2f9192f896b7c63eea05a9d57f06547ad0cec8".to_string(),
            vk_gamma_2: "198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c21800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa".to_string(),
            vk_delta_2: "2178a9c3805dd82071b2b28bb4c0ffc8178cad913c8c990b98b4863284dc3a5d175c0be554fc060c27c551e5e32effef015b918a0f5a2dc1b92909b8272719301c521d5f6542db5ea4775a42d32159c356a696599c1a3df011ec00559ae1c2b60d860f7e6513a7d20feaeaca401863e35a0f691dd7d30ce06d07946840de1ec8".to_string(),
            vk_ic0: "19126a54a9b6d0d415f892c246485cb2889487cf9c4a8cd88dab5e1140e1d0630d1d76ef4652df8887c9dc557aa57f25e221db7e5b2e4cf618a362bece107f5c".to_string(),
            vk_ic1: "0632e625fefc7172e8aec1070c4d32b90b6c482f6f3806773a4c55a03877c2d716cfd935eb3e3883f580c93f56adbf3a253ce3c208c52fb784f9d8fec139c617".to_string(),
        };

        let groth16_process_vkeys = format_vkey(&groth16_process_vkey)?;

        let groth16_tally_vkey = Groth16VKeyType {
            vk_alpha1: "2d4d9aa7e302d9df41749d5507949d05dbea33fbb16c643b22f599a2be6df2e214bedd503c37ceb061d8ec60209fe345ce89830a19230301f076caff004d1926".to_string(),
            vk_beta_2: "0967032fcbf776d1afc985f88877f182d38480a653f2decaa9794cbc3bf3060c0e187847ad4c798374d0d6732bf501847dd68bc0e071241e0213bc7fc13db7ab304cfbd1e08a704a99f5e847d93f8c3caafddec46b7a0d379da69a4d112346a71739c1b1a457a8c7313123d24d2f9192f896b7c63eea05a9d57f06547ad0cec8".to_string(),
            vk_gamma_2: "198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c21800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa".to_string(),
            vk_delta_2: "2e9fad39728c543c5213599111e1a44b01720c999a6785e8136c3e3b3bf8e07e248e1933d477969ca6e27cb7a74bca18cac7e3bbdf9371be5c54fe151f6376a30955609ec69b89329322a2f435b706ca248d1312c7513853a50ef37ed0f7826c25a5c57bf07789d89e538bc24017cf2722811f21480b0bb8030ed0028ecb7cd8".to_string(),
            vk_ic0: "1bc1a1a3444256469c07cd6f4d1cfd9f7c9ddce596a306e0af077ca9e9c0fe9602db2a9aecef76a9dc4c19bf88c0099b04fc75410cc9004f0966440825e3790a".to_string(),
            vk_ic1: "05b8b475f2bfedba4fa04ab1972006da9764c2c3e6fb65d6dd0aac938fd298112a560e13770b06a3f709a49fddf016331ea205fa125026993f6666eff69f4def".to_string(),
        };
        let groth16_tally_vkeys = format_vkey(&groth16_tally_vkey)?;

        let vkeys = OracleVkeyParams {
            process_vkey: groth16_process_vkeys,
            tally_vkey: groth16_tally_vkeys,
        };
        return Ok(vkeys);
    } else if parameters.state_tree_depth == Uint256::from_u128(4)
        && parameters.int_state_tree_depth == Uint256::from_u128(2)
        && parameters.vote_option_tree_depth == Uint256::from_u128(2)
        && parameters.message_batch_size == Uint256::from_u128(25)
    {
        // vkey for 4-2-2-25 scale
        let groth16_process_vkey = Groth16VKeyType {
            vk_alpha1: "2d4d9aa7e302d9df41749d5507949d05dbea33fbb16c643b22f599a2be6df2e214bedd503c37ceb061d8ec60209fe345ce89830a19230301f076caff004d1926".to_string(),
            vk_beta_2: "0967032fcbf776d1afc985f88877f182d38480a653f2decaa9794cbc3bf3060c0e187847ad4c798374d0d6732bf501847dd68bc0e071241e0213bc7fc13db7ab304cfbd1e08a704a99f5e847d93f8c3caafddec46b7a0d379da69a4d112346a71739c1b1a457a8c7313123d24d2f9192f896b7c63eea05a9d57f06547ad0cec8".to_string(),
            vk_gamma_2: "198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c21800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa".to_string(),
            vk_delta_2: "0d0fe390b9dd4d1d0f486787b6ea96765cbeaa8f00310fecc3429673c5866c081a27998596ba296f66f3f7b5e2450d1ce1bcc535c133b2e8b577ba07dc1ccb4c1895f7afb9b3168a6d628c9173157cd56ca51948cc66c129a25f80e3b665e4b12c9c50f0cc0d070978ed2fb8ce15956d67c5dc6c07c7f45f1facfb5522d7b656".to_string(),
            vk_ic0: "0ff2b22774da5c0ba94db4d759827b8c962aaf44db2649eb10407de02a40463a26497581d6d0979ad7f9057f26e048109158b0872700e2ad8447ffc9b4bf146b".to_string(),
            vk_ic1: "0a47be101a59d20641e1369c0b2b9fb839cd35ecbfbeac3866df43723b70c78d17e96303c417743d93b7726805b736f364d305036b50e4ad1b885fc41284daf5".to_string(),
        };

        let groth16_process_vkeys = format_vkey(&groth16_process_vkey)?;

        let groth16_tally_vkey = Groth16VKeyType {
            vk_alpha1: "2d4d9aa7e302d9df41749d5507949d05dbea33fbb16c643b22f599a2be6df2e214bedd503c37ceb061d8ec60209fe345ce89830a19230301f076caff004d1926".to_string(),
            vk_beta_2: "0967032fcbf776d1afc985f88877f182d38480a653f2decaa9794cbc3bf3060c0e187847ad4c798374d0d6732bf501847dd68bc0e071241e0213bc7fc13db7ab304cfbd1e08a704a99f5e847d93f8c3caafddec46b7a0d379da69a4d112346a71739c1b1a457a8c7313123d24d2f9192f896b7c63eea05a9d57f06547ad0cec8".to_string(),
            vk_gamma_2: "198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c21800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa".to_string(),
            vk_delta_2: "08e44e0876bd574d8a3411e374884eb61da7292ca52903fa96553c37311b66ce0f2f529e59b1d37e55794a575d0f87548ca0d03331c19689bc203a68c1c4bae20e9fd25a7bffaa9b7409e694a48bc0d57f42df164d4a01bd5deecffedd2d3a3125eff290efb93eaf9c578cc7ee9d00406137607b9602de02424ff413ac948690".to_string(),
            vk_ic0: "295c8e84b4b6b8de44b24f80eb5cae1df65e4877c4af8da2dbadfbfc3586dc790661b9e636f2c2a83028d11cbb7c753675481b65a5dfe32fff7a558231b3c9ef".to_string(),
            vk_ic1: "299cfb28054cde0470bd7ff280349089350226d1ca154dcf6544b2680bf3bea925026e6644668273d6066ef6766c2f561c3607c523fbbd1379c5002376ef69c3".to_string(),
        };
        let groth16_tally_vkeys = format_vkey(&groth16_tally_vkey)?;

        let vkeys = OracleVkeyParams {
            process_vkey: groth16_process_vkeys,
            tally_vkey: groth16_tally_vkeys,
        };
        return Ok(vkeys);
    } else if parameters.state_tree_depth == Uint256::from_u128(6)
        && parameters.int_state_tree_depth == Uint256::from_u128(3)
        && parameters.vote_option_tree_depth == Uint256::from_u128(3)
        && parameters.message_batch_size == Uint256::from_u128(125)
    {
        // vkey for 6-3-3-125 scale
        let groth16_process_vkey = Groth16VKeyType {
            vk_alpha1: "2d4d9aa7e302d9df41749d5507949d05dbea33fbb16c643b22f599a2be6df2e214bedd503c37ceb061d8ec60209fe345ce89830a19230301f076caff004d1926".to_string(),
            vk_beta_2: "0967032fcbf776d1afc985f88877f182d38480a653f2decaa9794cbc3bf3060c0e187847ad4c798374d0d6732bf501847dd68bc0e071241e0213bc7fc13db7ab304cfbd1e08a704a99f5e847d93f8c3caafddec46b7a0d379da69a4d112346a71739c1b1a457a8c7313123d24d2f9192f896b7c63eea05a9d57f06547ad0cec8".to_string(),
            vk_gamma_2: "198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c21800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa".to_string(),
            vk_delta_2: "057f25675851ef5a79a6d8706a43a6cd8e494cfb12c241ede46991d9174cf30605b081ff44f3ede774dab68ea9324c12308c13cb09cbb129adf94401b9134f5b16137d952fd32ab2d4243ebff4cb15d17206948ef17909ea8606886a8109bdad082f7d27e1cbf98925f055b39d1c89f9bcc4f6d92fdb920934ff5e37ba4d9b49".to_string(),
            vk_ic0: "27c937c032a18a320566e934448a0ffceea7050492a509c45a3bcb7e8ff8905d20789ada31729a833a4f595ff9f49f88adb66f2ab987de15a15deccb0e785bf4".to_string(),
            vk_ic1: "0ed2cefc103a2234dbc6bbd8634812d65332218b7589f4079b2c08eb5a4f5f63113a7f3cb53797a7f5819d7de7e3f0b2197d1c34790685a4a59af4314810420b".to_string(),
        };

        let groth16_process_vkeys = format_vkey(&groth16_process_vkey)?;

        let groth16_tally_vkey = Groth16VKeyType {
            vk_alpha1: "2d4d9aa7e302d9df41749d5507949d05dbea33fbb16c643b22f599a2be6df2e214bedd503c37ceb061d8ec60209fe345ce89830a19230301f076caff004d1926".to_string(),
            vk_beta_2: "0967032fcbf776d1afc985f88877f182d38480a653f2decaa9794cbc3bf3060c0e187847ad4c798374d0d6732bf501847dd68bc0e071241e0213bc7fc13db7ab304cfbd1e08a704a99f5e847d93f8c3caafddec46b7a0d379da69a4d112346a71739c1b1a457a8c7313123d24d2f9192f896b7c63eea05a9d57f06547ad0cec8".to_string(),
            vk_gamma_2: "198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c21800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa".to_string(),
            vk_delta_2: "2065e91c00fcc5cbc3d974cf52e24de972bdb1b4d8ded629dec20b5c904c3fa327ffe02402094795ff4d02588c8268fcad738f69eb4c732a0c98b485035e1f4913ede11b074ff143a929673e581a547717c58ce01af87d9d8b28f65f506093a61013e367b93e6782129362065840a0af9b77d7d9659a84577176e64a918d8d4c".to_string(),
            vk_ic0: "11db4a022aab89a265f06ff62aa18c74b21e913a8b23e7fce9cb46f76d1c4d9f2a7475b1eeb7be0a0dc457e6d52536ba351b621b63a7d77da75d4e773048537e".to_string(),
            vk_ic1: "0f298d235d0822ad281386abdf511853529af4c864b0cd54140facebfc1356a3059cd6d0d4b27b39e5683548fe12025e2a6b2e2724c2ca87d2008ef932ed3801".to_string(),
        };
        let groth16_tally_vkeys = format_vkey(&groth16_tally_vkey)?;

        let vkeys = OracleVkeyParams {
            process_vkey: groth16_process_vkeys,
            tally_vkey: groth16_tally_vkeys,
        };
        return Ok(vkeys);
    } else {
        return Err(ContractError::NotMatchCircuitSize {});
    }
}
