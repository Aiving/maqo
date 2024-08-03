pub trait StrExt {
    fn strip_id(&self) -> &str;
    fn as_id(&self) -> String;
}

impl StrExt for &str {
    fn strip_id(&self) -> &str {
        self.strip_prefix("minecraft:").unwrap_or(self)
    }

    fn as_id(&self) -> String {
        if self.starts_with("minecraft:") {
            self.to_string()
        } else {
            format!("minecraft:{self}")
        }
    }
}

impl StrExt for &String {
    fn strip_id(&self) -> &str {
        self.strip_prefix("minecraft:").unwrap_or(self)
    }

    fn as_id(&self) -> String {
        if self.starts_with("minecraft:") {
            self.to_string()
        } else {
            format!("minecraft:{self}")
        }
    }
}
