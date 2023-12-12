use network_tables::Value;

pub trait NTValue {
    fn try_to_string(&self) -> Option<String>;
    fn try_to_string_array(&self) -> Option<Vec<String>>;
}

impl NTValue for Value {
    fn try_to_string(&self) -> Option<String> {
        if let Self::String(string) = self {
            string.clone().into_str()
        } else {
            None
        }
    }

    fn try_to_string_array(&self) -> Option<Vec<String>> {
        if let Self::Array(array) = self {
            array.iter().map(NTValue::try_to_string).collect()
        } else {
            None
        }
    }
}
