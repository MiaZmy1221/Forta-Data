extern crate libsofl_core;
extern crate libsofl_utils;
extern crate libsofl_reth;
extern crate libsofl_knowledge_index;
extern crate reth_provider;
extern crate serde_json;
extern crate serde;

use std::{collections::HashSet, sync::Arc};
use std::i128;
use std::convert::TryFrom;

use libsofl_core::{
    blockchain::{
        provider::{BcProvider, BcStateProvider},
        transaction::Tx,
        tx_position::TxPosition,
    },
    conversion::ConvertTo,
    engine::{
        inspector::no_inspector, state::BcState,
        transition::TransitionSpec, types::TxHash, types::BlockNumber,
    },
};

use libsofl_reth::blockchain::provider::RethProvider;
use libsofl_reth::blockchain::provider::RethBlockchainProvider;
use libsofl_reth::config::RethConfig;
use libsofl_utils::config::Config;
use reth_provider::ReceiptProvider;

use libsofl_knowledge_index::inspectors::{
    extract_creation::ExtractCreationInspector,
    extract_invocation::ExtractInvocationInspector,
    extract_money_flow::ExtractMoneyFlowinspector,
    extract_mf_and_fc::ExtractMFAndFCinspector,
    extract_mf_and_fc_and_op::ExtractMFAndFCAndOPinspector,
};

use std::fs;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

// index, from_addr, to_addr, token_addr, amount, tokenid, call_stack
// usize, String, String, String, String, String, Vec<usize>
#[derive(Default, Clone, Debug, Serialize)]
pub struct MoneyFlow {
    pub money_flow_index: usize,
    pub from: String,
    pub to: String,
    pub token: String,
    pub amount: String,
    pub tokenid: String, // if nft token, this field is not empty
    pub call_stack: Vec<usize>,
}


// #[derive(Default, Clone, Debug, Serialize)]
// pub struct TxInfo {
//     pub moneys: Vec;
//     pub function_calls: Vec<Trace>;
// }

fn replay_tx(hash: &str) {
    // intialization
    let cfg = RethConfig::must_load();
    let bp = cfg.bc_provider().unwrap();
    let tx_hash_type: TxHash = hash.cvt();
    let tx_hash_str = hash.cvt();
    let spec = TransitionSpec::from_tx_hash(&bp, tx_hash_type).unwrap();
    let tx = bp.tx(tx_hash_str).unwrap();
    let mut state = bp.bc_state_at(tx.position().unwrap()).unwrap();

    // // use a ExtractCreationInspector
    // let mut creation_insp = ExtractCreationInspector::default();
    // let r = state.transit(spec, &mut creation_insp).unwrap().pop().unwrap();
    // println!("{} {}", r.is_success(), r.logs().len());
    // let creations: Vec<(TxHash, String, bool)> = creation_insp
    //             .created
    //             .iter()
    //             .map(|(addr, destruct)| {
    //                 (tx_hash_type.clone(), ConvertTo::<String>::cvt(addr), *destruct)
    //             })
    //             .collect();
    // println!("{:?}", creations);


    // // use a ExtractCreationInspector
    // let mut invocation_insp = ExtractInvocationInspector::default();
    // let r = state.transit(spec, &mut invocation_insp).unwrap().pop().unwrap();
    // println!("{} {}", r.is_success(), r.logs().len());
    // let invocations: Vec<String> = invocation_insp
    //             .invocations
    //             .iter()
    //             .map(|addr| ConvertTo::<String>::cvt(addr))
    //             .collect();
    // println!("{:?}", invocations);

    // // use a ExtractMoneyFlowinspector
    // let mut moneyflow_insp = ExtractMoneyFlowinspector::default();
    // let r = state.transit(spec, &mut moneyflow_insp).unwrap().pop().unwrap();
    // println!("{} {}", r.is_success(), r.logs().len());
    // let moneys: Vec<(u32, String, String, String, String, String)> = moneyflow_insp
    //                 .moneys
    //                 .iter()
    //                 .map(|(index, from_addr, to_addr, token_addr, amount, tokenid)|{
    //                     (*index, 
    //                     if (&from_addr).len() < usize::try_from(66).unwrap() {
    //                         from_addr.clone().to_lowercase()
    //                     } else {
    //                         format!("0x{}", &from_addr.clone().to_lowercase()[26..66])
    //                     },
    //                     if (&to_addr).len() < usize::try_from(66).unwrap() {
    //                         to_addr.clone().to_lowercase()
    //                     } else {
    //                         format!("0x{}", &to_addr.clone().to_lowercase()[26..66])
    //                     },
    //                     token_addr.clone().to_lowercase(), 
    //                     if token_addr == "ETH" {
    //                         amount.clone()
    //                     } else {
    //                         if amount == "" {
    //                             // erc1155
    //                             // println!("erc1155 {} {} {}", &tokenid.clone().len(), &tokenid.clone()[0..66], &tokenid.clone()[67..130]);
    //                             i128::from_str_radix(&tokenid.clone()[67..130], 16).unwrap().to_string()
    //                         } else {
    //                             i128::from_str_radix(amount.clone().trim_start_matches("0x"), 16).unwrap().to_string()
    //                         }
    //                     },
    //                     if amount == "" {
    //                         // erc1155
    //                         "0x".to_string() + tokenid.clone()[2..66].to_string().trim_start_matches('0')
    //                     } else {
    //                         tokenid.clone()
    //                     })
    //                 })
    //                 .collect();
    // println!("{:?}", moneys);
    

    // // use ExtractMFAndFCinspector
    // let mut mf_and_fc_insp = ExtractMFAndFCinspector::default();
    // let r = state.transit(spec, &mut mf_and_fc_insp).unwrap().pop().unwrap();
    // println!("{} {}", r.is_success(), r.logs().len());
    // let moneys: Vec<(usize, String, String, String, String, String, Vec<usize>)> = mf_and_fc_insp
    //                 .moneys
    //                 .iter()
    //                 .map(|(index, from_addr, to_addr, token_addr, amount, tokenid, call_stack)|{
    //                     (*index, 
    //                     if (&from_addr).len() < usize::try_from(66).unwrap() {
    //                         from_addr.clone().to_lowercase()
    //                     } else {
    //                         format!("0x{}", &from_addr.clone().to_lowercase()[26..66])
    //                     },
    //                     if (&to_addr).len() < usize::try_from(66).unwrap() {
    //                         to_addr.clone().to_lowercase()
    //                     } else {
    //                         format!("0x{}", &to_addr.clone().to_lowercase()[26..66])
    //                     },
    //                     token_addr.clone().to_lowercase(), 
    //                     if token_addr == "ETH" {
    //                         amount.clone()
    //                     } else {
    //                         if amount == "" {
    //                             // erc1155
    //                             // println!("erc1155 {} {} {}", &tokenid.clone().len(), &tokenid.clone()[0..66], &tokenid.clone()[67..130]);
    //                             i128::from_str_radix(&tokenid.clone()[67..130], 16).unwrap().to_string()
    //                         } else {
    //                             i128::from_str_radix(amount.clone().trim_start_matches("0x"), 16).unwrap().to_string()
    //                         }
    //                     },
    //                     if amount == "" {
    //                         // erc1155
    //                         "0x".to_string() + tokenid.clone()[2..66].to_string().trim_start_matches('0')
    //                     } else {
    //                         tokenid.clone()
    //                     },
    //                     call_stack.clone())
    //                 })
    //                 .collect();
    // println!("{:?}", moneys);
    // println!("{:?}", mf_and_fc_insp.traces);

    // use ExtractMFAndFCAndOPinspector
    let mut mf_and_fc_and_op_insp = ExtractMFAndFCAndOPinspector::default();
    let r = state.transit(spec, &mut mf_and_fc_and_op_insp).unwrap().pop().unwrap();
    let moneys: Vec<(usize, String, String, String, String, String, Vec<usize>)> = mf_and_fc_and_op_insp
                    .moneys
                    .iter()
                    .map(|(index, from_addr, to_addr, token_addr, amount, tokenid, call_stack)|{
                        (*index, 
                        if (&from_addr).len() < usize::try_from(66).unwrap() {
                            from_addr.clone().to_lowercase()
                        } else {
                            format!("0x{}", &from_addr.clone().to_lowercase()[26..66])
                        },
                        if (&to_addr).len() < usize::try_from(66).unwrap() {
                            to_addr.clone().to_lowercase()
                        } else {
                            format!("0x{}", &to_addr.clone().to_lowercase()[26..66])
                        },
                        token_addr.clone().to_lowercase(), 
                        if token_addr == "ETH" {
                            amount.clone()
                        } else {
                            if amount == "" {
                                // erc1155
                                // println!("erc1155 {} {} {}", &tokenid.clone().len(), &tokenid.clone()[0..66], &tokenid.clone()[67..130]);
                                i128::from_str_radix(&tokenid.clone()[67..130], 16).unwrap().to_string()
                            } else {
                                i128::from_str_radix(amount.clone().trim_start_matches("0x"), 16).unwrap().to_string()
                            }
                        },
                        if amount == "" {
                            // erc1155
                            "0x".to_string() + tokenid.clone()[2..66].to_string().trim_start_matches('0')
                        } else {
                            tokenid.clone()
                        },
                        call_stack.clone())
                    })
                    .collect();
                        
    // write it into a file
    let file_path = format!("{}.txt", hash);
    if let Err(e) = fs::remove_file(&file_path) {
        if e.kind() != std::io::ErrorKind::NotFound {
            eprintln!("Error removing {}: {}", file_path, e);
        }
    }
    let json_string1 = serde_json::to_string(&moneys).expect("Failed to serialize data to JSON");
    let json_string2 = serde_json::to_string(&mf_and_fc_and_op_insp.traces).expect("Failed to serialize data to JSON");
    match fs::write(file_path.clone(), format!("{} \n {}", json_string1,json_string2)) {
        Ok(()) => println!("Result has been written to {}", file_path.clone()),
        Err(e) => eprintln!("Error writing to {}: {}", file_path.clone(), e),
    }
}

// fn main() -> io::Result<()> {
//     let mut index = 1;
//     // get the transactions and execute one by one
//     let file = File::open("./attack_transaction.txt")?;
//     let reader = io::BufReader::new(file);

//     // Create a vector to store the strings starting with "0x"
//     let mut tx_vector: Vec<String> = Vec::new();

//     // Iterate over each line in the file
//     for line in reader.lines() {
//         if let Ok(tx_list) = line {
//             // Split the line based on ","
//             let txs: Vec<&str> = tx_list.split(',').collect();
//             // Iterate over each item in the split list
//             for tx in txs {
//                 // Trim whitespace and check if the item starts with "0x"
//                 if let Some(tx_str) = tx.trim().strip_prefix("0x") {
//                     // Add the string to the vector
//                     replay_tx(&format!("0x{}", tx_str));
//                     tx_vector.push(format!("0x{}", tx_str));
//                     println!("{} {}", index, format!("0x{}", tx_str));
//                     index += 1;
//                 }
//             }
//         }
//     }
//     Ok(())
// }


// // get normal tranasctions information
// fn main() {
//     // first attack transaction in block 18476513
//     // second attack transaction in block 18523344
//     // let's get the middle 3 blocks: 18477000; 18478000; 18479000
//     let cfg = RethConfig::must_load();
//     let bp = cfg.bc_provider().unwrap();
//     let txs = bp.txs_in_block(18479000u64.cvt()).unwrap();
//     let mut index = 1;
//     for tx in txs {
//         replay_tx(&tx.hash().to_string());
//         println!("{} 18479000 {}", index, tx.hash().to_string());
//         index += 1
//     }
// }


fn main() {
    // test ExtractMoneyFlowinspector
    // erc20 and suicide and eth transfer from call
    // let hash1 = "0x32c83905db61047834f29385ff8ce8cb6f3d24f97e24e6101d8301619efee96e";
    // // erc777
    // let hash2 = "0x855b19e03d7a87b0f925f0ebdced4669f086cfd12fd242bc5a7044c7707d3842"; 
    // // erc721
    // let hash3 = "0x0cd617a3cc204b159c2a88cf64b559a46d8b9c93cd3c2d7abe7adcbb132a73d8";
    // // create
    // let hash4 = "0x0d8fc644c8178a352cd39aed1c3ae58434e422390c96f182b3fd68c53ec070e2";
    // // // error: erc11155 tokenid is too big to convert
    // // erc11155 single
    // let hash5 = "0xd61e017c1fdc7b9464eccf880db17b1c66ff87cca9def4c5de0b62f9b05c4c56";
    // // erc11155 batch
    // let hash6 = "0x8f09040707aeaa5bf1c3e98f8c867a2b416c1b3bbac10d19b7f0de8f51b15dba";
    // // erc4626: it seems this type of token emits Transfer()
    // let hash7 = "0x79f41f29b5f637012ca435e11b07cfc2e71b073698a792e30b0b1b6d3c7890d3";
    // replay_tx(hash7);

    // // test ExtractMFAndFCinspector
    // replay_tx(hash1);

    // test ExtractMFAndFCAndOPinspector
    let hash1 = "0x32c83905db61047834f29385ff8ce8cb6f3d24f97e24e6101d8301619efee96e";
    replay_tx(hash1);
}