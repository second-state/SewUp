use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_derive::Deserialize;

#[derive(Clone)]
pub struct Raw([u8; 32]);

#[derive(Deserialize, Debug, PartialEq)]
struct RawHelper {
    e01: u8,
    e02: u8,
    e03: u8,
    e04: u8,
    e05: u8,
    e06: u8,
    e07: u8,
    e08: u8,
    e09: u8,
    e10: u8,
    e11: u8,
    e12: u8,
    e13: u8,
    e14: u8,
    e15: u8,
    e16: u8,
    e17: u8,
    e18: u8,
    e19: u8,
    e20: u8,
    e21: u8,
    e22: u8,
    e23: u8,
    e24: u8,
    e25: u8,
    e26: u8,
    e27: u8,
    e28: u8,
    e29: u8,
    e30: u8,
    e31: u8,
    e32: u8,
}

impl Serialize for Raw {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Raw {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Deserialize::deserialize(deserializer).map(
            |RawHelper {
                 e01,
                 e02,
                 e03,
                 e04,
                 e05,
                 e06,
                 e07,
                 e08,
                 e09,
                 e10,
                 e11,
                 e12,
                 e13,
                 e14,
                 e15,
                 e16,
                 e17,
                 e18,
                 e19,
                 e20,
                 e21,
                 e22,
                 e23,
                 e24,
                 e25,
                 e26,
                 e27,
                 e28,
                 e29,
                 e30,
                 e31,
                 e32,
             }| {
                let bytes_32: [u8; 32] = [
                    e01, e02, e03, e04, e05, e06, e07, e08, e09, e10, e11, e12, e13, e14, e15, e16,
                    e17, e18, e19, e20, e21, e22, e23, e24, e25, e26, e27, e28, e29, e30, e31, e32,
                ];
                Raw(bytes_32)
            },
        )
    }
}

impl Default for Raw {
    fn default() -> Self {
        Raw([0; 32])
    }
}

#[test]
fn test_ser_de() {
    let mut raw = Raw::default();
    raw.0[1] = 1;
    assert!(
        raw.0
            == [
                0u8, 1u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
                0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8
            ]
    );
    let bin = bincode::serialize(&raw).expect("serialize raw fail");
    let load: Raw = bincode::deserialize(&bin).expect("load raw binary fail");
    assert_eq!(raw.0, load.0);
}

#[test]
fn test_ser_de2() {
    let mut raw = Raw::default();
    raw.0 = [
        0u8, 1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8, 8u8, 9u8, 10u8, 11u8, 12u8, 13u8, 14u8, 15u8,
        200u8, 201u8, 202u8, 203u8, 204u8, 205u8, 206u8, 207u8, 208u8, 209u8, 210u8, 211u8, 212u8,
        213u8, 214u8, 215u8,
    ];
    let bin = bincode::serialize(&raw).expect("serialize raw fail");
    let load: Raw = bincode::deserialize(&bin).expect("load raw binary fail");
    assert_eq!(raw.0, load.0);
}
