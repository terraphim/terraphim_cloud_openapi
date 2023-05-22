fn string_from_redis_value(v: &Value) -> Option<String> {
    match v {
        Value::Data(d) => String::from_utf8(d.to_vec()).ok(),
        _ => None,
    }
}
