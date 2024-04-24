// the third version of machine learning data: money flow, function call, and opcodes
use serde::Serialize;
use libsofl_core::engine::{
    inspector::EvmInspector,
    state::BcState,
    types::{
        Address, Bytes, CreateInputs, EVMData, Gas, Inspector,
        InstructionResult, U256, B256,
    },
};
use revm::interpreter::{opcode, CallScheme, CallInputs, CreateScheme, Interpreter};
use revm::inspectors::GasInspector;

// (index starting from 1, 
    //  opcode string,
    //  gas_remaining,)
#[derive(Default, Clone, Debug, Serialize)]
pub struct Opcode {
    pub index: usize,
    pub opcode: String,
    pub gas_remaining: u64, 
}

// (index starting from 1, 
    //     function call from address, 
    //     function call to address, 
    //     function call input,
    //     function call output,
    //     function call value,
    //     type: call/delegatecall/staticcall/callcode/create/create2/suicide 
    //     create address if any, 
    //     beneficiary address if any,
    //     opcodes) 
    // (usize, String, String, String, String, String, String, String, String, Vec<Opcode>)
#[derive(Default, Clone, Debug, Serialize)]
pub struct Trace {
    pub function_call_index: usize,
    pub from: String,
    pub to: String,
    pub input: String,
    pub output: String,
    pub value: String,
    pub call_type: String,
    pub creator: String,
    pub beneficiary: String,
    pub opcodes: Vec<Opcode>,
}

#[derive(Default, Clone, Debug)]
pub struct ExtractMFAndFCAndOPinspector {
    pub mf_index: usize,
    // (index starting from 1, 
    //     token transfer from address, 
    //     token transfer to address, 
    //     token address, 
    //     token amount without decimals, 
    //     tokenId if nft token
    //     trace stack) 
    pub moneys: Vec<(usize, String, String, String, String, String, Vec<usize>)>, 
    pub trace_index: usize,
    pub trace_stack: Vec<usize>,
    pub traces: Vec<Trace>, 
    pub gas_inspector: GasInspector,
    pub opcode_index: usize,
    pub opcodes: Vec<Opcode>,
}

impl<BS: BcState> Inspector<BS> for ExtractMFAndFCAndOPinspector {
    fn initialize_interp(&mut self, interp: &mut Interpreter<'_>, evm_data: &mut EVMData<'_, BS>) {
        self.gas_inspector.initialize_interp(interp, evm_data);
    }

    fn step(&mut self, interp: &mut Interpreter<'_>, evm_data: &mut EVMData<'_, BS>) {
        let opcode = interp.current_opcode();
        let opcode_str = opcode::OPCODE_JUMPMAP[opcode as usize];
        self.opcode_index += 1;
        let op = Opcode {
            index: self.opcode_index,
            opcode: opcode_str.unwrap().to_string(),
            gas_remaining: self.gas_inspector.gas_remaining(),
        };
        self.opcodes.push(op);
        self.gas_inspector.step(interp, evm_data);
    }

    fn step_end(&mut self, interp: &mut Interpreter, evm_data: &mut EVMData<'_, BS>) {
        self.gas_inspector.step_end(interp, evm_data);
    }


    fn call (
        &mut self,
        _evm_data: &mut EVMData<'_, BS>,
        inputs: &mut CallInputs,
    ) -> (InstructionResult, Gas, Bytes) {
        // create a type
        let mut call_type = "";
        if inputs.context.scheme == CallScheme::Call {
            call_type = "CALL";
        } else if inputs.context.scheme == CallScheme::CallCode {
            call_type = "CALLCODE";
        } else if inputs.context.scheme == CallScheme::StaticCall {
            call_type = "STATICCALL";
        } else if inputs.context.scheme == CallScheme::DelegateCall {
            call_type = "DELEGATECALL";
        }

        // put the opcodes into the last call
        if self.trace_index != 0 {
            let out_index = self.trace_stack.last().unwrap();
            let mut trace = &mut self.traces[out_index-1];
            let mut combined: Vec<Opcode> = Vec::new();
            combined.extend(trace.opcodes.clone());
            combined.extend(self.opcodes.clone());
            // modify the opcodes
            trace.opcodes = combined.clone();
            self.traces[out_index-1] = trace.clone();
            // println!("CALL out_index:{} {} {:?} {} {} opcode_INDEX:{}", out_index-1, self.trace_index, self.trace_stack, combined.len(), self.traces.len(), self.opcode_index);
            self.opcodes.clear();
        } 
        // else {
        //     println!("CALL out_index:null {} {:?} {} {} opcode_INDEX:{}", self.trace_index, self.trace_stack, self.opcodes.len(), self.traces.len(), self.opcode_index);
        // }

        // create a trace
        self.trace_index += 1;
        self.trace_stack.push(self.trace_index);
        let tempt_trace = Trace {
            function_call_index: self.trace_index,
            from: inputs.context.caller.to_string().to_lowercase(),
            to: inputs.context.address.to_string().to_lowercase(),
            input: inputs.input.to_string(),
            output: "".to_string(),
            value: inputs.transfer.value.to_string(),
            call_type: call_type.to_string(),
            creator: "".to_string(),
            beneficiary: "".to_string(),
            opcodes: self.opcodes.clone(),
        };
        self.traces.push(tempt_trace.clone());

        // create a transferlog when eth value is not 0
        if !inputs.transfer.value.is_zero() && inputs.context.scheme == CallScheme::Call {
            self.mf_index += 1;
            self.moneys.push((self.mf_index, inputs.transfer.source.to_string().to_lowercase(), inputs.transfer.target.to_string().to_lowercase(), "ETH".to_string(), inputs.transfer.value.to_string(), "".to_string(), self.trace_stack.clone()));
        }
        (InstructionResult::Continue, Gas::new(0), Bytes::new())
    }

    fn call_end(
        &mut self,
        _data: &mut EVMData<'_, BS>,
        _inputs: &CallInputs,
        gas: Gas,
        ret: InstructionResult,
        out: Bytes,
    ) -> (InstructionResult, Gas, Bytes) {
        let out_index = self.trace_stack.last().unwrap();
        let mut trace = &mut self.traces[out_index-1];
        let mut combined: Vec<Opcode> = Vec::new();
        combined.extend(trace.opcodes.clone());
        combined.extend(self.opcodes.clone());
        // modify the opcodes and outputs
        trace.output = out.clone().to_string();
        trace.opcodes = combined.clone();
        self.traces[out_index-1] = trace.clone();
        // println!("CALL_END out_index:{} {} {:?} {} {} opcode_INDEX:{}", out_index, self.trace_index, self.trace_stack, self.opcodes.len(), self.traces.len(), self.opcode_index);
        self.trace_stack.pop();
        self.opcodes.clear();

        (ret, gas, out)
    }

    // money flow from eth transfer
    fn create(
        &mut self,
        evm_data: &mut EVMData<'_, BS>,
        inputs: &mut CreateInputs,
    ) -> (InstructionResult, Option<Address>, Gas, Bytes) {
        // create a type
        let mut create_type = "";
        if inputs.scheme == CreateScheme::Create {
            create_type = "CREATE";
        } else {
            create_type = "CREATE2";
        } 

        // get the created address
        let nonce = evm_data.journaled_state.account(inputs.caller).info.nonce;
        let addr = inputs.created_address(nonce);

        // put the opcodes into the last call
        if self.trace_index != 0 {
            let out_index = self.trace_stack.last().unwrap();
            let mut trace = &mut self.traces[out_index-1];
            let mut combined: Vec<Opcode> = Vec::new();
            combined.extend(trace.opcodes.clone());
            combined.extend(self.opcodes.clone());
            // modify the opcodes
            trace.opcodes = combined.clone();
            self.traces[out_index-1] = trace.clone();
            // println!("CREATE out_index:{} {} {:?} {} {} opcode_INDEX:{}", out_index-1, self.trace_index, self.trace_stack, combined.len(), self.traces.len(), self.opcode_index);
            self.opcodes.clear();
        } 
        // else {
        //     println!("CREATE out_index:null {} {:?} {} {} opcode_INDEX:{}", self.trace_index, self.trace_stack, self.opcodes.len(), self.traces.len(), self.opcode_index);
        // }
        
        // create a trace
        self.trace_index += 1;
        self.trace_stack.push(self.trace_index);
        let tempt_trace = Trace {
            function_call_index: self.trace_index,
            from: inputs.caller.to_string().to_lowercase(),
            to: "".to_string(),
            input: inputs.init_code.to_string(),
            output: "".to_string(),
            value: inputs.value.to_string(),
            call_type: create_type.to_string(),
            creator: addr.to_string(),
            beneficiary: "".to_string(),
            opcodes: self.opcodes.clone(),
        };
        self.traces.push(tempt_trace.clone());

        // create a transferlog when eth value is not 0
        if !inputs.value.is_zero() {
            self.mf_index += 1;
            self.moneys.push((self.mf_index, inputs.caller.to_string().to_lowercase(), addr.to_string().to_lowercase(), "ETH".to_string(), inputs.value.to_string(), "".to_string(), self.trace_stack.clone()));
        }
        (InstructionResult::Continue, None, Gas::new(inputs.gas_limit), Bytes::default())
    }

    fn create_end(
        &mut self,
        _data: &mut EVMData<'_, BS>,
        _inputs: &CreateInputs,
        ret: InstructionResult,
        address: Option<Address>,
        remaining_gas: Gas,
        out: Bytes,
    ) -> (InstructionResult, Option<Address>, Gas, Bytes) {
        let out_index = self.trace_stack.last().unwrap();
        let mut trace = &mut self.traces[out_index-1];
        let mut combined: Vec<Opcode> = Vec::new();
        combined.extend(trace.opcodes.clone());
        combined.extend(self.opcodes.clone());
        // modify the opcodes and outputs
        trace.output = out.clone().to_string();
        trace.opcodes = combined.clone();
        self.traces[out_index-1] = trace.clone();
        // println!("CREATE_END out_index:{} {} {:?} {} {} opcode_INDEX:{}", out_index, self.trace_index, self.trace_stack, self.opcodes.len(), self.traces.len(), self.opcode_index);
        self.trace_stack.pop();
        self.opcodes.clear();

        (ret, address, remaining_gas, out)
    }


    // money flow from eth transfer
    fn selfdestruct(
        &mut self,
        contract: Address,
        target: Address,
        value: U256,
    ) {
        // create a trace
        self.trace_index += 1;
        self.trace_stack.push(self.trace_index);
        let empty_opcodes: Vec<Opcode> = Vec::new();
        let tempt_trace = Trace {
            function_call_index: self.trace_index,
            from: "".to_string(),
            to: contract.to_string().to_lowercase(),
            input: "".to_string(),
            output: "".to_string(),
            value: value.to_string(),
            call_type: "SELFDESTRUCT".to_string(),
            creator: "".to_string(),
            beneficiary: target.to_string(),
            opcodes: empty_opcodes,
        };
        self.traces.push(tempt_trace.clone());
        // println!("SUICIDE {} {:?} {} opcode_INDEX:{}", self.trace_index, self.trace_stack, self.traces.len(), self.opcode_index);

        // money flow
        if !value.is_zero() {
            self.mf_index += 1;
            self.moneys.push((self.mf_index, contract.to_string().to_lowercase(), target.to_string().to_lowercase(), "ETH".to_string(), value.to_string(), "".to_string(), self.trace_stack.clone()));
        }

        // pop
        self.trace_stack.pop();
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
            self.moneys.push((self.mf_index, topics[1].to_string().to_lowercase(), topics[2].to_string().to_lowercase(), (*address).to_string().to_lowercase(), data.to_string(), "".to_string(), self.trace_stack.clone()));
        // erc721 -> token id is hexstring
        } else if topics.len() == 4 && topics[0].to_string() == "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef" {
            self.mf_index += 1;
            let token_id = "0x".to_string() + topics[3].to_string().trim_start_matches("0x").trim_start_matches('0');
            self.moneys.push((self.mf_index, topics[1].to_string().to_lowercase(), topics[2].to_string().to_lowercase(), (*address).to_string().to_lowercase(), "1".to_string(), token_id, self.trace_stack.clone()));
        // erc1155: transferSingle -> token id is hexstring
        } else if topics.len() == 4 && topics[0].to_string() == "0xc3d58168c5ae7397731d063d5bbf3d657854427343f4c083240f7aacaa2d0f62" {
            self.mf_index += 1;
            // if value == "" -> erc1155
            self.moneys.push((self.mf_index, topics[2].to_string().to_lowercase(), topics[3].to_string().to_lowercase(), (*address).to_string().to_lowercase(), "".to_string(), data.to_string(), self.trace_stack.clone()));
        // erc1155: transferBatch -> token id is hexstring
        } else if topics.len() == 4 && topics[0].to_string() == "0x4a39dc06d4c0dbc64b70af90fd698a233a518aa5d07e595d983b8c0526c8f7fb" {
            let transfers: usize = (data.to_string().len() - 258) / 128;
            if transfers == 0 {
                self.mf_index += 1;
                let transfer_data = "0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
                self.moneys.push((self.mf_index, topics[2].to_string().to_lowercase(), topics[3].to_string().to_lowercase(), (*address).to_string().to_lowercase(), "".to_string(), transfer_data.to_string(), self.trace_stack.clone()));
            } else {
                let transfer_len = usize::from_str_radix(&data.to_string()[130..194], 16).unwrap();
                // println!("data_len {}: {} {}", data.to_string().len(), transfer_len, transfers);
                for i in 0..transfers {
                    self.mf_index += 1;
                    // construct the data == "0x" + 64 length of id + 64 length of value in hex string
                    let cur_id = 194 + i * 64;
                    let cur_val = 194 + 64 * transfers + (i+1) * 64;
                    let transfer_data = "0x".to_string() + &data.to_string()[cur_id..(cur_id+64)].to_string() + &data.to_string()[cur_val..(cur_val+64)].to_string();
                    // if value == "" -> erc1155
                    self.moneys.push((self.mf_index, topics[2].to_string().to_lowercase(), topics[3].to_string().to_lowercase(), (*address).to_string().to_lowercase(), "".to_string(), transfer_data.to_string(), self.trace_stack.clone()));
                }
            }
        }        
    }
}

impl<BS: BcState> EvmInspector<BS> for ExtractMFAndFCAndOPinspector {}