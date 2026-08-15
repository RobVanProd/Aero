use crate::ir::{EnumSchema, LogicalType};
use crate::primitive_contract::PrimitiveKind;

/// Private physical primitive-lane policy for recursively lowered CopyData.
///
/// Production recursive aggregates currently select [`Self::Legacy`]. The
/// exact policy is selected only by the existing exact-profile direct scalar
/// and admitted flat-array roots in the backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CopyDataLayoutPolicy {
    Legacy,
    ExactI32,
}

impl CopyDataLayoutPolicy {
    fn primitive_llvm_type(self, primitive: PrimitiveKind) -> &'static str {
        match (self, primitive) {
            (Self::ExactI32, PrimitiveKind::Int) => "i32",
            _ => primitive.copy_data_llvm_type(),
        }
    }

    fn primitive_zero(self, primitive: PrimitiveKind) -> &'static str {
        match (self, primitive) {
            (Self::ExactI32, PrimitiveKind::Int) => "0",
            _ => primitive.copy_data_zero(),
        }
    }

    fn primitive_alignment(self, primitive: PrimitiveKind) -> usize {
        match (self, primitive) {
            (Self::ExactI32, PrimitiveKind::Int) => 4,
            _ => primitive.alignment(),
        }
    }
}

/// One recursive physical descriptor for a logical CopyData value.
#[derive(Debug, Clone, Copy)]
pub(crate) struct CopyDataLayout<'a> {
    logical_type: &'a LogicalType,
    policy: CopyDataLayoutPolicy,
}

impl<'a> CopyDataLayout<'a> {
    pub(crate) fn legacy(logical_type: &'a LogicalType) -> Self {
        Self::with_policy(logical_type, CopyDataLayoutPolicy::Legacy)
    }

    pub(crate) fn with_policy(logical_type: &'a LogicalType, policy: CopyDataLayoutPolicy) -> Self {
        Self {
            logical_type,
            policy,
        }
    }

    /// Render with the verifier's accepted raw private-struct spelling for
    /// module-local contract controls.
    #[cfg(test)]
    pub(crate) fn llvm_type(self) -> String {
        self.llvm_type_with(&|name| format!("%aero.struct.{name}"))
    }

    /// Preserve the verifier's historical diagnostic hint for an unsupported
    /// logical type without making that type renderable by the backend.
    pub(crate) fn physical_hint(self) -> String {
        self.try_llvm_type_with(&|name| format!("%aero.struct.{name}"))
            .unwrap_or_else(|| self.logical_type.to_string())
    }

    /// Render while allowing the backend to preserve its accepted generic
    /// private-struct symbol spelling. The callback owns spelling only; this
    /// descriptor remains the recursive topology and primitive-lane authority.
    pub(crate) fn llvm_type_with(self, named_struct: &impl Fn(&str) -> String) -> String {
        self.try_llvm_type_with(named_struct)
            .expect("verified CopyData layout excludes non-CopyData logical types")
    }

    fn try_llvm_type_with(self, named_struct: &impl Fn(&str) -> String) -> Option<String> {
        if let Some(primitive) = PrimitiveKind::from_logical_type(self.logical_type) {
            return Some(self.policy.primitive_llvm_type(primitive).to_string());
        }
        match self.logical_type {
            LogicalType::Int | LogicalType::Float | LogicalType::Bool | LogicalType::Char => {
                unreachable!("primitive logical types returned above")
            }
            LogicalType::Array { element, count } => Some(format!(
                "[{count} x {}]",
                Self::with_policy(element, self.policy).try_llvm_type_with(named_struct)?
            )),
            LogicalType::Struct { name, .. } => Some(named_struct(name)),
            LogicalType::Tuple { elements } => Some(format!(
                "{{ {} }}",
                elements
                    .iter()
                    .map(|element| {
                        Self::with_policy(element, self.policy).try_llvm_type_with(named_struct)
                    })
                    .collect::<Option<Vec<_>>>()?
                    .join(", ")
            )),
            LogicalType::EnumFields { fields } => Some(format!(
                "{{ {} }}",
                fields
                    .iter()
                    .map(|field| {
                        Self::with_policy(field, self.policy).try_llvm_type_with(named_struct)
                    })
                    .collect::<Option<Vec<_>>>()?
                    .join(", ")
            )),
            LogicalType::Void
            | LogicalType::String
            | LogicalType::ImmutableReference { .. }
            | LogicalType::MutableReference { .. }
            | LogicalType::Enum { .. } => None,
        }
    }

    pub(crate) fn zero_value(self) -> String {
        if let Some(primitive) = PrimitiveKind::from_logical_type(self.logical_type) {
            return self.policy.primitive_zero(primitive).to_string();
        }
        match self.logical_type {
            LogicalType::Int | LogicalType::Float | LogicalType::Bool | LogicalType::Char => {
                unreachable!("primitive logical types returned above")
            }
            LogicalType::Array { .. }
            | LogicalType::Struct { .. }
            | LogicalType::Tuple { .. }
            | LogicalType::EnumFields { .. } => "zeroinitializer".to_string(),
            LogicalType::Void
            | LogicalType::String
            | LogicalType::ImmutableReference { .. }
            | LogicalType::MutableReference { .. }
            | LogicalType::Enum { .. } => {
                unreachable!("verified CopyData zeros exclude non-CopyData logical types")
            }
        }
    }

    /// Maximum primitive-leaf alignment in the descriptor. LLVM emission keeps
    /// its accepted explicit alignment sites; this query centralizes leaf
    /// identity without introducing a new aggregate ABI rule.
    pub(crate) fn alignment(self) -> Option<usize> {
        if let Some(primitive) = PrimitiveKind::from_logical_type(self.logical_type) {
            return Some(self.policy.primitive_alignment(primitive));
        }
        match self.logical_type {
            LogicalType::Array { element, .. } => {
                Self::with_policy(element, self.policy).alignment()
            }
            LogicalType::Struct { fields, .. } | LogicalType::EnumFields { fields } => fields
                .iter()
                .filter_map(|field| Self::with_policy(field, self.policy).alignment())
                .max(),
            LogicalType::Tuple { elements } => elements
                .iter()
                .filter_map(|element| Self::with_policy(element, self.policy).alignment())
                .max(),
            LogicalType::Int | LogicalType::Float | LogicalType::Bool | LogicalType::Char => {
                unreachable!("primitive logical types returned above")
            }
            LogicalType::Void
            | LogicalType::String
            | LogicalType::ImmutableReference { .. }
            | LogicalType::MutableReference { .. }
            | LogicalType::Enum { .. } => None,
        }
    }
}

/// Shared storage topology for unit, compact-scalar, and general enums.
#[derive(Debug, Clone, Copy)]
pub(crate) struct EnumStorageLayout<'a> {
    schema: &'a EnumSchema,
    policy: CopyDataLayoutPolicy,
}

impl<'a> EnumStorageLayout<'a> {
    const TAG_LANE: usize = 0;
    const COMPACT_NUMERIC_LANE: usize = 1;
    const COMPACT_BOOLEAN_LANE: usize = 2;

    pub(crate) fn legacy(schema: &'a EnumSchema) -> Self {
        Self {
            schema,
            policy: CopyDataLayoutPolicy::Legacy,
        }
    }

    pub(crate) fn with_policy(schema: &'a EnumSchema, policy: CopyDataLayoutPolicy) -> Self {
        Self { schema, policy }
    }

    pub(crate) fn is_unit(self) -> bool {
        self.schema.is_unit()
    }

    pub(crate) fn is_compact(self) -> bool {
        self.schema
            .variants
            .iter()
            .filter_map(|variant| variant.payload.as_ref())
            .all(|payload| {
                matches!(
                    payload,
                    LogicalType::Int | LogicalType::Float | LogicalType::Bool
                )
            })
    }

    pub(crate) fn tag_lane(self) -> usize {
        Self::TAG_LANE
    }

    pub(crate) fn compact_numeric_lane(self) -> Option<usize> {
        (!self.is_unit() && self.is_compact()).then_some(Self::COMPACT_NUMERIC_LANE)
    }

    pub(crate) fn compact_boolean_lane(self) -> Option<usize> {
        (!self.is_unit() && self.is_compact()).then_some(Self::COMPACT_BOOLEAN_LANE)
    }

    fn compact_numeric_primitive(self) -> PrimitiveKind {
        if self
            .schema
            .variants
            .iter()
            .any(|variant| matches!(variant.payload.as_ref(), Some(LogicalType::Float)))
        {
            PrimitiveKind::Float
        } else if self
            .schema
            .variants
            .iter()
            .any(|variant| matches!(variant.payload.as_ref(), Some(LogicalType::Int)))
        {
            PrimitiveKind::Int
        } else {
            // Preserve the accepted otherwise-unused numeric lane for bool-only
            // compact enums.
            PrimitiveKind::Float
        }
    }

    pub(crate) fn enum_llvm_type(self) -> String {
        self.enum_llvm_type_with(&|name| format!("%aero.struct.{name}"))
    }

    pub(crate) fn enum_llvm_type_with(self, named_struct: &impl Fn(&str) -> String) -> String {
        if self.is_unit() {
            return "i32".to_string();
        }
        if self.is_compact() {
            return format!(
                "{{ i32, {}, i1 }}",
                self.policy
                    .primitive_llvm_type(self.compact_numeric_primitive())
            );
        }
        let mut lanes = vec!["i32".to_string()];
        lanes.extend(
            self.schema
                .variants
                .iter()
                .filter_map(|variant| variant.payload.as_ref())
                .map(|payload| {
                    CopyDataLayout::with_policy(payload, self.policy).llvm_type_with(named_struct)
                }),
        );
        format!("{{ {} }}", lanes.join(", "))
    }

    pub(crate) fn payload_lane(self, variant_index: usize) -> Option<usize> {
        let payload = self.schema.variants.get(variant_index)?.payload.as_ref()?;
        if self.is_compact() {
            return match payload {
                LogicalType::Int | LogicalType::Float => Some(Self::COMPACT_NUMERIC_LANE),
                LogicalType::Bool => Some(Self::COMPACT_BOOLEAN_LANE),
                _ => None,
            };
        }
        Some(
            Self::TAG_LANE
                + 1
                + self.schema.variants[..variant_index]
                    .iter()
                    .filter(|variant| variant.payload.is_some())
                    .count(),
        )
    }

    pub(crate) fn payload_variants(self) -> Vec<(usize, usize, &'a LogicalType)> {
        self.schema
            .variants
            .iter()
            .enumerate()
            .filter_map(|(variant_index, variant)| {
                variant.payload.as_ref().map(|payload| {
                    (
                        variant_index,
                        self.payload_lane(variant_index)
                            .expect("payload-bearing enum variant has a storage lane"),
                        payload,
                    )
                })
            })
            .collect()
    }

    pub(crate) fn lane_llvm_type(
        self,
        lane: usize,
        named_struct: &impl Fn(&str) -> String,
    ) -> Option<String> {
        match lane {
            Self::TAG_LANE => Some("i32".to_string()),
            Self::COMPACT_NUMERIC_LANE if self.is_compact() => Some(
                self.policy
                    .primitive_llvm_type(self.compact_numeric_primitive())
                    .to_string(),
            ),
            Self::COMPACT_BOOLEAN_LANE if self.is_compact() => Some("i1".to_string()),
            _ if !self.is_compact() => self
                .schema
                .variants
                .iter()
                .filter_map(|variant| variant.payload.as_ref())
                .nth(lane.checked_sub(1)?)
                .map(|payload| {
                    CopyDataLayout::with_policy(payload, self.policy).llvm_type_with(named_struct)
                }),
            _ => None,
        }
    }

    pub(crate) fn lane_zero_value(self, lane: usize) -> Option<String> {
        match lane {
            Self::TAG_LANE => Some("0".to_string()),
            Self::COMPACT_NUMERIC_LANE if self.is_compact() => Some(
                self.policy
                    .primitive_zero(self.compact_numeric_primitive())
                    .to_string(),
            ),
            Self::COMPACT_BOOLEAN_LANE if self.is_compact() => Some("false".to_string()),
            _ if !self.is_compact() => self
                .schema
                .variants
                .iter()
                .filter_map(|variant| variant.payload.as_ref())
                .nth(lane.checked_sub(1)?)
                .map(|payload| CopyDataLayout::with_policy(payload, self.policy).zero_value()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::EnumVariantSchema;

    fn variant(name: &str, payload: Option<LogicalType>) -> EnumVariantSchema {
        EnumVariantSchema {
            name: name.to_string(),
            payload,
        }
    }

    #[test]
    fn recursive_layout_preserves_legacy_and_exact_lane_policies() {
        let logical_type = LogicalType::Struct {
            name: "Frame".to_string(),
            fields: vec![
                LogicalType::Array {
                    element: Box::new(LogicalType::Int),
                    count: 2,
                },
                LogicalType::Tuple {
                    elements: vec![LogicalType::Bool, LogicalType::Int],
                },
            ],
        };
        let fields = match &logical_type {
            LogicalType::Struct { fields, .. } => fields,
            _ => unreachable!(),
        };
        assert_eq!(
            CopyDataLayout::legacy(&fields[0]).llvm_type(),
            "[2 x double]"
        );
        assert_eq!(
            CopyDataLayout::with_policy(&fields[0], CopyDataLayoutPolicy::ExactI32).llvm_type(),
            "[2 x i32]"
        );
        assert_eq!(
            CopyDataLayout::legacy(&fields[1]).llvm_type(),
            "{ i1, double }"
        );
        assert_eq!(CopyDataLayout::legacy(&logical_type).alignment(), Some(8));
        assert_eq!(
            CopyDataLayout::with_policy(&logical_type, CopyDataLayoutPolicy::ExactI32).alignment(),
            Some(4)
        );
        assert_eq!(
            CopyDataLayout::legacy(&logical_type).zero_value(),
            "zeroinitializer"
        );
        let nested_zero_array = LogicalType::Array {
            element: Box::new(LogicalType::Array {
                element: Box::new(LogicalType::Int),
                count: 2,
            }),
            count: 0,
        };
        assert_eq!(
            CopyDataLayout::legacy(&nested_zero_array).llvm_type(),
            "[0 x [2 x double]]"
        );
        assert_eq!(
            CopyDataLayout::with_policy(&nested_zero_array, CopyDataLayoutPolicy::ExactI32,)
                .llvm_type(),
            "[0 x [2 x i32]]"
        );
    }

    #[test]
    fn primitive_type_zero_and_alignment_matrix_is_centralized() {
        for (logical_type, llvm_type, zero, alignment) in [
            (LogicalType::Int, "double", "0x0000000000000000", 8),
            (LogicalType::Float, "double", "0x0000000000000000", 8),
            (LogicalType::Bool, "i1", "false", 1),
            (LogicalType::Char, "i32", "0", 4),
        ] {
            let layout = CopyDataLayout::legacy(&logical_type);
            assert_eq!(layout.llvm_type(), llvm_type);
            assert_eq!(layout.zero_value(), zero);
            assert_eq!(layout.alignment(), Some(alignment));
        }

        for (logical_type, llvm_type, zero, alignment) in [
            (LogicalType::Int, "i32", "0", 4),
            (LogicalType::Float, "double", "0x0000000000000000", 8),
            (LogicalType::Bool, "i1", "false", 1),
            (LogicalType::Char, "i32", "0", 4),
        ] {
            let layout = CopyDataLayout::with_policy(&logical_type, CopyDataLayoutPolicy::ExactI32);
            assert_eq!(layout.llvm_type(), llvm_type);
            assert_eq!(layout.zero_value(), zero);
            assert_eq!(layout.alignment(), Some(alignment));
        }
    }

    #[test]
    fn unsupported_types_remain_diagnostic_hints_only() {
        assert_eq!(
            CopyDataLayout::legacy(&LogicalType::String).physical_hint(),
            "String"
        );
    }

    #[test]
    #[should_panic(expected = "verified CopyData layout excludes non-CopyData logical types")]
    fn unsupported_types_cannot_be_rendered_for_the_backend() {
        let _ = CopyDataLayout::legacy(&LogicalType::String).llvm_type();
    }

    #[test]
    fn named_struct_spelling_is_an_emitter_callback() {
        let logical_type = LogicalType::Struct {
            name: "Window$Int".to_string(),
            fields: vec![LogicalType::Int],
        };
        assert_eq!(
            CopyDataLayout::legacy(&logical_type).llvm_type(),
            "%aero.struct.Window$Int"
        );
        assert_eq!(
            CopyDataLayout::legacy(&logical_type)
                .llvm_type_with(&|name| format!("%\"aero.struct.{name}\"")),
            "%\"aero.struct.Window$Int\""
        );
    }

    #[test]
    fn enum_storage_owns_compact_and_general_lane_topology() {
        let unit = EnumSchema {
            name: "State".to_string(),
            variants: vec![variant("Idle", None), variant("Ready", None)],
        };
        let unit = EnumStorageLayout::legacy(&unit);
        assert!(unit.is_unit());
        assert_eq!(unit.tag_lane(), 0);
        assert_eq!(unit.enum_llvm_type(), "i32");
        assert_eq!(
            unit.lane_llvm_type(unit.tag_lane(), &|name| format!("%aero.struct.{name}")),
            Some("i32".to_string())
        );
        assert_eq!(unit.lane_zero_value(unit.tag_lane()).as_deref(), Some("0"));
        assert!(unit.payload_variants().is_empty());

        let compact = EnumSchema {
            name: "Scalar".to_string(),
            variants: vec![
                variant("Empty", None),
                variant("Number", Some(LogicalType::Int)),
                variant("Flag", Some(LogicalType::Bool)),
            ],
        };
        let compact = EnumStorageLayout::legacy(&compact);
        assert!(compact.is_compact());
        assert_eq!(compact.tag_lane(), 0);
        assert_eq!(compact.compact_numeric_lane(), Some(1));
        assert_eq!(compact.compact_boolean_lane(), Some(2));
        assert_eq!(compact.enum_llvm_type(), "{ i32, double, i1 }");
        assert_eq!(compact.payload_lane(1), Some(1));
        assert_eq!(compact.payload_lane(2), Some(2));
        assert_eq!(
            compact.lane_zero_value(1).as_deref(),
            Some("0x0000000000000000")
        );
        assert_eq!(compact.lane_zero_value(2).as_deref(), Some("false"));
        assert_eq!(
            compact
                .payload_variants()
                .into_iter()
                .map(|(variant, lane, payload)| (variant, lane, payload.clone()))
                .collect::<Vec<_>>(),
            vec![(1, 1, LogicalType::Int), (2, 2, LogicalType::Bool)]
        );

        let general = EnumSchema {
            name: "Outcome".to_string(),
            variants: vec![
                variant("Empty", None),
                variant(
                    "Pair",
                    Some(LogicalType::Tuple {
                        elements: vec![LogicalType::Int, LogicalType::Bool],
                    }),
                ),
                variant("Code", Some(LogicalType::Char)),
            ],
        };
        let general = EnumStorageLayout::legacy(&general);
        assert!(!general.is_compact());
        assert_eq!(general.enum_llvm_type(), "{ i32, { double, i1 }, i32 }");
        assert_eq!(general.payload_lane(1), Some(1));
        assert_eq!(general.payload_lane(2), Some(2));
        assert_eq!(general.compact_numeric_lane(), None);
        assert_eq!(general.compact_boolean_lane(), None);
        assert_eq!(
            general
                .payload_variants()
                .into_iter()
                .map(|(variant, lane, _)| (variant, lane))
                .collect::<Vec<_>>(),
            vec![(1, 1), (2, 2)]
        );
        assert_eq!(
            general.lane_llvm_type(1, &|name| format!("%aero.struct.{name}")),
            Some("{ double, i1 }".to_string())
        );
        assert_eq!(
            general.lane_zero_value(1).as_deref(),
            Some("zeroinitializer")
        );

        let result = EnumSchema {
            name: "Result$Frame$Int".to_string(),
            variants: vec![
                variant(
                    "Ok",
                    Some(LogicalType::Struct {
                        name: "Frame".to_string(),
                        fields: vec![LogicalType::Int],
                    }),
                ),
                variant("Err", Some(LogicalType::Int)),
            ],
        };
        let result = EnumStorageLayout::legacy(&result);
        assert_eq!(
            result.enum_llvm_type(),
            "{ i32, %aero.struct.Frame, double }"
        );
        assert_eq!(result.payload_lane(0), Some(1));
        assert_eq!(result.payload_lane(1), Some(2));
        assert_eq!(
            result.lane_zero_value(1).as_deref(),
            Some("zeroinitializer")
        );
        assert_eq!(
            result.lane_zero_value(2).as_deref(),
            Some("0x0000000000000000")
        );
    }

    #[test]
    fn alternate_int_policy_is_not_selected_by_legacy_enum_layout() {
        let schema = EnumSchema {
            name: "MaybeInt".to_string(),
            variants: vec![
                variant("None", None),
                variant("Some", Some(LogicalType::Int)),
            ],
        };
        assert_eq!(
            EnumStorageLayout::with_policy(&schema, CopyDataLayoutPolicy::ExactI32)
                .enum_llvm_type(),
            "{ i32, i32, i1 }"
        );
        assert_eq!(
            EnumStorageLayout::legacy(&schema).enum_llvm_type(),
            "{ i32, double, i1 }"
        );
    }
}
