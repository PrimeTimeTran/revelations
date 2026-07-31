# Ownership / Flow Analysis Final Exam

Core question:

"Given this subject, can we explain:
1. where it originated,
2. how it moved,
3. who can access it,
4. who can mutate it,
5. when it dies?"

---

## 1. IDENTITY

What is this thing?
This is the "who am I?" layer.

Examples:
```rust
let x = 1;
```
`x` is:
- identifier
- local binding
- i32 value
- stack location

Concepts:

- Declaration
- Identifier
- Symbol
- Binding
- Shadowing
- Alias identity
- Memory identity

Questions:

"Are these two names the same thing?"
```rust
let x = 1;
let y = x;
```

```mermaid
%%{init: {"flowchart": {"htmlLabels": true}, "themeVariables": {"fontSize": "16px"}}}%%
flowchart TD
    x["<b>x</b><br/><i>i32 = 1</i>"]
    y["<b>y</b><br/><i>i32 = 1</i>"]

    x --> x_value["<b>Value</b><br/><i>1</i>"]
    y --> y_value["<b>Value</b><br/><i>1</i>"]

    classDef name fill:#e8f3ff,stroke:#4a90e2,stroke-width:2px,color:#1a365d
    classDef value fill:#e8f7ee,stroke:#38a169,stroke-width:2px,color:#1c4532

    class x,y name
    class x_value,y_value value
```
<div align="center">
Figure ... — Independent bindings with identical values
</div>

---
## 2. ALLOCATION

Where does the memory come from?

Stack:

```rust
let x = 1;
```

```mermaid
flowchart TD
    subgraph Stack["<b>Stack</b>"]
        x["<b>x</b><br/><i>i32 = 1</i>"]
        ptr["<b>ptr</b><br/><i>Box pointer</i>"]
    end

    subgraph Heap["<b>Heap</b>"]
        value["<b>Heap allocation</b><br/><i>value: 1</i>"]
    end

    x --> x_value["<b>Value</b><br/><i>stored directly</i>"]
    ptr --> value

    classDef binding fill:#e8f3ff,stroke:#4a90e2,stroke-width:2px,color:#1a365d
    classDef ownership fill:#eadcf8,stroke:#8e44ad,stroke-width:2px,color:#2c1838
    classDef memory fill:#e8f7ee,stroke:#38a169,stroke-width:2px,color:#1c4532
    classDef container fill:#f5f5f5,stroke:#888,stroke-width:2px,color:#333

    class x binding
    class ptr ownership
    class value,x_value memory
```
<div align="center">
Figure
</div>

Concepts:

- Stack allocation
- Heap allocation
- Box
- Vec
- String
- Rc allocation
- Arc allocation
- Static memory

Questions:

"Where is the actual data?"

---
## 3. VALUE CREATION

How did this value come into existence?


Sources:

Literal:
```rust
let x = 5;
```
Copy:
```rust
let y = x;
```
Move:
```rust
let y = String::new();
```
Clone:
```rust
let y = x.clone();
```
Function return:
```rust
let x = foo();
```
Struct:
```rust
let x = MyStruct {};
```
Enum:
```rust
let x = Some(value);
```


Questions:

"What is the origin node?"

---

## 4. FLOW / TRANSFER

How did identity travel?

This section tracks how a value moves from one binding to another.

Important distinction:
- Data flow: a value was used to produce another value.
- Move: ownership transferred.
- Borrow: access was temporarily granted.
- Clone: a new handle/value relationship was created.

---

### Value flow

Example:
```rust
let a = String::from("hello");
let b = transform(a);
```
Question:
"What value did b come from?"

This is a dependency relationship.
It does not necessarily mean the same memory exists.

```mermaid
flowchart LR
    a["a<br/>Original value"]
    flow["data flow"]
    b["b<br/>Derived value"]

    a --> flow
    flow --> b

    classDef variable fill:#e3f2fd,stroke:#1565c0
    classDef action fill:#fff3e0,stroke:#ef6c00

    class a,b variable
    class flow action
```
<div align="center">
Figure
</div>

###  Ownership transfer

Example:
```rust
let s1 = String::from("hello");
let s2 = s1;
```

Question:
"Who owns the allocation now?"

The allocation did not move.
The ownership binding moved.


```mermaid
flowchart LR
    s1["s1<br/>Owner"]
    move["move<br/>ownership transfer"]
    s2["s2<br/>New owner"]

    s1 --> move
    move --> s2

    classDef owner fill:#e3f2fd,stroke:#1565c0
    classDef action fill:#fff3e0,stroke:#ef6c00

    class s1,s2 owner
    class move action
```


###  Borrow flow

Example:
let x = String::from("hello");
let y = &x;

Question:
"Who temporarily has access?"

Ownership stays with x.
y is only a reference.

```mermaid
flowchart LR
    x["x<br/>Owner"]
    borrow["&<br/>shared borrow"]
    y["y<br/>Reference"]

    x --> borrow
    borrow --> y

    classDef owner fill:#e3f2fd,stroke:#1565c0
    classDef reference fill:#fff3e0,stroke:#ef6c00

    class x owner
    class borrow,y reference
```
<div align="center">
Figure
</div>
###  Mutable borrow flow

Example:
let mut x = 1;
let y = &mut x;

Question:
"Who temporarily controls mutation?"

Only one mutable reference may exist at a time.
```mermaid
flowchart LR
    x["x<br/>Owner"]
    borrow["&mut<br/>exclusive access"]
    y["y<br/>Mutable reference"]

    x --> borrow
    borrow --> y

    classDef owner fill:#e3f2fd,stroke:#1565c0
    classDef mutation fill:#ffebee,stroke:#c62828

    class x owner
    class borrow,y mutation
```

###  Clone flow
Example:
```rust
let x = Arc::new(value);
let y = x.clone();
```
Question:
"Did we copy the value, or copy access to the value?"

Arc clone creates another owner handle.
The underlying allocation is shared.

```mermaid
flowchart LR
    x["<b>x</b><br/><i>Arc owner handle</i><br/><span style='font-size:12px'>shared ownership</span>"]
    clone["<b>clone()</b><br/><i>ref count +1</i><br/><span style='font-size:12px'>creates another owner</span>"]
    y["<b>y</b><br/><i>Arc owner handle</i><br/><span style='font-size:12px'>same allocation</span>"]

    heap["<b>Heap Allocation</b><br/><i>Shared value</i><br/><span style='font-size:12px'>owned memory</span>"]

    x --> heap
    clone --> y
    y --> heap

    classDef owner fill:#eadcf8,stroke:#8e44ad,stroke-width:3px,color:#2c1838
    classDef action fill:#fdebd0,stroke:#d68910,stroke-width:3px,color:#5b3a00
    classDef value fill:#d5f5e3,stroke:#239b56,stroke-width:3px,color:#123d22

    class x,y owner
    class clone action
    class heap value
```
<div align="center">
Figure
</div>
### Summary
#### Data Flow

"b depends on a"

```mermaid
flowchart LR
    a["<b>a</b><br/><i>source value</i><br/><span style='font-size:12px'>origin of data</span>"]
    b["<b>b</b><br/><i>derived value</i><br/><span style='font-size:12px'>depends on source</span>"]
    a -->|data flow| b
    classDef value fill:#dbeafe,stroke:#1565c0,stroke-width:3px,color:#102a43
    class a,b value
```

---

#### Move

"ownership changed"

```mermaid
flowchart TD
    a["<b>a</b><br/><i>old owner</i>"]
    move["<b>move</b><br/><i>ownership transfer</i>"]
    b["<b>b</b><br/><i>new owner</i>"]

    a -->|ownership changes| move
    move -->|binding updated| b

    classDef owner fill:#eadcf8,stroke:#8e44ad,stroke-width:3px,color:#2c1838
    classDef action fill:#fdebd0,stroke:#d68910,stroke-width:3px,color:#5b3a00

    class a,b owner
    class move action
```
<div align="center">
Figure
</div>
---

#### Borrow

"temporary access"

```mermaid
flowchart LR
    owner["owner<br/>owns value"]
    borrow["& / &mut<br/>borrow"]
    reference["reference<br/>temporary access"]

    owner --> borrow
    borrow --> reference

    classDef owner fill:#e3f2fd,stroke:#1565c0
    classDef action fill:#fff3e0,stroke:#ef6c00
    classDef reference fill:#e8f5e9,stroke:#2e7d32

    class owner owner
    class borrow action
    class reference reference
```

---

#### Clone

"new handle, shared value"

```mermaid
flowchart LR
    x["x<br/>handle"]
    clone["clone"]
    y["y<br/>handle"]

    value["shared value"]

    x --> value
    clone --> y
    y --> value

    classDef handle fill:#e3f2fd,stroke:#1565c0
    classDef action fill:#fff3e0,stroke:#ef6c00
    classDef value fill:#e8f5e9,stroke:#2e7d32

    class x,y handle
    class clone action
    class value value
```
<div align="center">
Figure
</div>
I would keep this exact four-way split because it maps cleanly to the questions your future analyzer needs to answer:

- **Data flow:** "Where did this value come from?"
- **Move:** "Who owns it now?"
- **Borrow:** "Who can temporarily access it?"
- **Clone:** "Who shares the same underlying thing?"

Analyzer questions:
- Where did this value originate?
- Who owns the allocation?
- Who can mutate it?
- Is this a copy, move, borrow, or alias?

Concepts:
- Move
- Copy
- Clone
- Borrow
- Reference
- Pointer
- Function argument
- Return value

Question:

"How did this subject become this subject?"

---

## 5. ACCESS MODEL

Who can see this value?

Exclusive:
```rust
&mut T
```
One writer.

Shared:
```rust
&T
```
Many readers.

Shared ownership:
```rust
Rc<T>
Arc<T>
```

Raw:
```rust
*const T
*mut T
```

Concepts:
- Borrowing
- References
- Raw pointers
- Dereference
- Aliasing

Question:

"Who can reach this memory?"

---

## 6. MUTATION MODEL
Who can change this value?
Direct:
```rust
x = 5;
```
Through borrow:
```rust
*reference = 5;
```
Through interior mutability:
```rust
RefCell
Mutex
RwLock
Atomic
```
Through shared ownership:
```rust
Arc<Mutex<T>>
```
Concepts:
- Assignment
- Field mutation
- Index mutation
- Mutable borrow
- Interior mutability
- Synchronization
Question:
"What can change this?"

---

## 7. SCOPE / VISIBILITY

Where does this exist?

Levels:
```mermaid
flowchart LR
    program["Program"]
    crate["Crate"]
    module["Module"]
    function["Function"]
    block["Block"]
    closure["Closure"]

    program --> crate
    crate --> module
    module --> function
    function --> block
    block --> closure

    classDef scope fill:#e3f2fd,stroke:#1565c0

    class program,crate,module,function,block,closure scope
```
<div align="center">
Figure
</div>
Concepts:
- Global
- Static
- Const
- Module
- Function scope
- Block scope
- Closure capture

Question:
"Where is this name valid?"

---

## 8. LIFETIME

How long can this exist?

Local:
```rust
{
   let x = 1;
}
fn foo<'a>() -> &'a T
&'static T
```

Concepts:

- Lifetime
- Borrow validity
- Scope duration
- Drop timing

Question:

"How long is this relationship valid?"

---

## 9. DATA SHAPE

What kind of thing is flowing?

Primitive:
```rust
i32
bool
```

Owned:
```rust
String
Vec<T>
```

Composite:
```rust
struct
enum
tuple
```

Container:
```rust
Box<T>
Rc<T>
Arc<T>
```

Question:

"What rules apply because of this type?"

---

## 10. SPECIAL BOUNDARIES

Where normal reasoning breaks.

Unsafe:
- raw pointer
- FFI
- union
Async:
- state machines
- Pin
Drop:
- destructor effects
MaybeUninit:
- allocation without value
Cycles:
- Rc graphs
Question:
"What hidden behavior exists?"

---

## FINAL MODEL

A subject can be described as:

```mermaid
flowchart LR
    identity["Identity"]
    allocation["Allocation"]
    value["Value Creation"]
    flow["Flow"]
    access["Access"]
    mutation["Mutation"]
    lifetime["Lifetime"]
    drop["Drop"]

    identity --> allocation
    allocation --> value
    value --> flow
    flow --> access
    access --> mutation
    mutation --> lifetime
    lifetime --> drop

    classDef concept fill:#e3f2fd,stroke:#1565c0

    class identity,allocation,value,flow,access,mutation,lifetime,drop concept
```
<div align="center">
Figure
</div>

Cross-cutting constraints:
Scope
Data Type
Ownership Model
Unsafe Boundaries

## Mermaid Ownership Color System

Purple:
- Ownership
- Authority
- Source of truth
- Allocation owner

Blue:
- Binding
- Identifier
- Variable name
- Local handle

Orange:
- Access path
- Reference
- Pointer
- Borrow
- Arc/Rc handle
- Indirection

Green:
- Value / Memory
- Actual data
- Heap allocation
- Produced result

Red:
- Mutation
- Unsafe boundary
- Invalidated state
- Exclusive access

Gray:
- Metadata
- Scope
- Compiler concept
- Annotation

**Centering**
For actual centering, Mermaid itself usually inherits the markdown renderer's .mermaid CSS. In Obsidian you can add a CSS snippet: