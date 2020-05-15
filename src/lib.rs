// These structs are the same struct but in different times, so version 2 knows about version 1 but not version 3 etc.

// Notes:
// 
// - We wrap the fields that we aren't sure about in `Option`s.
// - When decoding from v1 to v2 (where we are sure about a field), we just unwrap the `Option`.
// - When encoding from v2, we still need to encode `None`s for the fields we have removed,
//   so that decoding v1 structs works. We could instead use the number of bytes remaining in the
//   input as an indicator of a field's existance, but this would only work for the last field.  

mod version_1 {
    use parity_scale_codec::{Encode, Decode};

    #[derive(Debug, Clone, Encode, Decode)]
    pub struct Struct {
        pub version: u8,
        pub bytes: Option<[u8; 4]>,
        pub names: Option<Vec<String>>
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

mod version_2_a {
    use parity_scale_codec::{Encode, Decode, Input, Output, Error};

    #[derive(Debug, Clone)]
    pub struct Struct {
        pub version: u8,
        pub bytes: [u8; 4],
    }

    impl Encode for Struct {
        fn size_hint(&self) -> usize {
            self.version.size_hint() + Some(self.bytes).size_hint() + None::<Vec<String>>.size_hint()
        }

        fn encode_to<T: Output>(&self, dest: &mut T) {
            dest.push(&self.version);
            dest.push(&Some(self.bytes));
            dest.push(&None::<Vec<String>>);
        }
    }

    impl Decode for Struct {
        fn decode<I: Input>(input: &mut I) -> Result<Self, Error> {
            let version = u8::decode(input)?;
            let bytes = Option::decode(input)?.unwrap();

            Ok(Self {
                version, bytes
            })
        }
    }

    impl PartialEq<Self> for Struct {
        fn eq(&self, other: &Self) -> bool {
            self.bytes == other.bytes
        }
    }
}

mod version_2_b {
    use parity_scale_codec::{Encode, Decode, Input, Output, Error};

    #[derive(Debug, Clone)]
    pub struct Struct {
        pub version: u8,
        pub names: Vec<String>,
    }

    impl Encode for Struct {
        fn size_hint(&self) -> usize {
            self.version.size_hint() + None::<[u8; 4]>.size_hint() + Some(&self.names).size_hint()
        }

        fn encode_to<T: Output>(&self, dest: &mut T) {
            dest.push(&self.version);
            dest.push(&None::<[u8; 4]>);
            dest.push(&Some(&self.names));
        }
    }

    impl Decode for Struct {
        fn decode<I: Input>(input: &mut I) -> Result<Self, Error> {
            let version = u8::decode(input)?;
            let _ = Option::<[u8; 4]>::decode(input)?;
            let names = Option::decode(input)?.unwrap();

            Ok(Self {
                version, names
            })
        }
    }

    impl PartialEq<Self> for Struct {
        fn eq(&self, other: &Self) -> bool {
            self.names == other.names
        }
    }
}


#[cfg(test)]
use parity_scale_codec::{Encode, Decode};

#[cfg(test)]
fn test_structs() -> (version_1::Struct, version_2_a::Struct, version_2_b::Struct) {
    let version_1_struct = version_1::Struct {
        version: 1,
        bytes: Some([55; 4]),
        names: Some(vec!["hello".into(), "world".into()])
    };

    let version_2_a_struct = version_2_a::Struct {
        version: 2,
        bytes: [55; 4],
    };

    let version_2_b_struct = version_2_b::Struct {
        version: 2,
        names: vec!["hello".into(), "world".into()]
    };

    (version_1_struct, version_2_a_struct, version_2_b_struct)
}

#[test]
fn self_compatibility() {
    let (version_1_struct, version_2_a_struct, version_2_b_struct) = test_structs();
    assert_eq!(version_1_struct, version_1::Struct::decode(&mut (&*version_1_struct.encode())).unwrap());
    assert_eq!(version_2_a_struct, version_2_a::Struct::decode(&mut (&*version_2_a_struct.encode())).unwrap());
    assert_eq!(version_2_b_struct, version_2_b::Struct::decode(&mut (&*version_2_b_struct.encode())).unwrap());
}

#[test]
fn backwards_compatibility() {
    let (version_1_struct, version_2_a_struct, version_2_b_struct) = test_structs();
    assert_eq!(version_2_a_struct, version_2_a::Struct::decode(&mut (&*version_1_struct.encode())).unwrap());
    assert_eq!(version_2_b_struct, version_2_b::Struct::decode(&mut (&*version_1_struct.encode())).unwrap());
}

#[test]
fn forwards_compatibility() {
    let (version_1_struct, version_2_a_struct, version_2_b_struct) = test_structs();
    assert_eq!(version_1_struct, version_1::Struct::decode(&mut (&*version_2_a_struct.encode())).unwrap());
    assert_eq!(version_1_struct, version_1::Struct::decode(&mut (&*version_2_b_struct.encode())).unwrap());
}

#[test]
fn reencoding() {
    let (_, version_2_a_struct, version_2_b_struct) = test_structs();

    let v2_a_to_v1 = version_1::Struct::decode(&mut (&*version_2_a_struct.encode())).unwrap();
    assert_eq!(version_2_a_struct, version_2_a::Struct::decode(&mut (&*v2_a_to_v1.encode())).unwrap());

    let v2_b_to_v1 = version_1::Struct::decode(&mut (&*version_2_b_struct.encode())).unwrap();
    assert_eq!(version_2_b_struct, version_2_b::Struct::decode(&mut (&*v2_b_to_v1.encode())).unwrap());
}
