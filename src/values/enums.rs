use llvm_sys::core::{LLVMGetTypeKind, LLVMTypeOf};
use llvm_sys::prelude::LLVMValueRef;
use llvm_sys::LLVMTypeKind;

use types::{AnyTypeEnum, BasicTypeEnum};
use values::traits::{AsValueRef, NamedValue};
use values::{
    ArrayValue, FloatValue, FunctionValue, InstructionValue, IntValue, MetadataValue, PhiValue,
    PointerValue, StructValue, VectorValue,
};

macro_rules! enum_value_set {
    ($enum_name:ident: $($args:ident),*) => (
        #[derive(Debug, EnumAsGetters, EnumIntoGetters, EnumIsA, Clone, Copy, PartialEq, Eq)]
        pub enum $enum_name {
            $(
                $args($args),
            )*
        }

        impl AsValueRef for $enum_name {
            fn as_value_ref(&self) -> LLVMValueRef {
                match *self {
                    $(
                        $enum_name::$args(ref t) => t.as_value_ref(),
                    )*
                }
            }
        }

        impl NamedValue for $enum_name {
            fn get_name(&self) -> &std::ffi::CStr {
                match *self {
                    $(
                        $enum_name::$args(ref t) => t.get_name(),
                    )*
                }
            }

            fn set_name(&self, name: &str) {
                match *self {
                    $(
                        $enum_name::$args(ref t) => t.set_name(name),
                    )*
                }
            }
        }

        $(
            impl From<$args> for $enum_name {
                fn from(value: $args) -> $enum_name {
                    $enum_name::$args(value)
                }
            }
        )*

        // REVIEW: Possible encompassing methods to implement:
        // as_instruction, is_sized, get/set metadata
    );
    (@$enum_name:ident: $($args:ident),*) => (
        #[derive(Debug, EnumAsGetters, EnumIntoGetters, EnumIsA, Clone, Copy, PartialEq, Eq)]
        pub enum $enum_name {
            $(
                $args($args),
            )*
        }

        impl AsValueRef for $enum_name {
            fn as_value_ref(&self) -> LLVMValueRef {
                match *self {
                    $(
                        $enum_name::$args(ref t) => t.as_value_ref(),
                    )*
                }
            }
        }

        $(
            impl From<$args> for $enum_name {
                fn from(value: $args) -> $enum_name {
                    $enum_name::$args(value)
                }
            }
        )*

        // REVIEW: Possible encompassing methods to implement:
        // as_instruction, is_sized, get/set metadata
    );
}

enum_value_set! {AggregateValueEnum: ArrayValue, StructValue}
enum_value_set! {AnyValueEnum: ArrayValue, IntValue, FloatValue, PhiValue, FunctionValue, PointerValue, StructValue, VectorValue, InstructionValue}
enum_value_set! {BasicValueEnum: ArrayValue, IntValue, FloatValue, PointerValue, StructValue, VectorValue}
enum_value_set! {@BasicMetadataValueEnum: ArrayValue, IntValue, FloatValue, PointerValue, StructValue, VectorValue, MetadataValue}

impl AnyValueEnum {
    pub(crate) fn new(value: LLVMValueRef) -> AnyValueEnum {
        let type_kind = unsafe { LLVMGetTypeKind(LLVMTypeOf(value)) };

        match type_kind {
            LLVMTypeKind::LLVMFloatTypeKind
            | LLVMTypeKind::LLVMFP128TypeKind
            | LLVMTypeKind::LLVMDoubleTypeKind
            | LLVMTypeKind::LLVMHalfTypeKind
            | LLVMTypeKind::LLVMX86_FP80TypeKind
            | LLVMTypeKind::LLVMPPC_FP128TypeKind => {
                AnyValueEnum::FloatValue(FloatValue::new(value))
            }
            LLVMTypeKind::LLVMIntegerTypeKind => AnyValueEnum::IntValue(IntValue::new(value)),
            LLVMTypeKind::LLVMStructTypeKind => AnyValueEnum::StructValue(StructValue::new(value)),
            LLVMTypeKind::LLVMPointerTypeKind => {
                AnyValueEnum::PointerValue(PointerValue::new(value))
            }
            LLVMTypeKind::LLVMArrayTypeKind => AnyValueEnum::ArrayValue(ArrayValue::new(value)),
            LLVMTypeKind::LLVMVectorTypeKind => AnyValueEnum::VectorValue(VectorValue::new(value)),
            LLVMTypeKind::LLVMFunctionTypeKind => {
                AnyValueEnum::FunctionValue(FunctionValue::new(value).unwrap())
            }
            LLVMTypeKind::LLVMVoidTypeKind => panic!("Void values shouldn't exist."),
            LLVMTypeKind::LLVMMetadataTypeKind => {
                panic!("Metadata values are not supported as AnyValue's.")
            }
            _ => panic!("The given type is not supported."),
        }
    }

    pub fn get_type(&self) -> AnyTypeEnum {
        let type_ = unsafe { LLVMTypeOf(self.as_value_ref()) };

        AnyTypeEnum::new(type_)
    }
}

impl BasicValueEnum {
    pub(crate) fn new(value: LLVMValueRef) -> BasicValueEnum {
        let type_kind = unsafe { LLVMGetTypeKind(LLVMTypeOf(value)) };

        match type_kind {
            LLVMTypeKind::LLVMFloatTypeKind
            | LLVMTypeKind::LLVMFP128TypeKind
            | LLVMTypeKind::LLVMDoubleTypeKind
            | LLVMTypeKind::LLVMHalfTypeKind
            | LLVMTypeKind::LLVMX86_FP80TypeKind
            | LLVMTypeKind::LLVMPPC_FP128TypeKind => {
                BasicValueEnum::FloatValue(FloatValue::new(value))
            }
            LLVMTypeKind::LLVMIntegerTypeKind => BasicValueEnum::IntValue(IntValue::new(value)),
            LLVMTypeKind::LLVMStructTypeKind => {
                BasicValueEnum::StructValue(StructValue::new(value))
            }
            LLVMTypeKind::LLVMPointerTypeKind => {
                BasicValueEnum::PointerValue(PointerValue::new(value))
            }
            LLVMTypeKind::LLVMArrayTypeKind => BasicValueEnum::ArrayValue(ArrayValue::new(value)),
            LLVMTypeKind::LLVMVectorTypeKind => {
                BasicValueEnum::VectorValue(VectorValue::new(value))
            }
            _ => unreachable!("The given type is not a basic type."),
        }
    }

    pub fn get_type(&self) -> BasicTypeEnum {
        let type_ = unsafe { LLVMTypeOf(self.as_value_ref()) };

        BasicTypeEnum::new(type_)
    }

    pub fn as_instruction(&self) -> Option<InstructionValue> {
        match *self {
            BasicValueEnum::ArrayValue(ref val) => val.as_instruction(),
            BasicValueEnum::IntValue(ref val) => val.as_instruction(),
            BasicValueEnum::FloatValue(ref val) => val.as_instruction(),
            BasicValueEnum::StructValue(ref val) => val.as_instruction(),
            BasicValueEnum::PointerValue(ref val) => val.as_instruction(),
            BasicValueEnum::VectorValue(ref val) => val.as_instruction(),
        }
    }
}

impl AggregateValueEnum {
    pub(crate) fn new(value: LLVMValueRef) -> AggregateValueEnum {
        let type_kind = unsafe { LLVMGetTypeKind(LLVMTypeOf(value)) };

        match type_kind {
            LLVMTypeKind::LLVMArrayTypeKind => {
                AggregateValueEnum::ArrayValue(ArrayValue::new(value))
            }
            LLVMTypeKind::LLVMStructTypeKind => {
                AggregateValueEnum::StructValue(StructValue::new(value))
            }
            _ => unreachable!("The given type is not an aggregate type."),
        }
    }
}

impl BasicMetadataValueEnum {
    pub(crate) fn new(value: LLVMValueRef) -> BasicMetadataValueEnum {
        let type_kind = unsafe { LLVMGetTypeKind(LLVMTypeOf(value)) };

        match type_kind {
            LLVMTypeKind::LLVMFloatTypeKind
            | LLVMTypeKind::LLVMFP128TypeKind
            | LLVMTypeKind::LLVMDoubleTypeKind
            | LLVMTypeKind::LLVMHalfTypeKind
            | LLVMTypeKind::LLVMX86_FP80TypeKind
            | LLVMTypeKind::LLVMPPC_FP128TypeKind => {
                BasicMetadataValueEnum::FloatValue(FloatValue::new(value))
            }
            LLVMTypeKind::LLVMIntegerTypeKind => {
                BasicMetadataValueEnum::IntValue(IntValue::new(value))
            }
            LLVMTypeKind::LLVMStructTypeKind => {
                BasicMetadataValueEnum::StructValue(StructValue::new(value))
            }
            LLVMTypeKind::LLVMPointerTypeKind => {
                BasicMetadataValueEnum::PointerValue(PointerValue::new(value))
            }
            LLVMTypeKind::LLVMArrayTypeKind => {
                BasicMetadataValueEnum::ArrayValue(ArrayValue::new(value))
            }
            LLVMTypeKind::LLVMVectorTypeKind => {
                BasicMetadataValueEnum::VectorValue(VectorValue::new(value))
            }
            LLVMTypeKind::LLVMMetadataTypeKind => {
                BasicMetadataValueEnum::MetadataValue(MetadataValue::new(value))
            }
            _ => unreachable!("Unsupported type"),
        }
    }
}

impl From<BasicValueEnum> for AnyValueEnum {
    fn from(value: BasicValueEnum) -> AnyValueEnum {
        AnyValueEnum::new(value.as_value_ref())
    }
}
