// These structs are the same struct but in different times, so version 2 knows about version 1 but not version 3 etc.

mod version_1 {
    use parity_scale_codec::{Encode, Decode};

    #[derive(Debug, Encode, Decode, Clone)]
    pub struct Struct {
        pub version: u8,
        pub string: String,
        pub value: u8
    }

    // Don't compare struct versions
    impl PartialEq<Self> for Struct {
        fn eq(&self, other: &Self) -> bool {
            self.string == other.string && self.value == other.value
        }
    }
}

mod version_2 {
    use parity_scale_codec::{Encode, Decode, Input, Error};

    #[derive(Debug, Encode, Clone)]
    pub struct Struct {
        pub version: u8,
        pub string: String,
        pub value: u8,
        pub string_2: String,
    }

    impl Decode for Struct {
        fn decode<I: Input>(input: &mut I) -> Result<Self, Error> {
            let version = u8::decode(input)?;
            let string = String::decode(input)?;
            let value = u8::decode(input)?;

            let string_2 = if version == 2 {
                String::decode(input)?
            } else {
                String::new()
            };

            Ok(Self {
                version, string, value, string_2
            })
        }
    }

    impl PartialEq<Self> for Struct {
        fn eq(&self, other: &Self) -> bool {
            let version_1_eq = self.string == other.string && self.value == other.value;

            if self.version != 2 || other.version != 2 {
                version_1_eq
            } else {
                let version_2_eq = self.string_2 == other.string_2;
                version_1_eq && version_2_eq
            }
        }
    }
}

mod version_3 {
    use parity_scale_codec::{Encode, Decode, Input, Error};

    #[derive(Debug, Encode, Clone)]
    pub struct Struct {
        pub version: u8,
        pub string: String,
        pub value: u8,
        pub value_2: u32,
    }

    impl Decode for Struct {
        fn decode<I: Input>(input: &mut I) -> Result<Self, Error> {
            let version = u8::decode(input)?;
            let string = String::decode(input)?;
            let value = u8::decode(input)?;

            let value_2 = if version == 3 {
                u32::decode(input)?
            } else {
                0
            };

            Ok(Self {
                version, string, value, value_2
            })
        }
    }

    impl PartialEq<Self> for Struct {
        fn eq(&self, other: &Self) -> bool {
            let version_1_2_eq = self.string == other.string && self.value == other.value;

            if self.version != 3 || other.version != 3 {
                version_1_2_eq
            } else {
                let version_3_eq = self.value_2 == other.value_2;
                version_1_2_eq && version_3_eq
            }
        }
    }
}

#[cfg(test)]
use parity_scale_codec::{Encode, Decode};

fn test_structs() -> (version_1::Struct, version_2::Struct, version_3::Struct) {
    let version_1_struct = version_1::Struct {
        version: 1,
        string: "abc".into(),
        value: 21
    };

    let version_2_struct = version_2::Struct {
        version: 2,
        string: "abc".into(),
        value: 21,
        string_2: "baa".into()
    };

    let version_3_struct = version_3::Struct {
        version: 3,
        string: "abc".into(),
        value: 21,
        value_2: 8,
    };

    (version_1_struct, version_2_struct, version_3_struct)
}

#[test]
fn backwards_compatibility() {
    let (version_1_struct, version_2_struct, version_3_struct) = test_structs();

    assert_eq!(version_2_struct, version_2::Struct::decode(&mut (&*version_1_struct.encode())).unwrap());
    assert_eq!(version_3_struct, version_3::Struct::decode(&mut (&*version_1_struct.encode())).unwrap());
    assert_eq!(version_3_struct, version_3::Struct::decode(&mut (&*version_2_struct.encode())).unwrap())
}

#[test]
fn forwards_compatibility() {
    let (version_1_struct, version_2_struct, version_3_struct) = test_structs();

    assert_eq!(version_1_struct, version_1::Struct::decode(&mut (&*version_2_struct.encode())).unwrap());
    assert_eq!(version_1_struct, version_1::Struct::decode(&mut (&*version_3_struct.encode())).unwrap());
    assert_eq!(version_2_struct, version_2::Struct::decode(&mut (&*version_3_struct.encode())).unwrap());
}

#[test]
fn arrays() {
    let (version_1_struct, _, version_3_struct) = test_structs();

    type V1Array = [version_1::Struct; 2];
    type V3Array = [version_3::Struct; 2];

    let version_1_array = [version_1_struct.clone(), version_1_struct.clone()];
    let version_3_array = [version_3_struct.clone(), version_3_struct.clone()];
    
    // Arrays are backwards compatible...
    assert_eq!(version_3_array, V3Array::decode(&mut (&*version_1_array.encode())).unwrap());
    // But not forwards compatible
    assert_eq!(version_1_array, V1Array::decode(&mut (&*version_3_array.encode())).unwrap());
}
