impl serde::Serialize for ActionCompaction {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("antigravity.localharness.ActionCompaction", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ActionCompaction {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                            Ok(GeneratedField::__SkipField__)
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ActionCompaction;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.ActionCompaction")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ActionCompaction, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map_.next_key::<GeneratedField>()?.is_some() {
                    let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(ActionCompaction {
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.ActionCompaction", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ActionCreateFile {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.file_path.is_some() {
            len += 1;
        }
        if self.contents.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.ActionCreateFile", len)?;
        if let Some(v) = self.file_path.as_ref() {
            struct_ser.serialize_field("filePath", v)?;
        }
        if let Some(v) = self.contents.as_ref() {
            struct_ser.serialize_field("contents", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ActionCreateFile {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "file_path",
            "filePath",
            "contents",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            FilePath,
            Contents,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "filePath" | "file_path" => Ok(GeneratedField::FilePath),
                            "contents" => Ok(GeneratedField::Contents),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ActionCreateFile;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.ActionCreateFile")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ActionCreateFile, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut file_path__ = None;
                let mut contents__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::FilePath => {
                            if file_path__.is_some() {
                                return Err(serde::de::Error::duplicate_field("filePath"));
                            }
                            file_path__ = map_.next_value()?;
                        }
                        GeneratedField::Contents => {
                            if contents__.is_some() {
                                return Err(serde::de::Error::duplicate_field("contents"));
                            }
                            contents__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ActionCreateFile {
                    file_path: file_path__,
                    contents: contents__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.ActionCreateFile", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ActionCustomTool {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.tool_call.is_some() {
            len += 1;
        }
        if self.tool_response.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.ActionCustomTool", len)?;
        if let Some(v) = self.tool_call.as_ref() {
            struct_ser.serialize_field("toolCall", v)?;
        }
        if let Some(v) = self.tool_response.as_ref() {
            struct_ser.serialize_field("toolResponse", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ActionCustomTool {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "tool_call",
            "toolCall",
            "tool_response",
            "toolResponse",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ToolCall,
            ToolResponse,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "toolCall" | "tool_call" => Ok(GeneratedField::ToolCall),
                            "toolResponse" | "tool_response" => Ok(GeneratedField::ToolResponse),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ActionCustomTool;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.ActionCustomTool")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ActionCustomTool, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut tool_call__ = None;
                let mut tool_response__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ToolCall => {
                            if tool_call__.is_some() {
                                return Err(serde::de::Error::duplicate_field("toolCall"));
                            }
                            tool_call__ = map_.next_value()?;
                        }
                        GeneratedField::ToolResponse => {
                            if tool_response__.is_some() {
                                return Err(serde::de::Error::duplicate_field("toolResponse"));
                            }
                            tool_response__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ActionCustomTool {
                    tool_call: tool_call__,
                    tool_response: tool_response__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.ActionCustomTool", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ActionEditFile {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.file_path.is_some() {
            len += 1;
        }
        if !self.diff_block.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.ActionEditFile", len)?;
        if let Some(v) = self.file_path.as_ref() {
            struct_ser.serialize_field("filePath", v)?;
        }
        if !self.diff_block.is_empty() {
            struct_ser.serialize_field("diffBlock", &self.diff_block)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ActionEditFile {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "file_path",
            "filePath",
            "diff_block",
            "diffBlock",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            FilePath,
            DiffBlock,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "filePath" | "file_path" => Ok(GeneratedField::FilePath),
                            "diffBlock" | "diff_block" => Ok(GeneratedField::DiffBlock),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ActionEditFile;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.ActionEditFile")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ActionEditFile, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut file_path__ = None;
                let mut diff_block__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::FilePath => {
                            if file_path__.is_some() {
                                return Err(serde::de::Error::duplicate_field("filePath"));
                            }
                            file_path__ = map_.next_value()?;
                        }
                        GeneratedField::DiffBlock => {
                            if diff_block__.is_some() {
                                return Err(serde::de::Error::duplicate_field("diffBlock"));
                            }
                            diff_block__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ActionEditFile {
                    file_path: file_path__,
                    diff_block: diff_block__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.ActionEditFile", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for action_edit_file::DiffBlock {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.start_line.is_some() {
            len += 1;
        }
        if self.end_line.is_some() {
            len += 1;
        }
        if !self.lines.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.ActionEditFile.DiffBlock", len)?;
        if let Some(v) = self.start_line.as_ref() {
            struct_ser.serialize_field("startLine", v)?;
        }
        if let Some(v) = self.end_line.as_ref() {
            struct_ser.serialize_field("endLine", v)?;
        }
        if !self.lines.is_empty() {
            struct_ser.serialize_field("lines", &self.lines)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for action_edit_file::DiffBlock {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "start_line",
            "startLine",
            "end_line",
            "endLine",
            "lines",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            StartLine,
            EndLine,
            Lines,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "startLine" | "start_line" => Ok(GeneratedField::StartLine),
                            "endLine" | "end_line" => Ok(GeneratedField::EndLine),
                            "lines" => Ok(GeneratedField::Lines),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = action_edit_file::DiffBlock;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.ActionEditFile.DiffBlock")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<action_edit_file::DiffBlock, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut start_line__ = None;
                let mut end_line__ = None;
                let mut lines__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::StartLine => {
                            if start_line__.is_some() {
                                return Err(serde::de::Error::duplicate_field("startLine"));
                            }
                            start_line__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::EndLine => {
                            if end_line__.is_some() {
                                return Err(serde::de::Error::duplicate_field("endLine"));
                            }
                            end_line__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::Lines => {
                            if lines__.is_some() {
                                return Err(serde::de::Error::duplicate_field("lines"));
                            }
                            lines__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(action_edit_file::DiffBlock {
                    start_line: start_line__,
                    end_line: end_line__,
                    lines: lines__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.ActionEditFile.DiffBlock", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for action_edit_file::DiffLine {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.text.is_some() {
            len += 1;
        }
        if self.action.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.ActionEditFile.DiffLine", len)?;
        if let Some(v) = self.text.as_ref() {
            struct_ser.serialize_field("text", v)?;
        }
        if let Some(v) = self.action.as_ref() {
            let v = action_edit_file::diff_line::LineAction::try_from(*v)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", *v)))?;
            struct_ser.serialize_field("action", &v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for action_edit_file::DiffLine {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "text",
            "action",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Text,
            Action,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "text" => Ok(GeneratedField::Text),
                            "action" => Ok(GeneratedField::Action),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = action_edit_file::DiffLine;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.ActionEditFile.DiffLine")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<action_edit_file::DiffLine, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut text__ = None;
                let mut action__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Text => {
                            if text__.is_some() {
                                return Err(serde::de::Error::duplicate_field("text"));
                            }
                            text__ = map_.next_value()?;
                        }
                        GeneratedField::Action => {
                            if action__.is_some() {
                                return Err(serde::de::Error::duplicate_field("action"));
                            }
                            action__ = map_.next_value::<::std::option::Option<action_edit_file::diff_line::LineAction>>()?.map(|x| x as i32);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(action_edit_file::DiffLine {
                    text: text__,
                    action: action__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.ActionEditFile.DiffLine", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for action_edit_file::diff_line::LineAction {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "LINE_ACTION_UNSPECIFIED",
            Self::Insert => "LINE_ACTION_INSERT",
            Self::Delete => "LINE_ACTION_DELETE",
            Self::None => "LINE_ACTION_NONE",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for action_edit_file::diff_line::LineAction {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "LINE_ACTION_UNSPECIFIED",
            "LINE_ACTION_INSERT",
            "LINE_ACTION_DELETE",
            "LINE_ACTION_NONE",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = action_edit_file::diff_line::LineAction;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "LINE_ACTION_UNSPECIFIED" => Ok(action_edit_file::diff_line::LineAction::Unspecified),
                    "LINE_ACTION_INSERT" => Ok(action_edit_file::diff_line::LineAction::Insert),
                    "LINE_ACTION_DELETE" => Ok(action_edit_file::diff_line::LineAction::Delete),
                    "LINE_ACTION_NONE" => Ok(action_edit_file::diff_line::LineAction::None),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for ActionError {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.error_message.is_some() {
            len += 1;
        }
        if self.http_code.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.ActionError", len)?;
        if let Some(v) = self.error_message.as_ref() {
            struct_ser.serialize_field("errorMessage", v)?;
        }
        if let Some(v) = self.http_code.as_ref() {
            struct_ser.serialize_field("httpCode", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ActionError {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "error_message",
            "errorMessage",
            "http_code",
            "httpCode",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ErrorMessage,
            HttpCode,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "errorMessage" | "error_message" => Ok(GeneratedField::ErrorMessage),
                            "httpCode" | "http_code" => Ok(GeneratedField::HttpCode),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ActionError;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.ActionError")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ActionError, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut error_message__ = None;
                let mut http_code__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ErrorMessage => {
                            if error_message__.is_some() {
                                return Err(serde::de::Error::duplicate_field("errorMessage"));
                            }
                            error_message__ = map_.next_value()?;
                        }
                        GeneratedField::HttpCode => {
                            if http_code__.is_some() {
                                return Err(serde::de::Error::duplicate_field("httpCode"));
                            }
                            http_code__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ActionError {
                    error_message: error_message__,
                    http_code: http_code__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.ActionError", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ActionFindFile {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.directory_path.is_some() {
            len += 1;
        }
        if self.query.is_some() {
            len += 1;
        }
        if self.output.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.ActionFindFile", len)?;
        if let Some(v) = self.directory_path.as_ref() {
            struct_ser.serialize_field("directoryPath", v)?;
        }
        if let Some(v) = self.query.as_ref() {
            struct_ser.serialize_field("query", v)?;
        }
        if let Some(v) = self.output.as_ref() {
            struct_ser.serialize_field("output", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ActionFindFile {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "directory_path",
            "directoryPath",
            "query",
            "output",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            DirectoryPath,
            Query,
            Output,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "directoryPath" | "directory_path" => Ok(GeneratedField::DirectoryPath),
                            "query" => Ok(GeneratedField::Query),
                            "output" => Ok(GeneratedField::Output),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ActionFindFile;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.ActionFindFile")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ActionFindFile, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut directory_path__ = None;
                let mut query__ = None;
                let mut output__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::DirectoryPath => {
                            if directory_path__.is_some() {
                                return Err(serde::de::Error::duplicate_field("directoryPath"));
                            }
                            directory_path__ = map_.next_value()?;
                        }
                        GeneratedField::Query => {
                            if query__.is_some() {
                                return Err(serde::de::Error::duplicate_field("query"));
                            }
                            query__ = map_.next_value()?;
                        }
                        GeneratedField::Output => {
                            if output__.is_some() {
                                return Err(serde::de::Error::duplicate_field("output"));
                            }
                            output__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ActionFindFile {
                    directory_path: directory_path__,
                    query: query__,
                    output: output__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.ActionFindFile", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ActionFinish {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.output_string.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.ActionFinish", len)?;
        if let Some(v) = self.output_string.as_ref() {
            struct_ser.serialize_field("outputString", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ActionFinish {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "output_string",
            "outputString",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            OutputString,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "outputString" | "output_string" => Ok(GeneratedField::OutputString),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ActionFinish;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.ActionFinish")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ActionFinish, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut output_string__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::OutputString => {
                            if output_string__.is_some() {
                                return Err(serde::de::Error::duplicate_field("outputString"));
                            }
                            output_string__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ActionFinish {
                    output_string: output_string__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.ActionFinish", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ActionGenerateImage {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.prompt.is_some() {
            len += 1;
        }
        if !self.image_paths.is_empty() {
            len += 1;
        }
        if self.image_name.is_some() {
            len += 1;
        }
        if self.aspect_ratio.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.ActionGenerateImage", len)?;
        if let Some(v) = self.prompt.as_ref() {
            struct_ser.serialize_field("prompt", v)?;
        }
        if !self.image_paths.is_empty() {
            struct_ser.serialize_field("imagePaths", &self.image_paths)?;
        }
        if let Some(v) = self.image_name.as_ref() {
            struct_ser.serialize_field("imageName", v)?;
        }
        if let Some(v) = self.aspect_ratio.as_ref() {
            struct_ser.serialize_field("aspectRatio", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ActionGenerateImage {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "prompt",
            "image_paths",
            "imagePaths",
            "image_name",
            "imageName",
            "aspect_ratio",
            "aspectRatio",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Prompt,
            ImagePaths,
            ImageName,
            AspectRatio,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "prompt" => Ok(GeneratedField::Prompt),
                            "imagePaths" | "image_paths" => Ok(GeneratedField::ImagePaths),
                            "imageName" | "image_name" => Ok(GeneratedField::ImageName),
                            "aspectRatio" | "aspect_ratio" => Ok(GeneratedField::AspectRatio),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ActionGenerateImage;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.ActionGenerateImage")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ActionGenerateImage, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut prompt__ = None;
                let mut image_paths__ = None;
                let mut image_name__ = None;
                let mut aspect_ratio__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Prompt => {
                            if prompt__.is_some() {
                                return Err(serde::de::Error::duplicate_field("prompt"));
                            }
                            prompt__ = map_.next_value()?;
                        }
                        GeneratedField::ImagePaths => {
                            if image_paths__.is_some() {
                                return Err(serde::de::Error::duplicate_field("imagePaths"));
                            }
                            image_paths__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ImageName => {
                            if image_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("imageName"));
                            }
                            image_name__ = map_.next_value()?;
                        }
                        GeneratedField::AspectRatio => {
                            if aspect_ratio__.is_some() {
                                return Err(serde::de::Error::duplicate_field("aspectRatio"));
                            }
                            aspect_ratio__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ActionGenerateImage {
                    prompt: prompt__,
                    image_paths: image_paths__.unwrap_or_default(),
                    image_name: image_name__,
                    aspect_ratio: aspect_ratio__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.ActionGenerateImage", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ActionInvokeSubagent {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("antigravity.localharness.ActionInvokeSubagent", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ActionInvokeSubagent {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                            Ok(GeneratedField::__SkipField__)
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ActionInvokeSubagent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.ActionInvokeSubagent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ActionInvokeSubagent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map_.next_key::<GeneratedField>()?.is_some() {
                    let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(ActionInvokeSubagent {
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.ActionInvokeSubagent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ActionListDirectory {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.directory_path.is_some() {
            len += 1;
        }
        if !self.results.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.ActionListDirectory", len)?;
        if let Some(v) = self.directory_path.as_ref() {
            struct_ser.serialize_field("directoryPath", v)?;
        }
        if !self.results.is_empty() {
            struct_ser.serialize_field("results", &self.results)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ActionListDirectory {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "directory_path",
            "directoryPath",
            "results",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            DirectoryPath,
            Results,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "directoryPath" | "directory_path" => Ok(GeneratedField::DirectoryPath),
                            "results" => Ok(GeneratedField::Results),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ActionListDirectory;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.ActionListDirectory")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ActionListDirectory, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut directory_path__ = None;
                let mut results__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::DirectoryPath => {
                            if directory_path__.is_some() {
                                return Err(serde::de::Error::duplicate_field("directoryPath"));
                            }
                            directory_path__ = map_.next_value()?;
                        }
                        GeneratedField::Results => {
                            if results__.is_some() {
                                return Err(serde::de::Error::duplicate_field("results"));
                            }
                            results__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ActionListDirectory {
                    directory_path: directory_path__,
                    results: results__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.ActionListDirectory", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for action_list_directory::Result {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.name.is_some() {
            len += 1;
        }
        if self.info.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.ActionListDirectory.Result", len)?;
        if let Some(v) = self.name.as_ref() {
            struct_ser.serialize_field("name", v)?;
        }
        if let Some(v) = self.info.as_ref() {
            match v {
                action_list_directory::result::Info::IsDirectory(v) => {
                    struct_ser.serialize_field("isDirectory", v)?;
                }
                action_list_directory::result::Info::FileSize(v) => {
                    #[allow(clippy::needless_borrow)]
                    struct_ser.serialize_field("fileSize", ToString::to_string(&v).as_str())?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for action_list_directory::Result {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "name",
            "is_directory",
            "isDirectory",
            "file_size",
            "fileSize",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Name,
            IsDirectory,
            FileSize,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "name" => Ok(GeneratedField::Name),
                            "isDirectory" | "is_directory" => Ok(GeneratedField::IsDirectory),
                            "fileSize" | "file_size" => Ok(GeneratedField::FileSize),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = action_list_directory::Result;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.ActionListDirectory.Result")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<action_list_directory::Result, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut name__ = None;
                let mut info__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = map_.next_value()?;
                        }
                        GeneratedField::IsDirectory => {
                            if info__.is_some() {
                                return Err(serde::de::Error::duplicate_field("isDirectory"));
                            }
                            info__ = map_.next_value::<::std::option::Option<_>>()?.map(action_list_directory::result::Info::IsDirectory);
                        }
                        GeneratedField::FileSize => {
                            if info__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fileSize"));
                            }
                            info__ = map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| action_list_directory::result::Info::FileSize(x.0));
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(action_list_directory::Result {
                    name: name__,
                    info: info__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.ActionListDirectory.Result", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ActionMcpTool {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.server_name.is_some() {
            len += 1;
        }
        if self.tool_name.is_some() {
            len += 1;
        }
        if self.arguments_json.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.ActionMcpTool", len)?;
        if let Some(v) = self.server_name.as_ref() {
            struct_ser.serialize_field("serverName", v)?;
        }
        if let Some(v) = self.tool_name.as_ref() {
            struct_ser.serialize_field("toolName", v)?;
        }
        if let Some(v) = self.arguments_json.as_ref() {
            struct_ser.serialize_field("argumentsJson", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ActionMcpTool {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "server_name",
            "serverName",
            "tool_name",
            "toolName",
            "arguments_json",
            "argumentsJson",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ServerName,
            ToolName,
            ArgumentsJson,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "serverName" | "server_name" => Ok(GeneratedField::ServerName),
                            "toolName" | "tool_name" => Ok(GeneratedField::ToolName),
                            "argumentsJson" | "arguments_json" => Ok(GeneratedField::ArgumentsJson),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ActionMcpTool;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.ActionMcpTool")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ActionMcpTool, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut server_name__ = None;
                let mut tool_name__ = None;
                let mut arguments_json__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ServerName => {
                            if server_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("serverName"));
                            }
                            server_name__ = map_.next_value()?;
                        }
                        GeneratedField::ToolName => {
                            if tool_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("toolName"));
                            }
                            tool_name__ = map_.next_value()?;
                        }
                        GeneratedField::ArgumentsJson => {
                            if arguments_json__.is_some() {
                                return Err(serde::de::Error::duplicate_field("argumentsJson"));
                            }
                            arguments_json__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ActionMcpTool {
                    server_name: server_name__,
                    tool_name: tool_name__,
                    arguments_json: arguments_json__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.ActionMcpTool", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ActionReadUrlContent {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.url.is_some() {
            len += 1;
        }
        if self.title.is_some() {
            len += 1;
        }
        if self.summary.is_some() {
            len += 1;
        }
        if self.content_path.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.ActionReadUrlContent", len)?;
        if let Some(v) = self.url.as_ref() {
            struct_ser.serialize_field("url", v)?;
        }
        if let Some(v) = self.title.as_ref() {
            struct_ser.serialize_field("title", v)?;
        }
        if let Some(v) = self.summary.as_ref() {
            struct_ser.serialize_field("summary", v)?;
        }
        if let Some(v) = self.content_path.as_ref() {
            struct_ser.serialize_field("contentPath", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ActionReadUrlContent {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "url",
            "title",
            "summary",
            "content_path",
            "contentPath",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Url,
            Title,
            Summary,
            ContentPath,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "url" => Ok(GeneratedField::Url),
                            "title" => Ok(GeneratedField::Title),
                            "summary" => Ok(GeneratedField::Summary),
                            "contentPath" | "content_path" => Ok(GeneratedField::ContentPath),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ActionReadUrlContent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.ActionReadUrlContent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ActionReadUrlContent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut url__ = None;
                let mut title__ = None;
                let mut summary__ = None;
                let mut content_path__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Url => {
                            if url__.is_some() {
                                return Err(serde::de::Error::duplicate_field("url"));
                            }
                            url__ = map_.next_value()?;
                        }
                        GeneratedField::Title => {
                            if title__.is_some() {
                                return Err(serde::de::Error::duplicate_field("title"));
                            }
                            title__ = map_.next_value()?;
                        }
                        GeneratedField::Summary => {
                            if summary__.is_some() {
                                return Err(serde::de::Error::duplicate_field("summary"));
                            }
                            summary__ = map_.next_value()?;
                        }
                        GeneratedField::ContentPath => {
                            if content_path__.is_some() {
                                return Err(serde::de::Error::duplicate_field("contentPath"));
                            }
                            content_path__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ActionReadUrlContent {
                    url: url__,
                    title: title__,
                    summary: summary__,
                    content_path: content_path__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.ActionReadUrlContent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ActionRunCommand {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.command_line.is_some() {
            len += 1;
        }
        if self.working_dir.is_some() {
            len += 1;
        }
        if self.exit_code.is_some() {
            len += 1;
        }
        if self.combined_output.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.ActionRunCommand", len)?;
        if let Some(v) = self.command_line.as_ref() {
            struct_ser.serialize_field("commandLine", v)?;
        }
        if let Some(v) = self.working_dir.as_ref() {
            struct_ser.serialize_field("workingDir", v)?;
        }
        if let Some(v) = self.exit_code.as_ref() {
            struct_ser.serialize_field("exitCode", v)?;
        }
        if let Some(v) = self.combined_output.as_ref() {
            struct_ser.serialize_field("combinedOutput", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ActionRunCommand {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "command_line",
            "commandLine",
            "working_dir",
            "workingDir",
            "exit_code",
            "exitCode",
            "combined_output",
            "combinedOutput",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            CommandLine,
            WorkingDir,
            ExitCode,
            CombinedOutput,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "commandLine" | "command_line" => Ok(GeneratedField::CommandLine),
                            "workingDir" | "working_dir" => Ok(GeneratedField::WorkingDir),
                            "exitCode" | "exit_code" => Ok(GeneratedField::ExitCode),
                            "combinedOutput" | "combined_output" => Ok(GeneratedField::CombinedOutput),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ActionRunCommand;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.ActionRunCommand")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ActionRunCommand, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut command_line__ = None;
                let mut working_dir__ = None;
                let mut exit_code__ = None;
                let mut combined_output__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::CommandLine => {
                            if command_line__.is_some() {
                                return Err(serde::de::Error::duplicate_field("commandLine"));
                            }
                            command_line__ = map_.next_value()?;
                        }
                        GeneratedField::WorkingDir => {
                            if working_dir__.is_some() {
                                return Err(serde::de::Error::duplicate_field("workingDir"));
                            }
                            working_dir__ = map_.next_value()?;
                        }
                        GeneratedField::ExitCode => {
                            if exit_code__.is_some() {
                                return Err(serde::de::Error::duplicate_field("exitCode"));
                            }
                            exit_code__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::CombinedOutput => {
                            if combined_output__.is_some() {
                                return Err(serde::de::Error::duplicate_field("combinedOutput"));
                            }
                            combined_output__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ActionRunCommand {
                    command_line: command_line__,
                    working_dir: working_dir__,
                    exit_code: exit_code__,
                    combined_output: combined_output__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.ActionRunCommand", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ActionSearchDirectory {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.directory_path.is_some() {
            len += 1;
        }
        if self.query.is_some() {
            len += 1;
        }
        if self.num_results.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.ActionSearchDirectory", len)?;
        if let Some(v) = self.directory_path.as_ref() {
            struct_ser.serialize_field("directoryPath", v)?;
        }
        if let Some(v) = self.query.as_ref() {
            struct_ser.serialize_field("query", v)?;
        }
        if let Some(v) = self.num_results.as_ref() {
            struct_ser.serialize_field("numResults", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ActionSearchDirectory {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "directory_path",
            "directoryPath",
            "query",
            "num_results",
            "numResults",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            DirectoryPath,
            Query,
            NumResults,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "directoryPath" | "directory_path" => Ok(GeneratedField::DirectoryPath),
                            "query" => Ok(GeneratedField::Query),
                            "numResults" | "num_results" => Ok(GeneratedField::NumResults),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ActionSearchDirectory;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.ActionSearchDirectory")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ActionSearchDirectory, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut directory_path__ = None;
                let mut query__ = None;
                let mut num_results__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::DirectoryPath => {
                            if directory_path__.is_some() {
                                return Err(serde::de::Error::duplicate_field("directoryPath"));
                            }
                            directory_path__ = map_.next_value()?;
                        }
                        GeneratedField::Query => {
                            if query__.is_some() {
                                return Err(serde::de::Error::duplicate_field("query"));
                            }
                            query__ = map_.next_value()?;
                        }
                        GeneratedField::NumResults => {
                            if num_results__.is_some() {
                                return Err(serde::de::Error::duplicate_field("numResults"));
                            }
                            num_results__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ActionSearchDirectory {
                    directory_path: directory_path__,
                    query: query__,
                    num_results: num_results__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.ActionSearchDirectory", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ActionSearchWeb {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.query.is_some() {
            len += 1;
        }
        if self.domain.is_some() {
            len += 1;
        }
        if self.summary.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.ActionSearchWeb", len)?;
        if let Some(v) = self.query.as_ref() {
            struct_ser.serialize_field("query", v)?;
        }
        if let Some(v) = self.domain.as_ref() {
            struct_ser.serialize_field("domain", v)?;
        }
        if let Some(v) = self.summary.as_ref() {
            struct_ser.serialize_field("summary", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ActionSearchWeb {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "query",
            "domain",
            "summary",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Query,
            Domain,
            Summary,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "query" => Ok(GeneratedField::Query),
                            "domain" => Ok(GeneratedField::Domain),
                            "summary" => Ok(GeneratedField::Summary),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ActionSearchWeb;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.ActionSearchWeb")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ActionSearchWeb, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut query__ = None;
                let mut domain__ = None;
                let mut summary__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Query => {
                            if query__.is_some() {
                                return Err(serde::de::Error::duplicate_field("query"));
                            }
                            query__ = map_.next_value()?;
                        }
                        GeneratedField::Domain => {
                            if domain__.is_some() {
                                return Err(serde::de::Error::duplicate_field("domain"));
                            }
                            domain__ = map_.next_value()?;
                        }
                        GeneratedField::Summary => {
                            if summary__.is_some() {
                                return Err(serde::de::Error::duplicate_field("summary"));
                            }
                            summary__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ActionSearchWeb {
                    query: query__,
                    domain: domain__,
                    summary: summary__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.ActionSearchWeb", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ActionViewFile {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.file_path.is_some() {
            len += 1;
        }
        if self.start_line.is_some() {
            len += 1;
        }
        if self.end_line.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.ActionViewFile", len)?;
        if let Some(v) = self.file_path.as_ref() {
            struct_ser.serialize_field("filePath", v)?;
        }
        if let Some(v) = self.start_line.as_ref() {
            struct_ser.serialize_field("startLine", v)?;
        }
        if let Some(v) = self.end_line.as_ref() {
            struct_ser.serialize_field("endLine", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ActionViewFile {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "file_path",
            "filePath",
            "start_line",
            "startLine",
            "end_line",
            "endLine",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            FilePath,
            StartLine,
            EndLine,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "filePath" | "file_path" => Ok(GeneratedField::FilePath),
                            "startLine" | "start_line" => Ok(GeneratedField::StartLine),
                            "endLine" | "end_line" => Ok(GeneratedField::EndLine),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ActionViewFile;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.ActionViewFile")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ActionViewFile, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut file_path__ = None;
                let mut start_line__ = None;
                let mut end_line__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::FilePath => {
                            if file_path__.is_some() {
                                return Err(serde::de::Error::duplicate_field("filePath"));
                            }
                            file_path__ = map_.next_value()?;
                        }
                        GeneratedField::StartLine => {
                            if start_line__.is_some() {
                                return Err(serde::de::Error::duplicate_field("startLine"));
                            }
                            start_line__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::EndLine => {
                            if end_line__.is_some() {
                                return Err(serde::de::Error::duplicate_field("endLine"));
                            }
                            end_line__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ActionViewFile {
                    file_path: file_path__,
                    start_line: start_line__,
                    end_line: end_line__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.ActionViewFile", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AppendedSystemInstructions {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.custom_identity.is_some() {
            len += 1;
        }
        if !self.appended_sections.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.AppendedSystemInstructions", len)?;
        if let Some(v) = self.custom_identity.as_ref() {
            struct_ser.serialize_field("customIdentity", v)?;
        }
        if !self.appended_sections.is_empty() {
            struct_ser.serialize_field("appendedSections", &self.appended_sections)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AppendedSystemInstructions {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "custom_identity",
            "customIdentity",
            "appended_sections",
            "appendedSections",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            CustomIdentity,
            AppendedSections,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "customIdentity" | "custom_identity" => Ok(GeneratedField::CustomIdentity),
                            "appendedSections" | "appended_sections" => Ok(GeneratedField::AppendedSections),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AppendedSystemInstructions;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.AppendedSystemInstructions")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AppendedSystemInstructions, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut custom_identity__ = None;
                let mut appended_sections__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::CustomIdentity => {
                            if custom_identity__.is_some() {
                                return Err(serde::de::Error::duplicate_field("customIdentity"));
                            }
                            custom_identity__ = map_.next_value()?;
                        }
                        GeneratedField::AppendedSections => {
                            if appended_sections__.is_some() {
                                return Err(serde::de::Error::duplicate_field("appendedSections"));
                            }
                            appended_sections__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(AppendedSystemInstructions {
                    custom_identity: custom_identity__,
                    appended_sections: appended_sections__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.AppendedSystemInstructions", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for appended_system_instructions::Section {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.title.is_some() {
            len += 1;
        }
        if self.content.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.AppendedSystemInstructions.Section", len)?;
        if let Some(v) = self.title.as_ref() {
            struct_ser.serialize_field("title", v)?;
        }
        if let Some(v) = self.content.as_ref() {
            struct_ser.serialize_field("content", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for appended_system_instructions::Section {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "title",
            "content",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Title,
            Content,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "title" => Ok(GeneratedField::Title),
                            "content" => Ok(GeneratedField::Content),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = appended_system_instructions::Section;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.AppendedSystemInstructions.Section")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<appended_system_instructions::Section, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut title__ = None;
                let mut content__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Title => {
                            if title__.is_some() {
                                return Err(serde::de::Error::duplicate_field("title"));
                            }
                            title__ = map_.next_value()?;
                        }
                        GeneratedField::Content => {
                            if content__.is_some() {
                                return Err(serde::de::Error::duplicate_field("content"));
                            }
                            content__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(appended_system_instructions::Section {
                    title: title__,
                    content: content__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.AppendedSystemInstructions.Section", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CallHookRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.request_id.is_some() {
            len += 1;
        }
        if self.name.is_some() {
            len += 1;
        }
        if self.r#type.is_some() {
            len += 1;
        }
        if self.args.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.CallHookRequest", len)?;
        if let Some(v) = self.request_id.as_ref() {
            struct_ser.serialize_field("requestId", v)?;
        }
        if let Some(v) = self.name.as_ref() {
            struct_ser.serialize_field("name", v)?;
        }
        if let Some(v) = self.r#type.as_ref() {
            let v = LifecycleHook::try_from(*v)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", *v)))?;
            struct_ser.serialize_field("type", &v)?;
        }
        if let Some(v) = self.args.as_ref() {
            match v {
                call_hook_request::Args::PreTurnArgs(v) => {
                    struct_ser.serialize_field("preTurnArgs", v)?;
                }
                call_hook_request::Args::PostTurnArgs(v) => {
                    struct_ser.serialize_field("postTurnArgs", v)?;
                }
                call_hook_request::Args::PreToolArgs(v) => {
                    struct_ser.serialize_field("preToolArgs", v)?;
                }
                call_hook_request::Args::PostToolArgs(v) => {
                    struct_ser.serialize_field("postToolArgs", v)?;
                }
                call_hook_request::Args::OnToolErrorArgs(v) => {
                    struct_ser.serialize_field("onToolErrorArgs", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CallHookRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "request_id",
            "requestId",
            "name",
            "type",
            "pre_turn_args",
            "preTurnArgs",
            "post_turn_args",
            "postTurnArgs",
            "pre_tool_args",
            "preToolArgs",
            "post_tool_args",
            "postToolArgs",
            "on_tool_error_args",
            "onToolErrorArgs",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            RequestId,
            Name,
            Type,
            PreTurnArgs,
            PostTurnArgs,
            PreToolArgs,
            PostToolArgs,
            OnToolErrorArgs,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "requestId" | "request_id" => Ok(GeneratedField::RequestId),
                            "name" => Ok(GeneratedField::Name),
                            "type" => Ok(GeneratedField::Type),
                            "preTurnArgs" | "pre_turn_args" => Ok(GeneratedField::PreTurnArgs),
                            "postTurnArgs" | "post_turn_args" => Ok(GeneratedField::PostTurnArgs),
                            "preToolArgs" | "pre_tool_args" => Ok(GeneratedField::PreToolArgs),
                            "postToolArgs" | "post_tool_args" => Ok(GeneratedField::PostToolArgs),
                            "onToolErrorArgs" | "on_tool_error_args" => Ok(GeneratedField::OnToolErrorArgs),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CallHookRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.CallHookRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CallHookRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut request_id__ = None;
                let mut name__ = None;
                let mut r#type__ = None;
                let mut args__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::RequestId => {
                            if request_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("requestId"));
                            }
                            request_id__ = map_.next_value()?;
                        }
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = map_.next_value()?;
                        }
                        GeneratedField::Type => {
                            if r#type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("type"));
                            }
                            r#type__ = map_.next_value::<::std::option::Option<LifecycleHook>>()?.map(|x| x as i32);
                        }
                        GeneratedField::PreTurnArgs => {
                            if args__.is_some() {
                                return Err(serde::de::Error::duplicate_field("preTurnArgs"));
                            }
                            args__ = map_.next_value::<::std::option::Option<_>>()?.map(call_hook_request::Args::PreTurnArgs)
;
                        }
                        GeneratedField::PostTurnArgs => {
                            if args__.is_some() {
                                return Err(serde::de::Error::duplicate_field("postTurnArgs"));
                            }
                            args__ = map_.next_value::<::std::option::Option<_>>()?.map(call_hook_request::Args::PostTurnArgs)
;
                        }
                        GeneratedField::PreToolArgs => {
                            if args__.is_some() {
                                return Err(serde::de::Error::duplicate_field("preToolArgs"));
                            }
                            args__ = map_.next_value::<::std::option::Option<_>>()?.map(call_hook_request::Args::PreToolArgs)
;
                        }
                        GeneratedField::PostToolArgs => {
                            if args__.is_some() {
                                return Err(serde::de::Error::duplicate_field("postToolArgs"));
                            }
                            args__ = map_.next_value::<::std::option::Option<_>>()?.map(call_hook_request::Args::PostToolArgs)
;
                        }
                        GeneratedField::OnToolErrorArgs => {
                            if args__.is_some() {
                                return Err(serde::de::Error::duplicate_field("onToolErrorArgs"));
                            }
                            args__ = map_.next_value::<::std::option::Option<_>>()?.map(call_hook_request::Args::OnToolErrorArgs)
;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(CallHookRequest {
                    request_id: request_id__,
                    name: name__,
                    r#type: r#type__,
                    args: args__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.CallHookRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CallHookResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.request_id.is_some() {
            len += 1;
        }
        if self.result.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.CallHookResponse", len)?;
        if let Some(v) = self.request_id.as_ref() {
            struct_ser.serialize_field("requestId", v)?;
        }
        if let Some(v) = self.result.as_ref() {
            match v {
                call_hook_response::Result::PreTurnResult(v) => {
                    struct_ser.serialize_field("preTurnResult", v)?;
                }
                call_hook_response::Result::PreToolResult(v) => {
                    struct_ser.serialize_field("preToolResult", v)?;
                }
                call_hook_response::Result::EmptyResult(v) => {
                    struct_ser.serialize_field("emptyResult", v)?;
                }
                call_hook_response::Result::ErrorMessage(v) => {
                    struct_ser.serialize_field("errorMessage", v)?;
                }
                call_hook_response::Result::OnToolErrorResult(v) => {
                    struct_ser.serialize_field("onToolErrorResult", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CallHookResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "request_id",
            "requestId",
            "pre_turn_result",
            "preTurnResult",
            "pre_tool_result",
            "preToolResult",
            "empty_result",
            "emptyResult",
            "error_message",
            "errorMessage",
            "on_tool_error_result",
            "onToolErrorResult",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            RequestId,
            PreTurnResult,
            PreToolResult,
            EmptyResult,
            ErrorMessage,
            OnToolErrorResult,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "requestId" | "request_id" => Ok(GeneratedField::RequestId),
                            "preTurnResult" | "pre_turn_result" => Ok(GeneratedField::PreTurnResult),
                            "preToolResult" | "pre_tool_result" => Ok(GeneratedField::PreToolResult),
                            "emptyResult" | "empty_result" => Ok(GeneratedField::EmptyResult),
                            "errorMessage" | "error_message" => Ok(GeneratedField::ErrorMessage),
                            "onToolErrorResult" | "on_tool_error_result" => Ok(GeneratedField::OnToolErrorResult),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CallHookResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.CallHookResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CallHookResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut request_id__ = None;
                let mut result__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::RequestId => {
                            if request_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("requestId"));
                            }
                            request_id__ = map_.next_value()?;
                        }
                        GeneratedField::PreTurnResult => {
                            if result__.is_some() {
                                return Err(serde::de::Error::duplicate_field("preTurnResult"));
                            }
                            result__ = map_.next_value::<::std::option::Option<_>>()?.map(call_hook_response::Result::PreTurnResult)
;
                        }
                        GeneratedField::PreToolResult => {
                            if result__.is_some() {
                                return Err(serde::de::Error::duplicate_field("preToolResult"));
                            }
                            result__ = map_.next_value::<::std::option::Option<_>>()?.map(call_hook_response::Result::PreToolResult)
;
                        }
                        GeneratedField::EmptyResult => {
                            if result__.is_some() {
                                return Err(serde::de::Error::duplicate_field("emptyResult"));
                            }
                            result__ = map_.next_value::<::std::option::Option<_>>()?.map(call_hook_response::Result::EmptyResult)
;
                        }
                        GeneratedField::ErrorMessage => {
                            if result__.is_some() {
                                return Err(serde::de::Error::duplicate_field("errorMessage"));
                            }
                            result__ = map_.next_value::<::std::option::Option<_>>()?.map(call_hook_response::Result::ErrorMessage);
                        }
                        GeneratedField::OnToolErrorResult => {
                            if result__.is_some() {
                                return Err(serde::de::Error::duplicate_field("onToolErrorResult"));
                            }
                            result__ = map_.next_value::<::std::option::Option<_>>()?.map(call_hook_response::Result::OnToolErrorResult)
;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(CallHookResponse {
                    request_id: request_id__,
                    result: result__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.CallHookResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ClientInfo {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.language.is_some() {
            len += 1;
        }
        if self.version.is_some() {
            len += 1;
        }
        if self.language_version.is_some() {
            len += 1;
        }
        if self.os.is_some() {
            len += 1;
        }
        if self.os_version.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.ClientInfo", len)?;
        if let Some(v) = self.language.as_ref() {
            struct_ser.serialize_field("language", v)?;
        }
        if let Some(v) = self.version.as_ref() {
            struct_ser.serialize_field("version", v)?;
        }
        if let Some(v) = self.language_version.as_ref() {
            struct_ser.serialize_field("languageVersion", v)?;
        }
        if let Some(v) = self.os.as_ref() {
            struct_ser.serialize_field("os", v)?;
        }
        if let Some(v) = self.os_version.as_ref() {
            struct_ser.serialize_field("osVersion", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ClientInfo {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "language",
            "version",
            "language_version",
            "languageVersion",
            "os",
            "os_version",
            "osVersion",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Language,
            Version,
            LanguageVersion,
            Os,
            OsVersion,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "language" => Ok(GeneratedField::Language),
                            "version" => Ok(GeneratedField::Version),
                            "languageVersion" | "language_version" => Ok(GeneratedField::LanguageVersion),
                            "os" => Ok(GeneratedField::Os),
                            "osVersion" | "os_version" => Ok(GeneratedField::OsVersion),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ClientInfo;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.ClientInfo")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ClientInfo, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut language__ = None;
                let mut version__ = None;
                let mut language_version__ = None;
                let mut os__ = None;
                let mut os_version__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Language => {
                            if language__.is_some() {
                                return Err(serde::de::Error::duplicate_field("language"));
                            }
                            language__ = map_.next_value()?;
                        }
                        GeneratedField::Version => {
                            if version__.is_some() {
                                return Err(serde::de::Error::duplicate_field("version"));
                            }
                            version__ = map_.next_value()?;
                        }
                        GeneratedField::LanguageVersion => {
                            if language_version__.is_some() {
                                return Err(serde::de::Error::duplicate_field("languageVersion"));
                            }
                            language_version__ = map_.next_value()?;
                        }
                        GeneratedField::Os => {
                            if os__.is_some() {
                                return Err(serde::de::Error::duplicate_field("os"));
                            }
                            os__ = map_.next_value()?;
                        }
                        GeneratedField::OsVersion => {
                            if os_version__.is_some() {
                                return Err(serde::de::Error::duplicate_field("osVersion"));
                            }
                            os_version__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ClientInfo {
                    language: language__,
                    version: version__,
                    language_version: language_version__,
                    os: os__,
                    os_version: os_version__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.ClientInfo", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CustomAgent {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.name.is_some() {
            len += 1;
        }
        if self.description.is_some() {
            len += 1;
        }
        if self.system_instructions.is_some() {
            len += 1;
        }
        if self.harness_side_tools.is_some() {
            len += 1;
        }
        if !self.tools.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.CustomAgent", len)?;
        if let Some(v) = self.name.as_ref() {
            struct_ser.serialize_field("name", v)?;
        }
        if let Some(v) = self.description.as_ref() {
            struct_ser.serialize_field("description", v)?;
        }
        if let Some(v) = self.system_instructions.as_ref() {
            struct_ser.serialize_field("systemInstructions", v)?;
        }
        if let Some(v) = self.harness_side_tools.as_ref() {
            struct_ser.serialize_field("harnessSideTools", v)?;
        }
        if !self.tools.is_empty() {
            struct_ser.serialize_field("tools", &self.tools)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CustomAgent {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "name",
            "description",
            "system_instructions",
            "systemInstructions",
            "harness_side_tools",
            "harnessSideTools",
            "tools",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Name,
            Description,
            SystemInstructions,
            HarnessSideTools,
            Tools,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "name" => Ok(GeneratedField::Name),
                            "description" => Ok(GeneratedField::Description),
                            "systemInstructions" | "system_instructions" => Ok(GeneratedField::SystemInstructions),
                            "harnessSideTools" | "harness_side_tools" => Ok(GeneratedField::HarnessSideTools),
                            "tools" => Ok(GeneratedField::Tools),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CustomAgent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.CustomAgent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CustomAgent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut name__ = None;
                let mut description__ = None;
                let mut system_instructions__ = None;
                let mut harness_side_tools__ = None;
                let mut tools__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = map_.next_value()?;
                        }
                        GeneratedField::Description => {
                            if description__.is_some() {
                                return Err(serde::de::Error::duplicate_field("description"));
                            }
                            description__ = map_.next_value()?;
                        }
                        GeneratedField::SystemInstructions => {
                            if system_instructions__.is_some() {
                                return Err(serde::de::Error::duplicate_field("systemInstructions"));
                            }
                            system_instructions__ = map_.next_value()?;
                        }
                        GeneratedField::HarnessSideTools => {
                            if harness_side_tools__.is_some() {
                                return Err(serde::de::Error::duplicate_field("harnessSideTools"));
                            }
                            harness_side_tools__ = map_.next_value()?;
                        }
                        GeneratedField::Tools => {
                            if tools__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tools"));
                            }
                            tools__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(CustomAgent {
                    name: name__,
                    description: description__,
                    system_instructions: system_instructions__,
                    harness_side_tools: harness_side_tools__,
                    tools: tools__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.CustomAgent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CustomEndpoint {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.backend_type.is_some() {
            len += 1;
        }
        if self.config_json.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.CustomEndpoint", len)?;
        if let Some(v) = self.backend_type.as_ref() {
            struct_ser.serialize_field("backendType", v)?;
        }
        if let Some(v) = self.config_json.as_ref() {
            struct_ser.serialize_field("configJson", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CustomEndpoint {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "backend_type",
            "backendType",
            "config_json",
            "configJson",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            BackendType,
            ConfigJson,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "backendType" | "backend_type" => Ok(GeneratedField::BackendType),
                            "configJson" | "config_json" => Ok(GeneratedField::ConfigJson),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CustomEndpoint;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.CustomEndpoint")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CustomEndpoint, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut backend_type__ = None;
                let mut config_json__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::BackendType => {
                            if backend_type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("backendType"));
                            }
                            backend_type__ = map_.next_value()?;
                        }
                        GeneratedField::ConfigJson => {
                            if config_json__.is_some() {
                                return Err(serde::de::Error::duplicate_field("configJson"));
                            }
                            config_json__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(CustomEndpoint {
                    backend_type: backend_type__,
                    config_json: config_json__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.CustomEndpoint", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CustomSystemInstructions {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.part.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.CustomSystemInstructions", len)?;
        if !self.part.is_empty() {
            struct_ser.serialize_field("part", &self.part)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CustomSystemInstructions {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "part",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Part,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "part" => Ok(GeneratedField::Part),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CustomSystemInstructions;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.CustomSystemInstructions")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CustomSystemInstructions, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut part__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Part => {
                            if part__.is_some() {
                                return Err(serde::de::Error::duplicate_field("part"));
                            }
                            part__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(CustomSystemInstructions {
                    part: part__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.CustomSystemInstructions", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for custom_system_instructions::Part {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.part.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.CustomSystemInstructions.Part", len)?;
        if let Some(v) = self.part.as_ref() {
            match v {
                custom_system_instructions::part::Part::Text(v) => {
                    struct_ser.serialize_field("text", v)?;
                }
                custom_system_instructions::part::Part::Template(v) => {
                    struct_ser.serialize_field("template", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for custom_system_instructions::Part {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "text",
            "template",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Text,
            Template,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "text" => Ok(GeneratedField::Text),
                            "template" => Ok(GeneratedField::Template),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = custom_system_instructions::Part;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.CustomSystemInstructions.Part")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<custom_system_instructions::Part, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut part__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Text => {
                            if part__.is_some() {
                                return Err(serde::de::Error::duplicate_field("text"));
                            }
                            part__ = map_.next_value::<::std::option::Option<_>>()?.map(custom_system_instructions::part::Part::Text);
                        }
                        GeneratedField::Template => {
                            if part__.is_some() {
                                return Err(serde::de::Error::duplicate_field("template"));
                            }
                            part__ = map_.next_value::<::std::option::Option<_>>()?.map(custom_system_instructions::part::Part::Template)
;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(custom_system_instructions::Part {
                    part: part__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.CustomSystemInstructions.Part", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for custom_system_instructions::SystemInstructionTemplate {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.template_name.is_some() {
            len += 1;
        }
        if !self.args.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.CustomSystemInstructions.SystemInstructionTemplate", len)?;
        if let Some(v) = self.template_name.as_ref() {
            struct_ser.serialize_field("templateName", v)?;
        }
        if !self.args.is_empty() {
            struct_ser.serialize_field("args", &self.args)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for custom_system_instructions::SystemInstructionTemplate {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "template_name",
            "templateName",
            "args",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TemplateName,
            Args,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "templateName" | "template_name" => Ok(GeneratedField::TemplateName),
                            "args" => Ok(GeneratedField::Args),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = custom_system_instructions::SystemInstructionTemplate;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.CustomSystemInstructions.SystemInstructionTemplate")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<custom_system_instructions::SystemInstructionTemplate, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut template_name__ = None;
                let mut args__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::TemplateName => {
                            if template_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("templateName"));
                            }
                            template_name__ = map_.next_value()?;
                        }
                        GeneratedField::Args => {
                            if args__.is_some() {
                                return Err(serde::de::Error::duplicate_field("args"));
                            }
                            args__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(custom_system_instructions::SystemInstructionTemplate {
                    template_name: template_name__,
                    args: args__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.CustomSystemInstructions.SystemInstructionTemplate", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for custom_system_instructions::system_instruction_template::Arg {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.name.is_some() {
            len += 1;
        }
        if self.value.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.CustomSystemInstructions.SystemInstructionTemplate.Arg", len)?;
        if let Some(v) = self.name.as_ref() {
            struct_ser.serialize_field("name", v)?;
        }
        if let Some(v) = self.value.as_ref() {
            struct_ser.serialize_field("value", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for custom_system_instructions::system_instruction_template::Arg {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "name",
            "value",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Name,
            Value,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "name" => Ok(GeneratedField::Name),
                            "value" => Ok(GeneratedField::Value),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = custom_system_instructions::system_instruction_template::Arg;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.CustomSystemInstructions.SystemInstructionTemplate.Arg")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<custom_system_instructions::system_instruction_template::Arg, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut name__ = None;
                let mut value__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = map_.next_value()?;
                        }
                        GeneratedField::Value => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("value"));
                            }
                            value__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(custom_system_instructions::system_instruction_template::Arg {
                    name: name__,
                    value: value__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.CustomSystemInstructions.SystemInstructionTemplate.Arg", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for EmptyResult {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("antigravity.localharness.EmptyResult", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for EmptyResult {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                            Ok(GeneratedField::__SkipField__)
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = EmptyResult;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.EmptyResult")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<EmptyResult, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map_.next_key::<GeneratedField>()?.is_some() {
                    let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(EmptyResult {
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.EmptyResult", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for FileEditToolConfig {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.enabled.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.FileEditToolConfig", len)?;
        if let Some(v) = self.enabled.as_ref() {
            struct_ser.serialize_field("enabled", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for FileEditToolConfig {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "enabled",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Enabled,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "enabled" => Ok(GeneratedField::Enabled),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = FileEditToolConfig;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.FileEditToolConfig")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<FileEditToolConfig, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut enabled__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Enabled => {
                            if enabled__.is_some() {
                                return Err(serde::de::Error::duplicate_field("enabled"));
                            }
                            enabled__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(FileEditToolConfig {
                    enabled: enabled__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.FileEditToolConfig", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for FilesystemWorkspace {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.directory.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.FilesystemWorkspace", len)?;
        if let Some(v) = self.directory.as_ref() {
            struct_ser.serialize_field("directory", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for FilesystemWorkspace {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "directory",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Directory,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "directory" => Ok(GeneratedField::Directory),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = FilesystemWorkspace;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.FilesystemWorkspace")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<FilesystemWorkspace, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut directory__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Directory => {
                            if directory__.is_some() {
                                return Err(serde::de::Error::duplicate_field("directory"));
                            }
                            directory__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(FilesystemWorkspace {
                    directory: directory__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.FilesystemWorkspace", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for FindToolConfig {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.enabled.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.FindToolConfig", len)?;
        if let Some(v) = self.enabled.as_ref() {
            struct_ser.serialize_field("enabled", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for FindToolConfig {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "enabled",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Enabled,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "enabled" => Ok(GeneratedField::Enabled),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = FindToolConfig;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.FindToolConfig")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<FindToolConfig, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut enabled__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Enabled => {
                            if enabled__.is_some() {
                                return Err(serde::de::Error::duplicate_field("enabled"));
                            }
                            enabled__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(FindToolConfig {
                    enabled: enabled__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.FindToolConfig", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GeminiApiEndpoint {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.base_url.is_some() {
            len += 1;
        }
        if !self.http_headers.is_empty() {
            len += 1;
        }
        if self.api_key.is_some() {
            len += 1;
        }
        if self.options.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.GeminiAPIEndpoint", len)?;
        if let Some(v) = self.base_url.as_ref() {
            struct_ser.serialize_field("baseUrl", v)?;
        }
        if !self.http_headers.is_empty() {
            struct_ser.serialize_field("httpHeaders", &self.http_headers)?;
        }
        if let Some(v) = self.api_key.as_ref() {
            struct_ser.serialize_field("apiKey", v)?;
        }
        if let Some(v) = self.options.as_ref() {
            struct_ser.serialize_field("options", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GeminiApiEndpoint {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "base_url",
            "baseUrl",
            "http_headers",
            "httpHeaders",
            "api_key",
            "apiKey",
            "options",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            BaseUrl,
            HttpHeaders,
            ApiKey,
            Options,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "baseUrl" | "base_url" => Ok(GeneratedField::BaseUrl),
                            "httpHeaders" | "http_headers" => Ok(GeneratedField::HttpHeaders),
                            "apiKey" | "api_key" => Ok(GeneratedField::ApiKey),
                            "options" => Ok(GeneratedField::Options),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GeminiApiEndpoint;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.GeminiAPIEndpoint")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GeminiApiEndpoint, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut base_url__ = None;
                let mut http_headers__ = None;
                let mut api_key__ = None;
                let mut options__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::BaseUrl => {
                            if base_url__.is_some() {
                                return Err(serde::de::Error::duplicate_field("baseUrl"));
                            }
                            base_url__ = map_.next_value()?;
                        }
                        GeneratedField::HttpHeaders => {
                            if http_headers__.is_some() {
                                return Err(serde::de::Error::duplicate_field("httpHeaders"));
                            }
                            http_headers__ = Some(
                                map_.next_value::<std::collections::HashMap<_, _>>()?
                            );
                        }
                        GeneratedField::ApiKey => {
                            if api_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("apiKey"));
                            }
                            api_key__ = map_.next_value()?;
                        }
                        GeneratedField::Options => {
                            if options__.is_some() {
                                return Err(serde::de::Error::duplicate_field("options"));
                            }
                            options__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(GeminiApiEndpoint {
                    base_url: base_url__,
                    http_headers: http_headers__.unwrap_or_default(),
                    api_key: api_key__,
                    options: options__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.GeminiAPIEndpoint", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GeminiModelOptions {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.thinking_level.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.GeminiModelOptions", len)?;
        if let Some(v) = self.thinking_level.as_ref() {
            struct_ser.serialize_field("thinkingLevel", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GeminiModelOptions {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "thinking_level",
            "thinkingLevel",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ThinkingLevel,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "thinkingLevel" | "thinking_level" => Ok(GeneratedField::ThinkingLevel),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GeminiModelOptions;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.GeminiModelOptions")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GeminiModelOptions, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut thinking_level__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ThinkingLevel => {
                            if thinking_level__.is_some() {
                                return Err(serde::de::Error::duplicate_field("thinkingLevel"));
                            }
                            thinking_level__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(GeminiModelOptions {
                    thinking_level: thinking_level__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.GeminiModelOptions", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GemmaEndpoint {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.base_url.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.GemmaEndpoint", len)?;
        if let Some(v) = self.base_url.as_ref() {
            struct_ser.serialize_field("baseUrl", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GemmaEndpoint {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "base_url",
            "baseUrl",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            BaseUrl,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "baseUrl" | "base_url" => Ok(GeneratedField::BaseUrl),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GemmaEndpoint;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.GemmaEndpoint")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GemmaEndpoint, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut base_url__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::BaseUrl => {
                            if base_url__.is_some() {
                                return Err(serde::de::Error::duplicate_field("baseUrl"));
                            }
                            base_url__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(GemmaEndpoint {
                    base_url: base_url__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.GemmaEndpoint", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GenerateImageToolConfig {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.enabled.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.GenerateImageToolConfig", len)?;
        if let Some(v) = self.enabled.as_ref() {
            struct_ser.serialize_field("enabled", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GenerateImageToolConfig {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "enabled",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Enabled,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "enabled" => Ok(GeneratedField::Enabled),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GenerateImageToolConfig;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.GenerateImageToolConfig")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GenerateImageToolConfig, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut enabled__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Enabled => {
                            if enabled__.is_some() {
                                return Err(serde::de::Error::duplicate_field("enabled"));
                            }
                            enabled__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(GenerateImageToolConfig {
                    enabled: enabled__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.GenerateImageToolConfig", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GrepSearchToolConfig {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.enabled.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.GrepSearchToolConfig", len)?;
        if let Some(v) = self.enabled.as_ref() {
            struct_ser.serialize_field("enabled", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GrepSearchToolConfig {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "enabled",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Enabled,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "enabled" => Ok(GeneratedField::Enabled),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GrepSearchToolConfig;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.GrepSearchToolConfig")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GrepSearchToolConfig, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut enabled__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Enabled => {
                            if enabled__.is_some() {
                                return Err(serde::de::Error::duplicate_field("enabled"));
                            }
                            enabled__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(GrepSearchToolConfig {
                    enabled: enabled__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.GrepSearchToolConfig", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for HarnessConfig {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.cascade_id.is_some() {
            len += 1;
        }
        if self.session_continuation_mode.is_some() {
            len += 1;
        }
        if self.system_instructions.is_some() {
            len += 1;
        }
        if !self.tools.is_empty() {
            len += 1;
        }
        if self.harness_side_tools.is_some() {
            len += 1;
        }
        if self.compaction_threshold.is_some() {
            len += 1;
        }
        if !self.workspaces.is_empty() {
            len += 1;
        }
        if !self.skills_paths.is_empty() {
            len += 1;
        }
        if self.finish_tool_schema_json.is_some() {
            len += 1;
        }
        if self.initial_trajectory.is_some() {
            len += 1;
        }
        if self.app_data_dir.is_some() {
            len += 1;
        }
        if !self.mcp_servers.is_empty() {
            len += 1;
        }
        if !self.models.is_empty() {
            len += 1;
        }
        if !self.enabled_hooks.is_empty() {
            len += 1;
        }
        if !self.custom_subagents.is_empty() {
            len += 1;
        }
        if self.tool_output_truncation.is_some() {
            len += 1;
        }
        if self.retry_config.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.HarnessConfig", len)?;
        if let Some(v) = self.cascade_id.as_ref() {
            struct_ser.serialize_field("cascadeId", v)?;
        }
        if let Some(v) = self.session_continuation_mode.as_ref() {
            let v = harness_config::SessionContinuationMode::try_from(*v)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", *v)))?;
            struct_ser.serialize_field("sessionContinuationMode", &v)?;
        }
        if let Some(v) = self.system_instructions.as_ref() {
            struct_ser.serialize_field("systemInstructions", v)?;
        }
        if !self.tools.is_empty() {
            struct_ser.serialize_field("tools", &self.tools)?;
        }
        if let Some(v) = self.harness_side_tools.as_ref() {
            struct_ser.serialize_field("harnessSideTools", v)?;
        }
        if let Some(v) = self.compaction_threshold.as_ref() {
            struct_ser.serialize_field("compactionThreshold", v)?;
        }
        if !self.workspaces.is_empty() {
            struct_ser.serialize_field("workspaces", &self.workspaces)?;
        }
        if !self.skills_paths.is_empty() {
            struct_ser.serialize_field("skillsPaths", &self.skills_paths)?;
        }
        if let Some(v) = self.finish_tool_schema_json.as_ref() {
            struct_ser.serialize_field("finishToolSchemaJson", v)?;
        }
        if let Some(v) = self.initial_trajectory.as_ref() {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("initialTrajectory", pbjson::private::base64::encode(&v).as_str())?;
        }
        if let Some(v) = self.app_data_dir.as_ref() {
            struct_ser.serialize_field("appDataDir", v)?;
        }
        if !self.mcp_servers.is_empty() {
            struct_ser.serialize_field("mcpServers", &self.mcp_servers)?;
        }
        if !self.models.is_empty() {
            struct_ser.serialize_field("models", &self.models)?;
        }
        if !self.enabled_hooks.is_empty() {
            let v = self.enabled_hooks.iter().cloned().map(|v| {
                LifecycleHook::try_from(v)
                    .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", v)))
                }).collect::<Result<Vec<_>, _>>()?;
            struct_ser.serialize_field("enabledHooks", &v)?;
        }
        if !self.custom_subagents.is_empty() {
            struct_ser.serialize_field("customSubagents", &self.custom_subagents)?;
        }
        if let Some(v) = self.tool_output_truncation.as_ref() {
            struct_ser.serialize_field("toolOutputTruncation", v)?;
        }
        if let Some(v) = self.retry_config.as_ref() {
            struct_ser.serialize_field("retryConfig", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for HarnessConfig {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "cascade_id",
            "cascadeId",
            "session_continuation_mode",
            "sessionContinuationMode",
            "system_instructions",
            "systemInstructions",
            "tools",
            "harness_side_tools",
            "harnessSideTools",
            "compaction_threshold",
            "compactionThreshold",
            "workspaces",
            "skills_paths",
            "skillsPaths",
            "finish_tool_schema_json",
            "finishToolSchemaJson",
            "initial_trajectory",
            "initialTrajectory",
            "app_data_dir",
            "appDataDir",
            "mcp_servers",
            "mcpServers",
            "models",
            "enabled_hooks",
            "enabledHooks",
            "custom_subagents",
            "customSubagents",
            "tool_output_truncation",
            "toolOutputTruncation",
            "retry_config",
            "retryConfig",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            CascadeId,
            SessionContinuationMode,
            SystemInstructions,
            Tools,
            HarnessSideTools,
            CompactionThreshold,
            Workspaces,
            SkillsPaths,
            FinishToolSchemaJson,
            InitialTrajectory,
            AppDataDir,
            McpServers,
            Models,
            EnabledHooks,
            CustomSubagents,
            ToolOutputTruncation,
            RetryConfig,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "cascadeId" | "cascade_id" => Ok(GeneratedField::CascadeId),
                            "sessionContinuationMode" | "session_continuation_mode" => Ok(GeneratedField::SessionContinuationMode),
                            "systemInstructions" | "system_instructions" => Ok(GeneratedField::SystemInstructions),
                            "tools" => Ok(GeneratedField::Tools),
                            "harnessSideTools" | "harness_side_tools" => Ok(GeneratedField::HarnessSideTools),
                            "compactionThreshold" | "compaction_threshold" => Ok(GeneratedField::CompactionThreshold),
                            "workspaces" => Ok(GeneratedField::Workspaces),
                            "skillsPaths" | "skills_paths" => Ok(GeneratedField::SkillsPaths),
                            "finishToolSchemaJson" | "finish_tool_schema_json" => Ok(GeneratedField::FinishToolSchemaJson),
                            "initialTrajectory" | "initial_trajectory" => Ok(GeneratedField::InitialTrajectory),
                            "appDataDir" | "app_data_dir" => Ok(GeneratedField::AppDataDir),
                            "mcpServers" | "mcp_servers" => Ok(GeneratedField::McpServers),
                            "models" => Ok(GeneratedField::Models),
                            "enabledHooks" | "enabled_hooks" => Ok(GeneratedField::EnabledHooks),
                            "customSubagents" | "custom_subagents" => Ok(GeneratedField::CustomSubagents),
                            "toolOutputTruncation" | "tool_output_truncation" => Ok(GeneratedField::ToolOutputTruncation),
                            "retryConfig" | "retry_config" => Ok(GeneratedField::RetryConfig),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = HarnessConfig;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.HarnessConfig")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<HarnessConfig, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut cascade_id__ = None;
                let mut session_continuation_mode__ = None;
                let mut system_instructions__ = None;
                let mut tools__ = None;
                let mut harness_side_tools__ = None;
                let mut compaction_threshold__ = None;
                let mut workspaces__ = None;
                let mut skills_paths__ = None;
                let mut finish_tool_schema_json__ = None;
                let mut initial_trajectory__ = None;
                let mut app_data_dir__ = None;
                let mut mcp_servers__ = None;
                let mut models__ = None;
                let mut enabled_hooks__ = None;
                let mut custom_subagents__ = None;
                let mut tool_output_truncation__ = None;
                let mut retry_config__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::CascadeId => {
                            if cascade_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("cascadeId"));
                            }
                            cascade_id__ = map_.next_value()?;
                        }
                        GeneratedField::SessionContinuationMode => {
                            if session_continuation_mode__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sessionContinuationMode"));
                            }
                            session_continuation_mode__ = map_.next_value::<::std::option::Option<harness_config::SessionContinuationMode>>()?.map(|x| x as i32);
                        }
                        GeneratedField::SystemInstructions => {
                            if system_instructions__.is_some() {
                                return Err(serde::de::Error::duplicate_field("systemInstructions"));
                            }
                            system_instructions__ = map_.next_value()?;
                        }
                        GeneratedField::Tools => {
                            if tools__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tools"));
                            }
                            tools__ = Some(map_.next_value()?);
                        }
                        GeneratedField::HarnessSideTools => {
                            if harness_side_tools__.is_some() {
                                return Err(serde::de::Error::duplicate_field("harnessSideTools"));
                            }
                            harness_side_tools__ = map_.next_value()?;
                        }
                        GeneratedField::CompactionThreshold => {
                            if compaction_threshold__.is_some() {
                                return Err(serde::de::Error::duplicate_field("compactionThreshold"));
                            }
                            compaction_threshold__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::Workspaces => {
                            if workspaces__.is_some() {
                                return Err(serde::de::Error::duplicate_field("workspaces"));
                            }
                            workspaces__ = Some(map_.next_value()?);
                        }
                        GeneratedField::SkillsPaths => {
                            if skills_paths__.is_some() {
                                return Err(serde::de::Error::duplicate_field("skillsPaths"));
                            }
                            skills_paths__ = Some(map_.next_value()?);
                        }
                        GeneratedField::FinishToolSchemaJson => {
                            if finish_tool_schema_json__.is_some() {
                                return Err(serde::de::Error::duplicate_field("finishToolSchemaJson"));
                            }
                            finish_tool_schema_json__ = map_.next_value()?;
                        }
                        GeneratedField::InitialTrajectory => {
                            if initial_trajectory__.is_some() {
                                return Err(serde::de::Error::duplicate_field("initialTrajectory"));
                            }
                            initial_trajectory__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::BytesDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::AppDataDir => {
                            if app_data_dir__.is_some() {
                                return Err(serde::de::Error::duplicate_field("appDataDir"));
                            }
                            app_data_dir__ = map_.next_value()?;
                        }
                        GeneratedField::McpServers => {
                            if mcp_servers__.is_some() {
                                return Err(serde::de::Error::duplicate_field("mcpServers"));
                            }
                            mcp_servers__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Models => {
                            if models__.is_some() {
                                return Err(serde::de::Error::duplicate_field("models"));
                            }
                            models__ = Some(map_.next_value()?);
                        }
                        GeneratedField::EnabledHooks => {
                            if enabled_hooks__.is_some() {
                                return Err(serde::de::Error::duplicate_field("enabledHooks"));
                            }
                            enabled_hooks__ = Some(map_.next_value::<Vec<LifecycleHook>>()?.into_iter().map(|x| x as i32).collect());
                        }
                        GeneratedField::CustomSubagents => {
                            if custom_subagents__.is_some() {
                                return Err(serde::de::Error::duplicate_field("customSubagents"));
                            }
                            custom_subagents__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ToolOutputTruncation => {
                            if tool_output_truncation__.is_some() {
                                return Err(serde::de::Error::duplicate_field("toolOutputTruncation"));
                            }
                            tool_output_truncation__ = map_.next_value()?;
                        }
                        GeneratedField::RetryConfig => {
                            if retry_config__.is_some() {
                                return Err(serde::de::Error::duplicate_field("retryConfig"));
                            }
                            retry_config__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(HarnessConfig {
                    cascade_id: cascade_id__,
                    session_continuation_mode: session_continuation_mode__,
                    system_instructions: system_instructions__,
                    tools: tools__.unwrap_or_default(),
                    harness_side_tools: harness_side_tools__,
                    compaction_threshold: compaction_threshold__,
                    workspaces: workspaces__.unwrap_or_default(),
                    skills_paths: skills_paths__.unwrap_or_default(),
                    finish_tool_schema_json: finish_tool_schema_json__,
                    initial_trajectory: initial_trajectory__,
                    app_data_dir: app_data_dir__,
                    mcp_servers: mcp_servers__.unwrap_or_default(),
                    models: models__.unwrap_or_default(),
                    enabled_hooks: enabled_hooks__.unwrap_or_default(),
                    custom_subagents: custom_subagents__.unwrap_or_default(),
                    tool_output_truncation: tool_output_truncation__,
                    retry_config: retry_config__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.HarnessConfig", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for harness_config::SessionContinuationMode {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "SESSION_CONTINUATION_MODE_UNSPECIFIED",
            Self::Resume => "RESUME",
            Self::CreateOrResume => "CREATE_OR_RESUME",
            Self::CreateOnly => "CREATE_ONLY",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for harness_config::SessionContinuationMode {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "SESSION_CONTINUATION_MODE_UNSPECIFIED",
            "RESUME",
            "CREATE_OR_RESUME",
            "CREATE_ONLY",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = harness_config::SessionContinuationMode;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "SESSION_CONTINUATION_MODE_UNSPECIFIED" => Ok(harness_config::SessionContinuationMode::Unspecified),
                    "RESUME" => Ok(harness_config::SessionContinuationMode::Resume),
                    "CREATE_OR_RESUME" => Ok(harness_config::SessionContinuationMode::CreateOrResume),
                    "CREATE_ONLY" => Ok(harness_config::SessionContinuationMode::CreateOnly),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for HarnessSideTools {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.find.is_some() {
            len += 1;
        }
        if self.run_command.is_some() {
            len += 1;
        }
        if self.subagents.is_some() {
            len += 1;
        }
        if self.user_questions.is_some() {
            len += 1;
        }
        if self.file_edit.is_some() {
            len += 1;
        }
        if self.view_file.is_some() {
            len += 1;
        }
        if self.write_to_file.is_some() {
            len += 1;
        }
        if self.grep_search.is_some() {
            len += 1;
        }
        if self.list_dir.is_some() {
            len += 1;
        }
        if self.permissions.is_some() {
            len += 1;
        }
        if self.generate_image.is_some() {
            len += 1;
        }
        if self.search_web.is_some() {
            len += 1;
        }
        if self.read_url_content.is_some() {
            len += 1;
        }
        if self.tool_search_config.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.HarnessSideTools", len)?;
        if let Some(v) = self.find.as_ref() {
            struct_ser.serialize_field("find", v)?;
        }
        if let Some(v) = self.run_command.as_ref() {
            struct_ser.serialize_field("runCommand", v)?;
        }
        if let Some(v) = self.subagents.as_ref() {
            struct_ser.serialize_field("subagents", v)?;
        }
        if let Some(v) = self.user_questions.as_ref() {
            struct_ser.serialize_field("userQuestions", v)?;
        }
        if let Some(v) = self.file_edit.as_ref() {
            struct_ser.serialize_field("fileEdit", v)?;
        }
        if let Some(v) = self.view_file.as_ref() {
            struct_ser.serialize_field("viewFile", v)?;
        }
        if let Some(v) = self.write_to_file.as_ref() {
            struct_ser.serialize_field("writeToFile", v)?;
        }
        if let Some(v) = self.grep_search.as_ref() {
            struct_ser.serialize_field("grepSearch", v)?;
        }
        if let Some(v) = self.list_dir.as_ref() {
            struct_ser.serialize_field("listDir", v)?;
        }
        if let Some(v) = self.permissions.as_ref() {
            struct_ser.serialize_field("permissions", v)?;
        }
        if let Some(v) = self.generate_image.as_ref() {
            struct_ser.serialize_field("generateImage", v)?;
        }
        if let Some(v) = self.search_web.as_ref() {
            struct_ser.serialize_field("searchWeb", v)?;
        }
        if let Some(v) = self.read_url_content.as_ref() {
            struct_ser.serialize_field("readUrlContent", v)?;
        }
        if let Some(v) = self.tool_search_config.as_ref() {
            struct_ser.serialize_field("toolSearchConfig", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for HarnessSideTools {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "find",
            "run_command",
            "runCommand",
            "subagents",
            "user_questions",
            "userQuestions",
            "file_edit",
            "fileEdit",
            "view_file",
            "viewFile",
            "write_to_file",
            "writeToFile",
            "grep_search",
            "grepSearch",
            "list_dir",
            "listDir",
            "permissions",
            "generate_image",
            "generateImage",
            "search_web",
            "searchWeb",
            "read_url_content",
            "readUrlContent",
            "tool_search_config",
            "toolSearchConfig",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Find,
            RunCommand,
            Subagents,
            UserQuestions,
            FileEdit,
            ViewFile,
            WriteToFile,
            GrepSearch,
            ListDir,
            Permissions,
            GenerateImage,
            SearchWeb,
            ReadUrlContent,
            ToolSearchConfig,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "find" => Ok(GeneratedField::Find),
                            "runCommand" | "run_command" => Ok(GeneratedField::RunCommand),
                            "subagents" => Ok(GeneratedField::Subagents),
                            "userQuestions" | "user_questions" => Ok(GeneratedField::UserQuestions),
                            "fileEdit" | "file_edit" => Ok(GeneratedField::FileEdit),
                            "viewFile" | "view_file" => Ok(GeneratedField::ViewFile),
                            "writeToFile" | "write_to_file" => Ok(GeneratedField::WriteToFile),
                            "grepSearch" | "grep_search" => Ok(GeneratedField::GrepSearch),
                            "listDir" | "list_dir" => Ok(GeneratedField::ListDir),
                            "permissions" => Ok(GeneratedField::Permissions),
                            "generateImage" | "generate_image" => Ok(GeneratedField::GenerateImage),
                            "searchWeb" | "search_web" => Ok(GeneratedField::SearchWeb),
                            "readUrlContent" | "read_url_content" => Ok(GeneratedField::ReadUrlContent),
                            "toolSearchConfig" | "tool_search_config" => Ok(GeneratedField::ToolSearchConfig),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = HarnessSideTools;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.HarnessSideTools")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<HarnessSideTools, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut find__ = None;
                let mut run_command__ = None;
                let mut subagents__ = None;
                let mut user_questions__ = None;
                let mut file_edit__ = None;
                let mut view_file__ = None;
                let mut write_to_file__ = None;
                let mut grep_search__ = None;
                let mut list_dir__ = None;
                let mut permissions__ = None;
                let mut generate_image__ = None;
                let mut search_web__ = None;
                let mut read_url_content__ = None;
                let mut tool_search_config__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Find => {
                            if find__.is_some() {
                                return Err(serde::de::Error::duplicate_field("find"));
                            }
                            find__ = map_.next_value()?;
                        }
                        GeneratedField::RunCommand => {
                            if run_command__.is_some() {
                                return Err(serde::de::Error::duplicate_field("runCommand"));
                            }
                            run_command__ = map_.next_value()?;
                        }
                        GeneratedField::Subagents => {
                            if subagents__.is_some() {
                                return Err(serde::de::Error::duplicate_field("subagents"));
                            }
                            subagents__ = map_.next_value()?;
                        }
                        GeneratedField::UserQuestions => {
                            if user_questions__.is_some() {
                                return Err(serde::de::Error::duplicate_field("userQuestions"));
                            }
                            user_questions__ = map_.next_value()?;
                        }
                        GeneratedField::FileEdit => {
                            if file_edit__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fileEdit"));
                            }
                            file_edit__ = map_.next_value()?;
                        }
                        GeneratedField::ViewFile => {
                            if view_file__.is_some() {
                                return Err(serde::de::Error::duplicate_field("viewFile"));
                            }
                            view_file__ = map_.next_value()?;
                        }
                        GeneratedField::WriteToFile => {
                            if write_to_file__.is_some() {
                                return Err(serde::de::Error::duplicate_field("writeToFile"));
                            }
                            write_to_file__ = map_.next_value()?;
                        }
                        GeneratedField::GrepSearch => {
                            if grep_search__.is_some() {
                                return Err(serde::de::Error::duplicate_field("grepSearch"));
                            }
                            grep_search__ = map_.next_value()?;
                        }
                        GeneratedField::ListDir => {
                            if list_dir__.is_some() {
                                return Err(serde::de::Error::duplicate_field("listDir"));
                            }
                            list_dir__ = map_.next_value()?;
                        }
                        GeneratedField::Permissions => {
                            if permissions__.is_some() {
                                return Err(serde::de::Error::duplicate_field("permissions"));
                            }
                            permissions__ = map_.next_value()?;
                        }
                        GeneratedField::GenerateImage => {
                            if generate_image__.is_some() {
                                return Err(serde::de::Error::duplicate_field("generateImage"));
                            }
                            generate_image__ = map_.next_value()?;
                        }
                        GeneratedField::SearchWeb => {
                            if search_web__.is_some() {
                                return Err(serde::de::Error::duplicate_field("searchWeb"));
                            }
                            search_web__ = map_.next_value()?;
                        }
                        GeneratedField::ReadUrlContent => {
                            if read_url_content__.is_some() {
                                return Err(serde::de::Error::duplicate_field("readUrlContent"));
                            }
                            read_url_content__ = map_.next_value()?;
                        }
                        GeneratedField::ToolSearchConfig => {
                            if tool_search_config__.is_some() {
                                return Err(serde::de::Error::duplicate_field("toolSearchConfig"));
                            }
                            tool_search_config__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(HarnessSideTools {
                    find: find__,
                    run_command: run_command__,
                    subagents: subagents__,
                    user_questions: user_questions__,
                    file_edit: file_edit__,
                    view_file: view_file__,
                    write_to_file: write_to_file__,
                    grep_search: grep_search__,
                    list_dir: list_dir__,
                    permissions: permissions__,
                    generate_image: generate_image__,
                    search_web: search_web__,
                    read_url_content: read_url_content__,
                    tool_search_config: tool_search_config__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.HarnessSideTools", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for InitializeConversationEvent {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.config.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.InitializeConversationEvent", len)?;
        if let Some(v) = self.config.as_ref() {
            struct_ser.serialize_field("config", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for InitializeConversationEvent {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "config",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Config,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "config" => Ok(GeneratedField::Config),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = InitializeConversationEvent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.InitializeConversationEvent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<InitializeConversationEvent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut config__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Config => {
                            if config__.is_some() {
                                return Err(serde::de::Error::duplicate_field("config"));
                            }
                            config__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(InitializeConversationEvent {
                    config: config__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.InitializeConversationEvent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for InitializeConversationResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.cascade_id.is_some() {
            len += 1;
        }
        if !self.history.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.InitializeConversationResponse", len)?;
        if let Some(v) = self.cascade_id.as_ref() {
            struct_ser.serialize_field("cascadeId", v)?;
        }
        if !self.history.is_empty() {
            struct_ser.serialize_field("history", &self.history)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for InitializeConversationResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "cascade_id",
            "cascadeId",
            "history",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            CascadeId,
            History,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "cascadeId" | "cascade_id" => Ok(GeneratedField::CascadeId),
                            "history" => Ok(GeneratedField::History),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = InitializeConversationResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.InitializeConversationResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<InitializeConversationResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut cascade_id__ = None;
                let mut history__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::CascadeId => {
                            if cascade_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("cascadeId"));
                            }
                            cascade_id__ = map_.next_value()?;
                        }
                        GeneratedField::History => {
                            if history__.is_some() {
                                return Err(serde::de::Error::duplicate_field("history"));
                            }
                            history__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(InitializeConversationResponse {
                    cascade_id: cascade_id__,
                    history: history__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.InitializeConversationResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for InputConfig {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.storage_directory.is_some() {
            len += 1;
        }
        if self.port.is_some() {
            len += 1;
        }
        if self.bind_address.is_some() {
            len += 1;
        }
        if self.client_info.is_some() {
            len += 1;
        }
        if !self.env.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.InputConfig", len)?;
        if let Some(v) = self.storage_directory.as_ref() {
            struct_ser.serialize_field("storageDirectory", v)?;
        }
        if let Some(v) = self.port.as_ref() {
            struct_ser.serialize_field("port", v)?;
        }
        if let Some(v) = self.bind_address.as_ref() {
            struct_ser.serialize_field("bindAddress", v)?;
        }
        if let Some(v) = self.client_info.as_ref() {
            struct_ser.serialize_field("clientInfo", v)?;
        }
        if !self.env.is_empty() {
            struct_ser.serialize_field("env", &self.env)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for InputConfig {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "storage_directory",
            "storageDirectory",
            "port",
            "bind_address",
            "bindAddress",
            "client_info",
            "clientInfo",
            "env",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            StorageDirectory,
            Port,
            BindAddress,
            ClientInfo,
            Env,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "storageDirectory" | "storage_directory" => Ok(GeneratedField::StorageDirectory),
                            "port" => Ok(GeneratedField::Port),
                            "bindAddress" | "bind_address" => Ok(GeneratedField::BindAddress),
                            "clientInfo" | "client_info" => Ok(GeneratedField::ClientInfo),
                            "env" => Ok(GeneratedField::Env),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = InputConfig;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.InputConfig")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<InputConfig, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut storage_directory__ = None;
                let mut port__ = None;
                let mut bind_address__ = None;
                let mut client_info__ = None;
                let mut env__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::StorageDirectory => {
                            if storage_directory__.is_some() {
                                return Err(serde::de::Error::duplicate_field("storageDirectory"));
                            }
                            storage_directory__ = map_.next_value()?;
                        }
                        GeneratedField::Port => {
                            if port__.is_some() {
                                return Err(serde::de::Error::duplicate_field("port"));
                            }
                            port__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::BindAddress => {
                            if bind_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("bindAddress"));
                            }
                            bind_address__ = map_.next_value()?;
                        }
                        GeneratedField::ClientInfo => {
                            if client_info__.is_some() {
                                return Err(serde::de::Error::duplicate_field("clientInfo"));
                            }
                            client_info__ = map_.next_value()?;
                        }
                        GeneratedField::Env => {
                            if env__.is_some() {
                                return Err(serde::de::Error::duplicate_field("env"));
                            }
                            env__ = Some(
                                map_.next_value::<std::collections::HashMap<_, _>>()?
                            );
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(InputConfig {
                    storage_directory: storage_directory__,
                    port: port__,
                    bind_address: bind_address__,
                    client_info: client_info__,
                    env: env__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.InputConfig", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for InputEvent {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.event.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.InputEvent", len)?;
        if let Some(v) = self.event.as_ref() {
            match v {
                input_event::Event::UserInput(v) => {
                    struct_ser.serialize_field("userInput", v)?;
                }
                input_event::Event::ComplexUserInput(v) => {
                    struct_ser.serialize_field("complexUserInput", v)?;
                }
                input_event::Event::ToolConfirmation(v) => {
                    struct_ser.serialize_field("toolConfirmation", v)?;
                }
                input_event::Event::ToolResponse(v) => {
                    struct_ser.serialize_field("toolResponse", v)?;
                }
                input_event::Event::QuestionResponse(v) => {
                    struct_ser.serialize_field("questionResponse", v)?;
                }
                input_event::Event::HaltRequest(v) => {
                    struct_ser.serialize_field("haltRequest", v)?;
                }
                input_event::Event::AutomatedTrigger(v) => {
                    struct_ser.serialize_field("automatedTrigger", v)?;
                }
                input_event::Event::CallHookResponse(v) => {
                    struct_ser.serialize_field("callHookResponse", v)?;
                }
                input_event::Event::SessionEndRequest(v) => {
                    struct_ser.serialize_field("sessionEndRequest", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for InputEvent {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "user_input",
            "userInput",
            "complex_user_input",
            "complexUserInput",
            "tool_confirmation",
            "toolConfirmation",
            "tool_response",
            "toolResponse",
            "question_response",
            "questionResponse",
            "halt_request",
            "haltRequest",
            "automated_trigger",
            "automatedTrigger",
            "call_hook_response",
            "callHookResponse",
            "session_end_request",
            "sessionEndRequest",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            UserInput,
            ComplexUserInput,
            ToolConfirmation,
            ToolResponse,
            QuestionResponse,
            HaltRequest,
            AutomatedTrigger,
            CallHookResponse,
            SessionEndRequest,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "userInput" | "user_input" => Ok(GeneratedField::UserInput),
                            "complexUserInput" | "complex_user_input" => Ok(GeneratedField::ComplexUserInput),
                            "toolConfirmation" | "tool_confirmation" => Ok(GeneratedField::ToolConfirmation),
                            "toolResponse" | "tool_response" => Ok(GeneratedField::ToolResponse),
                            "questionResponse" | "question_response" => Ok(GeneratedField::QuestionResponse),
                            "haltRequest" | "halt_request" => Ok(GeneratedField::HaltRequest),
                            "automatedTrigger" | "automated_trigger" => Ok(GeneratedField::AutomatedTrigger),
                            "callHookResponse" | "call_hook_response" => Ok(GeneratedField::CallHookResponse),
                            "sessionEndRequest" | "session_end_request" => Ok(GeneratedField::SessionEndRequest),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = InputEvent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.InputEvent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<InputEvent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut event__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::UserInput => {
                            if event__.is_some() {
                                return Err(serde::de::Error::duplicate_field("userInput"));
                            }
                            event__ = map_.next_value::<::std::option::Option<_>>()?.map(input_event::Event::UserInput);
                        }
                        GeneratedField::ComplexUserInput => {
                            if event__.is_some() {
                                return Err(serde::de::Error::duplicate_field("complexUserInput"));
                            }
                            event__ = map_.next_value::<::std::option::Option<_>>()?.map(input_event::Event::ComplexUserInput)
;
                        }
                        GeneratedField::ToolConfirmation => {
                            if event__.is_some() {
                                return Err(serde::de::Error::duplicate_field("toolConfirmation"));
                            }
                            event__ = map_.next_value::<::std::option::Option<_>>()?.map(input_event::Event::ToolConfirmation)
;
                        }
                        GeneratedField::ToolResponse => {
                            if event__.is_some() {
                                return Err(serde::de::Error::duplicate_field("toolResponse"));
                            }
                            event__ = map_.next_value::<::std::option::Option<_>>()?.map(input_event::Event::ToolResponse)
;
                        }
                        GeneratedField::QuestionResponse => {
                            if event__.is_some() {
                                return Err(serde::de::Error::duplicate_field("questionResponse"));
                            }
                            event__ = map_.next_value::<::std::option::Option<_>>()?.map(input_event::Event::QuestionResponse)
;
                        }
                        GeneratedField::HaltRequest => {
                            if event__.is_some() {
                                return Err(serde::de::Error::duplicate_field("haltRequest"));
                            }
                            event__ = map_.next_value::<::std::option::Option<_>>()?.map(input_event::Event::HaltRequest);
                        }
                        GeneratedField::AutomatedTrigger => {
                            if event__.is_some() {
                                return Err(serde::de::Error::duplicate_field("automatedTrigger"));
                            }
                            event__ = map_.next_value::<::std::option::Option<_>>()?.map(input_event::Event::AutomatedTrigger);
                        }
                        GeneratedField::CallHookResponse => {
                            if event__.is_some() {
                                return Err(serde::de::Error::duplicate_field("callHookResponse"));
                            }
                            event__ = map_.next_value::<::std::option::Option<_>>()?.map(input_event::Event::CallHookResponse)
;
                        }
                        GeneratedField::SessionEndRequest => {
                            if event__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sessionEndRequest"));
                            }
                            event__ = map_.next_value::<::std::option::Option<_>>()?.map(input_event::Event::SessionEndRequest);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(InputEvent {
                    event: event__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.InputEvent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for LifecycleHook {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "LIFECYCLE_HOOK_UNSPECIFIED",
            Self::OnSessionStart => "LIFECYCLE_HOOK_ON_SESSION_START",
            Self::OnSessionEnd => "LIFECYCLE_HOOK_ON_SESSION_END",
            Self::PreTurn => "LIFECYCLE_HOOK_PRE_TURN",
            Self::PostTurn => "LIFECYCLE_HOOK_POST_TURN",
            Self::PreTool => "LIFECYCLE_HOOK_PRE_TOOL",
            Self::PostTool => "LIFECYCLE_HOOK_POST_TOOL",
            Self::OnToolError => "LIFECYCLE_HOOK_ON_TOOL_ERROR",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for LifecycleHook {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "LIFECYCLE_HOOK_UNSPECIFIED",
            "LIFECYCLE_HOOK_ON_SESSION_START",
            "LIFECYCLE_HOOK_ON_SESSION_END",
            "LIFECYCLE_HOOK_PRE_TURN",
            "LIFECYCLE_HOOK_POST_TURN",
            "LIFECYCLE_HOOK_PRE_TOOL",
            "LIFECYCLE_HOOK_POST_TOOL",
            "LIFECYCLE_HOOK_ON_TOOL_ERROR",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = LifecycleHook;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "LIFECYCLE_HOOK_UNSPECIFIED" => Ok(LifecycleHook::Unspecified),
                    "LIFECYCLE_HOOK_ON_SESSION_START" => Ok(LifecycleHook::OnSessionStart),
                    "LIFECYCLE_HOOK_ON_SESSION_END" => Ok(LifecycleHook::OnSessionEnd),
                    "LIFECYCLE_HOOK_PRE_TURN" => Ok(LifecycleHook::PreTurn),
                    "LIFECYCLE_HOOK_POST_TURN" => Ok(LifecycleHook::PostTurn),
                    "LIFECYCLE_HOOK_PRE_TOOL" => Ok(LifecycleHook::PreTool),
                    "LIFECYCLE_HOOK_POST_TOOL" => Ok(LifecycleHook::PostTool),
                    "LIFECYCLE_HOOK_ON_TOOL_ERROR" => Ok(LifecycleHook::OnToolError),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for ListDirToolConfig {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.enabled.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.ListDirToolConfig", len)?;
        if let Some(v) = self.enabled.as_ref() {
            struct_ser.serialize_field("enabled", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ListDirToolConfig {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "enabled",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Enabled,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "enabled" => Ok(GeneratedField::Enabled),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ListDirToolConfig;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.ListDirToolConfig")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ListDirToolConfig, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut enabled__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Enabled => {
                            if enabled__.is_some() {
                                return Err(serde::de::Error::duplicate_field("enabled"));
                            }
                            enabled__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ListDirToolConfig {
                    enabled: enabled__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.ListDirToolConfig", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for McpHttpTransport {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.url.is_some() {
            len += 1;
        }
        if !self.headers.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.McpHttpTransport", len)?;
        if let Some(v) = self.url.as_ref() {
            struct_ser.serialize_field("url", v)?;
        }
        if !self.headers.is_empty() {
            struct_ser.serialize_field("headers", &self.headers)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for McpHttpTransport {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "url",
            "headers",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Url,
            Headers,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "url" => Ok(GeneratedField::Url),
                            "headers" => Ok(GeneratedField::Headers),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = McpHttpTransport;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.McpHttpTransport")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<McpHttpTransport, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut url__ = None;
                let mut headers__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Url => {
                            if url__.is_some() {
                                return Err(serde::de::Error::duplicate_field("url"));
                            }
                            url__ = map_.next_value()?;
                        }
                        GeneratedField::Headers => {
                            if headers__.is_some() {
                                return Err(serde::de::Error::duplicate_field("headers"));
                            }
                            headers__ = Some(
                                map_.next_value::<std::collections::HashMap<_, _>>()?
                            );
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(McpHttpTransport {
                    url: url__,
                    headers: headers__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.McpHttpTransport", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for McpServerConfig {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.name.is_some() {
            len += 1;
        }
        if !self.enabled_tools.is_empty() {
            len += 1;
        }
        if !self.disabled_tools.is_empty() {
            len += 1;
        }
        if self.auth_provider_type.is_some() {
            len += 1;
        }
        if self.timeout_seconds.is_some() {
            len += 1;
        }
        if self.transport.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.McpServerConfig", len)?;
        if let Some(v) = self.name.as_ref() {
            struct_ser.serialize_field("name", v)?;
        }
        if !self.enabled_tools.is_empty() {
            struct_ser.serialize_field("enabledTools", &self.enabled_tools)?;
        }
        if !self.disabled_tools.is_empty() {
            struct_ser.serialize_field("disabledTools", &self.disabled_tools)?;
        }
        if let Some(v) = self.auth_provider_type.as_ref() {
            let v = mcp_server_config::AuthProviderType::try_from(*v)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", *v)))?;
            struct_ser.serialize_field("authProviderType", &v)?;
        }
        if let Some(v) = self.timeout_seconds.as_ref() {
            struct_ser.serialize_field("timeoutSeconds", v)?;
        }
        if let Some(v) = self.transport.as_ref() {
            match v {
                mcp_server_config::Transport::Stdio(v) => {
                    struct_ser.serialize_field("stdio", v)?;
                }
                mcp_server_config::Transport::Http(v) => {
                    struct_ser.serialize_field("http", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for McpServerConfig {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "name",
            "enabled_tools",
            "enabledTools",
            "disabled_tools",
            "disabledTools",
            "auth_provider_type",
            "authProviderType",
            "timeout_seconds",
            "timeoutSeconds",
            "stdio",
            "http",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Name,
            EnabledTools,
            DisabledTools,
            AuthProviderType,
            TimeoutSeconds,
            Stdio,
            Http,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "name" => Ok(GeneratedField::Name),
                            "enabledTools" | "enabled_tools" => Ok(GeneratedField::EnabledTools),
                            "disabledTools" | "disabled_tools" => Ok(GeneratedField::DisabledTools),
                            "authProviderType" | "auth_provider_type" => Ok(GeneratedField::AuthProviderType),
                            "timeoutSeconds" | "timeout_seconds" => Ok(GeneratedField::TimeoutSeconds),
                            "stdio" => Ok(GeneratedField::Stdio),
                            "http" => Ok(GeneratedField::Http),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = McpServerConfig;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.McpServerConfig")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<McpServerConfig, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut name__ = None;
                let mut enabled_tools__ = None;
                let mut disabled_tools__ = None;
                let mut auth_provider_type__ = None;
                let mut timeout_seconds__ = None;
                let mut transport__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = map_.next_value()?;
                        }
                        GeneratedField::EnabledTools => {
                            if enabled_tools__.is_some() {
                                return Err(serde::de::Error::duplicate_field("enabledTools"));
                            }
                            enabled_tools__ = Some(map_.next_value()?);
                        }
                        GeneratedField::DisabledTools => {
                            if disabled_tools__.is_some() {
                                return Err(serde::de::Error::duplicate_field("disabledTools"));
                            }
                            disabled_tools__ = Some(map_.next_value()?);
                        }
                        GeneratedField::AuthProviderType => {
                            if auth_provider_type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("authProviderType"));
                            }
                            auth_provider_type__ = map_.next_value::<::std::option::Option<mcp_server_config::AuthProviderType>>()?.map(|x| x as i32);
                        }
                        GeneratedField::TimeoutSeconds => {
                            if timeout_seconds__.is_some() {
                                return Err(serde::de::Error::duplicate_field("timeoutSeconds"));
                            }
                            timeout_seconds__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::Stdio => {
                            if transport__.is_some() {
                                return Err(serde::de::Error::duplicate_field("stdio"));
                            }
                            transport__ = map_.next_value::<::std::option::Option<_>>()?.map(mcp_server_config::Transport::Stdio)
;
                        }
                        GeneratedField::Http => {
                            if transport__.is_some() {
                                return Err(serde::de::Error::duplicate_field("http"));
                            }
                            transport__ = map_.next_value::<::std::option::Option<_>>()?.map(mcp_server_config::Transport::Http)
;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(McpServerConfig {
                    name: name__,
                    enabled_tools: enabled_tools__.unwrap_or_default(),
                    disabled_tools: disabled_tools__.unwrap_or_default(),
                    auth_provider_type: auth_provider_type__,
                    timeout_seconds: timeout_seconds__,
                    transport: transport__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.McpServerConfig", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for mcp_server_config::AuthProviderType {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "AUTH_PROVIDER_TYPE_UNSPECIFIED",
            Self::GoogleCredentials => "AUTH_PROVIDER_TYPE_GOOGLE_CREDENTIALS",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for mcp_server_config::AuthProviderType {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "AUTH_PROVIDER_TYPE_UNSPECIFIED",
            "AUTH_PROVIDER_TYPE_GOOGLE_CREDENTIALS",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = mcp_server_config::AuthProviderType;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "AUTH_PROVIDER_TYPE_UNSPECIFIED" => Ok(mcp_server_config::AuthProviderType::Unspecified),
                    "AUTH_PROVIDER_TYPE_GOOGLE_CREDENTIALS" => Ok(mcp_server_config::AuthProviderType::GoogleCredentials),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for McpStdioTransport {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.command.is_some() {
            len += 1;
        }
        if !self.args.is_empty() {
            len += 1;
        }
        if !self.env.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.McpStdioTransport", len)?;
        if let Some(v) = self.command.as_ref() {
            struct_ser.serialize_field("command", v)?;
        }
        if !self.args.is_empty() {
            struct_ser.serialize_field("args", &self.args)?;
        }
        if !self.env.is_empty() {
            struct_ser.serialize_field("env", &self.env)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for McpStdioTransport {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "command",
            "args",
            "env",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Command,
            Args,
            Env,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "command" => Ok(GeneratedField::Command),
                            "args" => Ok(GeneratedField::Args),
                            "env" => Ok(GeneratedField::Env),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = McpStdioTransport;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.McpStdioTransport")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<McpStdioTransport, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut command__ = None;
                let mut args__ = None;
                let mut env__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Command => {
                            if command__.is_some() {
                                return Err(serde::de::Error::duplicate_field("command"));
                            }
                            command__ = map_.next_value()?;
                        }
                        GeneratedField::Args => {
                            if args__.is_some() {
                                return Err(serde::de::Error::duplicate_field("args"));
                            }
                            args__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Env => {
                            if env__.is_some() {
                                return Err(serde::de::Error::duplicate_field("env"));
                            }
                            env__ = Some(
                                map_.next_value::<std::collections::HashMap<_, _>>()?
                            );
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(McpStdioTransport {
                    command: command__,
                    args: args__.unwrap_or_default(),
                    env: env__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.McpStdioTransport", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Media {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.mime_type.is_some() {
            len += 1;
        }
        if self.description.is_some() {
            len += 1;
        }
        if self.data.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.Media", len)?;
        if let Some(v) = self.mime_type.as_ref() {
            struct_ser.serialize_field("mimeType", v)?;
        }
        if let Some(v) = self.description.as_ref() {
            struct_ser.serialize_field("description", v)?;
        }
        if let Some(v) = self.data.as_ref() {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("data", pbjson::private::base64::encode(&v).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Media {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "mime_type",
            "mimeType",
            "description",
            "data",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            MimeType,
            Description,
            Data,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "mimeType" | "mime_type" => Ok(GeneratedField::MimeType),
                            "description" => Ok(GeneratedField::Description),
                            "data" => Ok(GeneratedField::Data),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Media;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.Media")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Media, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut mime_type__ = None;
                let mut description__ = None;
                let mut data__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::MimeType => {
                            if mime_type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("mimeType"));
                            }
                            mime_type__ = map_.next_value()?;
                        }
                        GeneratedField::Description => {
                            if description__.is_some() {
                                return Err(serde::de::Error::duplicate_field("description"));
                            }
                            description__ = map_.next_value()?;
                        }
                        GeneratedField::Data => {
                            if data__.is_some() {
                                return Err(serde::de::Error::duplicate_field("data"));
                            }
                            data__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::BytesDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(Media {
                    mime_type: mime_type__,
                    description: description__,
                    data: data__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.Media", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ModelApiRetryConfig {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.max_retries.is_some() {
            len += 1;
        }
        if self.initial_sleep_duration_ms.is_some() {
            len += 1;
        }
        if self.exponential_multiplier.is_some() {
            len += 1;
        }
        if self.jitter_range.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.ModelAPIRetryConfig", len)?;
        if let Some(v) = self.max_retries.as_ref() {
            struct_ser.serialize_field("maxRetries", v)?;
        }
        if let Some(v) = self.initial_sleep_duration_ms.as_ref() {
            struct_ser.serialize_field("initialSleepDurationMs", v)?;
        }
        if let Some(v) = self.exponential_multiplier.as_ref() {
            struct_ser.serialize_field("exponentialMultiplier", v)?;
        }
        if let Some(v) = self.jitter_range.as_ref() {
            struct_ser.serialize_field("jitterRange", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ModelApiRetryConfig {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "max_retries",
            "maxRetries",
            "initial_sleep_duration_ms",
            "initialSleepDurationMs",
            "exponential_multiplier",
            "exponentialMultiplier",
            "jitter_range",
            "jitterRange",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            MaxRetries,
            InitialSleepDurationMs,
            ExponentialMultiplier,
            JitterRange,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "maxRetries" | "max_retries" => Ok(GeneratedField::MaxRetries),
                            "initialSleepDurationMs" | "initial_sleep_duration_ms" => Ok(GeneratedField::InitialSleepDurationMs),
                            "exponentialMultiplier" | "exponential_multiplier" => Ok(GeneratedField::ExponentialMultiplier),
                            "jitterRange" | "jitter_range" => Ok(GeneratedField::JitterRange),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ModelApiRetryConfig;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.ModelAPIRetryConfig")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ModelApiRetryConfig, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut max_retries__ = None;
                let mut initial_sleep_duration_ms__ = None;
                let mut exponential_multiplier__ = None;
                let mut jitter_range__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::MaxRetries => {
                            if max_retries__.is_some() {
                                return Err(serde::de::Error::duplicate_field("maxRetries"));
                            }
                            max_retries__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::InitialSleepDurationMs => {
                            if initial_sleep_duration_ms__.is_some() {
                                return Err(serde::de::Error::duplicate_field("initialSleepDurationMs"));
                            }
                            initial_sleep_duration_ms__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::ExponentialMultiplier => {
                            if exponential_multiplier__.is_some() {
                                return Err(serde::de::Error::duplicate_field("exponentialMultiplier"));
                            }
                            exponential_multiplier__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::JitterRange => {
                            if jitter_range__.is_some() {
                                return Err(serde::de::Error::duplicate_field("jitterRange"));
                            }
                            jitter_range__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ModelApiRetryConfig {
                    max_retries: max_retries__,
                    initial_sleep_duration_ms: initial_sleep_duration_ms__,
                    exponential_multiplier: exponential_multiplier__,
                    jitter_range: jitter_range__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.ModelAPIRetryConfig", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ModelConfig {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.name.is_some() {
            len += 1;
        }
        if !self.types.is_empty() {
            len += 1;
        }
        if self.endpoint.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.ModelConfig", len)?;
        if let Some(v) = self.name.as_ref() {
            struct_ser.serialize_field("name", v)?;
        }
        if !self.types.is_empty() {
            let v = self.types.iter().cloned().map(|v| {
                ModelType::try_from(v)
                    .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", v)))
                }).collect::<Result<Vec<_>, _>>()?;
            struct_ser.serialize_field("types", &v)?;
        }
        if let Some(v) = self.endpoint.as_ref() {
            match v {
                model_config::Endpoint::GeminiApiEndpoint(v) => {
                    struct_ser.serialize_field("geminiApiEndpoint", v)?;
                }
                model_config::Endpoint::VertexEndpoint(v) => {
                    struct_ser.serialize_field("vertexEndpoint", v)?;
                }
                model_config::Endpoint::GemmaEndpoint(v) => {
                    struct_ser.serialize_field("gemmaEndpoint", v)?;
                }
                model_config::Endpoint::CustomEndpoint(v) => {
                    struct_ser.serialize_field("customEndpoint", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ModelConfig {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "name",
            "types",
            "gemini_api_endpoint",
            "geminiApiEndpoint",
            "vertex_endpoint",
            "vertexEndpoint",
            "gemma_endpoint",
            "gemmaEndpoint",
            "custom_endpoint",
            "customEndpoint",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Name,
            Types,
            GeminiApiEndpoint,
            VertexEndpoint,
            GemmaEndpoint,
            CustomEndpoint,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "name" => Ok(GeneratedField::Name),
                            "types" => Ok(GeneratedField::Types),
                            "geminiApiEndpoint" | "gemini_api_endpoint" => Ok(GeneratedField::GeminiApiEndpoint),
                            "vertexEndpoint" | "vertex_endpoint" => Ok(GeneratedField::VertexEndpoint),
                            "gemmaEndpoint" | "gemma_endpoint" => Ok(GeneratedField::GemmaEndpoint),
                            "customEndpoint" | "custom_endpoint" => Ok(GeneratedField::CustomEndpoint),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ModelConfig;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.ModelConfig")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ModelConfig, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut name__ = None;
                let mut types__ = None;
                let mut endpoint__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = map_.next_value()?;
                        }
                        GeneratedField::Types => {
                            if types__.is_some() {
                                return Err(serde::de::Error::duplicate_field("types"));
                            }
                            types__ = Some(map_.next_value::<Vec<ModelType>>()?.into_iter().map(|x| x as i32).collect());
                        }
                        GeneratedField::GeminiApiEndpoint => {
                            if endpoint__.is_some() {
                                return Err(serde::de::Error::duplicate_field("geminiApiEndpoint"));
                            }
                            endpoint__ = map_.next_value::<::std::option::Option<_>>()?.map(model_config::Endpoint::GeminiApiEndpoint)
;
                        }
                        GeneratedField::VertexEndpoint => {
                            if endpoint__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vertexEndpoint"));
                            }
                            endpoint__ = map_.next_value::<::std::option::Option<_>>()?.map(model_config::Endpoint::VertexEndpoint)
;
                        }
                        GeneratedField::GemmaEndpoint => {
                            if endpoint__.is_some() {
                                return Err(serde::de::Error::duplicate_field("gemmaEndpoint"));
                            }
                            endpoint__ = map_.next_value::<::std::option::Option<_>>()?.map(model_config::Endpoint::GemmaEndpoint)
;
                        }
                        GeneratedField::CustomEndpoint => {
                            if endpoint__.is_some() {
                                return Err(serde::de::Error::duplicate_field("customEndpoint"));
                            }
                            endpoint__ = map_.next_value::<::std::option::Option<_>>()?.map(model_config::Endpoint::CustomEndpoint)
;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ModelConfig {
                    name: name__,
                    types: types__.unwrap_or_default(),
                    endpoint: endpoint__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.ModelConfig", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ModelOutputRetryConfig {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.max_retries.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.ModelOutputRetryConfig", len)?;
        if let Some(v) = self.max_retries.as_ref() {
            struct_ser.serialize_field("maxRetries", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ModelOutputRetryConfig {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "max_retries",
            "maxRetries",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            MaxRetries,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "maxRetries" | "max_retries" => Ok(GeneratedField::MaxRetries),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ModelOutputRetryConfig;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.ModelOutputRetryConfig")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ModelOutputRetryConfig, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut max_retries__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::MaxRetries => {
                            if max_retries__.is_some() {
                                return Err(serde::de::Error::duplicate_field("maxRetries"));
                            }
                            max_retries__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ModelOutputRetryConfig {
                    max_retries: max_retries__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.ModelOutputRetryConfig", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ModelType {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "MODEL_TYPE_UNSPECIFIED",
            Self::Text => "MODEL_TYPE_TEXT",
            Self::Image => "MODEL_TYPE_IMAGE",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for ModelType {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "MODEL_TYPE_UNSPECIFIED",
            "MODEL_TYPE_TEXT",
            "MODEL_TYPE_IMAGE",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ModelType;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "MODEL_TYPE_UNSPECIFIED" => Ok(ModelType::Unspecified),
                    "MODEL_TYPE_TEXT" => Ok(ModelType::Text),
                    "MODEL_TYPE_IMAGE" => Ok(ModelType::Image),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for MultipleChoice {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.question.is_some() {
            len += 1;
        }
        if !self.choices.is_empty() {
            len += 1;
        }
        if self.is_multi_select.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.MultipleChoice", len)?;
        if let Some(v) = self.question.as_ref() {
            struct_ser.serialize_field("question", v)?;
        }
        if !self.choices.is_empty() {
            struct_ser.serialize_field("choices", &self.choices)?;
        }
        if let Some(v) = self.is_multi_select.as_ref() {
            struct_ser.serialize_field("isMultiSelect", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MultipleChoice {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "question",
            "choices",
            "is_multi_select",
            "isMultiSelect",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Question,
            Choices,
            IsMultiSelect,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "question" => Ok(GeneratedField::Question),
                            "choices" => Ok(GeneratedField::Choices),
                            "isMultiSelect" | "is_multi_select" => Ok(GeneratedField::IsMultiSelect),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MultipleChoice;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.MultipleChoice")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<MultipleChoice, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut question__ = None;
                let mut choices__ = None;
                let mut is_multi_select__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Question => {
                            if question__.is_some() {
                                return Err(serde::de::Error::duplicate_field("question"));
                            }
                            question__ = map_.next_value()?;
                        }
                        GeneratedField::Choices => {
                            if choices__.is_some() {
                                return Err(serde::de::Error::duplicate_field("choices"));
                            }
                            choices__ = Some(map_.next_value()?);
                        }
                        GeneratedField::IsMultiSelect => {
                            if is_multi_select__.is_some() {
                                return Err(serde::de::Error::duplicate_field("isMultiSelect"));
                            }
                            is_multi_select__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(MultipleChoice {
                    question: question__,
                    choices: choices__.unwrap_or_default(),
                    is_multi_select: is_multi_select__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.MultipleChoice", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MultipleChoiceAnswer {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.selected_choice_indices.is_empty() {
            len += 1;
        }
        if self.freeform_response.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.MultipleChoiceAnswer", len)?;
        if !self.selected_choice_indices.is_empty() {
            struct_ser.serialize_field("selectedChoiceIndices", &self.selected_choice_indices)?;
        }
        if let Some(v) = self.freeform_response.as_ref() {
            struct_ser.serialize_field("freeformResponse", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MultipleChoiceAnswer {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "selected_choice_indices",
            "selectedChoiceIndices",
            "freeform_response",
            "freeformResponse",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            SelectedChoiceIndices,
            FreeformResponse,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "selectedChoiceIndices" | "selected_choice_indices" => Ok(GeneratedField::SelectedChoiceIndices),
                            "freeformResponse" | "freeform_response" => Ok(GeneratedField::FreeformResponse),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MultipleChoiceAnswer;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.MultipleChoiceAnswer")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<MultipleChoiceAnswer, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut selected_choice_indices__ = None;
                let mut freeform_response__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::SelectedChoiceIndices => {
                            if selected_choice_indices__.is_some() {
                                return Err(serde::de::Error::duplicate_field("selectedChoiceIndices"));
                            }
                            selected_choice_indices__ = 
                                Some(map_.next_value::<Vec<::pbjson::private::NumberDeserialize<_>>>()?
                                    .into_iter().map(|x| x.0).collect())
                            ;
                        }
                        GeneratedField::FreeformResponse => {
                            if freeform_response__.is_some() {
                                return Err(serde::de::Error::duplicate_field("freeformResponse"));
                            }
                            freeform_response__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(MultipleChoiceAnswer {
                    selected_choice_indices: selected_choice_indices__.unwrap_or_default(),
                    freeform_response: freeform_response__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.MultipleChoiceAnswer", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for OnToolErrorArgs {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.tool_name.is_some() {
            len += 1;
        }
        if self.error_message.is_some() {
            len += 1;
        }
        if self.server_name.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.OnToolErrorArgs", len)?;
        if let Some(v) = self.tool_name.as_ref() {
            struct_ser.serialize_field("toolName", v)?;
        }
        if let Some(v) = self.error_message.as_ref() {
            struct_ser.serialize_field("errorMessage", v)?;
        }
        if let Some(v) = self.server_name.as_ref() {
            struct_ser.serialize_field("serverName", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for OnToolErrorArgs {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "tool_name",
            "toolName",
            "error_message",
            "errorMessage",
            "server_name",
            "serverName",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ToolName,
            ErrorMessage,
            ServerName,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "toolName" | "tool_name" => Ok(GeneratedField::ToolName),
                            "errorMessage" | "error_message" => Ok(GeneratedField::ErrorMessage),
                            "serverName" | "server_name" => Ok(GeneratedField::ServerName),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = OnToolErrorArgs;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.OnToolErrorArgs")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<OnToolErrorArgs, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut tool_name__ = None;
                let mut error_message__ = None;
                let mut server_name__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ToolName => {
                            if tool_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("toolName"));
                            }
                            tool_name__ = map_.next_value()?;
                        }
                        GeneratedField::ErrorMessage => {
                            if error_message__.is_some() {
                                return Err(serde::de::Error::duplicate_field("errorMessage"));
                            }
                            error_message__ = map_.next_value()?;
                        }
                        GeneratedField::ServerName => {
                            if server_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("serverName"));
                            }
                            server_name__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(OnToolErrorArgs {
                    tool_name: tool_name__,
                    error_message: error_message__,
                    server_name: server_name__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.OnToolErrorArgs", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for OnToolErrorResult {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.custom_error_message.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.OnToolErrorResult", len)?;
        if let Some(v) = self.custom_error_message.as_ref() {
            struct_ser.serialize_field("customErrorMessage", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for OnToolErrorResult {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "custom_error_message",
            "customErrorMessage",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            CustomErrorMessage,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "customErrorMessage" | "custom_error_message" => Ok(GeneratedField::CustomErrorMessage),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = OnToolErrorResult;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.OnToolErrorResult")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<OnToolErrorResult, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut custom_error_message__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::CustomErrorMessage => {
                            if custom_error_message__.is_some() {
                                return Err(serde::de::Error::duplicate_field("customErrorMessage"));
                            }
                            custom_error_message__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(OnToolErrorResult {
                    custom_error_message: custom_error_message__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.OnToolErrorResult", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for OutputConfig {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.port.is_some() {
            len += 1;
        }
        if self.api_key.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.OutputConfig", len)?;
        if let Some(v) = self.port.as_ref() {
            struct_ser.serialize_field("port", v)?;
        }
        if let Some(v) = self.api_key.as_ref() {
            struct_ser.serialize_field("apiKey", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for OutputConfig {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "port",
            "api_key",
            "apiKey",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Port,
            ApiKey,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "port" => Ok(GeneratedField::Port),
                            "apiKey" | "api_key" => Ok(GeneratedField::ApiKey),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = OutputConfig;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.OutputConfig")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<OutputConfig, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut port__ = None;
                let mut api_key__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Port => {
                            if port__.is_some() {
                                return Err(serde::de::Error::duplicate_field("port"));
                            }
                            port__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::ApiKey => {
                            if api_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("apiKey"));
                            }
                            api_key__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(OutputConfig {
                    port: port__,
                    api_key: api_key__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.OutputConfig", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for OutputEvent {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.seq_num.is_some() {
            len += 1;
        }
        if self.timestamp_micros.is_some() {
            len += 1;
        }
        if self.usage_metadata.is_some() {
            len += 1;
        }
        if self.event.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.OutputEvent", len)?;
        if let Some(v) = self.seq_num.as_ref() {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("seqNum", ToString::to_string(&v).as_str())?;
        }
        if let Some(v) = self.timestamp_micros.as_ref() {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("timestampMicros", ToString::to_string(&v).as_str())?;
        }
        if let Some(v) = self.usage_metadata.as_ref() {
            struct_ser.serialize_field("usageMetadata", v)?;
        }
        if let Some(v) = self.event.as_ref() {
            match v {
                output_event::Event::StepUpdate(v) => {
                    struct_ser.serialize_field("stepUpdate", v)?;
                }
                output_event::Event::TrajectoryStateUpdate(v) => {
                    struct_ser.serialize_field("trajectoryStateUpdate", v)?;
                }
                output_event::Event::ToolCall(v) => {
                    struct_ser.serialize_field("toolCall", v)?;
                }
                output_event::Event::InitializeConversationResponse(v) => {
                    struct_ser.serialize_field("initializeConversationResponse", v)?;
                }
                output_event::Event::CallHookRequest(v) => {
                    struct_ser.serialize_field("callHookRequest", v)?;
                }
                output_event::Event::SessionEndResponse(v) => {
                    struct_ser.serialize_field("sessionEndResponse", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for OutputEvent {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "seq_num",
            "seqNum",
            "timestamp_micros",
            "timestampMicros",
            "usage_metadata",
            "usageMetadata",
            "step_update",
            "stepUpdate",
            "trajectory_state_update",
            "trajectoryStateUpdate",
            "tool_call",
            "toolCall",
            "initialize_conversation_response",
            "initializeConversationResponse",
            "call_hook_request",
            "callHookRequest",
            "session_end_response",
            "sessionEndResponse",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            SeqNum,
            TimestampMicros,
            UsageMetadata,
            StepUpdate,
            TrajectoryStateUpdate,
            ToolCall,
            InitializeConversationResponse,
            CallHookRequest,
            SessionEndResponse,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "seqNum" | "seq_num" => Ok(GeneratedField::SeqNum),
                            "timestampMicros" | "timestamp_micros" => Ok(GeneratedField::TimestampMicros),
                            "usageMetadata" | "usage_metadata" => Ok(GeneratedField::UsageMetadata),
                            "stepUpdate" | "step_update" => Ok(GeneratedField::StepUpdate),
                            "trajectoryStateUpdate" | "trajectory_state_update" => Ok(GeneratedField::TrajectoryStateUpdate),
                            "toolCall" | "tool_call" => Ok(GeneratedField::ToolCall),
                            "initializeConversationResponse" | "initialize_conversation_response" => Ok(GeneratedField::InitializeConversationResponse),
                            "callHookRequest" | "call_hook_request" => Ok(GeneratedField::CallHookRequest),
                            "sessionEndResponse" | "session_end_response" => Ok(GeneratedField::SessionEndResponse),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = OutputEvent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.OutputEvent")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<OutputEvent, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut seq_num__ = None;
                let mut timestamp_micros__ = None;
                let mut usage_metadata__ = None;
                let mut event__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::SeqNum => {
                            if seq_num__.is_some() {
                                return Err(serde::de::Error::duplicate_field("seqNum"));
                            }
                            seq_num__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::TimestampMicros => {
                            if timestamp_micros__.is_some() {
                                return Err(serde::de::Error::duplicate_field("timestampMicros"));
                            }
                            timestamp_micros__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::UsageMetadata => {
                            if usage_metadata__.is_some() {
                                return Err(serde::de::Error::duplicate_field("usageMetadata"));
                            }
                            usage_metadata__ = map_.next_value()?;
                        }
                        GeneratedField::StepUpdate => {
                            if event__.is_some() {
                                return Err(serde::de::Error::duplicate_field("stepUpdate"));
                            }
                            event__ = map_.next_value::<::std::option::Option<_>>()?.map(output_event::Event::StepUpdate)
;
                        }
                        GeneratedField::TrajectoryStateUpdate => {
                            if event__.is_some() {
                                return Err(serde::de::Error::duplicate_field("trajectoryStateUpdate"));
                            }
                            event__ = map_.next_value::<::std::option::Option<_>>()?.map(output_event::Event::TrajectoryStateUpdate)
;
                        }
                        GeneratedField::ToolCall => {
                            if event__.is_some() {
                                return Err(serde::de::Error::duplicate_field("toolCall"));
                            }
                            event__ = map_.next_value::<::std::option::Option<_>>()?.map(output_event::Event::ToolCall)
;
                        }
                        GeneratedField::InitializeConversationResponse => {
                            if event__.is_some() {
                                return Err(serde::de::Error::duplicate_field("initializeConversationResponse"));
                            }
                            event__ = map_.next_value::<::std::option::Option<_>>()?.map(output_event::Event::InitializeConversationResponse)
;
                        }
                        GeneratedField::CallHookRequest => {
                            if event__.is_some() {
                                return Err(serde::de::Error::duplicate_field("callHookRequest"));
                            }
                            event__ = map_.next_value::<::std::option::Option<_>>()?.map(output_event::Event::CallHookRequest)
;
                        }
                        GeneratedField::SessionEndResponse => {
                            if event__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sessionEndResponse"));
                            }
                            event__ = map_.next_value::<::std::option::Option<_>>()?.map(output_event::Event::SessionEndResponse);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(OutputEvent {
                    seq_num: seq_num__,
                    timestamp_micros: timestamp_micros__,
                    usage_metadata: usage_metadata__,
                    event: event__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.OutputEvent", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for PermissionsConfig {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.enforce_workspace_validation.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.PermissionsConfig", len)?;
        if let Some(v) = self.enforce_workspace_validation.as_ref() {
            struct_ser.serialize_field("enforceWorkspaceValidation", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for PermissionsConfig {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "enforce_workspace_validation",
            "enforceWorkspaceValidation",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            EnforceWorkspaceValidation,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "enforceWorkspaceValidation" | "enforce_workspace_validation" => Ok(GeneratedField::EnforceWorkspaceValidation),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = PermissionsConfig;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.PermissionsConfig")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<PermissionsConfig, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut enforce_workspace_validation__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::EnforceWorkspaceValidation => {
                            if enforce_workspace_validation__.is_some() {
                                return Err(serde::de::Error::duplicate_field("enforceWorkspaceValidation"));
                            }
                            enforce_workspace_validation__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(PermissionsConfig {
                    enforce_workspace_validation: enforce_workspace_validation__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.PermissionsConfig", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for PostToolArgs {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.tool_name.is_some() {
            len += 1;
        }
        if self.result.is_some() {
            len += 1;
        }
        if self.error.is_some() {
            len += 1;
        }
        if self.server_name.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.PostToolArgs", len)?;
        if let Some(v) = self.tool_name.as_ref() {
            struct_ser.serialize_field("toolName", v)?;
        }
        if let Some(v) = self.result.as_ref() {
            struct_ser.serialize_field("result", v)?;
        }
        if let Some(v) = self.error.as_ref() {
            struct_ser.serialize_field("error", v)?;
        }
        if let Some(v) = self.server_name.as_ref() {
            struct_ser.serialize_field("serverName", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for PostToolArgs {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "tool_name",
            "toolName",
            "result",
            "error",
            "server_name",
            "serverName",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ToolName,
            Result,
            Error,
            ServerName,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "toolName" | "tool_name" => Ok(GeneratedField::ToolName),
                            "result" => Ok(GeneratedField::Result),
                            "error" => Ok(GeneratedField::Error),
                            "serverName" | "server_name" => Ok(GeneratedField::ServerName),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = PostToolArgs;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.PostToolArgs")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<PostToolArgs, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut tool_name__ = None;
                let mut result__ = None;
                let mut error__ = None;
                let mut server_name__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ToolName => {
                            if tool_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("toolName"));
                            }
                            tool_name__ = map_.next_value()?;
                        }
                        GeneratedField::Result => {
                            if result__.is_some() {
                                return Err(serde::de::Error::duplicate_field("result"));
                            }
                            result__ = map_.next_value()?;
                        }
                        GeneratedField::Error => {
                            if error__.is_some() {
                                return Err(serde::de::Error::duplicate_field("error"));
                            }
                            error__ = map_.next_value()?;
                        }
                        GeneratedField::ServerName => {
                            if server_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("serverName"));
                            }
                            server_name__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(PostToolArgs {
                    tool_name: tool_name__,
                    result: result__,
                    error: error__,
                    server_name: server_name__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.PostToolArgs", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for PostTurnArgs {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.response_text.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.PostTurnArgs", len)?;
        if let Some(v) = self.response_text.as_ref() {
            struct_ser.serialize_field("responseText", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for PostTurnArgs {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "response_text",
            "responseText",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ResponseText,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "responseText" | "response_text" => Ok(GeneratedField::ResponseText),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = PostTurnArgs;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.PostTurnArgs")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<PostTurnArgs, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut response_text__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ResponseText => {
                            if response_text__.is_some() {
                                return Err(serde::de::Error::duplicate_field("responseText"));
                            }
                            response_text__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(PostTurnArgs {
                    response_text: response_text__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.PostTurnArgs", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for PreToolArgs {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.tool_name.is_some() {
            len += 1;
        }
        if self.arguments_json.is_some() {
            len += 1;
        }
        if self.server_name.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.PreToolArgs", len)?;
        if let Some(v) = self.tool_name.as_ref() {
            struct_ser.serialize_field("toolName", v)?;
        }
        if let Some(v) = self.arguments_json.as_ref() {
            struct_ser.serialize_field("argumentsJson", v)?;
        }
        if let Some(v) = self.server_name.as_ref() {
            struct_ser.serialize_field("serverName", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for PreToolArgs {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "tool_name",
            "toolName",
            "arguments_json",
            "argumentsJson",
            "server_name",
            "serverName",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ToolName,
            ArgumentsJson,
            ServerName,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "toolName" | "tool_name" => Ok(GeneratedField::ToolName),
                            "argumentsJson" | "arguments_json" => Ok(GeneratedField::ArgumentsJson),
                            "serverName" | "server_name" => Ok(GeneratedField::ServerName),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = PreToolArgs;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.PreToolArgs")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<PreToolArgs, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut tool_name__ = None;
                let mut arguments_json__ = None;
                let mut server_name__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ToolName => {
                            if tool_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("toolName"));
                            }
                            tool_name__ = map_.next_value()?;
                        }
                        GeneratedField::ArgumentsJson => {
                            if arguments_json__.is_some() {
                                return Err(serde::de::Error::duplicate_field("argumentsJson"));
                            }
                            arguments_json__ = map_.next_value()?;
                        }
                        GeneratedField::ServerName => {
                            if server_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("serverName"));
                            }
                            server_name__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(PreToolArgs {
                    tool_name: tool_name__,
                    arguments_json: arguments_json__,
                    server_name: server_name__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.PreToolArgs", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for PreToolResult {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.decision.is_some() {
            len += 1;
        }
        if self.reason.is_some() {
            len += 1;
        }
        if self.modified_arguments_json.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.PreToolResult", len)?;
        if let Some(v) = self.decision.as_ref() {
            let v = pre_tool_result::Decision::try_from(*v)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", *v)))?;
            struct_ser.serialize_field("decision", &v)?;
        }
        if let Some(v) = self.reason.as_ref() {
            struct_ser.serialize_field("reason", v)?;
        }
        if let Some(v) = self.modified_arguments_json.as_ref() {
            struct_ser.serialize_field("modifiedArgumentsJson", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for PreToolResult {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "decision",
            "reason",
            "modified_arguments_json",
            "modifiedArgumentsJson",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Decision,
            Reason,
            ModifiedArgumentsJson,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "decision" => Ok(GeneratedField::Decision),
                            "reason" => Ok(GeneratedField::Reason),
                            "modifiedArgumentsJson" | "modified_arguments_json" => Ok(GeneratedField::ModifiedArgumentsJson),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = PreToolResult;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.PreToolResult")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<PreToolResult, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut decision__ = None;
                let mut reason__ = None;
                let mut modified_arguments_json__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Decision => {
                            if decision__.is_some() {
                                return Err(serde::de::Error::duplicate_field("decision"));
                            }
                            decision__ = map_.next_value::<::std::option::Option<pre_tool_result::Decision>>()?.map(|x| x as i32);
                        }
                        GeneratedField::Reason => {
                            if reason__.is_some() {
                                return Err(serde::de::Error::duplicate_field("reason"));
                            }
                            reason__ = map_.next_value()?;
                        }
                        GeneratedField::ModifiedArgumentsJson => {
                            if modified_arguments_json__.is_some() {
                                return Err(serde::de::Error::duplicate_field("modifiedArgumentsJson"));
                            }
                            modified_arguments_json__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(PreToolResult {
                    decision: decision__,
                    reason: reason__,
                    modified_arguments_json: modified_arguments_json__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.PreToolResult", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for pre_tool_result::Decision {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "DECISION_UNSPECIFIED",
            Self::Allow => "ALLOW",
            Self::Deny => "DENY",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for pre_tool_result::Decision {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "DECISION_UNSPECIFIED",
            "ALLOW",
            "DENY",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = pre_tool_result::Decision;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "DECISION_UNSPECIFIED" => Ok(pre_tool_result::Decision::Unspecified),
                    "ALLOW" => Ok(pre_tool_result::Decision::Allow),
                    "DENY" => Ok(pre_tool_result::Decision::Deny),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for PreTurnArgs {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.user_input.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.PreTurnArgs", len)?;
        if let Some(v) = self.user_input.as_ref() {
            struct_ser.serialize_field("userInput", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for PreTurnArgs {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "user_input",
            "userInput",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            UserInput,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "userInput" | "user_input" => Ok(GeneratedField::UserInput),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = PreTurnArgs;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.PreTurnArgs")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<PreTurnArgs, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut user_input__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::UserInput => {
                            if user_input__.is_some() {
                                return Err(serde::de::Error::duplicate_field("userInput"));
                            }
                            user_input__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(PreTurnArgs {
                    user_input: user_input__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.PreTurnArgs", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for PreTurnResult {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.decision.is_some() {
            len += 1;
        }
        if self.reason.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.PreTurnResult", len)?;
        if let Some(v) = self.decision.as_ref() {
            let v = pre_turn_result::Decision::try_from(*v)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", *v)))?;
            struct_ser.serialize_field("decision", &v)?;
        }
        if let Some(v) = self.reason.as_ref() {
            struct_ser.serialize_field("reason", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for PreTurnResult {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "decision",
            "reason",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Decision,
            Reason,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "decision" => Ok(GeneratedField::Decision),
                            "reason" => Ok(GeneratedField::Reason),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = PreTurnResult;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.PreTurnResult")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<PreTurnResult, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut decision__ = None;
                let mut reason__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Decision => {
                            if decision__.is_some() {
                                return Err(serde::de::Error::duplicate_field("decision"));
                            }
                            decision__ = map_.next_value::<::std::option::Option<pre_turn_result::Decision>>()?.map(|x| x as i32);
                        }
                        GeneratedField::Reason => {
                            if reason__.is_some() {
                                return Err(serde::de::Error::duplicate_field("reason"));
                            }
                            reason__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(PreTurnResult {
                    decision: decision__,
                    reason: reason__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.PreTurnResult", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for pre_turn_result::Decision {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "DECISION_UNSPECIFIED",
            Self::Allow => "ALLOW",
            Self::Deny => "DENY",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for pre_turn_result::Decision {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "DECISION_UNSPECIFIED",
            "ALLOW",
            "DENY",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = pre_turn_result::Decision;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "DECISION_UNSPECIFIED" => Ok(pre_turn_result::Decision::Unspecified),
                    "ALLOW" => Ok(pre_turn_result::Decision::Allow),
                    "DENY" => Ok(pre_turn_result::Decision::Deny),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for ReadUrlContentToolConfig {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.enabled.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.ReadUrlContentToolConfig", len)?;
        if let Some(v) = self.enabled.as_ref() {
            struct_ser.serialize_field("enabled", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ReadUrlContentToolConfig {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "enabled",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Enabled,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "enabled" => Ok(GeneratedField::Enabled),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ReadUrlContentToolConfig;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.ReadUrlContentToolConfig")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ReadUrlContentToolConfig, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut enabled__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Enabled => {
                            if enabled__.is_some() {
                                return Err(serde::de::Error::duplicate_field("enabled"));
                            }
                            enabled__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ReadUrlContentToolConfig {
                    enabled: enabled__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.ReadUrlContentToolConfig", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for RetryConfig {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.api_retry.is_some() {
            len += 1;
        }
        if self.model_output_retry.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.RetryConfig", len)?;
        if let Some(v) = self.api_retry.as_ref() {
            struct_ser.serialize_field("apiRetry", v)?;
        }
        if let Some(v) = self.model_output_retry.as_ref() {
            struct_ser.serialize_field("modelOutputRetry", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for RetryConfig {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "api_retry",
            "apiRetry",
            "model_output_retry",
            "modelOutputRetry",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ApiRetry,
            ModelOutputRetry,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "apiRetry" | "api_retry" => Ok(GeneratedField::ApiRetry),
                            "modelOutputRetry" | "model_output_retry" => Ok(GeneratedField::ModelOutputRetry),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = RetryConfig;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.RetryConfig")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<RetryConfig, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut api_retry__ = None;
                let mut model_output_retry__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ApiRetry => {
                            if api_retry__.is_some() {
                                return Err(serde::de::Error::duplicate_field("apiRetry"));
                            }
                            api_retry__ = map_.next_value()?;
                        }
                        GeneratedField::ModelOutputRetry => {
                            if model_output_retry__.is_some() {
                                return Err(serde::de::Error::duplicate_field("modelOutputRetry"));
                            }
                            model_output_retry__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(RetryConfig {
                    api_retry: api_retry__,
                    model_output_retry: model_output_retry__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.RetryConfig", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for RunCommandToolConfig {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.enabled.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.RunCommandToolConfig", len)?;
        if let Some(v) = self.enabled.as_ref() {
            struct_ser.serialize_field("enabled", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for RunCommandToolConfig {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "enabled",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Enabled,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "enabled" => Ok(GeneratedField::Enabled),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = RunCommandToolConfig;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.RunCommandToolConfig")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<RunCommandToolConfig, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut enabled__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Enabled => {
                            if enabled__.is_some() {
                                return Err(serde::de::Error::duplicate_field("enabled"));
                            }
                            enabled__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(RunCommandToolConfig {
                    enabled: enabled__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.RunCommandToolConfig", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SearchWebToolConfig {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.enabled.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.SearchWebToolConfig", len)?;
        if let Some(v) = self.enabled.as_ref() {
            struct_ser.serialize_field("enabled", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SearchWebToolConfig {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "enabled",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Enabled,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "enabled" => Ok(GeneratedField::Enabled),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SearchWebToolConfig;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.SearchWebToolConfig")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SearchWebToolConfig, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut enabled__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Enabled => {
                            if enabled__.is_some() {
                                return Err(serde::de::Error::duplicate_field("enabled"));
                            }
                            enabled__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(SearchWebToolConfig {
                    enabled: enabled__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.SearchWebToolConfig", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for StepUpdate {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.cascade_id.is_some() {
            len += 1;
        }
        if self.trajectory_id.is_some() {
            len += 1;
        }
        if self.step_index.is_some() {
            len += 1;
        }
        if self.state.is_some() {
            len += 1;
        }
        if self.source.is_some() {
            len += 1;
        }
        if self.target.is_some() {
            len += 1;
        }
        if self.error_message.is_some() {
            len += 1;
        }
        if self.thinking.is_some() {
            len += 1;
        }
        if self.text_delta.is_some() {
            len += 1;
        }
        if self.thinking_delta.is_some() {
            len += 1;
        }
        if self.text.is_some() {
            len += 1;
        }
        if self.list_directory.is_some() {
            len += 1;
        }
        if self.find_file.is_some() {
            len += 1;
        }
        if self.search_directory.is_some() {
            len += 1;
        }
        if self.view_file.is_some() {
            len += 1;
        }
        if self.create_file.is_some() {
            len += 1;
        }
        if self.edit_file.is_some() {
            len += 1;
        }
        if self.run_command.is_some() {
            len += 1;
        }
        if self.compaction.is_some() {
            len += 1;
        }
        if self.invoke_subagent.is_some() {
            len += 1;
        }
        if self.generate_image.is_some() {
            len += 1;
        }
        if self.finish.is_some() {
            len += 1;
        }
        if self.error.is_some() {
            len += 1;
        }
        if self.mcp_tool.is_some() {
            len += 1;
        }
        if self.search_web.is_some() {
            len += 1;
        }
        if self.read_url_content.is_some() {
            len += 1;
        }
        if self.custom_tool.is_some() {
            len += 1;
        }
        if self.request_text.is_some() {
            len += 1;
        }
        if self.tool_confirmation_request.is_some() {
            len += 1;
        }
        if self.questions_request.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.StepUpdate", len)?;
        if let Some(v) = self.cascade_id.as_ref() {
            struct_ser.serialize_field("cascadeId", v)?;
        }
        if let Some(v) = self.trajectory_id.as_ref() {
            struct_ser.serialize_field("trajectoryId", v)?;
        }
        if let Some(v) = self.step_index.as_ref() {
            struct_ser.serialize_field("stepIndex", v)?;
        }
        if let Some(v) = self.state.as_ref() {
            let v = step_update::State::try_from(*v)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", *v)))?;
            struct_ser.serialize_field("state", &v)?;
        }
        if let Some(v) = self.source.as_ref() {
            let v = step_update::Source::try_from(*v)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", *v)))?;
            struct_ser.serialize_field("source", &v)?;
        }
        if let Some(v) = self.target.as_ref() {
            let v = step_update::Target::try_from(*v)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", *v)))?;
            struct_ser.serialize_field("target", &v)?;
        }
        if let Some(v) = self.error_message.as_ref() {
            struct_ser.serialize_field("errorMessage", v)?;
        }
        if let Some(v) = self.thinking.as_ref() {
            struct_ser.serialize_field("thinking", v)?;
        }
        if let Some(v) = self.text_delta.as_ref() {
            struct_ser.serialize_field("textDelta", v)?;
        }
        if let Some(v) = self.thinking_delta.as_ref() {
            struct_ser.serialize_field("thinkingDelta", v)?;
        }
        if let Some(v) = self.text.as_ref() {
            struct_ser.serialize_field("text", v)?;
        }
        if let Some(v) = self.list_directory.as_ref() {
            struct_ser.serialize_field("listDirectory", v)?;
        }
        if let Some(v) = self.find_file.as_ref() {
            struct_ser.serialize_field("findFile", v)?;
        }
        if let Some(v) = self.search_directory.as_ref() {
            struct_ser.serialize_field("searchDirectory", v)?;
        }
        if let Some(v) = self.view_file.as_ref() {
            struct_ser.serialize_field("viewFile", v)?;
        }
        if let Some(v) = self.create_file.as_ref() {
            struct_ser.serialize_field("createFile", v)?;
        }
        if let Some(v) = self.edit_file.as_ref() {
            struct_ser.serialize_field("editFile", v)?;
        }
        if let Some(v) = self.run_command.as_ref() {
            struct_ser.serialize_field("runCommand", v)?;
        }
        if let Some(v) = self.compaction.as_ref() {
            struct_ser.serialize_field("compaction", v)?;
        }
        if let Some(v) = self.invoke_subagent.as_ref() {
            struct_ser.serialize_field("invokeSubagent", v)?;
        }
        if let Some(v) = self.generate_image.as_ref() {
            struct_ser.serialize_field("generateImage", v)?;
        }
        if let Some(v) = self.finish.as_ref() {
            struct_ser.serialize_field("finish", v)?;
        }
        if let Some(v) = self.error.as_ref() {
            struct_ser.serialize_field("error", v)?;
        }
        if let Some(v) = self.mcp_tool.as_ref() {
            struct_ser.serialize_field("mcpTool", v)?;
        }
        if let Some(v) = self.search_web.as_ref() {
            struct_ser.serialize_field("searchWeb", v)?;
        }
        if let Some(v) = self.read_url_content.as_ref() {
            struct_ser.serialize_field("readUrlContent", v)?;
        }
        if let Some(v) = self.custom_tool.as_ref() {
            struct_ser.serialize_field("customTool", v)?;
        }
        if let Some(v) = self.request_text.as_ref() {
            struct_ser.serialize_field("requestText", v)?;
        }
        if let Some(v) = self.tool_confirmation_request.as_ref() {
            struct_ser.serialize_field("toolConfirmationRequest", v)?;
        }
        if let Some(v) = self.questions_request.as_ref() {
            struct_ser.serialize_field("questionsRequest", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for StepUpdate {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "cascade_id",
            "cascadeId",
            "trajectory_id",
            "trajectoryId",
            "step_index",
            "stepIndex",
            "state",
            "source",
            "target",
            "error_message",
            "errorMessage",
            "thinking",
            "text_delta",
            "textDelta",
            "thinking_delta",
            "thinkingDelta",
            "text",
            "list_directory",
            "listDirectory",
            "find_file",
            "findFile",
            "search_directory",
            "searchDirectory",
            "view_file",
            "viewFile",
            "create_file",
            "createFile",
            "edit_file",
            "editFile",
            "run_command",
            "runCommand",
            "compaction",
            "invoke_subagent",
            "invokeSubagent",
            "generate_image",
            "generateImage",
            "finish",
            "error",
            "mcp_tool",
            "mcpTool",
            "search_web",
            "searchWeb",
            "read_url_content",
            "readUrlContent",
            "custom_tool",
            "customTool",
            "request_text",
            "requestText",
            "tool_confirmation_request",
            "toolConfirmationRequest",
            "questions_request",
            "questionsRequest",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            CascadeId,
            TrajectoryId,
            StepIndex,
            State,
            Source,
            Target,
            ErrorMessage,
            Thinking,
            TextDelta,
            ThinkingDelta,
            Text,
            ListDirectory,
            FindFile,
            SearchDirectory,
            ViewFile,
            CreateFile,
            EditFile,
            RunCommand,
            Compaction,
            InvokeSubagent,
            GenerateImage,
            Finish,
            Error,
            McpTool,
            SearchWeb,
            ReadUrlContent,
            CustomTool,
            RequestText,
            ToolConfirmationRequest,
            QuestionsRequest,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "cascadeId" | "cascade_id" => Ok(GeneratedField::CascadeId),
                            "trajectoryId" | "trajectory_id" => Ok(GeneratedField::TrajectoryId),
                            "stepIndex" | "step_index" => Ok(GeneratedField::StepIndex),
                            "state" => Ok(GeneratedField::State),
                            "source" => Ok(GeneratedField::Source),
                            "target" => Ok(GeneratedField::Target),
                            "errorMessage" | "error_message" => Ok(GeneratedField::ErrorMessage),
                            "thinking" => Ok(GeneratedField::Thinking),
                            "textDelta" | "text_delta" => Ok(GeneratedField::TextDelta),
                            "thinkingDelta" | "thinking_delta" => Ok(GeneratedField::ThinkingDelta),
                            "text" => Ok(GeneratedField::Text),
                            "listDirectory" | "list_directory" => Ok(GeneratedField::ListDirectory),
                            "findFile" | "find_file" => Ok(GeneratedField::FindFile),
                            "searchDirectory" | "search_directory" => Ok(GeneratedField::SearchDirectory),
                            "viewFile" | "view_file" => Ok(GeneratedField::ViewFile),
                            "createFile" | "create_file" => Ok(GeneratedField::CreateFile),
                            "editFile" | "edit_file" => Ok(GeneratedField::EditFile),
                            "runCommand" | "run_command" => Ok(GeneratedField::RunCommand),
                            "compaction" => Ok(GeneratedField::Compaction),
                            "invokeSubagent" | "invoke_subagent" => Ok(GeneratedField::InvokeSubagent),
                            "generateImage" | "generate_image" => Ok(GeneratedField::GenerateImage),
                            "finish" => Ok(GeneratedField::Finish),
                            "error" => Ok(GeneratedField::Error),
                            "mcpTool" | "mcp_tool" => Ok(GeneratedField::McpTool),
                            "searchWeb" | "search_web" => Ok(GeneratedField::SearchWeb),
                            "readUrlContent" | "read_url_content" => Ok(GeneratedField::ReadUrlContent),
                            "customTool" | "custom_tool" => Ok(GeneratedField::CustomTool),
                            "requestText" | "request_text" => Ok(GeneratedField::RequestText),
                            "toolConfirmationRequest" | "tool_confirmation_request" => Ok(GeneratedField::ToolConfirmationRequest),
                            "questionsRequest" | "questions_request" => Ok(GeneratedField::QuestionsRequest),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = StepUpdate;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.StepUpdate")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<StepUpdate, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut cascade_id__ = None;
                let mut trajectory_id__ = None;
                let mut step_index__ = None;
                let mut state__ = None;
                let mut source__ = None;
                let mut target__ = None;
                let mut error_message__ = None;
                let mut thinking__ = None;
                let mut text_delta__ = None;
                let mut thinking_delta__ = None;
                let mut text__ = None;
                let mut list_directory__ = None;
                let mut find_file__ = None;
                let mut search_directory__ = None;
                let mut view_file__ = None;
                let mut create_file__ = None;
                let mut edit_file__ = None;
                let mut run_command__ = None;
                let mut compaction__ = None;
                let mut invoke_subagent__ = None;
                let mut generate_image__ = None;
                let mut finish__ = None;
                let mut error__ = None;
                let mut mcp_tool__ = None;
                let mut search_web__ = None;
                let mut read_url_content__ = None;
                let mut custom_tool__ = None;
                let mut request_text__ = None;
                let mut tool_confirmation_request__ = None;
                let mut questions_request__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::CascadeId => {
                            if cascade_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("cascadeId"));
                            }
                            cascade_id__ = map_.next_value()?;
                        }
                        GeneratedField::TrajectoryId => {
                            if trajectory_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("trajectoryId"));
                            }
                            trajectory_id__ = map_.next_value()?;
                        }
                        GeneratedField::StepIndex => {
                            if step_index__.is_some() {
                                return Err(serde::de::Error::duplicate_field("stepIndex"));
                            }
                            step_index__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::State => {
                            if state__.is_some() {
                                return Err(serde::de::Error::duplicate_field("state"));
                            }
                            state__ = map_.next_value::<::std::option::Option<step_update::State>>()?.map(|x| x as i32);
                        }
                        GeneratedField::Source => {
                            if source__.is_some() {
                                return Err(serde::de::Error::duplicate_field("source"));
                            }
                            source__ = map_.next_value::<::std::option::Option<step_update::Source>>()?.map(|x| x as i32);
                        }
                        GeneratedField::Target => {
                            if target__.is_some() {
                                return Err(serde::de::Error::duplicate_field("target"));
                            }
                            target__ = map_.next_value::<::std::option::Option<step_update::Target>>()?.map(|x| x as i32);
                        }
                        GeneratedField::ErrorMessage => {
                            if error_message__.is_some() {
                                return Err(serde::de::Error::duplicate_field("errorMessage"));
                            }
                            error_message__ = map_.next_value()?;
                        }
                        GeneratedField::Thinking => {
                            if thinking__.is_some() {
                                return Err(serde::de::Error::duplicate_field("thinking"));
                            }
                            thinking__ = map_.next_value()?;
                        }
                        GeneratedField::TextDelta => {
                            if text_delta__.is_some() {
                                return Err(serde::de::Error::duplicate_field("textDelta"));
                            }
                            text_delta__ = map_.next_value()?;
                        }
                        GeneratedField::ThinkingDelta => {
                            if thinking_delta__.is_some() {
                                return Err(serde::de::Error::duplicate_field("thinkingDelta"));
                            }
                            thinking_delta__ = map_.next_value()?;
                        }
                        GeneratedField::Text => {
                            if text__.is_some() {
                                return Err(serde::de::Error::duplicate_field("text"));
                            }
                            text__ = map_.next_value()?;
                        }
                        GeneratedField::ListDirectory => {
                            if list_directory__.is_some() {
                                return Err(serde::de::Error::duplicate_field("listDirectory"));
                            }
                            list_directory__ = map_.next_value()?;
                        }
                        GeneratedField::FindFile => {
                            if find_file__.is_some() {
                                return Err(serde::de::Error::duplicate_field("findFile"));
                            }
                            find_file__ = map_.next_value()?;
                        }
                        GeneratedField::SearchDirectory => {
                            if search_directory__.is_some() {
                                return Err(serde::de::Error::duplicate_field("searchDirectory"));
                            }
                            search_directory__ = map_.next_value()?;
                        }
                        GeneratedField::ViewFile => {
                            if view_file__.is_some() {
                                return Err(serde::de::Error::duplicate_field("viewFile"));
                            }
                            view_file__ = map_.next_value()?;
                        }
                        GeneratedField::CreateFile => {
                            if create_file__.is_some() {
                                return Err(serde::de::Error::duplicate_field("createFile"));
                            }
                            create_file__ = map_.next_value()?;
                        }
                        GeneratedField::EditFile => {
                            if edit_file__.is_some() {
                                return Err(serde::de::Error::duplicate_field("editFile"));
                            }
                            edit_file__ = map_.next_value()?;
                        }
                        GeneratedField::RunCommand => {
                            if run_command__.is_some() {
                                return Err(serde::de::Error::duplicate_field("runCommand"));
                            }
                            run_command__ = map_.next_value()?;
                        }
                        GeneratedField::Compaction => {
                            if compaction__.is_some() {
                                return Err(serde::de::Error::duplicate_field("compaction"));
                            }
                            compaction__ = map_.next_value()?;
                        }
                        GeneratedField::InvokeSubagent => {
                            if invoke_subagent__.is_some() {
                                return Err(serde::de::Error::duplicate_field("invokeSubagent"));
                            }
                            invoke_subagent__ = map_.next_value()?;
                        }
                        GeneratedField::GenerateImage => {
                            if generate_image__.is_some() {
                                return Err(serde::de::Error::duplicate_field("generateImage"));
                            }
                            generate_image__ = map_.next_value()?;
                        }
                        GeneratedField::Finish => {
                            if finish__.is_some() {
                                return Err(serde::de::Error::duplicate_field("finish"));
                            }
                            finish__ = map_.next_value()?;
                        }
                        GeneratedField::Error => {
                            if error__.is_some() {
                                return Err(serde::de::Error::duplicate_field("error"));
                            }
                            error__ = map_.next_value()?;
                        }
                        GeneratedField::McpTool => {
                            if mcp_tool__.is_some() {
                                return Err(serde::de::Error::duplicate_field("mcpTool"));
                            }
                            mcp_tool__ = map_.next_value()?;
                        }
                        GeneratedField::SearchWeb => {
                            if search_web__.is_some() {
                                return Err(serde::de::Error::duplicate_field("searchWeb"));
                            }
                            search_web__ = map_.next_value()?;
                        }
                        GeneratedField::ReadUrlContent => {
                            if read_url_content__.is_some() {
                                return Err(serde::de::Error::duplicate_field("readUrlContent"));
                            }
                            read_url_content__ = map_.next_value()?;
                        }
                        GeneratedField::CustomTool => {
                            if custom_tool__.is_some() {
                                return Err(serde::de::Error::duplicate_field("customTool"));
                            }
                            custom_tool__ = map_.next_value()?;
                        }
                        GeneratedField::RequestText => {
                            if request_text__.is_some() {
                                return Err(serde::de::Error::duplicate_field("requestText"));
                            }
                            request_text__ = map_.next_value()?;
                        }
                        GeneratedField::ToolConfirmationRequest => {
                            if tool_confirmation_request__.is_some() {
                                return Err(serde::de::Error::duplicate_field("toolConfirmationRequest"));
                            }
                            tool_confirmation_request__ = map_.next_value()?;
                        }
                        GeneratedField::QuestionsRequest => {
                            if questions_request__.is_some() {
                                return Err(serde::de::Error::duplicate_field("questionsRequest"));
                            }
                            questions_request__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(StepUpdate {
                    cascade_id: cascade_id__,
                    trajectory_id: trajectory_id__,
                    step_index: step_index__,
                    state: state__,
                    source: source__,
                    target: target__,
                    error_message: error_message__,
                    thinking: thinking__,
                    text_delta: text_delta__,
                    thinking_delta: thinking_delta__,
                    text: text__,
                    list_directory: list_directory__,
                    find_file: find_file__,
                    search_directory: search_directory__,
                    view_file: view_file__,
                    create_file: create_file__,
                    edit_file: edit_file__,
                    run_command: run_command__,
                    compaction: compaction__,
                    invoke_subagent: invoke_subagent__,
                    generate_image: generate_image__,
                    finish: finish__,
                    error: error__,
                    mcp_tool: mcp_tool__,
                    search_web: search_web__,
                    read_url_content: read_url_content__,
                    custom_tool: custom_tool__,
                    request_text: request_text__,
                    tool_confirmation_request: tool_confirmation_request__,
                    questions_request: questions_request__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.StepUpdate", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for step_update::Source {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "SOURCE_UNSPECIFIED",
            Self::System => "SOURCE_SYSTEM",
            Self::User => "SOURCE_USER",
            Self::Model => "SOURCE_MODEL",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for step_update::Source {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "SOURCE_UNSPECIFIED",
            "SOURCE_SYSTEM",
            "SOURCE_USER",
            "SOURCE_MODEL",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = step_update::Source;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "SOURCE_UNSPECIFIED" => Ok(step_update::Source::Unspecified),
                    "SOURCE_SYSTEM" => Ok(step_update::Source::System),
                    "SOURCE_USER" => Ok(step_update::Source::User),
                    "SOURCE_MODEL" => Ok(step_update::Source::Model),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for step_update::State {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "STATE_UNSPECIFIED",
            Self::Active => "STATE_ACTIVE",
            Self::Done => "STATE_DONE",
            Self::WaitingForUser => "STATE_WAITING_FOR_USER",
            Self::Error => "STATE_ERROR",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for step_update::State {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "STATE_UNSPECIFIED",
            "STATE_ACTIVE",
            "STATE_DONE",
            "STATE_WAITING_FOR_USER",
            "STATE_ERROR",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = step_update::State;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "STATE_UNSPECIFIED" => Ok(step_update::State::Unspecified),
                    "STATE_ACTIVE" => Ok(step_update::State::Active),
                    "STATE_DONE" => Ok(step_update::State::Done),
                    "STATE_WAITING_FOR_USER" => Ok(step_update::State::WaitingForUser),
                    "STATE_ERROR" => Ok(step_update::State::Error),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for step_update::Target {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "TARGET_UNSPECIFIED",
            Self::User => "TARGET_USER",
            Self::Model => "TARGET_MODEL",
            Self::Environment => "TARGET_ENVIRONMENT",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for step_update::Target {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "TARGET_UNSPECIFIED",
            "TARGET_USER",
            "TARGET_MODEL",
            "TARGET_ENVIRONMENT",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = step_update::Target;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "TARGET_UNSPECIFIED" => Ok(step_update::Target::Unspecified),
                    "TARGET_USER" => Ok(step_update::Target::User),
                    "TARGET_MODEL" => Ok(step_update::Target::Model),
                    "TARGET_ENVIRONMENT" => Ok(step_update::Target::Environment),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for SubagentsConfig {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.enabled.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.SubagentsConfig", len)?;
        if let Some(v) = self.enabled.as_ref() {
            struct_ser.serialize_field("enabled", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SubagentsConfig {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "enabled",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Enabled,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "enabled" => Ok(GeneratedField::Enabled),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SubagentsConfig;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.SubagentsConfig")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SubagentsConfig, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut enabled__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Enabled => {
                            if enabled__.is_some() {
                                return Err(serde::de::Error::duplicate_field("enabled"));
                            }
                            enabled__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(SubagentsConfig {
                    enabled: enabled__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.SubagentsConfig", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SystemInstructions {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.r#type.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.SystemInstructions", len)?;
        if let Some(v) = self.r#type.as_ref() {
            match v {
                system_instructions::Type::Custom(v) => {
                    struct_ser.serialize_field("custom", v)?;
                }
                system_instructions::Type::Appended(v) => {
                    struct_ser.serialize_field("appended", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SystemInstructions {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "custom",
            "appended",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Custom,
            Appended,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "custom" => Ok(GeneratedField::Custom),
                            "appended" => Ok(GeneratedField::Appended),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SystemInstructions;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.SystemInstructions")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SystemInstructions, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut r#type__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Custom => {
                            if r#type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("custom"));
                            }
                            r#type__ = map_.next_value::<::std::option::Option<_>>()?.map(system_instructions::Type::Custom)
;
                        }
                        GeneratedField::Appended => {
                            if r#type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("appended"));
                            }
                            r#type__ = map_.next_value::<::std::option::Option<_>>()?.map(system_instructions::Type::Appended)
;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(SystemInstructions {
                    r#type: r#type__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.SystemInstructions", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Tool {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.name.is_some() {
            len += 1;
        }
        if self.description.is_some() {
            len += 1;
        }
        if self.parameters_json_schema.is_some() {
            len += 1;
        }
        if self.response_json_schema.is_some() {
            len += 1;
        }
        if self.defer_loading.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.Tool", len)?;
        if let Some(v) = self.name.as_ref() {
            struct_ser.serialize_field("name", v)?;
        }
        if let Some(v) = self.description.as_ref() {
            struct_ser.serialize_field("description", v)?;
        }
        if let Some(v) = self.parameters_json_schema.as_ref() {
            struct_ser.serialize_field("parametersJsonSchema", v)?;
        }
        if let Some(v) = self.response_json_schema.as_ref() {
            struct_ser.serialize_field("responseJsonSchema", v)?;
        }
        if let Some(v) = self.defer_loading.as_ref() {
            struct_ser.serialize_field("deferLoading", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Tool {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "name",
            "description",
            "parameters_json_schema",
            "parametersJsonSchema",
            "response_json_schema",
            "responseJsonSchema",
            "defer_loading",
            "deferLoading",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Name,
            Description,
            ParametersJsonSchema,
            ResponseJsonSchema,
            DeferLoading,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "name" => Ok(GeneratedField::Name),
                            "description" => Ok(GeneratedField::Description),
                            "parametersJsonSchema" | "parameters_json_schema" => Ok(GeneratedField::ParametersJsonSchema),
                            "responseJsonSchema" | "response_json_schema" => Ok(GeneratedField::ResponseJsonSchema),
                            "deferLoading" | "defer_loading" => Ok(GeneratedField::DeferLoading),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Tool;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.Tool")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Tool, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut name__ = None;
                let mut description__ = None;
                let mut parameters_json_schema__ = None;
                let mut response_json_schema__ = None;
                let mut defer_loading__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = map_.next_value()?;
                        }
                        GeneratedField::Description => {
                            if description__.is_some() {
                                return Err(serde::de::Error::duplicate_field("description"));
                            }
                            description__ = map_.next_value()?;
                        }
                        GeneratedField::ParametersJsonSchema => {
                            if parameters_json_schema__.is_some() {
                                return Err(serde::de::Error::duplicate_field("parametersJsonSchema"));
                            }
                            parameters_json_schema__ = map_.next_value()?;
                        }
                        GeneratedField::ResponseJsonSchema => {
                            if response_json_schema__.is_some() {
                                return Err(serde::de::Error::duplicate_field("responseJsonSchema"));
                            }
                            response_json_schema__ = map_.next_value()?;
                        }
                        GeneratedField::DeferLoading => {
                            if defer_loading__.is_some() {
                                return Err(serde::de::Error::duplicate_field("deferLoading"));
                            }
                            defer_loading__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(Tool {
                    name: name__,
                    description: description__,
                    parameters_json_schema: parameters_json_schema__,
                    response_json_schema: response_json_schema__,
                    defer_loading: defer_loading__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.Tool", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ToolCall {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.id.is_some() {
            len += 1;
        }
        if self.name.is_some() {
            len += 1;
        }
        if self.arguments_json.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.ToolCall", len)?;
        if let Some(v) = self.id.as_ref() {
            struct_ser.serialize_field("id", v)?;
        }
        if let Some(v) = self.name.as_ref() {
            struct_ser.serialize_field("name", v)?;
        }
        if let Some(v) = self.arguments_json.as_ref() {
            struct_ser.serialize_field("argumentsJson", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ToolCall {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "id",
            "name",
            "arguments_json",
            "argumentsJson",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Id,
            Name,
            ArgumentsJson,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "id" => Ok(GeneratedField::Id),
                            "name" => Ok(GeneratedField::Name),
                            "argumentsJson" | "arguments_json" => Ok(GeneratedField::ArgumentsJson),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ToolCall;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.ToolCall")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ToolCall, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut id__ = None;
                let mut name__ = None;
                let mut arguments_json__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Id => {
                            if id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("id"));
                            }
                            id__ = map_.next_value()?;
                        }
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = map_.next_value()?;
                        }
                        GeneratedField::ArgumentsJson => {
                            if arguments_json__.is_some() {
                                return Err(serde::de::Error::duplicate_field("argumentsJson"));
                            }
                            arguments_json__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ToolCall {
                    id: id__,
                    name: name__,
                    arguments_json: arguments_json__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.ToolCall", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ToolConfirmation {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.trajectory_id.is_some() {
            len += 1;
        }
        if self.step_index.is_some() {
            len += 1;
        }
        if self.accepted.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.ToolConfirmation", len)?;
        if let Some(v) = self.trajectory_id.as_ref() {
            struct_ser.serialize_field("trajectoryId", v)?;
        }
        if let Some(v) = self.step_index.as_ref() {
            struct_ser.serialize_field("stepIndex", v)?;
        }
        if let Some(v) = self.accepted.as_ref() {
            struct_ser.serialize_field("accepted", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ToolConfirmation {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "trajectory_id",
            "trajectoryId",
            "step_index",
            "stepIndex",
            "accepted",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TrajectoryId,
            StepIndex,
            Accepted,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "trajectoryId" | "trajectory_id" => Ok(GeneratedField::TrajectoryId),
                            "stepIndex" | "step_index" => Ok(GeneratedField::StepIndex),
                            "accepted" => Ok(GeneratedField::Accepted),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ToolConfirmation;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.ToolConfirmation")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ToolConfirmation, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut trajectory_id__ = None;
                let mut step_index__ = None;
                let mut accepted__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::TrajectoryId => {
                            if trajectory_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("trajectoryId"));
                            }
                            trajectory_id__ = map_.next_value()?;
                        }
                        GeneratedField::StepIndex => {
                            if step_index__.is_some() {
                                return Err(serde::de::Error::duplicate_field("stepIndex"));
                            }
                            step_index__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::Accepted => {
                            if accepted__.is_some() {
                                return Err(serde::de::Error::duplicate_field("accepted"));
                            }
                            accepted__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ToolConfirmation {
                    trajectory_id: trajectory_id__,
                    step_index: step_index__,
                    accepted: accepted__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.ToolConfirmation", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ToolConfirmationRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("antigravity.localharness.ToolConfirmationRequest", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ToolConfirmationRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                            Ok(GeneratedField::__SkipField__)
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ToolConfirmationRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.ToolConfirmationRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ToolConfirmationRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map_.next_key::<GeneratedField>()?.is_some() {
                    let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(ToolConfirmationRequest {
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.ToolConfirmationRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ToolOutputTruncation {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.strategy.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.ToolOutputTruncation", len)?;
        if let Some(v) = self.strategy.as_ref() {
            match v {
                tool_output_truncation::Strategy::Truncate(v) => {
                    struct_ser.serialize_field("truncate", v)?;
                }
                tool_output_truncation::Strategy::Error(v) => {
                    struct_ser.serialize_field("error", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ToolOutputTruncation {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "truncate",
            "error",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Truncate,
            Error,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "truncate" => Ok(GeneratedField::Truncate),
                            "error" => Ok(GeneratedField::Error),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ToolOutputTruncation;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.ToolOutputTruncation")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ToolOutputTruncation, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut strategy__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Truncate => {
                            if strategy__.is_some() {
                                return Err(serde::de::Error::duplicate_field("truncate"));
                            }
                            strategy__ = map_.next_value::<::std::option::Option<_>>()?.map(tool_output_truncation::Strategy::Truncate)
;
                        }
                        GeneratedField::Error => {
                            if strategy__.is_some() {
                                return Err(serde::de::Error::duplicate_field("error"));
                            }
                            strategy__ = map_.next_value::<::std::option::Option<_>>()?.map(tool_output_truncation::Strategy::Error)
;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ToolOutputTruncation {
                    strategy: strategy__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.ToolOutputTruncation", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for tool_output_truncation::ErrorStrategy {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.max_tokens.is_some() {
            len += 1;
        }
        if self.error_message.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.ToolOutputTruncation.ErrorStrategy", len)?;
        if let Some(v) = self.max_tokens.as_ref() {
            struct_ser.serialize_field("maxTokens", v)?;
        }
        if let Some(v) = self.error_message.as_ref() {
            struct_ser.serialize_field("errorMessage", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for tool_output_truncation::ErrorStrategy {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "max_tokens",
            "maxTokens",
            "error_message",
            "errorMessage",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            MaxTokens,
            ErrorMessage,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "maxTokens" | "max_tokens" => Ok(GeneratedField::MaxTokens),
                            "errorMessage" | "error_message" => Ok(GeneratedField::ErrorMessage),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = tool_output_truncation::ErrorStrategy;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.ToolOutputTruncation.ErrorStrategy")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<tool_output_truncation::ErrorStrategy, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut max_tokens__ = None;
                let mut error_message__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::MaxTokens => {
                            if max_tokens__.is_some() {
                                return Err(serde::de::Error::duplicate_field("maxTokens"));
                            }
                            max_tokens__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::ErrorMessage => {
                            if error_message__.is_some() {
                                return Err(serde::de::Error::duplicate_field("errorMessage"));
                            }
                            error_message__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(tool_output_truncation::ErrorStrategy {
                    max_tokens: max_tokens__,
                    error_message: error_message__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.ToolOutputTruncation.ErrorStrategy", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for tool_output_truncation::TruncateStrategy {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.max_tokens.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.ToolOutputTruncation.TruncateStrategy", len)?;
        if let Some(v) = self.max_tokens.as_ref() {
            struct_ser.serialize_field("maxTokens", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for tool_output_truncation::TruncateStrategy {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "max_tokens",
            "maxTokens",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            MaxTokens,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "maxTokens" | "max_tokens" => Ok(GeneratedField::MaxTokens),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = tool_output_truncation::TruncateStrategy;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.ToolOutputTruncation.TruncateStrategy")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<tool_output_truncation::TruncateStrategy, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut max_tokens__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::MaxTokens => {
                            if max_tokens__.is_some() {
                                return Err(serde::de::Error::duplicate_field("maxTokens"));
                            }
                            max_tokens__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(tool_output_truncation::TruncateStrategy {
                    max_tokens: max_tokens__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.ToolOutputTruncation.TruncateStrategy", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ToolResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.id.is_some() {
            len += 1;
        }
        if self.response_json.is_some() {
            len += 1;
        }
        if !self.supplemental_media.is_empty() {
            len += 1;
        }
        if self.error_message.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.ToolResponse", len)?;
        if let Some(v) = self.id.as_ref() {
            struct_ser.serialize_field("id", v)?;
        }
        if let Some(v) = self.response_json.as_ref() {
            struct_ser.serialize_field("responseJson", v)?;
        }
        if !self.supplemental_media.is_empty() {
            struct_ser.serialize_field("supplementalMedia", &self.supplemental_media)?;
        }
        if let Some(v) = self.error_message.as_ref() {
            struct_ser.serialize_field("errorMessage", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ToolResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "id",
            "response_json",
            "responseJson",
            "supplemental_media",
            "supplementalMedia",
            "error_message",
            "errorMessage",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Id,
            ResponseJson,
            SupplementalMedia,
            ErrorMessage,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "id" => Ok(GeneratedField::Id),
                            "responseJson" | "response_json" => Ok(GeneratedField::ResponseJson),
                            "supplementalMedia" | "supplemental_media" => Ok(GeneratedField::SupplementalMedia),
                            "errorMessage" | "error_message" => Ok(GeneratedField::ErrorMessage),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ToolResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.ToolResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ToolResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut id__ = None;
                let mut response_json__ = None;
                let mut supplemental_media__ = None;
                let mut error_message__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Id => {
                            if id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("id"));
                            }
                            id__ = map_.next_value()?;
                        }
                        GeneratedField::ResponseJson => {
                            if response_json__.is_some() {
                                return Err(serde::de::Error::duplicate_field("responseJson"));
                            }
                            response_json__ = map_.next_value()?;
                        }
                        GeneratedField::SupplementalMedia => {
                            if supplemental_media__.is_some() {
                                return Err(serde::de::Error::duplicate_field("supplementalMedia"));
                            }
                            supplemental_media__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ErrorMessage => {
                            if error_message__.is_some() {
                                return Err(serde::de::Error::duplicate_field("errorMessage"));
                            }
                            error_message__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ToolResponse {
                    id: id__,
                    response_json: response_json__,
                    supplemental_media: supplemental_media__.unwrap_or_default(),
                    error_message: error_message__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.ToolResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ToolSearchConfig {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.enabled.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.ToolSearchConfig", len)?;
        if let Some(v) = self.enabled.as_ref() {
            struct_ser.serialize_field("enabled", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ToolSearchConfig {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "enabled",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Enabled,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "enabled" => Ok(GeneratedField::Enabled),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ToolSearchConfig;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.ToolSearchConfig")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ToolSearchConfig, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut enabled__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Enabled => {
                            if enabled__.is_some() {
                                return Err(serde::de::Error::duplicate_field("enabled"));
                            }
                            enabled__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ToolSearchConfig {
                    enabled: enabled__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.ToolSearchConfig", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for TrajectoryStateUpdate {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.trajectory_id.is_some() {
            len += 1;
        }
        if self.state.is_some() {
            len += 1;
        }
        if self.error.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.TrajectoryStateUpdate", len)?;
        if let Some(v) = self.trajectory_id.as_ref() {
            struct_ser.serialize_field("trajectoryId", v)?;
        }
        if let Some(v) = self.state.as_ref() {
            let v = trajectory_state_update::State::try_from(*v)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", *v)))?;
            struct_ser.serialize_field("state", &v)?;
        }
        if let Some(v) = self.error.as_ref() {
            struct_ser.serialize_field("error", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for TrajectoryStateUpdate {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "trajectory_id",
            "trajectoryId",
            "state",
            "error",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TrajectoryId,
            State,
            Error,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "trajectoryId" | "trajectory_id" => Ok(GeneratedField::TrajectoryId),
                            "state" => Ok(GeneratedField::State),
                            "error" => Ok(GeneratedField::Error),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = TrajectoryStateUpdate;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.TrajectoryStateUpdate")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<TrajectoryStateUpdate, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut trajectory_id__ = None;
                let mut state__ = None;
                let mut error__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::TrajectoryId => {
                            if trajectory_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("trajectoryId"));
                            }
                            trajectory_id__ = map_.next_value()?;
                        }
                        GeneratedField::State => {
                            if state__.is_some() {
                                return Err(serde::de::Error::duplicate_field("state"));
                            }
                            state__ = map_.next_value::<::std::option::Option<trajectory_state_update::State>>()?.map(|x| x as i32);
                        }
                        GeneratedField::Error => {
                            if error__.is_some() {
                                return Err(serde::de::Error::duplicate_field("error"));
                            }
                            error__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(TrajectoryStateUpdate {
                    trajectory_id: trajectory_id__,
                    state: state__,
                    error: error__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.TrajectoryStateUpdate", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for trajectory_state_update::State {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "STATE_UNSPECIFIED",
            Self::Running => "STATE_RUNNING",
            Self::FullyIdle => "STATE_FULLY_IDLE",
            Self::Cancelled => "STATE_CANCELLED",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for trajectory_state_update::State {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "STATE_UNSPECIFIED",
            "STATE_RUNNING",
            "STATE_FULLY_IDLE",
            "STATE_CANCELLED",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = trajectory_state_update::State;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "STATE_UNSPECIFIED" => Ok(trajectory_state_update::State::Unspecified),
                    "STATE_RUNNING" => Ok(trajectory_state_update::State::Running),
                    "STATE_FULLY_IDLE" => Ok(trajectory_state_update::State::FullyIdle),
                    "STATE_CANCELLED" => Ok(trajectory_state_update::State::Cancelled),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for UsageMetadata {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.prompt_token_count.is_some() {
            len += 1;
        }
        if self.cached_content_token_count.is_some() {
            len += 1;
        }
        if self.candidates_token_count.is_some() {
            len += 1;
        }
        if self.thoughts_token_count.is_some() {
            len += 1;
        }
        if self.total_token_count.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.UsageMetadata", len)?;
        if let Some(v) = self.prompt_token_count.as_ref() {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("promptTokenCount", ToString::to_string(&v).as_str())?;
        }
        if let Some(v) = self.cached_content_token_count.as_ref() {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("cachedContentTokenCount", ToString::to_string(&v).as_str())?;
        }
        if let Some(v) = self.candidates_token_count.as_ref() {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("candidatesTokenCount", ToString::to_string(&v).as_str())?;
        }
        if let Some(v) = self.thoughts_token_count.as_ref() {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("thoughtsTokenCount", ToString::to_string(&v).as_str())?;
        }
        if let Some(v) = self.total_token_count.as_ref() {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("totalTokenCount", ToString::to_string(&v).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for UsageMetadata {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "prompt_token_count",
            "promptTokenCount",
            "cached_content_token_count",
            "cachedContentTokenCount",
            "candidates_token_count",
            "candidatesTokenCount",
            "thoughts_token_count",
            "thoughtsTokenCount",
            "total_token_count",
            "totalTokenCount",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PromptTokenCount,
            CachedContentTokenCount,
            CandidatesTokenCount,
            ThoughtsTokenCount,
            TotalTokenCount,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "promptTokenCount" | "prompt_token_count" => Ok(GeneratedField::PromptTokenCount),
                            "cachedContentTokenCount" | "cached_content_token_count" => Ok(GeneratedField::CachedContentTokenCount),
                            "candidatesTokenCount" | "candidates_token_count" => Ok(GeneratedField::CandidatesTokenCount),
                            "thoughtsTokenCount" | "thoughts_token_count" => Ok(GeneratedField::ThoughtsTokenCount),
                            "totalTokenCount" | "total_token_count" => Ok(GeneratedField::TotalTokenCount),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = UsageMetadata;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.UsageMetadata")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<UsageMetadata, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut prompt_token_count__ = None;
                let mut cached_content_token_count__ = None;
                let mut candidates_token_count__ = None;
                let mut thoughts_token_count__ = None;
                let mut total_token_count__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::PromptTokenCount => {
                            if prompt_token_count__.is_some() {
                                return Err(serde::de::Error::duplicate_field("promptTokenCount"));
                            }
                            prompt_token_count__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::CachedContentTokenCount => {
                            if cached_content_token_count__.is_some() {
                                return Err(serde::de::Error::duplicate_field("cachedContentTokenCount"));
                            }
                            cached_content_token_count__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::CandidatesTokenCount => {
                            if candidates_token_count__.is_some() {
                                return Err(serde::de::Error::duplicate_field("candidatesTokenCount"));
                            }
                            candidates_token_count__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::ThoughtsTokenCount => {
                            if thoughts_token_count__.is_some() {
                                return Err(serde::de::Error::duplicate_field("thoughtsTokenCount"));
                            }
                            thoughts_token_count__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::TotalTokenCount => {
                            if total_token_count__.is_some() {
                                return Err(serde::de::Error::duplicate_field("totalTokenCount"));
                            }
                            total_token_count__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(UsageMetadata {
                    prompt_token_count: prompt_token_count__,
                    cached_content_token_count: cached_content_token_count__,
                    candidates_token_count: candidates_token_count__,
                    thoughts_token_count: thoughts_token_count__,
                    total_token_count: total_token_count__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.UsageMetadata", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for UserInput {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.parts.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.UserInput", len)?;
        if !self.parts.is_empty() {
            struct_ser.serialize_field("parts", &self.parts)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for UserInput {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "parts",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Parts,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "parts" => Ok(GeneratedField::Parts),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = UserInput;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.UserInput")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<UserInput, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut parts__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Parts => {
                            if parts__.is_some() {
                                return Err(serde::de::Error::duplicate_field("parts"));
                            }
                            parts__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(UserInput {
                    parts: parts__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.UserInput", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for user_input::Media {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.mime_type.is_some() {
            len += 1;
        }
        if self.description.is_some() {
            len += 1;
        }
        if self.data.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.UserInput.Media", len)?;
        if let Some(v) = self.mime_type.as_ref() {
            struct_ser.serialize_field("mimeType", v)?;
        }
        if let Some(v) = self.description.as_ref() {
            struct_ser.serialize_field("description", v)?;
        }
        if let Some(v) = self.data.as_ref() {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("data", pbjson::private::base64::encode(&v).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for user_input::Media {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "mime_type",
            "mimeType",
            "description",
            "data",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            MimeType,
            Description,
            Data,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "mimeType" | "mime_type" => Ok(GeneratedField::MimeType),
                            "description" => Ok(GeneratedField::Description),
                            "data" => Ok(GeneratedField::Data),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = user_input::Media;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.UserInput.Media")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<user_input::Media, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut mime_type__ = None;
                let mut description__ = None;
                let mut data__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::MimeType => {
                            if mime_type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("mimeType"));
                            }
                            mime_type__ = map_.next_value()?;
                        }
                        GeneratedField::Description => {
                            if description__.is_some() {
                                return Err(serde::de::Error::duplicate_field("description"));
                            }
                            description__ = map_.next_value()?;
                        }
                        GeneratedField::Data => {
                            if data__.is_some() {
                                return Err(serde::de::Error::duplicate_field("data"));
                            }
                            data__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::BytesDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(user_input::Media {
                    mime_type: mime_type__,
                    description: description__,
                    data: data__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.UserInput.Media", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for user_input::Part {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.part.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.UserInput.Part", len)?;
        if let Some(v) = self.part.as_ref() {
            match v {
                user_input::part::Part::Text(v) => {
                    struct_ser.serialize_field("text", v)?;
                }
                user_input::part::Part::Media(v) => {
                    struct_ser.serialize_field("media", v)?;
                }
                user_input::part::Part::SlashCommand(v) => {
                    struct_ser.serialize_field("slashCommand", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for user_input::Part {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "text",
            "media",
            "slash_command",
            "slashCommand",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Text,
            Media,
            SlashCommand,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "text" => Ok(GeneratedField::Text),
                            "media" => Ok(GeneratedField::Media),
                            "slashCommand" | "slash_command" => Ok(GeneratedField::SlashCommand),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = user_input::Part;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.UserInput.Part")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<user_input::Part, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut part__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Text => {
                            if part__.is_some() {
                                return Err(serde::de::Error::duplicate_field("text"));
                            }
                            part__ = map_.next_value::<::std::option::Option<_>>()?.map(user_input::part::Part::Text);
                        }
                        GeneratedField::Media => {
                            if part__.is_some() {
                                return Err(serde::de::Error::duplicate_field("media"));
                            }
                            part__ = map_.next_value::<::std::option::Option<_>>()?.map(user_input::part::Part::Media)
;
                        }
                        GeneratedField::SlashCommand => {
                            if part__.is_some() {
                                return Err(serde::de::Error::duplicate_field("slashCommand"));
                            }
                            part__ = map_.next_value::<::std::option::Option<_>>()?.map(user_input::part::Part::SlashCommand)
;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(user_input::Part {
                    part: part__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.UserInput.Part", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for user_input::SlashCommand {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.name.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.UserInput.SlashCommand", len)?;
        if let Some(v) = self.name.as_ref() {
            struct_ser.serialize_field("name", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for user_input::SlashCommand {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "name",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Name,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "name" => Ok(GeneratedField::Name),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = user_input::SlashCommand;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.UserInput.SlashCommand")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<user_input::SlashCommand, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut name__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(user_input::SlashCommand {
                    name: name__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.UserInput.SlashCommand", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for UserQuestion {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.question_type.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.UserQuestion", len)?;
        if let Some(v) = self.question_type.as_ref() {
            match v {
                user_question::QuestionType::MultipleChoice(v) => {
                    struct_ser.serialize_field("multipleChoice", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for UserQuestion {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "multiple_choice",
            "multipleChoice",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            MultipleChoice,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "multipleChoice" | "multiple_choice" => Ok(GeneratedField::MultipleChoice),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = UserQuestion;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.UserQuestion")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<UserQuestion, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut question_type__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::MultipleChoice => {
                            if question_type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("multipleChoice"));
                            }
                            question_type__ = map_.next_value::<::std::option::Option<_>>()?.map(user_question::QuestionType::MultipleChoice)
;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(UserQuestion {
                    question_type: question_type__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.UserQuestion", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for UserQuestionAnswer {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.answer.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.UserQuestionAnswer", len)?;
        if let Some(v) = self.answer.as_ref() {
            match v {
                user_question_answer::Answer::Unanswered(v) => {
                    struct_ser.serialize_field("unanswered", v)?;
                }
                user_question_answer::Answer::MultipleChoiceAnswer(v) => {
                    struct_ser.serialize_field("multipleChoiceAnswer", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for UserQuestionAnswer {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "unanswered",
            "multiple_choice_answer",
            "multipleChoiceAnswer",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Unanswered,
            MultipleChoiceAnswer,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "unanswered" => Ok(GeneratedField::Unanswered),
                            "multipleChoiceAnswer" | "multiple_choice_answer" => Ok(GeneratedField::MultipleChoiceAnswer),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = UserQuestionAnswer;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.UserQuestionAnswer")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<UserQuestionAnswer, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut answer__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Unanswered => {
                            if answer__.is_some() {
                                return Err(serde::de::Error::duplicate_field("unanswered"));
                            }
                            answer__ = map_.next_value::<::std::option::Option<_>>()?.map(user_question_answer::Answer::Unanswered);
                        }
                        GeneratedField::MultipleChoiceAnswer => {
                            if answer__.is_some() {
                                return Err(serde::de::Error::duplicate_field("multipleChoiceAnswer"));
                            }
                            answer__ = map_.next_value::<::std::option::Option<_>>()?.map(user_question_answer::Answer::MultipleChoiceAnswer)
;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(UserQuestionAnswer {
                    answer: answer__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.UserQuestionAnswer", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for UserQuestionsConfig {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.enabled.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.UserQuestionsConfig", len)?;
        if let Some(v) = self.enabled.as_ref() {
            struct_ser.serialize_field("enabled", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for UserQuestionsConfig {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "enabled",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Enabled,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "enabled" => Ok(GeneratedField::Enabled),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = UserQuestionsConfig;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.UserQuestionsConfig")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<UserQuestionsConfig, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut enabled__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Enabled => {
                            if enabled__.is_some() {
                                return Err(serde::de::Error::duplicate_field("enabled"));
                            }
                            enabled__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(UserQuestionsConfig {
                    enabled: enabled__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.UserQuestionsConfig", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for UserQuestionsRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.questions.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.UserQuestionsRequest", len)?;
        if !self.questions.is_empty() {
            struct_ser.serialize_field("questions", &self.questions)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for UserQuestionsRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "questions",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Questions,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "questions" => Ok(GeneratedField::Questions),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = UserQuestionsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.UserQuestionsRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<UserQuestionsRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut questions__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Questions => {
                            if questions__.is_some() {
                                return Err(serde::de::Error::duplicate_field("questions"));
                            }
                            questions__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(UserQuestionsRequest {
                    questions: questions__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.UserQuestionsRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for UserQuestionsResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.trajectory_id.is_some() {
            len += 1;
        }
        if self.step_index.is_some() {
            len += 1;
        }
        if self.result.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.UserQuestionsResponse", len)?;
        if let Some(v) = self.trajectory_id.as_ref() {
            struct_ser.serialize_field("trajectoryId", v)?;
        }
        if let Some(v) = self.step_index.as_ref() {
            struct_ser.serialize_field("stepIndex", v)?;
        }
        if let Some(v) = self.result.as_ref() {
            match v {
                user_questions_response::Result::Cancelled(v) => {
                    struct_ser.serialize_field("cancelled", v)?;
                }
                user_questions_response::Result::Response(v) => {
                    struct_ser.serialize_field("response", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for UserQuestionsResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "trajectory_id",
            "trajectoryId",
            "step_index",
            "stepIndex",
            "cancelled",
            "response",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TrajectoryId,
            StepIndex,
            Cancelled,
            Response,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "trajectoryId" | "trajectory_id" => Ok(GeneratedField::TrajectoryId),
                            "stepIndex" | "step_index" => Ok(GeneratedField::StepIndex),
                            "cancelled" => Ok(GeneratedField::Cancelled),
                            "response" => Ok(GeneratedField::Response),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = UserQuestionsResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.UserQuestionsResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<UserQuestionsResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut trajectory_id__ = None;
                let mut step_index__ = None;
                let mut result__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::TrajectoryId => {
                            if trajectory_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("trajectoryId"));
                            }
                            trajectory_id__ = map_.next_value()?;
                        }
                        GeneratedField::StepIndex => {
                            if step_index__.is_some() {
                                return Err(serde::de::Error::duplicate_field("stepIndex"));
                            }
                            step_index__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::Cancelled => {
                            if result__.is_some() {
                                return Err(serde::de::Error::duplicate_field("cancelled"));
                            }
                            result__ = map_.next_value::<::std::option::Option<_>>()?.map(user_questions_response::Result::Cancelled);
                        }
                        GeneratedField::Response => {
                            if result__.is_some() {
                                return Err(serde::de::Error::duplicate_field("response"));
                            }
                            result__ = map_.next_value::<::std::option::Option<_>>()?.map(user_questions_response::Result::Response)
;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(UserQuestionsResponse {
                    trajectory_id: trajectory_id__,
                    step_index: step_index__,
                    result: result__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.UserQuestionsResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for user_questions_response::QuestionsResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.answers.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.UserQuestionsResponse.QuestionsResponse", len)?;
        if !self.answers.is_empty() {
            struct_ser.serialize_field("answers", &self.answers)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for user_questions_response::QuestionsResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "answers",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Answers,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "answers" => Ok(GeneratedField::Answers),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = user_questions_response::QuestionsResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.UserQuestionsResponse.QuestionsResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<user_questions_response::QuestionsResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut answers__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Answers => {
                            if answers__.is_some() {
                                return Err(serde::de::Error::duplicate_field("answers"));
                            }
                            answers__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(user_questions_response::QuestionsResponse {
                    answers: answers__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.UserQuestionsResponse.QuestionsResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for VertexEndpoint {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.base_url.is_some() {
            len += 1;
        }
        if !self.http_headers.is_empty() {
            len += 1;
        }
        if self.project.is_some() {
            len += 1;
        }
        if self.location.is_some() {
            len += 1;
        }
        if self.options.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.VertexEndpoint", len)?;
        if let Some(v) = self.base_url.as_ref() {
            struct_ser.serialize_field("baseUrl", v)?;
        }
        if !self.http_headers.is_empty() {
            struct_ser.serialize_field("httpHeaders", &self.http_headers)?;
        }
        if let Some(v) = self.project.as_ref() {
            struct_ser.serialize_field("project", v)?;
        }
        if let Some(v) = self.location.as_ref() {
            struct_ser.serialize_field("location", v)?;
        }
        if let Some(v) = self.options.as_ref() {
            struct_ser.serialize_field("options", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for VertexEndpoint {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "base_url",
            "baseUrl",
            "http_headers",
            "httpHeaders",
            "project",
            "location",
            "options",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            BaseUrl,
            HttpHeaders,
            Project,
            Location,
            Options,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "baseUrl" | "base_url" => Ok(GeneratedField::BaseUrl),
                            "httpHeaders" | "http_headers" => Ok(GeneratedField::HttpHeaders),
                            "project" => Ok(GeneratedField::Project),
                            "location" => Ok(GeneratedField::Location),
                            "options" => Ok(GeneratedField::Options),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = VertexEndpoint;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.VertexEndpoint")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<VertexEndpoint, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut base_url__ = None;
                let mut http_headers__ = None;
                let mut project__ = None;
                let mut location__ = None;
                let mut options__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::BaseUrl => {
                            if base_url__.is_some() {
                                return Err(serde::de::Error::duplicate_field("baseUrl"));
                            }
                            base_url__ = map_.next_value()?;
                        }
                        GeneratedField::HttpHeaders => {
                            if http_headers__.is_some() {
                                return Err(serde::de::Error::duplicate_field("httpHeaders"));
                            }
                            http_headers__ = Some(
                                map_.next_value::<std::collections::HashMap<_, _>>()?
                            );
                        }
                        GeneratedField::Project => {
                            if project__.is_some() {
                                return Err(serde::de::Error::duplicate_field("project"));
                            }
                            project__ = map_.next_value()?;
                        }
                        GeneratedField::Location => {
                            if location__.is_some() {
                                return Err(serde::de::Error::duplicate_field("location"));
                            }
                            location__ = map_.next_value()?;
                        }
                        GeneratedField::Options => {
                            if options__.is_some() {
                                return Err(serde::de::Error::duplicate_field("options"));
                            }
                            options__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(VertexEndpoint {
                    base_url: base_url__,
                    http_headers: http_headers__.unwrap_or_default(),
                    project: project__,
                    location: location__,
                    options: options__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.VertexEndpoint", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ViewFileToolConfig {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.enabled.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.ViewFileToolConfig", len)?;
        if let Some(v) = self.enabled.as_ref() {
            struct_ser.serialize_field("enabled", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ViewFileToolConfig {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "enabled",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Enabled,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "enabled" => Ok(GeneratedField::Enabled),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ViewFileToolConfig;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.ViewFileToolConfig")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ViewFileToolConfig, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut enabled__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Enabled => {
                            if enabled__.is_some() {
                                return Err(serde::de::Error::duplicate_field("enabled"));
                            }
                            enabled__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ViewFileToolConfig {
                    enabled: enabled__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.ViewFileToolConfig", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Workspace {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.workspace_type.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.Workspace", len)?;
        if let Some(v) = self.workspace_type.as_ref() {
            match v {
                workspace::WorkspaceType::FilesystemWorkspace(v) => {
                    struct_ser.serialize_field("filesystemWorkspace", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Workspace {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "filesystem_workspace",
            "filesystemWorkspace",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            FilesystemWorkspace,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "filesystemWorkspace" | "filesystem_workspace" => Ok(GeneratedField::FilesystemWorkspace),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Workspace;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.Workspace")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Workspace, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut workspace_type__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::FilesystemWorkspace => {
                            if workspace_type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("filesystemWorkspace"));
                            }
                            workspace_type__ = map_.next_value::<::std::option::Option<_>>()?.map(workspace::WorkspaceType::FilesystemWorkspace)
;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(Workspace {
                    workspace_type: workspace_type__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.Workspace", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for WriteToFileToolConfig {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.enabled.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("antigravity.localharness.WriteToFileToolConfig", len)?;
        if let Some(v) = self.enabled.as_ref() {
            struct_ser.serialize_field("enabled", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for WriteToFileToolConfig {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "enabled",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Enabled,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "enabled" => Ok(GeneratedField::Enabled),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = WriteToFileToolConfig;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct antigravity.localharness.WriteToFileToolConfig")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<WriteToFileToolConfig, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut enabled__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Enabled => {
                            if enabled__.is_some() {
                                return Err(serde::de::Error::duplicate_field("enabled"));
                            }
                            enabled__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(WriteToFileToolConfig {
                    enabled: enabled__,
                })
            }
        }
        deserializer.deserialize_struct("antigravity.localharness.WriteToFileToolConfig", FIELDS, GeneratedVisitor)
    }
}
