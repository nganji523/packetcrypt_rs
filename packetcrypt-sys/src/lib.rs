#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub mod difficulty;

use packetcrypt_util::util;

use std::convert::TryInto;

include!("../bindings.rs");

pub fn init() {
    sodiumoxide::init().unwrap();
}

pub struct ValidateCtx {
    raw: *mut PacketCrypt_ValidateCtx_t,
}
impl Drop for ValidateCtx {
    fn drop(&mut self) {
        unsafe {
            ValidateCtx_destroy(self.raw);
        }
    }
}
impl Default for ValidateCtx {
    fn default() -> ValidateCtx {
        ValidateCtx {
            raw: unsafe { ValidateCtx_create() },
        }
    }
}

pub fn hard_nonce(bytes: &[u8]) -> u32 {
    u32::from_le_bytes(bytes[4..8].try_into().unwrap())
}
pub fn work_bits(bytes: &[u8]) -> u32 {
    u32::from_le_bytes(bytes[8..12].try_into().unwrap())
}
pub fn parent_block_height(bytes: &[u8]) -> i32 {
    i32::from_le_bytes(bytes[12..16].try_into().unwrap())
}

#[derive(Clone, Debug)]
pub struct PacketCryptAnn {
    pub bytes: bytes::Bytes,
}
impl PacketCryptAnn {
    pub fn version(&self) -> u8 {
        self.bytes[0]
    }
    pub fn soft_nonce(&self) -> u32 {
        u32::from_le_bytes(self.bytes[..4].try_into().unwrap()) << 8
    }
    pub fn hard_nonce(&self) -> u32 {
        u32::from_le_bytes(self.bytes[4..8].try_into().unwrap())
    }
    pub fn work_bits(&self) -> u32 {
        u32::from_le_bytes(self.bytes[8..12].try_into().unwrap())
    }
    pub fn parent_block_height(&self) -> i32 {
        i32::from_le_bytes(self.bytes[12..16].try_into().unwrap())
    }
    pub fn content_hash(&self) -> &[u8] {
        &self.bytes[24..56]
    }
    pub fn signing_key(&self) -> &[u8] {
        &self.bytes[56..88]
    }
}

pub fn check_block_work(
    header: &[u8],
    low_nonce: u32,
    share_target: u32,
    anns: &[[u8; 1024]],
    coinbase: &[u8],
) -> Result<[u8; 32], &'static str> {
    let mut hap = [0_u8; 80 + 8 + (1024 * 4)];
    hap[0..80].copy_from_slice(header);
    hap[84..88].copy_from_slice(&low_nonce.to_le_bytes());
    for (ann, i) in anns.iter().zip(0..4) {
        let loc = 88 + (i * 1024);
        hap[loc..loc + 1024].copy_from_slice(ann);
    }
    let aligned_hap = util::aligned_bytes(&hap, 8);
    let aligned_coinbase = util::aligned_bytes(coinbase, 8);
    let mut hashout = [0_u8; 32];
    let res = unsafe {
        Validate_powOnly(
            aligned_hap.as_ptr() as *const PacketCrypt_HeaderAndProof_t,
            share_target,
            aligned_coinbase.as_ptr() as *const PacketCrypt_Coinbase_t,
            hashout.as_mut_ptr(),
        )
    };
    match res as i32 {
        0 => Ok(hashout),
        1 => Err("INVAL"),
        2 => Err("INVAL_ITEM4"),
        3 => Err("INSUF_POW"),
        4 => Err("SOFT_NONCE_HIGH"),
        _ => Err("UNKNOWN"),
    }
}

pub fn check_ann(
    ann: &PacketCryptAnn,
    parent_block_hash: &[u8; 32],
    vctx: &mut ValidateCtx,
) -> Result<[u8; 32], &'static str> {
    let mut hashout: [u8; 32] = [0; 32];
    let annptr = ann.bytes.as_ptr() as *const PacketCrypt_Announce_t;
    let res = unsafe {
        Validate_checkAnn(
            hashout.as_mut_ptr(),
            annptr,
            parent_block_hash.as_ptr(),
            vctx.raw,
        )
    };
    match res as i32 {
        0 => Ok(hashout),
        1 => Err("INVAL"),
        2 => Err("INVAL_ITEM4"),
        3 => Err("INSUF_POW"),
        4 => Err("SOFT_NONCE_HIGH"),
        _ => Err("UNKNOWN"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CStr;
    #[test]
    fn basic_test() {
        let res = unsafe { CStr::from_ptr(Validate_checkBlock_outToString(256)).to_str() };
        assert_eq!("Validate_checkBlock_SHARE_OK", res.unwrap());
    }
}
