#![allow(dead_code, unused_imports)]
use std::io::Read;

use memmap2::{Mmap, MmapMut};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout, SizeError};
use zerocopy::{TryFromBytes, ValidityError};

use crate::flatten;
use crate::types::*;

/* -------------------------------------------------------------------------- */
/*                              Writing to buffer                             */
/* -------------------------------------------------------------------------- */

/// Mmaps a new file, returning a handle to the mmap-ed buffer
pub fn mmap_new_file(filename: &str, size: u64) -> MmapMut {
    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .truncate(true)
        .create(true)
        .open(filename)
        .unwrap();
    file.set_len(size).unwrap();
    let buffer = unsafe { MmapMut::map_mut(&file) };
    buffer.unwrap()
}

/// Writes `data` to the first `len` bytes of `buffer`,
/// (where `len = sizeof(data)`), & returns a mutable reference to
/// `buffer[len..]` (i.e. the suffix of `buffer` after the first `len` bytes).
/// - Note: Use `write_bump` when `data` has type `T` where `T` is *not* a slice
///   (i.e. when the size of `T` is not statically known)
fn write_bump<'a, T: IntoBytes + Immutable + ?Sized>(
    buffer: &'a mut [u8],
    data: &'a T,
) -> Result<&'a mut [u8], SizeError<&'a T, &'a mut [u8]>> {
    let len = size_of_val(data);
    data.write_to_prefix(buffer)?;
    Ok(&mut buffer[len..])
}

/// Writes `data` to the first `len` bytes of `buffer`
/// (where `len = sizeof(data)`), and returns a mutable reference to
/// `buffer[len..]` (i.e. the suffix of `buffer` after the first `len` bytes).
/// - Note: `write_bytes` is a specialized version of `write_bump` where
///  `data: &[u8]` (i.e. `data` is just a slice containing bytes)
fn write_bytes<'a>(buffer: &'a mut [u8], data: &[u8]) -> Option<&'a mut [u8]> {
    let len = data.len();
    buffer[0..len].copy_from_slice(data);
    Some(&mut buffer[len..])
}

/// Writes the `InstrView` to a buffer (note: `buffer` is modified in place)
fn dump_to_buffer(instr_view: &InstrView, buffer: &mut [u8]) {
    // Write the table of contents to the buffer
    let toc = instr_view.get_sizes();
    println!("original toc = {:?}", toc);

    let new_buffer = write_bump(buffer, &toc).unwrap();

    // Write the acutal contents of the `InstrView` to the buffer
    let new_buffer = write_bytes(new_buffer, instr_view.func_name).unwrap();
    let new_buffer = write_bump(new_buffer, instr_view.func_args).unwrap();
    let func_ret_ty = instr_view.func_ret_ty;
    let new_buffer = write_bump(new_buffer, &func_ret_ty).unwrap();
    let new_buffer = write_bytes(new_buffer, instr_view.var_store).unwrap();
    let new_buffer =
        write_bump(new_buffer, instr_view.arg_idxes_store).unwrap();
    let new_buffer =
        write_bump(new_buffer, instr_view.labels_idxes_store).unwrap();
    let new_buffer = write_bytes(new_buffer, instr_view.labels_store).unwrap();
    let new_buffer = write_bytes(new_buffer, instr_view.funcs_store).unwrap();
    write_bump(new_buffer, instr_view.instrs).unwrap();
}

/* -------------------------------------------------------------------------- */
/*                             Reading from buffer                            */
/* -------------------------------------------------------------------------- */

/// Consume `size.len` items from a byte slice,
/// skip the remainder of `size.capacity`
/// elements, and return the items and the rest of the slice.
fn slice_prefix<T: TryFromBytes + Immutable>(
    data: &[u8],
    size: usize,
) -> (&[T], &[u8]) {
    println!("buffer = {:p}", data);

    <[T]>::try_ref_from_prefix_with_elems(data, size)
        .expect("Deserialization error in slice_prefix")
}

/// Reads the table of contents from a prefix of the byte buffer
fn read_toc(data: &[u8]) -> (&Toc, &[u8]) {
    let (toc, remaining_buffer) = Toc::ref_from_prefix(data).unwrap();
    (toc, remaining_buffer)
}

/// Get an `InstrView` backed by the data in a byte buffer
fn get_instr_view(data: &[u8]) -> InstrView {
    let (toc, buffer) = read_toc(data);
    println!("read toc = {:?}", toc);

    let (func_name, new_buffer) = slice_prefix::<u8>(buffer, toc.func_name);
    let (func_args, new_buffer) =
        slice_prefix::<FlatFuncArg>(new_buffer, toc.func_args);

    let (func_ret_ty, new_buffer) =
        <FlatType>::try_read_from_prefix(new_buffer)
            .expect("error deserializing func_ret_ty");

    let (var_store, new_buffer) = slice_prefix::<u8>(new_buffer, toc.var_store);
    let (arg_idxes_store, new_buffer) =
        slice_prefix::<I32Pair>(new_buffer, toc.arg_idxes_store);
    let (labels_idxes_store, new_buffer) =
        slice_prefix::<I32Pair>(new_buffer, toc.labels_idxes_store);
    let (labels_store, new_buffer) =
        slice_prefix::<u8>(new_buffer, toc.labels_store);
    let (funcs_store, new_buffer) =
        slice_prefix::<u8>(new_buffer, toc.funcs_store);
    let (instrs, _) = slice_prefix::<FlatInstr>(new_buffer, toc.instrs);

    InstrView {
        func_name,
        func_args,
        func_ret_ty,
        var_store,
        arg_idxes_store,
        labels_idxes_store,
        labels_store,
        funcs_store,
        instrs,
    }
}

/* -------------------------------------------------------------------------- */
/*                                Actual logic                                */
/* -------------------------------------------------------------------------- */

pub fn main() {
    // Enable stack backtrace for debugging
    unsafe {
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    println!("memfile");

    // Read in the JSON representation of a Bril file from stdin
    let mut buffer = String::new();
    std::io::stdin()
        .read_to_string(&mut buffer)
        .expect("Unable to read from stdin");

    // Parse the JSON into serde_json's `Value` datatype
    let json: serde_json::Value =
        serde_json::from_str(&buffer).expect("Unable to parse malformed JSON");
    let functions = json["functions"]
        .as_array()
        .expect("Expected `functions` to be a JSON array");
    for func in functions {
        let instr_store = flatten::flatten_instrs(func);
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
        let flat_instrs: &[FlatInstr] = flat_instrs_vec.as_slice();
        let instr_view = InstrView {
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

        // TODO: come up with some file name and appropriate file size
        let mut mmap = mmap_new_file("fbril", 1000000000);
        dump_to_buffer(&instr_view, &mut mmap);
        println!("wrote to buffer!");

        let new_instr_view = get_instr_view(&mut mmap);
        println!("read from buffer!");

        println!("new_instr_view = {:#?}", new_instr_view);
    }
}
