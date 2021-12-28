impl JsonPathQuery for Box<Value> {
    fn path(self, query: &str) -> Result<Value, String> {
        let p = JsonPathInst::from_str(query)?;
        Ok(JsonPathFinder::new(self, Box::new(p)).find())
    }
}

impl JsonPathQuery for Value {
    fn path(self, query: &str) -> Result<Value, String> {
        let p = JsonPathInst::from_str(query)?;
        Ok(JsonPathFinder::new(Box::from(self), Box::new(p)).find())
    }
}
