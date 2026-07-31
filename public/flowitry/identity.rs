// ============================================================
// Ownership / Flow Analysis Final Exam

// Core question:

// "Given this subject, can we explain:
// 1. where it originated,
// 2. how it moved,
// 3. who can access it,
// 4. who can mutate it,
// 5. when it dies?"

// ============================================================

// ============================================================
// 1. IDENTITY

// What is this thing?

// This is the "who am I?" layer.

// ============================================================

// Examples:

// let x = 1;

// x is:
// - identifier
// - local binding
// - i32 value
// - stack location

// Concepts:

// - Declaration
// - Identifier
// - Symbol
// - Binding
// - Shadowing
// - Alias identity
// - Memory identity

// Questions:

// "Are these two names the same thing?"

// x
// |
// y

// or

// x --> value
// y --> same value

// ============================================================

// ============================================================
// 2. ALLOCATION

// Where does the memory come from?

// ============================================================

// Stack:

// let x = 1;

// Concepts:

// - Stack allocation
// - Heap allocation
// - Box
// - Vec
// - String
// - Rc allocation
// - Arc allocation
// - Static memory

// Questions:

// "Where is the actual data?"

// ============================================================

// ============================================================
// 3. VALUE CREATION

// How did this value come into existence?

// ============================================================

// Sources:

// Literal:

// let x = 5;

// Copy:

// let y = x;

// Move:

// let y = String::new();

// Clone:

// let y = x.clone();

// Function return:

// let x = foo();

// Struct:

// let x = MyStruct {};

// Enum:

// let x = Some(value);

// Questions:

// "What is the origin node?"

// ============================================================

// ============================================================
// 4. FLOW / TRANSFER

// How did identity travel?

// ============================================================

// Value flow:

// a
// |
// v
// b

// Ownership transfer:

// String:

// s1
// |
// move
// v
// s2

// Reference flow:

// x
// |
// &
// v
// y

// Shared ownership:

// x
// |
// clone
// +----> y

// Concepts:

// - Move
// - Copy
// - Clone
// - Borrow
// - Reference
// - Pointer
// - Function argument
// - Return value

// Question:

// "How did this subject become this subject?"

// ============================================================

// ============================================================
// 5. ACCESS MODEL

// Who can see this value?

// ============================================================

// Exclusive:

// &mut T

// One writer.

// Shared:

// &T

// Many readers.

// Shared ownership:

// Rc<T>
// Arc<T>

// Raw:

// *const T
// *mut T

// Concepts:

// - Borrowing
// - References
// - Raw pointers
// - Dereference
// - Aliasing

// Question:

// "Who can reach this memory?"

// ============================================================

// ============================================================
// 6. MUTATION MODEL

// Who can change this value?

// ============================================================

// Direct:

// x = 5;

// Through borrow:

// *reference = 5;

// Through interior mutability:

// RefCell
// Mutex
// RwLock
// Atomic

// Through shared ownership:

// Arc<Mutex<T>>

// Concepts:

// - Assignment
// - Field mutation
// - Index mutation
// - Mutable borrow
// - Interior mutability
// - Synchronization

// Question:

// "What can change this?"

// ============================================================

// ============================================================
// 7. SCOPE / VISIBILITY

// Where does this exist?

// ============================================================

// Levels:

// Program
//  └── Crate
//       └── Module
//            └── Function
//                 └── Block
//                      └── Closure

// Concepts:

// - Global
// - Static
// - Const
// - Module
// - Function scope
// - Block scope
// - Closure capture

// Question:

// "Where is this name valid?"

// ============================================================

// ============================================================
// 8. LIFETIME

// How long can this exist?

// ============================================================

// Examples:

// Local:

// {
//    let x = 1;
// }

// Returned:

// fn foo<'a>() -> &'a T

// Static:

// &'static T

// Concepts:

// - Lifetime
// - Borrow validity
// - Scope duration
// - Drop timing

// Question:

// "How long is this relationship valid?"

// ============================================================

// ============================================================
// 9. DATA SHAPE

// What kind of thing is flowing?

// ============================================================

// Primitive:

// i32
// bool

// Owned:

// String
// Vec<T>

// Composite:

// struct
// enum
// tuple

// Container:

// Box<T>
// Rc<T>
// Arc<T>

// Question:

// "What rules apply because of this type?"

// ============================================================

// ============================================================
// 10. SPECIAL BOUNDARIES

// Where normal reasoning breaks.

// ============================================================

// Unsafe:

// - raw pointer
// - FFI
// - union

// Async:

// - state machines
// - Pin

// Drop:

// - destructor effects

// MaybeUninit:

// - allocation without value

// Cycles:

// - Rc graphs

// Question:

// "What hidden behavior exists?"

// ============================================================

// ============================================================
// FINAL MODEL
// ============================================================

// Cross-cutting constraints:

// Scope
// Data Type
// Ownership Model
// Unsafe Boundaries

// ============================================================

/// ------------------------------------------------------------
static CHAPTER_IDENTITY: i32 = 1;
/// ------------------------------------------------------------
// # Origin Analysis Test Matrix
// ## Identity
// - [ ] Local binding
// - [ ] Variable shadowing
// - [ ] Constant
// - [ ] Static
// Expected behavior:
// Clicking the marked variable should resolve to the origin declaration.
fn identity_local_binding() {
    let a = 1;
    let b = a;
    // EXPECT_ORIGIN: a
    // EXPECT_LINE: declaration of `a`
    assert_eq!(b, 1);
}
fn identity_variable_shadowing() {
    let value = 1;
    let value = value + 1;
    // EXPECT_ORIGIN: second `value` binding
    // EXPECT_LINE: declaration of shadowed `value`
    assert_eq!(value, 2);
}
fn identity_constant() {
    const MAX_SIZE: usize = 100;

    let size = MAX_SIZE;

    // EXPECT_ORIGIN: MAX_SIZE
    // EXPECT_LINE: const declaration
    assert_eq!(size, 100);
}

static GLOBAL_COUNTER: i32 = 10;
/// ------------------------------------------------------------
fn identity_static() {
    let counter = GLOBAL_COUNTER;
    // EXPECT_ORIGIN: GLOBAL_COUNTER
    // EXPECT_LINE: static declaration
    assert_eq!(counter, 10);
}
