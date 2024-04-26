# Forta-Data
This is a Github repo for generating customized data for transactions, such as money flow graph and function call graph.

## Preparations
You need to have a rust-based archive node and set the local node path *datadir* in *scripts/config.toml*. 

## LibSOFL
This repo uses the library for rust-based node developed in the [LibSOFL repo](https://github.com/Troublor/LibSOFL.git) and make some modifications such as developing a customized inspector --- *LibSOFL/crates/knowledge/index/src/inspectors/extract_mf_and_fc_and_op.rs*.

## Scripts
There is an example in the main.rs. Execute command *cargo run main.rs*. 

