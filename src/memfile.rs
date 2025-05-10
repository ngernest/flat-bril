#![allow(dead_code, unused_imports)]

use crate::types::*;

/// Converts an `InstrStore` to a `InstrView`
fn flatten_instr_store<'a>(instr_store: InstrStore) {
    let flat_func_name = instr_store.func_name.as_slice();
    let flat_func_arg_vec: Vec<FlatFuncArg> = instr_store
        .func_args
        .into_iter()
        .map(|func_arg| func_arg.into())
        .collect();

    let flat_func_args: &[FlatFuncArg] = flat_func_arg_vec.as_slice();
    let flat_func_ret_ty: FlatType = instr_store.func_ret_ty.into();

    let flat_var_store: &[u8] = instr_store.var_store.as_slice();
    let flat_arg_idxes_vec: Vec<I32Pair> = instr_store
        .args_idxes_store
        .into_iter()
        .map(|arg_idxes| arg_idxes.into())
        .collect();
    let flat_arg_idxes_store = flat_arg_idxes_vec.as_slice();

    let flat_label_idxes_vec: Vec<I32Pair> = instr_store
        .labels_idxes_store
        .into_iter()
        .map(|lbl_idx| lbl_idx.into())
        .collect();
    let flat_label_idxes = flat_label_idxes_vec.as_slice();

    let flat_labels_store = instr_store.labels_store.as_slice();
    let flat_funcs_store = instr_store.funcs_store.as_slice();
    let flat_instrs_vec: Vec<FlatInstr> = instr_store
        .instrs
        .into_iter()
        .map(|instr| instr.into())
        .collect();
    let flat_instrs: &[FlatInstr] = &flat_instrs_vec.as_slice();
    let _instr_view = InstrView {
        func_name: flat_func_name,
        func_args: flat_func_args,
        func_ret_ty: flat_func_ret_ty,
        var_store: flat_var_store,
        arg_idxes_store: flat_arg_idxes_store,
        labels_idxes_store: flat_label_idxes,
        labels_store: flat_labels_store,
        funcs_store: flat_funcs_store,
        instrs: flat_instrs,
    };

    todo!("do something with instr_view")
}
