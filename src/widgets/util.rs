use network_tables::Value;

pub trait NTValue {
    fn try_to_string(&self) -> Option<String>;
    fn try_to_string_array(&self) -> Option<Vec<String>>;
}

impl NTValue for Value {
    fn try_to_string(&self) -> Option<String> {
        if let Value::String(string) = self {
            string.clone().into_str()
        } else {
            None
        }
    }

    fn try_to_string_array(&self) -> Option<Vec<String>> {
        if let Value::Array(array) = self {
            array.iter().map(|value| value.try_to_string()).collect()
        } else {
            None
        }
    }
}
