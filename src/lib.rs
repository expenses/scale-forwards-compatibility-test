// These structs are the same struct but in different times, so version 2 knows about version 1 but not version 3 etc.

mod version_1 {
    use parity_scale_codec::{Encode, Decode, Input, Error};

    #[derive(Debug, Encode, Clone)]
    pub struct Struct {
        pub version: u8,
        pub bytes: Option<[u8; 4]>,
        pub names: Option<Vec<String>>
    }

    impl Decode for Struct {
        fn decode<I: Input>(input: &mut I) -> Result<Self, Error> {
            let version = u8::decode(input)?;
            
            let bytes = if input.remaining_len()? != Some(0) {
                Option::decode(input)?
            } else {
                None
            };

            let names = if input.remaining_len()? != Some(0) {
                Option::decode(input)?
            } else {
                None
            };

            Ok(Self {
                version, bytes, names
            })
        }
    }

    impl PartialEq<Self> for Struct {
        fn eq(&self, other: &Self) -> bool {
            let mut eq = true;

            if self.bytes.is_some() && other.bytes.is_some() {
                eq &= self.bytes == other.bytes;
            }

            if self.names.is_some() && other.names.is_some() {
                eq &= self.names == other.names;
            }

            eq
        }
    }
}

mod version_2 {
    use parity_scale_codec::{Encode, Decode, Input, Output, Error};

    #[derive(Debug, Clone)]
    pub struct Struct {
        pub version: u8,
        pub bytes: [u8; 5],
    }

    impl Encode for Struct {
        fn size_hint(&self) -> usize {
            self.version.size_hint() + Some(self.bytes).size_hint()
        }

        fn encode_to<T: Output>(&self, dest: &mut T) {
            dest.push(&self.version);
            dest.push(&Some(self.bytes));
        }
    }

    impl Decode for Struct {
        fn decode<I: Input>(input: &mut I) -> Result<Self, Error> {
            let version = u8::decode(input)?;
            let bytes = if version == 2 {
                Option::decode(input)?.unwrap()
            } else {
                let bytes = Option::<[u8; 4]>::decode(input)?.unwrap();
                [bytes[0], bytes[1], bytes[2], bytes[3], 0]
            };

            Ok(Self {
                version, bytes
            })
        }
    }

    impl PartialEq<Self> for Struct {
        fn eq(&self, other: &Self) -> bool {
            if self.version == 2 && other.version == 2 {
                self.bytes == other.bytes
            } else {
                self.bytes[..4] == other.bytes[..4]
            }
        }
    }
}

#[cfg(test)]
use parity_scale_codec::{Encode, Decode};

fn test_structs() -> (version_1::Struct, version_2::Struct) {
    let version_1_struct = version_1::Struct {
        version: 1,
        bytes: Some([55; 4]),
        names: Some(vec!["hello".into(), "world".into()])
    };

    let version_2_struct = version_2::Struct {
        version: 2,
        bytes: [55, 55, 55, 55, 55],
    };

    (version_1_struct, version_2_struct)
}

#[test]
fn self_compatibility() {
    let (version_1_struct, version_2_struct) = test_structs();
    assert_eq!(version_1_struct, version_1::Struct::decode(&mut (&*version_1_struct.encode())).unwrap());
    assert_eq!(version_2_struct, version_2::Struct::decode(&mut (&*version_2_struct.encode())).unwrap());
}

#[test]
fn backwards_compatibility() {
    let (version_1_struct, version_2_struct) = test_structs();
    assert_eq!(version_2_struct, version_2::Struct::decode(&mut (&*version_1_struct.encode())).unwrap());
}

#[test]
fn forwards_compatibility() {
    let (version_1_struct, version_2_struct) = test_structs();
    assert_eq!(version_1_struct, version_1::Struct::decode(&mut (&*version_2_struct.encode())).unwrap());
}
