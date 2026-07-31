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

/// ------------------------------------------------------------
static CHAPTER_VALUE: i32 = 1;
/// ------------------------------------------------------------
// # Origin Analysis Test Matrix
// ## Value Creation
// - [ ] Literal
// - [ ] Copy
// - [ ] Move
// - [ ] Clone
// - [ ] Function return
// - [ ] Struct construction
// - [ ] Enum construction
// - [ ] Tuple construction
//
// Expected behavior:
// Each value-producing operation should resolve to the expression or declaration
// that created the value being tracked.

#[derive(Debug, Clone)]
struct User {
    id: i32,
}
#[derive(Debug)]
enum Status {
    Active,
    Inactive,
}
fn create_value() -> i32 {
    // EXPECT_ORIGIN: literal `42`
    42
}
fn value_creation_literal() {
    let value = 10;

    // EXPECT_ORIGIN: literal `10`
    assert_eq!(value, 10);
}
fn value_creation_copy() {
    let a: i32 = 5;
    let b = a;

    // EXPECT_ORIGIN: a
    // EXPECT_VALUE_ORIGIN: literal `5`
    assert_eq!(b, 5);
}
fn value_creation_move() {
    let a = String::from("hello");
    let b = a;

    // EXPECT_ORIGIN: a
    // EXPECT_VALUE_ORIGIN: String allocation
    assert_eq!(b, "hello");
}
fn value_creation_clone() {
    let a = String::from("hello");
    let b = a.clone();

    // EXPECT_ORIGIN: a
    // EXPECT_VALUE_ORIGIN: clone operation
    assert_eq!(b, "hello");
}
fn value_creation_function_return() {
    let value = create_value();

    // EXPECT_ORIGIN: create_value return expression
    // EXPECT_VALUE_ORIGIN: literal `42` inside create_value
    assert_eq!(value, 42);
}
fn value_creation_struct_construction() {
    let user = User {
        name: "hi".to_string(),
        age: 18,
    };

    // EXPECT_ORIGIN: User struct construction
    // EXPECT_FIELD_ORIGIN: literal `100`
    assert_eq!(user.id, 100);
}
fn value_creation_enum_construction() {
    let status = Status::Active;

    // EXPECT_ORIGIN: Status::Active constructor
    assert!(matches!(status, Status::Active));
}
fn value_creation_tuple_construction() {
    let point = (10, 20);

    // EXPECT_ORIGIN: tuple construction
    // EXPECT_FIELD_ORIGIN:
    // point.0 -> literal `10`
    // point.1 -> literal `20`
    assert_eq!(point.0, 10);
    assert_eq!(point.1, 20);
}

/// ------------------------------------------------------------
static CHAPTER_BORROW: i32 = 1;
/// ------------------------------------------------------------
// # Origin Analysis Test Matrix
//
// ## Borrowing
// - [ ] Shared borrow (&T)
// - [ ] Mutable borrow (&mut T)
// - [ ] Reborrow
// - [ ] Nested borrow
//
// ## Commands
//
// Run this file:
// cargo test --test origin_borrowing
//
// Or if standalone:
// rustc origin_borrowing.rs && ./origin_borrowing
//
// Useful inspection:
// cargo expand --test origin_borrowing
//
// MIR inspection:
// rustc --emit=mir origin_borrowing.rs
//
// Or nightly:
// rustc -Zunpretty=mir origin_borrowing.rs
//
// Ownership inspection:
// cargo check
// cargo clippy
//
// Expected behavior:
// Each borrow should resolve back to the allocation/value being borrowed.
// A borrow is not a new value origin. It is an access path to an existing origin.
fn borrowing_shared() {
    let a = String::from("hello");

    let b = &a;

    // EXPECT_ORIGIN:
    // b -> a
    //
    // EXPECT_ALLOCATION_ORIGIN:
    // String allocation inside String::from
    //
    // EXPECT_MUTATION:
    // none allowed through b

    assert_eq!(b, "hello");
}
fn borrowing_mutable() {
    let mut a = String::from("hello");

    let b = &mut a;

    b.push_str(" world");

    // EXPECT_ORIGIN:
    // b -> a
    //
    // EXPECT_ALLOCATION_ORIGIN:
    // String allocation inside String::from
    //
    // EXPECT_MUTATION:
    // b mutates a

    assert_eq!(b, "hello world");
}
fn borrowing_reborrow() {
    let mut a = String::from("hello");

    let b = &mut a;

    {
        let c = &mut *b;

        c.push_str(" world");

        // EXPECT_ORIGIN:
        // c -> b -> a
        //
        // EXPECT_MUTATION:
        // c mutates original allocation
    }

    assert_eq!(b, "hello world");
}
fn borrowing_nested() {
    let mut value = 10;

    let first = &mut value;

    {
        let second = &mut *first;

        *second += 5;

        // EXPECT_ORIGIN:
        // second -> first -> value
        //
        // EXPECT_MUTATION:
        // second changes value
    }

    assert_eq!(value, 15);
}
fn borrowing_shared_nested() {
    let value = String::from("hello");

    let first = &value;
    let second = &first;

    // EXPECT_ORIGIN:
    // second -> first -> value
    //
    // EXPECT_ALLOCATION_ORIGIN:
    // String::from allocation

    assert_eq!(**second, "hello");
}

/// ------------------------------------------------------------
static CHAPTER_REFERENCE: i32 = 1;
/// ------------------------------------------------------------
// # Origin Analysis Test Matrix
//
// ## References / Indirection
// - [ ] Box
// - [ ] Rc
// - [ ] Arc
// - [ ] Raw pointer
// - [ ] Unsafe dereference
//
// ## Commands
//
// Run this file:
// cargo test --test origin_indirection
//
// MIR inspection:
// rustc --emit=mir origin_indirection.rs
//
// Or nightly:
// rustc -Zunpretty=mir origin_indirection.rs
//
// Ownership inspection:
// cargo check
// cargo clippy
//
// For reference counting:
// cargo +nightly rustc -- -Zprint-type-sizes
//
// Expected behavior:
// Indirection should preserve the ability to resolve the original allocation.
// The analyzer should distinguish:
// - owner relationship
// - pointer/reference relationship
// - shared ownership relationship
// - unsafe unknown relationship
//

use std::rc::Rc;
use std::sync::{Arc, Mutex};

fn references_box() {
    let value = Box::new(42);

    let reference = &value;

    // EXPECT_ORIGIN:
    // reference -> value -> Box allocation
    //
    // EXPECT_ALLOCATION_ORIGIN:
    // Box::new(42)
    //
    // EXPECT_OWNERSHIP:
    // value owns heap allocation

    assert_eq!(**reference, 42);
}
fn references_box_move() {
    let value = Box::new(42);

    let moved = value;

    // EXPECT_ORIGIN:
    // moved -> previous value binding
    //
    // EXPECT_ALLOCATION_ORIGIN:
    // Box::new(42)
    //
    // EXPECT_OWNERSHIP:
    // ownership moved from value to moved

    assert_eq!(*moved, 42);
}
fn references_rc() {
    let value = Rc::new(42);

    let clone = Rc::clone(&value);

    // EXPECT_ORIGIN:
    // clone -> same Rc allocation as value
    //
    // EXPECT_ALLOCATION_ORIGIN:
    // Rc::new(42)
    //
    // EXPECT_OWNERSHIP:
    // shared ownership
    //
    // EXPECT_REFERENCE_COUNT:
    // strong_count == 2

    assert_eq!(*clone, 42);
    assert_eq!(Rc::strong_count(&value), 2);
}
fn references_arc() {
    let value = Arc::new(42);

    let clone = Arc::clone(&value);

    // EXPECT_ORIGIN:
    // clone -> same Arc allocation as value
    //
    // EXPECT_ALLOCATION_ORIGIN:
    // Arc::new(42)
    //
    // EXPECT_OWNERSHIP:
    // thread-safe shared ownership
    //
    // EXPECT_REFERENCE_COUNT:
    // strong_count == 2

    assert_eq!(*clone, 42);
    assert_eq!(Arc::strong_count(&value), 2);
}
fn references_arc_mutex_mutation() {
    let value = Arc::new(Mutex::new(0));

    let clone = Arc::clone(&value);

    *value.lock().unwrap() = 10;

    // EXPECT_ORIGIN:
    // clone -> same Arc allocation as value
    //
    // EXPECT_MUTATION:
    // value mutation affects clone observation
    //
    // EXPECT_FLOW:
    // mutation(value)
    //        |
    //        v
    // shared Mutex state
    //        |
    //        v
    // read(clone)

    assert_eq!(*clone.lock().unwrap(), 10);
}
fn references_raw_pointer() {
    let value = 42;

    let pointer: *const i32 = &value;

    unsafe {
        // EXPECT_ORIGIN:
        // pointer -> value address
        //
        // EXPECT_OWNERSHIP:
        // unknown
        //
        // EXPECT_SAFETY:
        // requires unsafe verification

        assert_eq!(*pointer, 42);
    }
}
fn references_raw_mut_pointer() {
    let mut value = 42;

    let pointer: *mut i32 = &mut value;

    unsafe {
        *pointer = 100;

        // EXPECT_ORIGIN:
        // pointer -> value
        //
        // EXPECT_MUTATION:
        // pointer mutates value
        //
        // EXPECT_SAFETY:
        // aliasing rules no longer enforced by compiler

    }

    assert_eq!(value, 100);
}
fn references_unsafe_dereference() {
    let value = Box::new(99);

    let pointer = Box::into_raw(value);

    unsafe {
        let recovered = Box::from_raw(pointer);

        // EXPECT_ORIGIN:
        // recovered -> original Box allocation
        //
        // EXPECT_ALLOCATION_ORIGIN:
        // Box::new(99)
        //
        // EXPECT_WARNING:
        // ownership reconstruction occurred manually

        assert_eq!(*recovered, 99);
    }
}

/// ------------------------------------------------------------
static CHAPTER_INTERIOR_MUTABILITY: i32 = 1;
/// ------------------------------------------------------------
// For your graph model, this category introduces a new relationship type:
// Value graph:
// Box
// ----
// value
//  |
//  v
// heap allocation


// Rc / Arc
// --------
// value ----+
//           |
//           v
//      shared allocation
//           ^
//           |
// clone ----+


// Raw pointer
// ------------
// pointer
//    |
//    v
// memory address

// (no ownership guarantee)

// The important distinction:

// * `Box<T>` → **one owner, heap indirection**
// * `Rc<T>` → **many owners, single-threaded**
// * `Arc<T>` → **many owners, thread-safe**
// * `*const T` / `*mut T` → **address relationship only**
// * `unsafe dereference` → **manual proof boundary**

// For a V1 "origin panel", I would represent these separately:

// enum Edge {
//     Copy,
//     Move,
//     Borrow,
//     MutBorrow,
//     Reborrow,
//     Clone,
//     OwnsAllocation,     // Box
//     SharesAllocation,   // Rc / Arc
//     PointsTo,           // raw pointers
//     Dereferences,       // unsafe access
// }

// This is also where your earlier intuition about `Arc` was right: it cannot be treated like a normal variable lineage. `Arc::clone()` does not create "a child value"; it creates another **owner handle to the same underlying state**. That is a different graph shape.


// # Origin Analysis Test Matrix
//
// ## Interior Mutability
// - [ ] Cell
// - [ ] RefCell
// - [ ] Mutex
// - [ ] RwLock
// - [ ] Atomic types
//
// ## Commands
//
// Run this file:
// cargo test --test origin_interior_mutability
//
// MIR inspection:
// rustc --emit=mir origin_interior_mutability.rs
//
// Or nightly:
// rustc -Zunpretty=mir origin_interior_mutability.rs
//
// Thread safety inspection:
// cargo check
// cargo clippy
//
// Expected behavior:
//
// Interior mutability changes the relationship between:
// - binding
// - mutation
// - ownership
//
// Normal Rust:
//     let mut x = 1;
//     *x can only mutate through &mut x
//
// Interior mutability:
//     let x = Cell::new(1);
//     x.set(2);
//
// The binding is immutable, but the contained value changes.
//
// The analyzer must distinguish:
//
// Binding:
//     who owns the container?
//
// Value:
//     what data is inside?
//
// Mutation:
//     what path can modify the contained data?
//
/// ------------------------------------------------------------
//
// Cell<T>
// --------
// Purpose:
// - Copy types only
// - Provides mutation through shared references
// - No runtime borrowing checks
//
// Relationship:
//
//     x
//     |
//     v
//   Cell allocation
//     |
//     v
//   contained value
//
// Mutation path:
//
//     x.set(new_value)
//          |
//          v
//       contained T
//
/// ------------------------------------------------------------
//
// RefCell<T>
// -----------
// Purpose:
// - Runtime borrow checking
// - Single-threaded interior mutability
//
// Relationship:
//
//     x
//     |
//     v
//   RefCell<T>
//     |
//     v
//   contained value
//
// Mutation:
//
//     borrow_mut()
//          |
//          v
//       contained T
//
// Analyzer note:
// The compiler cannot prove borrowing rules statically here.
// Runtime state determines whether access is valid.
//
/// ------------------------------------------------------------
//
// Mutex<T>
// --------
//
// Purpose:
// - Thread-safe interior mutability
// - Synchronizes access between threads
//
// Relationship:
//
//     x
//     |
//     v
//   Mutex allocation
//     |
//     v
//   protected T
//
// Mutation:
//
//     lock()
//       |
//       v
//   mutable guard
//       |
//       v
//   protected value
//
// Important:
//
// Arc<Mutex<T>>:
//
//     Arc handle A
//          |
//          v
//      Mutex<T>
//          ^
//          |
//     Arc handle B
//
// Mutation through A affects observation through B.
//
/// ------------------------------------------------------------
//
// RwLock<T>
// ---------
//
// Purpose:
// - Multiple readers OR one writer
//
// Relationship:
//
//     x
//     |
//     v
//   RwLock<T>
//     |
//     v
//   protected T
//
// Access:
//
//     read()
//       |
//       v
//     &T
//
//     write()
//       |
//       v
//    &mut T
//
// Analyzer note:
// Similar to Mutex but distinguishes read flow vs write flow.
//
/// ------------------------------------------------------------
//
// Atomic types
// ------------
//
// Purpose:
// - Lock-free synchronized mutation
// - Primitive shared state mutation
//
// Relationship:
//
//     x
//     |
//     v
//   Atomic<T>
//
// Mutation:
//
//     fetch_add()
//     store()
//     swap()
//
// Analyzer note:
//
// There is no traditional &mut T path.
// Mutation happens through atomic operations.
//
// The origin graph must track:
//
//     operation
//          |
//          v
//     atomic memory location
//
/// ------------------------------------------------------------
//
// Edge types introduced by this category:
//
// enum Edge {
//
//     OwnsContainer,
//     ContainsValue,
//     InteriorMutation,
//     RuntimeBorrowCheck,
//     LockedAccess,
//     AtomicMutation,
//
// }
//
// These are different from normal mutation:
//
// let mut x = String::new();
//
// means:
//
//     binding -> mutable owner
//
//
// Mutex:
//
//     binding -> shared owner
//                 |
//                 v
//             mutable interior state
//
/// ------------------------------------------------------------

use std::cell::{Cell, RefCell};
use std::sync::{
    atomic::{AtomicI32, Ordering},
    // Mutex,
    RwLock,
};

fn interior_cell() {
    let value = Cell::new(10);

    value.set(20);

    // EXPECT_ORIGIN:
    // value -> Cell allocation
    //
    // EXPECT_MUTATION:
    // Cell::set mutates contained i32
    //
    // EXPECT_OWNERSHIP:
    // single owner

    assert_eq!(value.get(), 20);
}
fn interior_refcell() {
    let value = RefCell::new(String::from("hello"));

    {
        let mut borrowed = value.borrow_mut();
        borrowed.push_str(" world");

        // EXPECT_ORIGIN:
        // borrowed -> RefCell -> String allocation
        //
        // EXPECT_MUTATION:
        // borrow_mut allows mutation of inner String
    }

    assert_eq!(value.borrow().as_str(), "hello world");
}
fn interior_mutex() {
    let value = Mutex::new(10);

    {
        let mut locked = value.lock().unwrap();

        *locked += 1;

        // EXPECT_ORIGIN:
        // locked -> Mutex -> i32
        //
        // EXPECT_MUTATION:
        // lock guard mutates protected value
        //
        // EXPECT_ACCESS:
        // mutation requires lock ownership
    }

    assert_eq!(*value.lock().unwrap(), 11);
}
fn interior_rwlock_read() {
    let value = RwLock::new(String::from("hello"));

    let read = value.read().unwrap();

    // EXPECT_ORIGIN:
    // read -> RwLock -> String allocation
    //
    // EXPECT_ACCESS:
    // shared read access
    //
    // EXPECT_MUTATION:
    // not allowed

    assert_eq!(read.as_str(), "hello");
}
fn interior_rwlock_write() {
    let value = RwLock::new(String::from("hello"));

    {
        let mut write = value.write().unwrap();

        write.push_str(" world");

        // EXPECT_ORIGIN:
        // write -> RwLock -> String allocation
        //
        // EXPECT_MUTATION:
        // write guard mutates inner String
    }

    assert_eq!(value.read().unwrap().as_str(), "hello world");
}
fn interior_atomic() {
    let value = AtomicI32::new(10);

    value.fetch_add(5, Ordering::SeqCst);

    // EXPECT_ORIGIN:
    // value -> AtomicI32 allocation
    //
    // EXPECT_MUTATION:
    // atomic operation mutates memory location
    //
    // EXPECT_ACCESS:
    // no &mut reference exists

    assert_eq!(value.load(Ordering::SeqCst), 15);
}
// # Origin Analysis Test Matrix
//
// ## Mutation
// - [ ] Assignment
// - [ ] Compound assignment (+=)
// - [ ] Field mutation
// - [ ] Index mutation
// - [ ] Mutation through borrow
// - [ ] Mutation through shared ownership
//
// ## Commands
//
// Run this file:
// cargo test --test origin_mutation
//
// MIR inspection:
// rustc --emit=mir origin_mutation.rs
//
// Or nightly:
// rustc -Zunpretty=mir origin_mutation.rs
//
// Ownership inspection:
// cargo check
// cargo clippy
//
/// ------------------------------------------------------------
//
// Core questions:
//
// When tracking a mutation:
//
// 1. WHAT memory location changed?
//
// 2. WHERE was that memory allocated?
//
// 3. WHO currently owns access to that memory?
//
// 4. WHAT paths can reach that memory?
//
// 5. CAN another variable observe this mutation?
//
/// ------------------------------------------------------------
//
// Mutation graph:
//
// Before:
//
//     binding
//        |
//        v
//     allocation
//        |
//        v
//      value
//
//
// After:
//
//     mutation operation
//            |
//            v
//     affected allocation
//            |
//            v
//        new value
//
/// ------------------------------------------------------------
//
// Important distinction:
//
// Assignment:
//
//     x = new_value;
//
// replaces the value associated with a binding.
//
//
//
// Interior/shared mutation:
//
//     Arc<Mutex<T>>
//
//     handle A
//        |
//        v
//     shared allocation
//        ^
//        |
//     handle B
//
//
// Mutation through A:
//
//     A.lock()
//        |
//        v
//     allocation changes
//        |
//        v
//     B observes new state
//
/// ------------------------------------------------------------
//
// Analyzer model:
//
// Mutation should not only create:
//
//     variable -> value
//
// It should create:
//
//     operation
//          |
//          v
//     memory location
//
// Example:
//
//     *x = 5;
//
// becomes:
//
//     Assignment
//          |
//          v
//     memory(x)
//          |
//          v
//       value(5)
//

/// ------------------------------------------------------------
static CHAPTER_MUTATION: i32 = 1;
/// ------------------------------------------------------------
struct User {
    name: String,
    age: u32,
}
fn mutation_assignment() {
    let mut value = 10;

    value = 20;

    // EXPECT_ORIGIN:
    // value -> initial literal 10
    //
    // EXPECT_MUTATION:
    // assignment replaces current value
    //
    // EXPECT_FINAL_VALUE:
    // value == 20
    //
    // GRAPH:
    //
    // value binding
    //      |
    //      v
    // old value(10)
    //
    // assignment
    //      |
    //      v
    // new value(20)

    assert_eq!(value, 20);
}
fn mutation_compound_assignment() {
    let mut value = 10;

    value += 5;

    // EXPECT_ORIGIN:
    // value -> literal 10
    //
    // EXPECT_MUTATION:
    // read existing value
    // compute new value
    // write back
    //
    // GRAPH:
    //
    //       value
    //         |
    //         v
    //       10
    //
    //       += 5
    //         |
    //         v
    //
    //       15

    assert_eq!(value, 15);
}
fn mutation_field() {
    #[derive(Debug, Clone)]
    struct User {
        id: i32,
        name: String,
        age: i32,
    }
    
    let mut user = User {
        id: 1,
        name: "Alice".to_string(),
        age: 20,
    };

    user.age = 21;

    // EXPECT_ORIGIN:
    // user -> struct allocation
    //
    // EXPECT_FIELD_ORIGIN:
    // user.age -> literal 20
    //
    // EXPECT_MUTATION:
    // only age field changes
    //
    // GRAPH:
    //
    // user
    //  |
    //  +-- name -> String allocation
    //  |
    //  +-- age
    //          |
    //          v
    //       assignment(21)

    assert_eq!(user.age, 21);
}
fn mutation_index() {
    let mut values = vec![1, 2, 3];

    values[1] = 10;

    // EXPECT_ORIGIN:
    // values -> Vec allocation
    //
    // EXPECT_MUTATION:
    // indexed element mutation
    //
    // GRAPH:
    //
    // values
    //    |
    //    v
    // Vec allocation
    //    |
    //    +-- index 0 -> 1
    //    +-- index 1 -> 2
    //    +-- index 2 -> 3
    //
    // mutation:
    //
    // index 1
    //    |
    //    v
    // replace 2 with 10

    assert_eq!(values[1], 10);
}
fn mutation_through_borrow() {
    let mut value = String::from("hello");

    let reference = &mut value;

    reference.push_str(" world");

    // EXPECT_ORIGIN:
    // reference -> value -> String allocation
    //
    // EXPECT_MUTATION:
    // mutable borrow allows mutation
    //
    // GRAPH:
    //
    // reference
    //     |
    //     v
    // value
    //     |
    //     v
    // String allocation
    //
    // mutation(reference)
    //     |
    //     v
    // String allocation

    assert_eq!(value, "hello world");
}
fn mutation_through_shared_ownership() {
    use std::sync::{Arc, Mutex};

    let x = Arc::new(Mutex::new(0));

    let y = Arc::clone(&x);

    *x.lock().unwrap() = 100;

    // EXPECT_ORIGIN:
    // x -> Arc allocation
    // y -> same Arc allocation
    //
    // EXPECT_MUTATION:
    // x mutation affects y observation
    //
    // GRAPH:
    //
    //        x
    //        |
    //        |
    //        v
    //     Arc<Mutex>
    //        ^
    //        |
    //        |
    //        y
    //
    // mutation:
    //
    // x.lock()
    //    |
    //    v
    // Mutex inner value
    //    |
    //    v
    // y.lock() observes change

    assert_eq!(*y.lock().unwrap(), 100);
}

/// ------------------------------------------------------------
static CHAPTER_FUNCTIONS: i32 = 1;
/// ------------------------------------------------------------
//
// Future expansion questions:
//
// Q1:
// If a variable changes, do we track:
//
//     binding mutation
//
// or:
//
//     allocation mutation?
//
// Example:
//
//     let mut x = String::new();
//     x = String::from("a");
//
// The binding changed, but the original allocation may be gone.
//
//
//
// Q2:
// If a field changes, does the parent object become "changed"?
//
// Example:
//
//     user.age = 30;
//
// Should the analyzer mark:
//
//     user.age   changed
//
// or:
//
//     user      changed
//
// or both?
//
//
// Q3:
// If two variables share memory, should mutation propagate?
//
// Example:
//
//     Arc<Mutex<T>>
//
// Should:
//
//     mutate(x)
//
// automatically highlight:
//
//     y
//
// because:
//
//     x -> shared allocation <- y
//
//
//
// Q4:
// Does mutation flow backwards to origin?
//
// Example:
//
//     let a = 1;
//     let b = a;
//     b += 1;
//
// Should the graph say:
//
//     b mutation affects a
//
// No.
//
// Because copy created a new value.
//
//
//
// Q5:
// What about moves?
//
// Example:
//
//     let a = String::from("x");
//     let b = a;
//
//     b.push_str("y");
//
//
// Should mutation highlight:
//
//     a?
//
// No.
//
// The allocation lineage is shared,
// but the binding ownership moved.
//
//
//
// Q6:
// What about references?
//
// Example:
//
//     let mut a = 1;
//     let b = &mut a;
//     *b = 2;
//
// Should highlight:
//
//     b
//     a
//
// Yes.
//
// Because they refer to the same memory.
//
/// ------------------------------------------------------------
//
// Likely graph edges:
//
// enum MutationEdge {
//
//     AssignsTo,
//     ReplacesValue,
//     MutatesField,
//     MutatesIndex,
//     MutatesThroughBorrow,
//     MutatesSharedAllocation,
//
// }
//
// Mutation analysis is not just:
//
// "who called this variable?"
//
// It is:
//
// "what memory location changed, and who can observe that change?"
//
/// ------------------------------------------------------------

// ## Functions
// - [ ] Pass by value
// - [ ] Pass by shared reference
// - [ ] Pass by mutable reference
// - [ ] Generic parameter
// - [ ] Trait object
// - [ ] Closure capture
// - [ ] Async function
/// ------------------------------------------------------------
// ## Functions
//
// - [ ] Pass by value
//      Ownership moves into the function.
//      Question:
//      "Does the origin follow the argument into the callee?"
//
//      Expected flow:
//      caller value
//          |
//          v
//      function parameter
//
//      Diagram:
//
//      stack
//      +-------+
//      | a     |  ---- move ---->  fn parameter
//      +-------+
//
//      After move:
//      caller binding is invalid.
//
//
// - [ ] Pass by shared reference
//      Borrowed relationship.
//      The function does not own the value.
//
//      Question:
//      "Can we trace this reference back to the original owner?"
//
//      Diagram:
//
//      owner
//        |
//        v
//      &T
//        |
//        v
//      function parameter
//
//
//
// - [ ] Pass by mutable reference
//      Exclusive mutation capability through a borrow.
//
//      Question:
//      "Does mutation through the parameter affect the original owner?"
//
//      Diagram:
//
//      owner
//        |
//        v
//      &mut T
//        |
//        v
//      mutation
//
//      Expected:
//      origin == owner
//      mutation_target == owner
//
//
// - [ ] Generic parameter
//      Type abstraction where ownership behavior depends on T.
//
//      Question:
//      "Can the analyzer preserve ownership flow through generic boundaries?"
//
//      Example challenges:
//      - T: Copy
//      - T: Clone
//      - T: Drop
//      - T: Deref
//
//
//
// - [ ] Trait object
//      Dynamic dispatch through indirection.
//
//      Question:
//      "Can we identify the underlying value behind dyn Trait?"
//
//      Diagram:
//
//      concrete type
//          |
//          v
//      Box<dyn Trait>
//          |
//          v
//      trait method call
//
//
//
// - [ ] Closure capture
//      Closures capture values from surrounding scope.
//
//      Questions:
//      - Was value moved?
//      - Was value borrowed?
//      - Was value mutably borrowed?
//
//      Diagram:
//
//      outer scope
//          |
//          v
//      closure environment
//          |
//          v
//      closure execution
//
//
//
// - [ ] Async function
//      Values are stored inside generated futures.
//
//      Questions:
//      - What values cross await points?
//      - What values are moved into the future?
//      - What values remain borrowed?
//
//      Diagram:
//
//      local variables
//          |
//          v
//      Future state machine
//          |
//          v
//      poll()
//          |
//          v
//      execution resumes
//
//
/// ------------------------------------------------------------
// Test runner notes:
//
// Run:
//
// cargo test --test ownership_flow
//
// or:
//
// cargo test functions -- --nocapture
//
/// ------------------------------------------------------------


fn functions_pass_by_value() {
    let a = String::from("hello");

    consume(a);

    // EXPECTED:
    // origin:
    //   allocation inside this function
    //
    // binding:
    //   a -> String allocation
    //
    // after call:
    //   a moved
    //
    // analyzer should show:
    //   consume parameter originates from a

    fn consume(value: String) {
        assert_eq!(value, "hello");
    }

    consume(a);
}


fn functions_pass_by_shared_reference() {
    let a = String::from("hello");

    inspect(&a);

    // EXPECTED:
    // origin:
    //   a
    //
    // relationship:
    //   shared borrow
    //
    // mutation:
    //   impossible through &String

    fn inspect(value: &String) {
        assert_eq!(value, "hello");
    }

    inspect(&a);
}


fn functions_pass_by_mutable_reference() {
    let mut a = String::from("hello");

    modify(&mut a);

    // EXPECTED:
    // origin:
    //   a
    //
    // mutation:
    //   modify affects a

    fn modify(value: &mut String) {
        value.push_str(" world");
    }

    modify(&mut a);

    assert_eq!(a, "hello world");
}


fn functions_generic_parameter() {
    fn identity<T>(value: T) -> T {
        value
    }

    let a = String::from("hello");

    let b = identity(a);

    // EXPECTED:
    //
    // origin:
    //   allocation from a
    //
    // flow:
    //   a -> generic T -> b
    //
    // question:
    //   does generic boundary preserve identity?

    assert_eq!(b, "hello");
}


trait Animal {
    fn sound(&self) -> &'static str;
}


struct Dog;


impl Animal for Dog {
    fn sound(&self) -> &'static str {
        "woof"
    }
}


fn functions_trait_object() {
    let dog = Dog;

    let animal: Box<dyn Animal> = Box::new(dog);

    // EXPECTED:
    //
    // origin:
    //   Dog allocation
    //
    // indirection:
    //   Box
    //
    // dynamic dispatch:
    //   dyn Animal
    //
    // question:
    //   can analyzer trace through trait object?

    assert_eq!(animal.sound(), "woof");
}


fn functions_closure_capture() {
    let value = String::from("hello");

    let closure = || {
        println!("{}", value);
    };

    // EXPECTED:
    //
    // capture mode:
    //   shared borrow
    //
    // origin:
    //   value
    //
    // closure environment stores reference

    closure();
}


fn functions_closure_move_capture() {
    let value = String::from("hello");

    let closure = move || {
        println!("{}", value);
    };

    // EXPECTED:
    //
    // capture mode:
    //   move
    //
    // origin:
    //   value
    //
    // closure owns captured data
    //
    // diagram:
    //
    // stack
    // +-------+
    // | value |
    // +-------+
    //      |
    //      move
    //      v
    // closure environment

    closure();
}


async fn async_worker(value: String) -> usize {
    value.len()
}


async fn functions_async_function() {
    let value = String::from("hello");

    let future = async_worker(value);

    // EXPECTED:
    //
    // origin:
    //   value
    //
    // flow:
    //   value
    //      |
    //      v
    //   Future state
    //
    // question:
    //   does analyzer understand values stored across await points?

    assert_eq!(future.await, 5);
}


/// ------------------------------------------------------------
// Additional analyzer questions:
//
// 1. For every function boundary:
//
//    Can we answer:
//       "Where did this value originate?"
//
//
//
// 2. For every parameter:
//
//    Can we classify:
//
//       moved?
//       borrowed?
//       mutably borrowed?
//       copied?
//       cloned?
//
//
//
// 3. For every return value:
//
//    Can we trace:
//
//       return value
//            |
//            v
//       original allocation
//
//
//
// 4. For async:
//
//    Can we identify:
//
//       stack value
//          |
//          v
//       generated future struct
//          |
//          v
//       resumed execution
//
//
//
// 5. For trait objects:
//
//    Can we preserve:
//
//       concrete type
//          |
//          v
//       erased interface

/// ------------------------------------------------------------
static CHAPTER_DATA_STRUCTURES: i32 = 1;
/// ------------------------------------------------------------
// ## Data Structures
// - [ ] Struct field
// - [ ] Nested struct
// - [ ] Enum payload
// - [ ] Tuple field
// - [ ] Array element
// - [ ] Slice element
// - [ ] Vec element
// - [ ] HashMap value

/// ------------------------------------------------------------
// ## Data Structures
//
// - [ ] Struct field
//      A field is owned by the parent struct.
//
//      Question:
//      "Does the analyzer understand that a field originates
//       from the struct allocation?"
//
//      Diagram:
//
//      Struct allocation
//          |
//          v
//      +-------------+
//      | User        |
//      |-------------|
//      | name        | -----> String allocation
//      +-------------+
//
//      Expected:
//      user.name origin == String creation
//
//
// - [ ] Nested struct
//      Ownership relationship through multiple layers.
//
//      Question:
//      "Can we recursively expand through nested ownership?"
//
//      Diagram:
//
//      Outer
//        |
//        v
//      Inner
//        |
//        v
//      Value
//
//      Expected:
//
//      outer.inner.value
//          |
//          v
//      original allocation
//
//
// - [ ] Enum payload
//      Enum variants can contain owned values.
//
//      Question:
//      "Can the analyzer follow values hidden behind variants?"
//
//      Diagram:
//
//      Enum
//        |
//        +--> Variant A
//        |
//        +--> Variant B
//                |
//                v
//              Payload
//
//      Expected:
//      match arms preserve origin information
//
//
// - [ ] Tuple field
//      Tuple fields are positional ownership containers.
//
//      Question:
//      "Can index-based ownership paths be tracked?"
//
//      Diagram:
//
//      tuple
//       |
//       +-- .0 --> value A
//       |
//       +-- .1 --> value B
//
//
// - [ ] Array element
//      Fixed-size collection with inline storage.
//
//      Question:
//      "Can individual elements be traced independently?"
//
//      Diagram:
//
//      [A, B, C]
//
//       array allocation
//          |
//          +--> index 0
//          +--> index 1
//          +--> index 2
//
//
// - [ ] Slice element
//      Borrowed view into another allocation.
//
//      Question:
//      "Can we distinguish slice ownership from element ownership?"
//
//      Diagram:
//
//      Vec/String allocation
//              |
//              v
//          &[T]
//              |
//              v
//           element
//
//      Expected:
//
//      slice does not own data
//      origin belongs to backing allocation
//
//
// - [ ] Vec element
//      Heap allocation with dynamic elements.
//
//      Question:
//      "Can push/reallocation/index mutation be tracked?"
//
//      Diagram:
//
//      Vec
//       |
//       v
//      Heap buffer
//       |
//       +--> element 0
//       +--> element 1
//
//      Questions:
//      - Does vec[0] point to inserted value?
//      - Does mutation affect original element?
//      - Does resize change identity?
//
//
// - [ ] HashMap value
//      Key/value storage behind hashing.
//
//      Question:
//      "Can we trace through map indirection?"
//
//      Diagram:
//
//      HashMap
//          |
//          v
//      bucket
//          |
//          v
//      value
//
//      Questions:
//      - Does insert establish origin?
//      - Does get() preserve relationship?
//      - Does mutation through entry API track correctly?
//
//
/// ------------------------------------------------------------
// Test runner notes:
//
// Run:
//
// cargo test --test ownership_flow
//
// or:
//
// cargo test data_structures -- --nocapture
//
//
/// ------------------------------------------------------------
fn data_structures_struct_field() {
    #[derive(Debug)]
    struct User {
        name: String,
        age: u32,
    }
    let name = String::from("Alice");
    let user = User {
        name,
        age: 30,
    };
    // EXPECTED:
    //
    // origin:
    //   String::from("Alice")
    //
    // path:
    //   name
    //     |
    //     v
    //   user.name
    //
    // analyzer should not lose identity at field assignment

    assert_eq!(user.name, "Alice");
}
#[derive(Debug)]
struct Address {
    city: String,
}
#[derive(Debug)]
struct Person {
    address: Address,
}
fn data_structures_nested_struct() {
    let city = String::from("NYC");

    let address = Address {
        city,
    };

    let person = Person {
        address,
    };


    // EXPECTED:
    //
    // person
    //   |
    //   v
    // address
    //   |
    //   v
    // city
    //
    // origin:
    //   String::from("NYC")

    assert_eq!(person.address.city, "NYC");
}
enum Message {
    Text(String),
    Number(i32),
}
fn data_structures_enum_payload() {
    let text = String::from("hello");

    let message = Message::Text(text);


    // EXPECTED:
    //
    // text
    //  |
    //  move
    //  v
    // Enum::Text(payload)
    //
    // origin preserved through variant

    match message {
        Message::Text(value) => {
            assert_eq!(value, "hello");
        }
        _ => panic!(),
    }
}
fn data_structures_tuple_field() {
    let value = String::from("hello");

    let tuple = (value, 42);


    // EXPECTED:
    //
    // tuple.0
    //    |
    //    v
    // String allocation

    assert_eq!(tuple.0, "hello");
}
fn data_structures_array_element() {
    let a = String::from("A");
    let b = String::from("B");

    let array = [a, b];


    // EXPECTED:
    //
    // array[0]
    //    |
    //    v
    // String::from("A")
    //
    // array[1]
    //    |
    //    v
    // String::from("B")

    assert_eq!(array[0], "A");
    assert_eq!(array[1], "B");
}
fn data_structures_slice_element() {
    let values = vec![
        String::from("A"),
        String::from("B"),
    ];

    let slice = &values[..];


    // EXPECTED:
    //
    // values
    //   |
    //   v
    // &[String]
    //   |
    //   v
    // element
    //
    // slice does not own values

    assert_eq!(slice[0], "A");
}
fn data_structures_vec_element() {
    let value = String::from("hello");

    let mut values = Vec::new();

    values.push(value);


    // EXPECTED:
    //
    // value
    //   |
    //   move
    //   v
    // Vec buffer
    //   |
    //   v
    // values[0]
    //
    // origin:
    //   String::from("hello")

    assert_eq!(values[0], "hello");
}
use std::collections::HashMap;
fn data_structures_hashmap_value() {
    let value = String::from("hello");

    let mut map = HashMap::new();

    map.insert("key", value);


    // EXPECTED:
    //
    // value
    //   |
    //   move
    //   v
    // HashMap bucket
    //   |
    //   v
    // map["key"]
    //
    // question:
    // can analyzer cross hash indirection?

    assert_eq!(map["key"], "hello");
}
/// ------------------------------------------------------------
// Additional analyzer questions:
//
// 1. Struct field:
//
//    Can we answer:
//
//       "Which allocation created this field?"
//
//
//
// 2. Nested structures:
//
//    Can expansion continue:
//
//       object
//          -> field
//          -> nested field
//          -> allocation
//
//
//
// 3. Collections:
//
//    Can we distinguish:
//
//       container identity
//            vs
//       element identity
//
//
//
// 4. Mutation:
//
//    If:
//
//       user.name.push_str("!")
//
//    Can we report:
//
//       mutation target:
//          String allocation originally created here
//
//
//
// 5. Aliasing:
//
//    If:
//
//       let a = &vec[0];
//       let b = &vec[0];
//
//    Can we unify:
//
//       a == b
//       same underlying allocation
//
//
//
// 6. Reallocation:
//
//    For Vec:
//
//       push()
//       reserve()
//       resize()
//
//    Does the analyzer preserve identity after
//    memory movement?

/// ------------------------------------------------------------
static CHAPTER_ALIAS: i32 = 1;
/// ------------------------------------------------------------
// ## Aliasing
// - [ ] Multiple immutable aliases
// - [ ] Mutable alias
// - [ ] Rc alias
// - [ ] Arc alias
// - [ ] Arc<Mutex<T>>
// - [ ] Arc<RwLock<T>>
/// ------------------------------------------------------------
// ## Aliasing
//
// Aliasing answers:
//
// "Can multiple paths refer to the same underlying value?"
//
// Important distinction:
//
// Binding identity:
//     variable name -> value
//
// Memory identity:
//     allocation -> all references pointing to it
//
// The analyzer goal:
//
//     Given a subject:
//
//         "what can affect this value?"
//
//     Find every alias relationship that can influence it.
//
//
//
// - [ ] Multiple immutable aliases
//
//      Multiple readers of the same allocation.
//
//      Rule:
//
//      &T
//      - many allowed
//      - cannot mutate through reference
//
//      Question:
//
//      "Can the analyzer prove these references point
//       to the same origin?"
//
//
//      Diagram:
//
//      allocation
//          |
//          +------------+
//          |            |
//          v            v
//         &a           &b
//
//      Expected:
//
//      a.origin == b.origin
//
//
// - [ ] Mutable alias
//
//      Exclusive mutable access.
//
//      Rule:
//
//      &mut T
//      - only one active mutable alias
//      - mutation affects original owner
//
//
//      Question:
//
//      "Can mutation through this path be attributed
//       back to the original allocation?"
//
//
//      Diagram:
//
//      owner
//        |
//        v
//      &mut value
//        |
//        v
//      mutation
//
//
//      Expected:
//
//      mutation target == owner
//
//
// - [ ] Rc alias
//
//      Reference counted shared ownership.
//
//      Rule:
//
//      Rc<T>
//      - multiple owners
//      - single threaded
//      - shared ownership count
//
//
//      Question:
//
//      "Can we trace all Rc clones back to the same allocation?"
//
//
//      Diagram:
//
//                    +----------------+
//                    | Heap allocation|
//                    +----------------+
//                         ^
//                         |
//              +----------+----------+
//              |                     |
//              v                     v
//            Rc A                  Rc B
//
//      Expected:
//
//      Rc::clone(a)
//          |
//          v
//      same allocation identity
//
//
// - [ ] Arc alias
//
//      Thread-safe reference counted ownership.
//
//      Rule:
//
//      Arc<T>
//      - multiple owners
//      - Send + Sync when T allows
//
//
//
//      Question:
//
//      "Can we identify all owners of this shared memory?"
//
//
//      Diagram:
//
//                  allocation
//                      ^
//                      |
//              +-------+-------+
//              |               |
//              v               v
//             Arc x           Arc y
//
//
//      Expected:
//
//      x and y share origin
//
//
// - [ ] Arc<Mutex<T>>
//
//      Shared ownership + synchronized mutation.
//
//      This is one of the hardest common patterns.
//
//
//      Question:
//
//      "If one Arc owner mutates through Mutex,
//       who else can observe the change?"
//
//
//      Diagram:
//
//              Arc
//               |
//               v
//          Mutex<T>
//               |
//               v
//             value
//
//
//
//      Thread A:
//
//          Arc clone
//              |
//              v
//          lock()
//              |
//              v
//          mutate
//
//
//      Thread B:
//
//          Arc clone
//              |
//              v
//          lock()
//              |
//              v
//          observe mutation
//
//
//
//      Expected:
//
//      mutation path:
//
//          *x.lock()
//                |
//                v
//             Mutex<T>
//                |
//                v
//             shared allocation
//
//
//
// - [ ] Arc<RwLock<T>>
//
//      Shared ownership with multiple readers
//      or one writer.
//
//
//      Question:
//
//      "Can the analyzer distinguish read aliases
//       from write aliases?"
//
//
//      Diagram:
//
//                 Arc
//                  |
//                  v
//              RwLock<T>
//                  |
//          +-------+-------+
//          |               |
//          v               v
//       read()          write()
//
//
//
//      Expected:
//
//      read:
//
//          multiple allowed
//
//      write:
//
//          exclusive mutation
//
//
/// ------------------------------------------------------------
// Test runner notes:
//
// Run:
//
// cargo test --test ownership_flow
//
// or:
//
// cargo test aliasing -- --nocapture
//
//
/// ------------------------------------------------------------


use std::rc::Rc;
use std::sync::{Arc, Mutex, RwLock};


/// ------------------------------------------------------------
// Multiple immutable aliases
/// ------------------------------------------------------------

fn aliasing_multiple_immutable_aliases() {
    let value = String::from("hello");

    let a = &value;
    let b = &value;


    // EXPECTED:
    //
    // value
    //   |
    //   +----> a
    //   |
    //   +----> b
    //
    // same origin
    //
    // mutation:
    // impossible

    assert_eq!(a, b);
}


/// ------------------------------------------------------------
// Mutable alias
/// ------------------------------------------------------------

fn aliasing_mutable_alias() {
    let mut value = String::from("hello");


    let reference = &mut value;

    reference.push_str(" world");


    // EXPECTED:
    //
    // value
    //   |
    //   v
    // &mut value
    //   |
    //   v
    // mutation
    //
    // origin:
    // value
    //
    // mutation target:
    // value


    assert_eq!(value, "hello world");
}


/// ------------------------------------------------------------
// Rc alias
/// ------------------------------------------------------------

fn aliasing_rc_alias() {
    let value = Rc::new(String::from("hello"));


    let a = Rc::clone(&value);
    let b = Rc::clone(&value);


    // EXPECTED:
    //
    // String allocation
    //        ^
    //        |
    //    +---+---+
    //    |       |
    //    v       v
    //    a       b
    //
    // same heap identity


    assert_eq!(&*a, "hello");
    assert_eq!(&*b, "hello");
}


/// ------------------------------------------------------------
// Arc alias
/// ------------------------------------------------------------

fn aliasing_arc_alias() {
    let value = Arc::new(String::from("hello"));


    let a = Arc::clone(&value);
    let b = Arc::clone(&value);


    // EXPECTED:
    //
    // Arc allocation
    //       ^
    //       |
    //   +---+---+
    //   |       |
    //   v       v
    //   a       b
    //
    // shared ownership


    assert_eq!(&*a, "hello");
    assert_eq!(&*b, "hello");
}


/// ------------------------------------------------------------
// Arc<Mutex<T>>
/// ------------------------------------------------------------

fn aliasing_arc_mutex() {
    let value = Arc::new(Mutex::new(0));


    let a = Arc::clone(&value);
    let b = Arc::clone(&value);


    *a.lock().unwrap() = 10;


    // EXPECTED:
    //
    //             Arc
    //              |
    //              v
    //           Mutex
    //              |
    //              v
    //              0
    //
    //
    // mutation:
    //
    // a.lock()
    //    |
    //    v
    // shared value
    //
    //
    // b observes mutation


    assert_eq!(*b.lock().unwrap(), 10);
}


/// ------------------------------------------------------------
// Arc<RwLock<T>>
/// ------------------------------------------------------------

fn aliasing_arc_rwlock() {
    let value = Arc::new(RwLock::new(0));


    let reader = Arc::clone(&value);
    let writer = Arc::clone(&value);


    {
        let mut guard = writer.write().unwrap();
        *guard = 20;
    }


    let result = reader.read().unwrap();


    // EXPECTED:
    //
    // Arc
    //  |
    //  v
    // RwLock
    //  |
    //  +---- read alias
    //  |
    //  +---- write alias
    //
    //
    // write changes value seen by readers


    assert_eq!(*result, 20);
}


/// ------------------------------------------------------------
// Additional analyzer questions:
//
// 1. Identity:
//
//    Can we collapse:
//
//       x
//       y
//       z
//
//    into:
//
//       same allocation group?
//
//
//
// 2. Mutation:
//
//    If:
//
//       alias_a writes
//
//    Can we highlight:
//
//       alias_b
//
//    because it observes the same memory?
//
//
//
// 3. Ownership:
//
//    Can we distinguish:
//
//       Copy
//          |
//          new value
//
//       Clone
//          |
//          related value
//
//       Rc::clone / Arc::clone
//          |
//          same allocation
//
//
//
// 4. Interior mutability:
//
//    Can we detect that:
//
//       immutable binding
//              |
//              v
//       mutable interior state
//
//
//
// 5. Threading:
//
//    For:
//
//       Arc<Mutex<T>>
//
//    can we show:
//
//       "these are all possible mutation points"
//
//
//
// 6. Final feature goal:
//
//    Given:
//
//       cursor on value
//
//    show:
//
//       - original allocation
//       - all aliases
//       - mutation paths
//       - ownership boundaries

/// ------------------------------------------------------------
static CHAPTER_LIFETIME: i32 = 1;
/// ------------------------------------------------------------
// ## Lifetimes
// - [ ] Local lifetime
// - [ ] Returned reference
// - [ ] Generic lifetime
// - [ ] Static lifetime
/// ------------------------------------------------------------
// ## Lifetimes
//
// Lifetimes answer:
//
// "How long is this value guaranteed to remain valid?"
//
// Important distinction:
//
// Lifetime is NOT:
//     where memory lives
//
// Lifetime IS:
//     how long references are allowed to point there
//
// The analyzer goal:
//
//     Given a reference:
//
//         "why is this reference valid?"
//
//     Find:
//
//         - allocation origin
//         - owner lifetime
//         - borrow lifetime
//         - destruction boundary
//
//
//
// - [ ] Local lifetime
//
//      A reference cannot outlive the scope where the owner exists.
//
//      Question:
//
//      "Can the analyzer identify the scope boundary
//       that limits this reference?"
//
//
//      Diagram:
//
//      fn example() {
//
//          +-------------------+
//          | value             |
//          |                   |
//          |   &value -------->|
//          |                   |
//          +-------------------+
//
//      }
//
//      Expected:
//
//      reference lifetime <= value lifetime
//
//
//      Important:
//
//      The owner dies at the end of the block.
//      Any reference escaping is invalid.
//
//
//
// - [ ] Returned reference
//
//      Returning references requires the referenced
//      value to outlive the function call.
//
//      Question:
//
//      "Where did the returned reference come from?"
//
//
//      Valid:
//
//          caller owns value
//              |
//              v
//          function borrows
//              |
//              v
//          returns reference
//
//
//      Invalid:
//
//          function local
//              |
//              v
//          return &local
//
//              X
//              |
//              v
//          dangling reference
//
//
//
// - [ ] Generic lifetime
//
//      Lifetime parameters allow relationships between
//      multiple references.
//
//
//      Question:
//
//      "Can the analyzer preserve lifetime relationships
//       through generic abstractions?"
//
//
//      Example:
//
//          fn choose<'a>(
//              a: &'a str,
//              b: &'a str
//          ) -> &'a str
//
//
//      Diagram:
//
//          input A
//             |
//             |
//          input B
//             |
//             v
//        lifetime 'a
//             |
//             v
//        returned reference
//
//
//      Expected:
//
//      return lifetime tied to inputs
//
//
// - [ ] Static lifetime
//
//      Data guaranteed to live for the entire program.
//
//
//      Question:
//
//      "Can the analyzer identify values that escape
//       normal ownership boundaries?"
//
//
//      Diagram:
//
//
//      Program lifetime
//      +--------------------------------+
//
//          &'static value
//                 |
//                 v
//             global data
//
//
//
//      Examples:
//
//          string literals
//          static variables
//
//
//
/// ------------------------------------------------------------
// Test runner notes:
//
// Run:
//
// cargo test --test ownership_flow
//
// or:
//
// cargo test lifetimes -- --nocapture
//
//
/// ------------------------------------------------------------
/// ------------------------------------------------------------
// Local lifetime
/// ------------------------------------------------------------
fn lifetimes_local_lifetime() {
    let value = String::from("hello");


    let reference = &value;


    // EXPECTED:
    //
    // owner:
    //   value
    //
    // borrow:
    //   reference
    //
    // lifetime:
    //   reference exists inside value scope
    //
    //
    // Diagram:
    //
    // +----------------+
    // | value          |
    // |                |
    // | reference ----+
    // |                |
    // +----------------+


    assert_eq!(reference, "hello");
}
/// ------------------------------------------------------------
// Returned reference
/// ------------------------------------------------------------
fn lifetimes_returned_reference<'a>(value: &'a String) -> &'a String {
    value
}
fn lifetimes_returned_reference_test() {
    let value = String::from("hello");


    let result = lifetimes_returned_reference(&value);


    // EXPECTED:
    //
    // origin:
    //   value
    //
    // flow:
    //
    //   value
    //      |
    //      v
    //   function argument
    //      |
    //      v
    //   result
    //
    //
    // result cannot outlive value


    assert_eq!(result, "hello");
}
/// ------------------------------------------------------------
// Returned reference failure case
/// ------------------------------------------------------------
// This should NOT compile:
//
// fn invalid_return() -> &String {
//     let value = String::from("hello");
//     &value
// }
//
//
// EXPECTED ANALYZER RESULT:
//
// allocation:
//     value
//
// lifetime:
//     function scope
//
// return reference:
//     escapes scope
//
// result:
//     invalid
/// ------------------------------------------------------------
// Generic lifetime
/// ------------------------------------------------------------
fn lifetimes_generic<'a>(a: &'a str, b: &'a str) -> &'a str {

    if a.len() > b.len() {
        a
    } else {
        b
    }
}
fn lifetimes_generic_test() {
    let first = String::from("hello");
    let second = String::from("world");


    let result = lifetimes_generic(
        &first,
        &second,
    );


    // EXPECTED:
    //
    // result lifetime:
    //   minimum lifetime of inputs
    //
    //
    // Diagram:
    //
    // first
    //   |
    //   +------+
    //          |
    //          v
    //        'a
    //          ^
    //          |
    //   +------+
    //   |
    // second
    //
    //
    // return points to one of inputs


    assert_eq!(result.len(), 5);
}
/// ------------------------------------------------------------
// Static lifetime
/// ------------------------------------------------------------
static MESSAGE: &str = "hello";
fn lifetimes_static() {

    let value: &'static str = MESSAGE;


    // EXPECTED:
    //
    // origin:
    //   static allocation
    //
    // lifetime:
    //   entire program
    //
    //
    // Diagram:
    //
    // Program lifetime
    // +--------------------------------+
    //
    //       MESSAGE
    //          |
    //          v
    //      &'static str
    //
    //
    assert_eq!(value, "hello");
}
/// ------------------------------------------------------------
// Additional analyzer questions:
//
// 1. Lifetime visualization:
//
//    Given:
//
//        let r = &value;
//
//    Can we show:
//
//        owner lifetime
//             |
//             v
//        borrow lifetime
//
//
//
// 2. Escape analysis:
//
//    Can we detect:
//
//        local allocation
//             |
//             v
//        returned reference
//
//
//
//    and explain:
//
//        "reference cannot escape because owner dies here"
//
//
//
// 3. Lifetime propagation:
//
//    Through:
//
//        fn foo<'a>(&'a T) -> &'a T
//
//
//
//    Can we preserve:
//
//        input lifetime
//             |
//             v
//        output lifetime
//
//
//
// 4. Struct lifetime:
//
//    Future test:
//
//        struct Holder<'a> {
//            value: &'a String
//        }
//
//
//    Question:
//
//        Does field lifetime trace back
//        to original owner?
//
//
//
// 5. Async lifetime:
//
//    Future challenge:
//
//        Can a reference survive across await?
//
//
//        local value
//             |
//             v
//        Future state machine
//
//
//
// 6. Final feature goal:
//
//    Given cursor on reference:
//
//        &foo
//
//
//    show:
//
//        - origin allocation
//        - owner
//        - lifetime boundary
//        - invalidation points

/// ------------------------------------------------------------
static CHAPTER_TRAIT_DISPATCH: i32 = 1;
/// ------------------------------------------------------------
// ## Trait Dispatch
// - [ ] Trait method
// - [ ] Dynamic dispatch
// - [ ] Associated type
// - [ ] Generic impl
/// ------------------------------------------------------------
// ## Trait Dispatch
//
// Trait dispatch answers:
//
// "When I call this method, which implementation actually runs?"
//
// Important distinction:
//
// Static dispatch:
//     compiler knows concrete type
//
// Dynamic dispatch:
//     runtime chooses implementation
//
// The analyzer goal:
//
//     Given a method call:
//
//         subject.method()
//
//     determine:
//
//         - where implementation comes from
//         - what data is accessed
//         - what ownership/lifetime relationships exist
//
//
//
// - [ ] Trait method
//
//      A concrete type implementing a trait.
//
//      Question:
//
//      "Can we resolve the trait method back to
//       the concrete implementation?"
//
//
//      Diagram:
//
//
//      Trait
//        |
//        v
//      fn speak()
//        |
//        |
//        +------------+
//        |            |
//        v            v
//      Dog          Cat
//      impl          impl
//
//
//      Expected:
//
//      dog.speak()
//
//          resolves to:
//
//      Dog::speak()
//
//
// - [ ] Dynamic dispatch
//
//      Runtime-selected implementation.
//
//      Uses:
//
//          dyn Trait
//
//
//      Question:
//
//      "Can we show all possible implementations
//       behind a trait object?"
//
//
//      Diagram:
//
//
//          &dyn Animal
//              |
//              v
//          vtable
//              |
//        +-----+-----+
//        |           |
//        v           v
//       Dog         Cat
//
//
//
//      Expected:
//
//      analyzer knows:
//
//          possible targets:
//              Dog::speak
//              Cat::speak
//
//
//      Runtime decides actual call.
//
//
//
// - [ ] Associated type
//
//      Trait defines a related type.
//
//
//      Question:
//
//      "Can we resolve the hidden type relationship?"
//
//
//      Example:
//
//          trait Iterator {
//              type Item;
//          }
//
//
//      Diagram:
//
//
//      Iterator<T>
//           |
//           v
//      associated type
//           |
//           v
//        Item
//
//
//
//      Expected:
//
//      Iterator implementation determines:
//
//          Item == concrete type
//
//
//
// - [ ] Generic impl
//
//      Trait implementation parameterized by type.
//
//
//      Question:
//
//      "Can we expand generic constraints into
//       concrete behavior?"
//
//
//      Diagram:
//
//
//      impl<T> Trait for Wrapper<T>
//
//                 |
//                 v
//
//             Wrapper<String>
//
//
//
//      Expected:
//
//      generic implementation applies to
//      all valid T
//
//
//
/// ------------------------------------------------------------
// Test runner notes:
//
// Run:
//
// cargo test --test ownership_flow
//
// or:
//
// cargo test trait_dispatch -- --nocapture
//
//
/// ------------------------------------------------------------

/// ------------------------------------------------------------
// Trait method
/// ------------------------------------------------------------
trait Speak {
    fn speak(&self) -> &'static str;
}
struct Dog;
impl Speak for Dog {

    fn speak(&self) -> &'static str {
        "woof"
    }
}
struct Cat;
impl Speak for Cat {

    fn speak(&self) -> &'static str {
        "meow"
    }
}
fn trait_dispatch_trait_method() {

    let dog = Dog;
    let cat = Cat;


    let dog_sound = dog.speak();
    let cat_sound = cat.speak();


    // EXPECTED:
    //
    // dog.speak()
    //      |
    //      v
    // Dog::speak()
    //
    //
    // cat.speak()
    //      |
    //      v
    // Cat::speak()
    //
    //
    // Static dispatch:
    // compiler knows exact target


    assert_eq!(dog_sound, "woof");
    assert_eq!(cat_sound, "meow");
}
/// ------------------------------------------------------------
// Dynamic dispatch
/// ------------------------------------------------------------
fn trait_dispatch_dynamic_dispatch() {

    let dog = Dog;
    let cat = Cat;


    let animals: Vec<&dyn Speak> = vec![
        &dog,
        &cat,
    ];


    for animal in animals {

        let sound = animal.speak();


        // EXPECTED:
        //
        // animal
        //    |
        //    v
        // &dyn Speak
        //    |
        //    v
        // vtable lookup
        //    |
        //    +----> Dog::speak
        //    |
        //    +----> Cat::speak
        //
        //
        // runtime chooses implementation


        assert!(
            sound == "woof" ||
            sound == "meow"
        );
    }
}
/// ------------------------------------------------------------
// Associated type
/// ------------------------------------------------------------
trait Container {

    type Item;

    fn get(&self) -> Self::Item;
}
struct NumberContainer {
    value: i32,
}
impl Container for NumberContainer {

    type Item = i32;


    fn get(&self) -> Self::Item {
        self.value
    }
}
fn trait_dispatch_associated_type() {

    let container = NumberContainer {
        value: 42,
    };


    let value = container.get();


    // EXPECTED:
    //
    // Container
    //      |
    //      v
    // associated Item
    //      |
    //      v
    // i32
    //
    //
    // analyzer resolves:
    //
    // Self::Item == i32


    assert_eq!(value, 42);
}
/// ------------------------------------------------------------
// Generic impl
/// ------------------------------------------------------------
struct Wrapper<T> {
    value: T,
}
trait Describe {

    fn describe(&self) -> &'static str;

}
impl<T> Describe for Wrapper<T> {

    fn describe(&self) -> &'static str {
        "wrapper"
    }

}
fn trait_dispatch_generic_impl() {


    let string_wrapper = Wrapper {
        value: String::from("hello"),
    };


    let number_wrapper = Wrapper {
        value: 123,
    };


    let a = string_wrapper.describe();
    let b = number_wrapper.describe();



    // EXPECTED:
    //
    //
    // impl<T> Describe for Wrapper<T>
    //
    //                |
    //        +-------+-------+
    //        |               |
    //        v               v
    //
    // Wrapper<String>   Wrapper<i32>
    //
    //
    // same implementation applies


    assert_eq!(a, "wrapper");
    assert_eq!(b, "wrapper");
}
/// ------------------------------------------------------------
// Additional analyzer questions:
//
// 1. Method resolution:
//
//    Given:
//
//        value.method()
//
//    Can we show:
//
//        trait definition
//             |
//             v
//        selected impl
//             |
//             v
//        executed function
//
//
//
// 2. Dynamic dispatch:
//
//    Given:
//
//        Box<dyn Trait>
//
//    Can we list:
//
//        possible implementations?
//
//
//
// 3. Ownership through traits:
//
//    If:
//
//        trait method mutates self
//
//
//    Can we trace:
//
//        call site
//             |
//             v
//        receiver ownership
//             |
//             v
//        mutation target
//
//
//
// 4. Generic expansion:
//
//    Given:
//
//        impl<T> Trait for Wrapper<T>
//
//
//    Can we show:
//
//        Wrapper<String>
//             |
//             v
//        concrete implementation
//
//
//
// 5. Trait objects:
//
//    Future tests:
//
//        Box<dyn Trait>
//        Arc<dyn Trait>
//        &dyn Trait
//
//
//    Questions:
//
//        Who owns the object?
//        Where is the vtable?
//        What implementations are possible?
//
//
//
// 6. Final feature goal:
//
//    Cursor on:
//
//        object.method()
//
//
//    Show:
//
//        - trait
//        - implementation
//        - dispatch type
//        - ownership of receiver
//        - possible mutation paths

/// ------------------------------------------------------------
static CHAPTER_CONTROL_FLOW: i32 = 1;
/// ------------------------------------------------------------
// ## Control Flow
// - [ ] If/else
// - [ ] Match
// - [ ] Loop
// - [ ] While
// - [ ] For
// - [ ] Early return

/// ------------------------------------------------------------
// ## Control Flow
//
// Control flow answers:
//
// "How does execution move through the program,
// and which values can influence this path?"
//
// Important distinction:
//
// Data flow:
//     where values move
//
// Control flow:
//     which code paths execute
//
// The analyzer goal:
//
//     Given a subject:
//
//         "why can this value exist here?"
//
//     Understand:
//
//         - branches
//         - possible execution paths
//         - conditions
//         - early exits
//         - unreachable paths
//
//
//
// - [ ] If/else
//
//      Conditional execution.
//
//      Question:
//
//      "Can the analyzer understand that a value
//       depends on a condition?"
//
//
//      Diagram:
//
//
//              condition
//                  |
//          +-------+-------+
//          |               |
//          v               v
//        true            false
//          |               |
//          +-------+-------+
//                  |
//                  v
//              result
//
//
//
//      Expected:
//
//      result depends on condition
//
//
//
// - [ ] Match
//
//      Pattern-based branching.
//
//      Question:
//
//      "Can the analyzer follow values through
//       enum variants and pattern matching?"
//
//
//      Diagram:
//
//
//              enum value
//                   |
//                   v
//              match
//              /   \
//             /     \
//          Variant A Variant B
//              |         |
//              v         v
//           branch    branch
//
//
//      Expected:
//
//      each branch preserves origin information
//
//
//
// - [ ] Loop
//
//      Repeated execution.
//
//      Question:
//
//      "Can the analyzer handle values that
//       are updated across iterations?"
//
//
//      Diagram:
//
//
//          initial value
//               |
//               v
//          +---------+
//          |  loop   |
//          +---------+
//               |
//               v
//           mutation
//               |
//               +---- back
//
//
//      Expected:
//
//      loop-carried dependencies are tracked
//
//
//
// - [ ] While
//
//      Condition-controlled loop.
//
//      Question:
//
//      "Can the analyzer understand that execution
//       count depends on runtime state?"
//
//
//      Diagram:
//
//
//          condition
//              |
//        +-----+-----+
//        |           |
//       yes          no
//        |           |
//        v           |
//      body          |
//        |           |
//        +-----------+
//
//
//
//      Expected:
//
//      mutations inside body affect future checks
//
//
//
// - [ ] For
//
//      Iterator-based loop.
//
//      Question:
//
//      "Can the analyzer trace values coming from
//       iterator elements?"
//
//
//      Diagram:
//
//
//      collection
//          |
//          v
//      iterator
//          |
//          v
//       item
//          |
//          v
//       loop body
//
//
//      Expected:
//
//      item origin points back to collection
//
//
//
// - [ ] Early return
//
//      Control flow exits before the end.
//
//      Question:
//
//      "Can the analyzer understand that some paths
//       terminate execution?"
//
//
//      Diagram:
//
//
//          condition
//              |
//       +------+------+
//       |             |
//       v             v
//    return        continue
//       |
//       v
//    function end
//
//
//
//      Expected:
//
//      unreachable code after return is excluded
//
//
/// ------------------------------------------------------------
// Test runner notes:
//
// Run:
//
// cargo test --test ownership_flow
//
// or:
//
// cargo test control_flow -- --nocapture
//
/// ------------------------------------------------------------
/// ------------------------------------------------------------
// If / else
/// ------------------------------------------------------------
fn control_flow_if_else(input: bool) -> String {
    let value;
    if input {
        value = String::from("true");
    } else {
        value = String::from("false");
    }
    // EXPECTED:
    //
    // condition
    //     |
    // +---+---+
    // |       |
    // v       v
    // "true" "false"
    // |       |
    // +---+---+
    //     |
    //     v
    // value
    //
    //
    // value has two possible origins
    value
}
fn control_flow_if_else_test() {

    assert_eq!(
        control_flow_if_else(true),
        "true"
    );

    assert_eq!(
        control_flow_if_else(false),
        "false"
    );
}
/// ------------------------------------------------------------
// Match
/// ------------------------------------------------------------
enum Input {
    Text(String),
    Number(i32),
}
fn control_flow_match(input: Input) -> String {

    match input {

        Input::Text(value) => {

            // EXPECTED:
            //
            // Input::Text
            //       |
            //       v
            //     value

            value
        }


        Input::Number(value) => {

            // EXPECTED:
            //
            // Input::Number
            //       |
            //       v
            //    i32
            //

            value.to_string()
        }
    }
}
fn control_flow_match_test() {

    assert_eq!(
        control_flow_match(
            Input::Text(
                "hello".into()
            )
        ),
        "hello"
    );


    assert_eq!(
        control_flow_match(
            Input::Number(42)
        ),
        "42"
    );
}
/// ------------------------------------------------------------
// Loop
/// ------------------------------------------------------------
fn control_flow_loop() {

    let mut value = 0;


    loop {

        value += 1;


        if value == 3 {
            break;
        }
    }


    // EXPECTED:
    //
    // value
    //   |
    //   v
    // mutation
    //   |
    //   v
    // next iteration
    //
    //
    // loop creates repeated mutation path


    assert_eq!(value, 3);
}
/// ------------------------------------------------------------
// While
/// ------------------------------------------------------------
fn control_flow_while() {

    let mut value = 0;


    while value < 3 {

        value += 1;
    }


    // EXPECTED:
    //
    // value
    //   |
    //   v
    // condition
    //   |
    //   v
    // mutation
    //
    //
    // mutation influences next condition
    assert_eq!(value, 3);
}
/// ------------------------------------------------------------
// For
/// ------------------------------------------------------------
fn control_flow_for() {

    let values = vec![
        1,
        2,
        3,
    ];
    let mut total = 0;
    for value in values {

        total += value;
    }
    // EXPECTED:
    //
    // Vec
    //  |
    //  v
    // iterator
    //  |
    //  v
    // value
    //  |
    //  v
    // total mutation
    //
    //
    // total depends on every element
    assert_eq!(total, 6);
}
/// ------------------------------------------------------------
// Early return
/// ------------------------------------------------------------
fn control_flow_early_return(value: Option<String>) -> String {


    let value = match value {

        Some(value) => value,

        None => {
            return String::from("missing");
        }
    };


    // EXPECTED:
    //
    // Option
    //    |
    //    v
    // match
    //
    // Some:
    //    |
    //    v
    // continue
    //
    // None:
    //    |
    //    v
    // return
    //
    //
    // later code only sees Some path


    value
}
fn control_flow_early_return_test() {

    assert_eq!(
        control_flow_early_return(
            Some("hello".into())
        ),
        "hello"
    );


    assert_eq!(
        control_flow_early_return(None),
        "missing"
    );
}
/// ------------------------------------------------------------
// Additional analyzer questions:
//
// 1. Branch merging:
//
//    Given:
//
//        if condition {
//            a = x;
//        } else {
//            a = y;
//        }
//
//
//    Can we show:
//
//        a
//        |
//        +--> x
//        |
//        +--> y
//
//
//
// 2. Path sensitivity:
//
//    Can we distinguish:
//
//        if x.is_some()
//
//    from:
//
//        else
//
//
//
// 3. Loop dependencies:
//
//    Given:
//
//        x = x + 1;
//
//
//    Can we show:
//
//        previous x
//             |
//             v
//        current x
//
//
//
// 4. Match exhaustiveness:
//
//    Can we know:
//
//        enum variant
//             |
//             v
//        exact branch
//
//
//
// 5. Early exits:
//
//    Can we remove impossible paths:
//
//        return;
//
//        unreachable code;
//
//
//
// 6. Mutation tracking:
//
//    Given:
//
//        loop {
//            state.update()
//        }
//
//
//    Can we highlight:
//
//        every possible mutation source?
//
//
//
// 7. Final feature goal:
//
//    Cursor on:
//
//        variable
//
//
//    Show:
//
//        - possible origins
//        - possible branches
//        - execution paths
//        - mutation paths

/// ------------------------------------------------------------
static CHAPTER_CROSS_BOUNDARY: i32 = 1;
/// ------------------------------------------------------------
// ## Cross-Boundary
// - [ ] Module
// - [ ] Crate
// - [ ] External crate
/// ------------------------------------------------------------
// # Cross-Boundary Ownership / Identity Tests
// Purpose:
//     These tests verify how identity and data flow behave when crossing Rust
//     compilation boundaries.

// Important:
//     Unlike previous examples, these intentionally require multiple files.

// Recommended layout:

// cross_boundary_tests/
// ├── main.rs              <-- binary crate entry
// ├── local_module.rs      <-- module boundary
// ├── library_crate/
// │   ├── Cargo.toml       <-- separate crate
// │   └── src/
// │       └── lib.rs       <-- external crate boundary

// The expected analysis questions:

// 1. Module boundary
//     - Can we resolve identity through `mod`?
//     - Does the symbol keep its origin?
//     - Does visibility affect ownership tracking?

// 2. Crate boundary
//     - Can we trace values passed through public APIs?
//     - Does the exported function become the new "root"?
//     - Can we continue analysis into dependency metadata?

// 3. External crate boundary
//     - What happens when source is unavailable?
//     - Can rustc metadata / MIR / type information provide enough information?
//     - Do we treat external functions as opaque?

// The ideal UI behavior:

// Click:
//     let result = local_module::create_value();

// Panel:
//     Origin:
//         local_module.rs:10 create_value()

//     Flow:
//         local_module::create_value
//             ↓
//         main::cross_boundary_module
//             ↓
//         result

// ---

// # Questions

// Module:
//     - Is this just another scope?
//     - Does FileId matter?
//     - Can we resolve through the module tree?

// Crate:
//     - Is pub fn a controlled boundary?
//     - Should exported functions become "known transformers"?

// External crate:
//     - If Vec::push mutates memory:
//         who owns the allocation?
//     - If HashMap::insert changes state:
//         where is the root?

/// ------------------------------------------------------------
// Module Boundary
/// ------------------------------------------------------------
//
// File: local_module.rs
//
// pub fn create_value() -> i32 {
//     42
// }
//
// File: main.rs
//
// mod local_module;
//
// fn cross_boundary_module() {
//     let value = local_module::create_value();
//
//     assert_eq!(value, 42);
// }
//
// Expected:
//
// value
//   ↓
// local_module::create_value()
//   ↓
// literal 42
//
// Origin should resolve across file boundary.
#[allow(dead_code)]
mod local_module {
    pub fn create_value() -> i32 {
        42
    }

    pub fn mutate_value(value: &mut i32) {
        *value += 1;
    }
}
fn cross_boundary_module() {
    let mut value = local_module::create_value();

    local_module::mutate_value(&mut value);

    assert_eq!(value, 43);

    /*
    Analysis:

    Allocation:
        value created in main

    Value origin:
        local_module::create_value

    Mutation:
        local_module::mutate_value

    Expected flow:

        create_value()
             |
             v
        value
             |
             v
        mutate_value()
    */
}
/// ------------------------------------------------------------
// Crate Boundary
/// ------------------------------------------------------------
//
// Expected layout:
//
// workspace/
// ├── app/
// │   └── src/main.rs
// │
// └── helper/
//     └── src/lib.rs
//
//
// helper/src/lib.rs:
//
// pub fn create_number() -> i32 {
//     100
// }
//
// app/src/main.rs:
//
// use helper::create_number;
//
// fn cross_boundary_crate() {
//     let value = create_number();
//
//     assert_eq!(value, 100);
// }
//
// Expected:
//
// app
//  |
//  v
// helper crate
//  |
//  v
// literal 100
//
// Questions:
//
// Can analysis cross crate metadata?
//
// If source exists:
//     follow source.
//
// If source missing:
//     use rustc metadata / MIR / type information.
fn cross_boundary_crate_example() {
    // Placeholder for separate crate:
    //
    // let value = helper::create_number();
    //
    // assert_eq!(value, 100);


    /*
    Expected future representation:

    External Symbol:

        helper::create_number

    Known:
        return type = i32

    Unknown:
        internal mutation
        allocation details


    Boundary classification:

        [known API]
              |
              v
        [unknown implementation]
    */
}
/// ------------------------------------------------------------
// External Crate Boundary
/// ------------------------------------------------------------
//
// Examples:
// - std::collections::Vec
// - std::sync::Arc
// - tokio
// - serde
//
// These are NOT part of the current crate source tree.
//
// Example:
//
// fn cross_boundary_external() {
//     let mut values = Vec::new();
//     values.push(1);
//
//     assert_eq!(values[0], 1);
// }
//
//
//
// Expected:
//
// Allocation:
//
// Vec::new()
//      |
//      v
// heap allocation
//      |
//      v
// values
//
// Mutation:
//
// values.push(1)
//
//
//
// Questions:
//
// Can we know:
//
// - Vec owns memory?
// - push reallocates?
// - capacity changes?
// - element moved?
//
//
//
// Possible strategy:
//
// Layer 1:
//     Use rust-analyzer semantic info.
//
// Layer 2:
//     Use rustc metadata.
//
// Layer 3:
//     Use MIR if available.
//
// Layer 4:
//     Treat as opaque.
//
fn cross_boundary_external() {

    let mut values = Vec::new();

    values.push(1);

    assert_eq!(values[0], 1);


    /*
    Expected:

    Allocation:
        Vec allocation

    Binding:
        values owns Vec

    Mutation:
        push modifies Vec contents


    UI:

        values

        Origin:
            Vec::new()

        Mutated by:
            Vec::push()


    */
}
/// ------------------------------------------------------------
// Trait / Library Boundary Example
/// ------------------------------------------------------------
//
// Another important external boundary.
//
// A library exposes:
//
// pub trait Storage {
//     fn save(&mut self, value: String);
// }
//
//
//
// Application:
//
// let storage = Database::new();
//
// storage.save(data);
//
//
//
// Question:
//
// Does the IDE follow:
//
// call site
//      ↓
// trait method
//      ↓
// implementation
//
// or:
//
// call site
//      ↓
// unknown dispatch boundary
//
//
// Dynamic dispatch especially matters:
//
// Box<dyn Trait>
//
/// ------------------------------------------------------------
trait Storage {
    fn save(&mut self, value: String);
}
struct MemoryStorage {
    values: Vec<String>,
}
impl Storage for MemoryStorage {
    fn save(&mut self, value: String) {
        self.values.push(value);
    }
}
fn cross_boundary_trait_object() {
    let mut storage: Box<dyn Storage> =
        Box::new(MemoryStorage {
            values: Vec::new(),
        });
    storage.save("hello".to_string());
    /*
    Analysis:

    Static dispatch:

        caller
          |
          v
        MemoryStorage::save


    Dynamic dispatch:

        caller
          |
          v
        dyn Storage
          |
          v
        unknown implementation


    Expected UI:

    Confidence:

        HIGH:
            trait contract

        MEDIUM:
            known implementations

        LOW:
            runtime-selected implementation

    */
}
/// ------------------------------------------------------------
// Summary
/// ------------------------------------------------------------
//
// Cross-boundary hierarchy:
//
// Same function
//      ↓
// Same file
//      ↓
// Same module
//      ↓
// Same crate
//      ↓
// Workspace crate
//      ↓
// External crate
//      ↓
// Runtime boundary
//
//
// Each step increases uncertainty.
//
// The analyzer should not pretend every boundary is identical.
//
// Instead:
//
// Origin confidence:
//
// 100%
//   |
//   | local variable
//   |
// 90%
//   |
//   | module
//   |
// 70%
//   |
//   | crate
//   |
// 40%
//   |
//   | external dependency
//   |
// 10%
//   |
//   | runtime dynamic behavior
//
//
// The goal is not "always know".
//
// The goal is:
// "show exactly where knowledge stops."

/// ------------------------------------------------------------
static CHAPTER_UNSAFE: i32 = 1;
/// ------------------------------------------------------------
// ## Unsafe
// - [ ] Raw pointer mutation
// - [ ] FFI
// - [ ] Union
// # Ownership / Origin Analysis Test Suite
//
// Purpose:
// These tests define the expected behavior for a future ownership-flow analyzer.
//
// The analyzer goal:
// Given a clicked identifier, determine:
// - Where the data originated
// - How it reached this location
// - What operations can mutate it
// - What aliases can affect it
//
// This is not testing Rust compilation.
// This is testing the semantic model:
//
// Allocation
//     ↓
// Binding
//     ↓
// Value creation / movement
//     ↓
// References / aliases
//     ↓
// Mutation
//
// Expected output examples:
//
// Subject: y
// Origin:
//   x @ line 5
//
// Flow:
//   x
//    ↓ copy
//   y
//
// Mutation sources:
//   none
//
/// ------------------------------------------------------------
// ## Unsafe
//
// Unsafe introduces places where Rust's normal ownership guarantees
// are suspended.
//
// Analyzer questions:
//
// - Where was this memory allocated?
// - Who currently has authority to mutate it?
// - Is the pointer still valid?
// - Does the unsafe block create a new alias relationship?
//
// Important:
// Unsafe does not remove the need for ownership analysis.
// It moves the proof obligation from the compiler to the programmer.
//
/// ------------------------------------------------------------
/// ------------------------------------------------------------
// ## Raw pointer mutation
//
// Ownership meaning:
//
// *mut T:
// - stores an address
// - does NOT own the memory
// - does NOT track lifetime
// - can create aliases
//
// Analyzer expectation:
//
// Subject:
//     *ptr
//
// Origin:
//     value allocated in x
//
// Mutation path:
//
//     x
//      ↓
//     ptr (*mut i32)
//      ↓
//     unsafe dereference
//      ↓
//     mutation
//
// Question:
// If ptr changes the value, does x observe the change?
//
// Answer:
// Yes, if ptr points to x.
//
/// ------------------------------------------------------------

fn unsafe_raw_pointer_mutation() {
    let mut x = 10;

    let ptr: *mut i32 = &mut x;

    unsafe {
        *ptr = 20;
    }

    assert_eq!(x, 20);

    // Analyzer:
    //
    // x:
    //   allocation: stack
    //   owner: x
    //
    // ptr:
    //   allocation: none
    //   relationship: address alias
    //
    // mutation:
    //   ptr -> x
    //
    // Origin(x):
    //   x
    //
    // Mutation source:
    //   unsafe block
}


/// ------------------------------------------------------------
// ## FFI
//
// Foreign Function Interface.
//
// Rust calls code outside Rust's ownership model.
//
// Analyzer questions:
//
// - Did foreign code mutate this?
// - Does ownership transfer across boundary?
// - Is the pointer still valid?
//
// Important:
//
// Rust cannot prove what C does.
//
// FFI is another proof boundary like unsafe.
//
/// ------------------------------------------------------------


// pretend external C function:
//
// extern "C" {
//     fn mutate(value: *mut i32);
// }


// Example:
//
// C:
// void mutate(int* value) {
//     *value = 100;
// }
//
// Rust:
// unsafe {
//     mutate(&mut x);
// }
//
// Analyzer expectation:
//
// Before:
//
// x
//
// After:
//
// x
//  ↑
//  |
// FFI mutation
//
// The analyzer should mark:
//
// "unknown external mutation source"
//
// because Rust cannot inspect foreign implementation.
#[allow(dead_code)]
fn unsafe_ffi_placeholder() {
    let mut value = 1;

    // Imaginary:
    //
    // unsafe {
    //     mutate(&mut value);
    // }

    assert_eq!(value, 1);

    // Analyzer:
    //
    // value:
    //   origin: local binding
    //
    // mutation:
    //   possible external source
    //
    // confidence:
    //   unknown
}
/// ------------------------------------------------------------
// ## Union
//
// Rust unions allow multiple interpretations of the same memory.
//
// Example:
//
// A union stores either:
//     i32
// or
//     f32
//
// but the memory location is identical.
//
// Analyzer questions:
//
// - Which interpretation is currently valid?
// - Which field was last written?
// - Is this read valid?
//
/// ------------------------------------------------------------
union Number {
    integer: i32,
    float: f32,
}
/// ------------------------------------------------------------
// Union construction
//
// Memory:
//
// Allocation:
//
// Number
// +-------------+
// | same bytes  |
// +-------------+
//
// Binding:
//
// value
//    |
//    v
// Number memory
//
//
//
// Important:
// integer and float are NOT two values.
//
// They are two views of the same memory.
//
/// ------------------------------------------------------------
fn unsafe_union() {
    let value = Number {
        integer: 42,
    };
    unsafe {
        assert_eq!(value.integer, 42);
    }
    // Analyzer:
    //
    // value:
    //   allocation: union memory
    //
    // fields:
    //   integer
    //   float
    //
    // alias:
    //   integer and float overlap
    //
    // mutation:
    //   writing one field changes all views
}
/// ------------------------------------------------------------
// Additional Questions For Analyzer
/// ------------------------------------------------------------
//
// Raw pointers:
//
// Q:
// If two raw pointers point to the same address,
// are they aliases?
//
// Expected:
// Yes.
//
//
// Q:
// Can analyzer prove lifetime?
//
// Expected:
// No.
// Must mark unknown.
//
//
// Q:
// Does dereferencing *mut T create ownership?
//
// Expected:
// No.
//
//
//
// FFI:
//
// Q:
// Can analyzer know what C modifies?
//
// Expected:
// No unless metadata exists.
//
//
//
// Q:
// Should FFI calls invalidate assumptions?
//
// Expected:
// Yes.
//
//
//
// Union:
//
// Q:
// Are union fields independent values?
//
// Expected:
// No.
//
//
//
// Q:
// Can analyzer track active union variant?
//
// Expected:
// Only with additional user/compiler metadata.
//
//
//
/// ------------------------------------------------------------
//
// Release V1 expectation:
//
// Support:
// [ ] raw pointer relationship detection
// [ ] unsafe boundary marking
// [ ] mutation source marking
// [ ] unknown external mutation warnings
// [ ] union overlap detection
//
// Not required:
//
// [ ] proving unsafe correctness
// [ ] proving FFI behavior
// [ ] full alias analysis

/// ------------------------------------------------------------
static CHAPTER_SPECIAL_CASES: i32 = 1;
/// ------------------------------------------------------------
// ## Special Cases
// - [ ] Self-referential patterns
// - [ ] Cyclic Rc
// - [ ] Drop
// - [ ] Pin
// - [ ] MaybeUninit
// # Ownership / Origin Analysis Test Suite
//
/// ------------------------------------------------------------
// ## Special Cases
//
// These are the cases where simple "follow the variable back to
// its origin" analysis starts breaking down.
//
// These are important because they represent places where:
//
// Allocation
//      ↓
// Binding
//      ↓
// Value flow
//      ↓
// Mutation
//
// is no longer a simple tree.
//
// These introduce:
//
// - cycles
// - hidden ownership transitions
// - delayed destruction
// - pinned locations
// - partially initialized memory
//
/// ------------------------------------------------------------


/// ------------------------------------------------------------
// ## Self-referential patterns
//
// A self-referential struct contains a reference to data owned by
// itself.
//
// Example:
//
// struct
// +----------------+
// | String data    |
// | reference ----+|
// +----------------+
//                 |
//                 +----> inside same struct
//
// Problem:
//
// Moving the struct changes its address.
//
// The internal reference becomes invalid.
//
//
//
// Analyzer questions:
//
// - Does this value depend on its own address?
// - Is this movable?
// - Does the reference point inside allocation?
//
// Important:
//
// Normal ownership graph:
//
// A
// ↓
// B
//
//
// Self reference:
//
// A
// ↺
//
// This is a cycle inside one allocation.
//
/// ------------------------------------------------------------


// Rust normally prevents this without unsafe tricks.
//
// Example pattern only:
//
// struct SelfRef<'a> {
//     value: String,
//     reference: &'a str,
// }


// Analyzer expectation:
//
// Subject:
//
// self_ref.reference
//
//
// Origin:
//
// self_ref.value
//
//
// Relationship:
//
// internal pointer
//
// Risk:
//
// moving self_ref invalidates reference
//
//
//
// Additional questions:
//
// Q:
// Can this value be freely moved?
//
// Expected:
// No.
//
//
// Q:
// Is this the same as Rc cycle?
//
// Expected:
// No.
//
// Rc creates ownership graph cycles.
// Self-reference creates address dependency.
//
/// ------------------------------------------------------------



/// ------------------------------------------------------------
// ## Cyclic Rc
//
// Rc creates shared ownership.
//
// Normally:
//
// A
// |
// v
// B
//
// But cycles:
//
// A ----> B
// ^      |
// |      v
// +------
//
//
//
// Example:
//
/// ------------------------------------------------------------

use std::cell::RefCell;
use std::rc::Rc;
struct Node {
    next: Option<Rc<RefCell<Node>>>,
}
fn special_cyclic_rc() {
    let a = Rc::new(RefCell::new(Node {
        next: None,
    }));
    let b = Rc::new(RefCell::new(Node {
        next: Some(a.clone()),
    }));
    a.borrow_mut().next = Some(b.clone());
    // Analyzer:
    //
    // Ownership graph:
    //
    // a
    // |
    // v
    // b
    // |
    // v
    // a
    //
    //
    // Problem:
    //
    // Reference counting cannot reach zero.
    //
    // Memory leak possible.
}
// Analyzer questions:
//
// Q:
// Is Rc ownership always tree shaped?
//
// Expected:
// No.
//
//
// Q:
// Can reference counting prove cleanup?
//
// Expected:
// No.
//
//
// Q:
// Should cycles become their own graph category?
//
// Expected:
// Yes.
//
/// ------------------------------------------------------------
/// ------------------------------------------------------------
// ## Drop
//
// Drop is where ownership ends.
//
// A value's destructor can:
//
// - mutate global state
// - access other objects
// - release resources
//
//
//
// Example:
//
/// ------------------------------------------------------------
struct Resource {
    id: i32,
}
impl Drop for Resource {
    fn drop(&mut self) {
        println!("dropping {}", self.id);
    }
}
fn special_drop() {
    let resource = Resource {
        id: 1,
    };
    assert_eq!(resource.id, 1);

    // At end of scope:
    //
    // resource
    //    |
    //    v
    // Drop::drop()
}
// Analyzer:
//
// Resource lifecycle:
//
// allocation
//    ↓
// binding
//    ↓
// mutation
//    ↓
// drop
// Questions:
//
// Q:
// Is drop a mutation?
//
// Expected:
// Yes, potentially.
//
//
// Q:
// Can drop affect other values?
//
// Expected:
// Yes.
//
//
// Q:
// Should Drop implementations be analyzed as mutation sources?
//
// Expected:
// Yes.
//
/// ------------------------------------------------------------
/// ------------------------------------------------------------
// ## Pin
//
// Pin prevents moving a value.
//
// Important for:
//
// - async
// - generators
// - self references
//
//
// Normal:
//
// value
//
// can move:
//
// stack A
//    |
//    v
// stack B
//
// Pin:
//
// stack A
//    |
//    X
// cannot move
//
//
//
// Example:
//
/// ------------------------------------------------------------
use std::pin::Pin;
fn special_pin() {
    let value = Box::new(String::from("hello"));

    let pinned = Pin::new(value);

    assert_eq!(pinned.len(), 5);


    // Analyzer:
    //
    // Ownership:
    //
    // pinned
    //    |
    //    v
    // heap allocation
    //
    //
    // Constraint:
    //
    // address stability required
}
// Analyzer questions:
//
// Q:
// Does Pin create ownership?
//
// Expected:
// No.
//
//
// Q:
// Does Pin change movement rules?
//
// Expected:
// Yes.
//
//
// Q:
// Should Pin be treated as an address constraint?
//
// Expected:
// Yes.
//
/// ------------------------------------------------------------
/// ------------------------------------------------------------
// ## MaybeUninit
//
// Represents memory that has been allocated but not initialized.
//
//
//
// Normal:
//
// Allocation
//    |
//    v
// Value
//
//
//
// MaybeUninit:
//
// Allocation
//    |
//    v
// UNKNOWN MEMORY
//
//
//
// Example:
//
/// ------------------------------------------------------------
use std::mem::MaybeUninit;
fn special_maybe_uninit() {
    let mut value: MaybeUninit<i32> =
        MaybeUninit::uninit();


    unsafe {
        value.write(42);

        let initialized = value.assume_init();

        assert_eq!(initialized, 42);
    }
}
// Analyzer:
//
// Before write:
//
// value
//    |
//    v
// uninitialized memory
//
//
// After write:
//
// value
//    |
//    v
// i32
//
//
// Important:
//
// Initialization is a separate event from allocation.
//
//
//
// Questions:
//
// Q:
// Does allocation imply a valid value?
//
// Expected:
// No.
//
//
// Q:
// Is mutation required before reading?
//
// Expected:
// Yes.
//
//
// Q:
// Should uninitialized memory be tracked separately?
//
// Expected:
// Yes.
//
/// ------------------------------------------------------------

/// ------------------------------------------------------------
// Additional Questions For Analyzer
/// ------------------------------------------------------------
//
// Self reference:
//
// Q:
// Is ownership graph still a tree?
//
// Expected:
// No.
//
//
// Rc cycle:
//
// Q:
// Can reference count reach zero?
//
// Expected:
// Not necessarily.
//
//
// Drop:
//
// Q:
// Can destruction execute arbitrary code?
//
// Expected:
// Yes.
//
//
// Pin:
//
// Q:
// Does address become part of identity?
//
// Expected:
// Yes.
//
//
// MaybeUninit:
//
// Q:
// Can memory exist without a value?
//
// Expected:
// Yes.
//
//
/// ------------------------------------------------------------
//
// Release V1 Expectations:
//
// Support:
//
// [ ] Detect ownership graph cycles
// [ ] Mark self-referential address dependencies
// [ ] Treat Drop as mutation boundary
// [ ] Treat Pin as move restriction
// [ ] Track initialization state
//
// Not required:
//
// [ ] Prove unsafe self-reference correctness
// [ ] Solve all destructor side effects
// [ ] Verify MaybeUninit invariants
//
/// ------------------------------------------------------------
fn main() {
    let x = Arc::new(Mutex::new(0));
    let y = x.clone();
    *x.lock().unwrap() = 1;
    println!("{}", y.lock().unwrap());
}
