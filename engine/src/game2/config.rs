use crate::game2::data::KickTable;
use std::marker::ConstParamTy;

#[derive(ConstParamTy, PartialEq, Eq)]
pub struct ConstConfig {
  pub kicktable: KickTable,
  pub enable_180: bool,
}
