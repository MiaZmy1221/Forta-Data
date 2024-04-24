use std::collections::HashSet;

use libsofl_core::engine::{
    inspector::EvmInspector,
    state::BcState,
    types::{
        Address, Bytes, CallInputs, EVMData, Gas, Inspector, InstructionResult,
        // B256, U256, Database,
    },
};

#[derive(Default)]
pub struct ExtractInvocationInspector {
    pub invocations: HashSet<Address>, // invoked code addresses
}

impl<BS: BcState> Inspector<BS> for ExtractInvocationInspector {
    fn call(
        &mut self,
        data: &mut EVMData<'_, BS>,
        inputs: &mut CallInputs,
    ) -> (InstructionResult, Gas, Bytes) {
        let addr = inputs.context.code_address;
        let account_info =
            data.db.basic(addr).expect("failed to get account info");
        if account_info.is_none() || account_info.unwrap().is_empty_code_hash()
        {
            return (InstructionResult::Continue, Gas::new(0), Bytes::new());
        }
        self.invocations.insert(addr);
        (InstructionResult::Continue, Gas::new(0), Bytes::new())
    }
}

impl<BS: BcState> EvmInspector<BS> for ExtractInvocationInspector {}

#[cfg(test)]
mod tests {
    use alloy_dyn_abi::JsonAbiExt;
    use alloy_json_abi::Function;
    use libsofl_core::{
        conversion::ConvertTo,
        engine::{
            memory::MemoryBcState,
            types::{Address, U256},
        },
    };
    use libsofl_utils::solidity::{
        caller::HighLevelCaller,
        scripting::{deploy_contracts, SolScriptConfig},
    };

    #[test]
    fn test_extract_contract_call() {
        let mut state = MemoryBcState::fresh();
        let mut inspector = super::ExtractInvocationInspector::default();
        let addr_a = deploy_contracts(
            &mut state,
            "0.8.12",
            r#"
        contract A {
            function foo() public returns (uint) {
                return 1;
            }
        }
        "#,
            vec!["A"],
            Default::default(),
        )
        .unwrap()
        .remove(0);
        let code = format!(
            r#"
        interface A {{
            function foo() external returns (uint);
        }}
        contract B {{
            function foo() public returns (uint) {{
                A a = A({});
                return a.foo() + 1;
            }}
        }}
        "#,
            addr_a
        );
        let addr_b = deploy_contracts(
            &mut state,
            "0.8.12",
            code,
            vec!["B"],
            Default::default(),
        )
        .unwrap()
        .remove(0);

        let input = Function::parse("foo()")
            .unwrap()
            .abi_encode_input(&[])
            .unwrap();
        HighLevelCaller::default()
            .bypass_check()
            .call(&mut state, addr_b, input.cvt(), None, &mut inspector)
            .unwrap();
        let invocations = inspector.invocations;
        assert_eq!(invocations.len(), 2);
        assert!(invocations.contains(&addr_a));
        assert!(invocations.contains(&addr_b));
    }

    #[test]
    fn test_extract_non_contract_call() {
        let mut state = MemoryBcState::fresh();
        let mut inspector = super::ExtractInvocationInspector::default();

        let receiver: Address = 0x123456.cvt();
        let code = format!(
            r#"
            contract A {{
                constructor() payable {{ }}
                function foo() public returns (uint) {{
                    payable({}).transfer(1 ether);
                }}
            }}
            "#,
            receiver
        );
        let contract = deploy_contracts(
            &mut state,
            "0.8.12",
            code,
            vec!["A"],
            SolScriptConfig {
                prefund: U256::from(10).pow(U256::from(18)),
                ..Default::default()
            },
        )
        .unwrap()
        .remove(0);

        let input = Function::parse("foo()")
            .unwrap()
            .abi_encode_input(&[])
            .unwrap();

        HighLevelCaller::default()
            .bypass_check()
            .call(&mut state, contract, input.cvt(), None, &mut inspector)
            .unwrap();

        let invocations = inspector.invocations;
        assert_eq!(invocations.len(), 1);
        assert!(!invocations.contains(&receiver));
    }
}
