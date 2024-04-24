// the first version of machine learning data: only money flow 
use libsofl_core::engine::{
    inspector::EvmInspector,
    state::BcState,
    types::{
        Address, Bytes, CreateInputs, EVMData, Gas, Inspector,
        InstructionResult, U256, B256,
    },
};
use revm::interpreter::{CallScheme, CallInputs};

#[derive(Default)]
pub struct ExtractMoneyFlowinspector {
    pub mf_index: usize,
    // (index starting from 1, 
    //     token transfer from address, 
    //     token transfer to address, 
    //     token address, 
    //     token amount without decimals, 
    //     tokenId if nft token) 
    pub moneys: Vec<(usize, String, String, String, String, String)>, 
}


impl<BS: BcState> Inspector<BS> for ExtractMoneyFlowinspector {
    // money flow from eth transfer
    fn call (
        &mut self,
        _evm_data: &mut EVMData<'_, BS>,
        inputs: &mut CallInputs,
    ) -> (InstructionResult, Gas, Bytes) {
        // create a transferlog only when call
        if !inputs.transfer.value.is_zero() && inputs.context.scheme == CallScheme::Call {
            self.mf_index += 1;
            self.moneys.push((self.mf_index, inputs.transfer.source.to_string().to_lowercase(), inputs.transfer.target.to_string().to_lowercase(), "ETH".to_string(), inputs.transfer.value.to_string(), "".to_string()));
        }
        (InstructionResult::Continue, Gas::new(0), Bytes::new())
    }

    // money flow from eth transfer
    fn create(
        &mut self,
        evm_data: &mut EVMData<'_, BS>,
        inputs: &mut CreateInputs,
    ) -> (InstructionResult, Option<Address>, Gas, Bytes) {
        if !inputs.value.is_zero() {
            let nonce = evm_data.journaled_state.account(inputs.caller).info.nonce;
            let addr = inputs.created_address(nonce);
            self.mf_index += 1;
            self.moneys.push((self.mf_index, inputs.caller.to_string().to_lowercase(), addr.to_string().to_lowercase(), "ETH".to_string(), inputs.value.to_string(), "".to_string()));
        }
        (InstructionResult::Continue, None, Gas::new(inputs.gas_limit), Bytes::default())
    }

    // money flow from eth transfer
    fn selfdestruct(
        &mut self,
        contract: Address,
        target: Address,
        value: U256,
    ) {
        if !value.is_zero() {
            self.mf_index += 1;
            self.moneys.push((self.mf_index, contract.to_string().to_lowercase(), target.to_string().to_lowercase(), "ETH".to_string(), value.to_string(), "".to_string()));
        }
    }

    // money flow
    fn log(
        &mut self,
        _evm_data: &mut EVMData<'_, BS>,
        address: &Address,
        topics: &[B256],
        data: &Bytes,
    ) {
        // erc20 and erc777
        if topics.len() == 3 && topics[0].to_string() == "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef" {
            self.mf_index += 1;
            self.moneys.push((self.mf_index, topics[1].to_string().to_lowercase(), topics[2].to_string().to_lowercase(), (*address).to_string().to_lowercase(), data.to_string(), "".to_string()));
        // erc721 -> token id is hexstring
        } else if topics.len() == 4 && topics[0].to_string() == "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef" {
            self.mf_index += 1;
            let token_id = "0x".to_string() + topics[3].to_string().trim_start_matches("0x").trim_start_matches('0');
            self.moneys.push((self.mf_index, topics[1].to_string().to_lowercase(), topics[2].to_string().to_lowercase(), (*address).to_string().to_lowercase(), "1".to_string(), token_id));
        // erc1155: transferSingle -> token id is hexstring
        } else if topics.len() == 4 && topics[0].to_string() == "0xc3d58168c5ae7397731d063d5bbf3d657854427343f4c083240f7aacaa2d0f62" {
            self.mf_index += 1;
            // if value == "" -> erc1155
            self.moneys.push((self.mf_index, topics[2].to_string().to_lowercase(), topics[3].to_string().to_lowercase(), (*address).to_string().to_lowercase(), "".to_string(), data.to_string()));
        // erc1155: transferBatch -> token id is hexstring
        } else if topics.len() == 4 && topics[0].to_string() == "0x4a39dc06d4c0dbc64b70af90fd698a233a518aa5d07e595d983b8c0526c8f7fb" {
            let transfers: usize = (data.to_string().len() - 258) / 128;
            if transfers == 0 {
                self.mf_index += 1;
                let transfer_data = "0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
                self.moneys.push((self.mf_index, topics[2].to_string().to_lowercase(), topics[3].to_string().to_lowercase(), (*address).to_string().to_lowercase(), "".to_string(), transfer_data.to_string()));
            } else {
                let transfer_len = usize::from_str_radix(&data.to_string()[130..194], 16).unwrap();
                println!("data_len {}: {} {}", data.to_string().len(), transfer_len, transfers);
                for i in 0..transfers {
                    self.mf_index += 1;
                    // construct the data == "0x" + 64 length of id + 64 length of value in hex string
                    let cur_id = 194 + i * 64;
                    let cur_val = 194 + 64 * transfers + (i+1) * 64;
                    let transfer_data = "0x".to_string() + &data.to_string()[cur_id..(cur_id+64)].to_string() + &data.to_string()[cur_val..(cur_val+64)].to_string();
                    // if value == "" -> erc1155
                    self.moneys.push((self.mf_index, topics[2].to_string().to_lowercase(), topics[3].to_string().to_lowercase(), (*address).to_string().to_lowercase(), "".to_string(), transfer_data.to_string()));
                }
            }
        }        
    }
}

impl<BS: BcState> EvmInspector<BS> for ExtractMoneyFlowinspector {}